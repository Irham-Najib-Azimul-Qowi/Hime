pub mod client;
pub mod normalizer;
pub mod parser;

pub use client::{LrclibClient, LrclibResponse, NEGATIVE_CACHE_TTL_SECS};
pub use normalizer::{compute_cache_key, normalize_artist, normalize_title};
pub use parser::{parse_lrc, LyricLine, ParsedLyrics};

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};
use hime_storage::{CachedLyricsRecord, Database};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LyricsResult {
    pub cache_key: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_sec: u32,
    pub source: String,
    pub synced_lyrics: Option<String>,
    pub plain_lyrics: Option<String>,
    pub lines: Vec<LyricLine>,
    pub is_synced: bool,
    pub is_instrumental: bool,
    pub user_offset_ms: i32,
}

pub struct LyricsService {
    storage: Arc<Database>,
    client: LrclibClient,
}

impl LyricsService {
    pub fn new(storage: Arc<Database>) -> Self {
        Self {
            storage,
            client: LrclibClient::new(),
        }
    }

    pub fn with_client(storage: Arc<Database>, client: LrclibClient) -> Self {
        Self { storage, client }
    }

    fn now_secs() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    /// Fetches lyrics with hierarchical resolution: SQLite cache -> LRCLIB API -> Negative cache.
    pub async fn get_lyrics(
        &self,
        raw_title: &str,
        raw_artist: &str,
        album: Option<&str>,
        duration_sec: Option<u32>,
        allow_online: bool,
    ) -> Result<LyricsResult, String> {
        let clean_title = normalize_title(raw_title);
        let clean_artist = normalize_artist(raw_artist);
        let dur = duration_sec.unwrap_or(0);
        let cache_key = compute_cache_key(&clean_artist, &clean_title, dur);

        // 1. Check local SQLite cache
        if let Ok(Some(cached)) = self.storage.get_cached_lyrics(&cache_key) {
            if cached.source != "not_found" {
                // Valid cached lyrics
                let mut lines = Vec::new();
                let mut is_synced = false;
                let mut is_instrumental = cached.is_instrumental;

                if let Some(ref lrc_content) = cached.synced_lyrics {
                    let parsed = parse_lrc(lrc_content);
                    lines = parsed.lines;
                    is_synced = parsed.has_sync;
                    is_instrumental = is_instrumental || parsed.is_instrumental;
                } else if let Some(ref json_str) = cached.parsed_json {
                    if let Ok(deserialized) = serde_json::from_str::<Vec<LyricLine>>(json_str) {
                        lines = deserialized;
                        is_synced = !lines.is_empty();
                    }
                }

                // Apply user_offset_ms if non-zero
                if cached.user_offset_ms != 0 {
                    for line in &mut lines {
                        let adjusted = line.ms as i64 + cached.user_offset_ms as i64;
                        line.ms = if adjusted < 0 { 0 } else { adjusted as u32 };
                    }
                }

                info!(cache_key = %cache_key, "Lyrics retrieved from local SQLite cache");
                return Ok(LyricsResult {
                    cache_key,
                    title: cached.title,
                    artist: cached.artist,
                    album: cached.album,
                    duration_sec: cached.duration_sec,
                    source: "cache".to_string(),
                    synced_lyrics: cached.synced_lyrics,
                    plain_lyrics: cached.plain_lyrics,
                    lines,
                    is_synced,
                    is_instrumental,
                    user_offset_ms: cached.user_offset_ms,
                });
            } else if !allow_online {
                // Negative cache hit strictly when offline; if online is permitted, proceed to retry
                info!(cache_key = %cache_key, "Local negative cache hit for lyrics (offline)");
                return Ok(LyricsResult {
                    cache_key,
                    title: clean_title,
                    artist: clean_artist,
                    album: album.map(|s| s.to_string()),
                    duration_sec: dur,
                    source: "not_found".to_string(),
                    synced_lyrics: None,
                    plain_lyrics: None,
                    lines: Vec::new(),
                    is_synced: false,
                    is_instrumental: false,
                    user_offset_ms: 0,
                });
            }
        }

        // 2. Fallback to LRCLIB online lookup (if permitted)
        if allow_online {
            match self.client.fetch_lyrics(&clean_title, &clean_artist, album, duration_sec).await {
                Ok(Some(lrclib_resp)) => {
                    let is_inst = lrclib_resp.instrumental.unwrap_or(false);
                    let mut lines = Vec::new();
                    let mut is_synced = false;

                    if let Some(ref synced) = lrclib_resp.synced_lyrics {
                        let parsed = parse_lrc(synced);
                        lines = parsed.lines;
                        is_synced = parsed.has_sync;
                    }

                    let parsed_json = if !lines.is_empty() {
                        serde_json::to_string(&lines).ok()
                    } else {
                        None
                    };

                    let record = CachedLyricsRecord {
                        cache_key: cache_key.clone(),
                        title: clean_title.clone(),
                        artist: clean_artist.clone(),
                        album: lrclib_resp.album_name.clone().or_else(|| album.map(|s| s.to_string())),
                        duration_sec: dur,
                        source: "lrclib".to_string(),
                        synced_lyrics: lrclib_resp.synced_lyrics.clone(),
                        parsed_json,
                        plain_lyrics: lrclib_resp.plain_lyrics.clone(),
                        is_instrumental: is_inst,
                        user_offset_ms: 0,
                        cached_at: Self::now_secs(),
                        expires_at: Some(Self::now_secs() + 30 * 86400), // 30 days
                    };

                    let _ = self.storage.store_cached_lyrics(&record);
                    info!(cache_key = %cache_key, "Lyrics fetched from LRCLIB and cached");

                    return Ok(LyricsResult {
                        cache_key,
                        title: clean_title,
                        artist: clean_artist,
                        album: record.album,
                        duration_sec: dur,
                        source: "lrclib".to_string(),
                        synced_lyrics: lrclib_resp.synced_lyrics,
                        plain_lyrics: lrclib_resp.plain_lyrics,
                        lines,
                        is_synced,
                        is_instrumental: is_inst,
                        user_offset_ms: 0,
                    });
                }
                Ok(None) => {
                    // Do NOT write negative cache on non-matches to allow retries and not lock out songs
                    info!(cache_key = %cache_key, "Lyrics not found on LRCLIB; skipping negative cache");
                }
                Err(err) => {
                    // Do NOT write negative cache on temporary network errors
                    warn!("LRCLIB fetch error: {}", err);
                }
            }
        }

        // Return empty not found state
        Ok(LyricsResult {
            cache_key,
            title: clean_title,
            artist: clean_artist,
            album: album.map(|s| s.to_string()),
            duration_sec: dur,
            source: "not_found".to_string(),
            synced_lyrics: None,
            plain_lyrics: None,
            lines: Vec::new(),
            is_synced: false,
            is_instrumental: false,
            user_offset_ms: 0,
        })
    }

    /// Updates the user offset in milliseconds for a given cached track.
    pub fn set_offset(&self, cache_key: &str, offset_ms: i32) -> Result<bool, String> {
        self.storage
            .update_lyrics_offset(cache_key, offset_ms)
            .map_err(|e| e.to_string())
    }

    /// Clears the lyrics cache.
    pub fn clear_cache(&self) -> Result<usize, String> {
        self.storage
            .clear_lyrics_cache()
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lyrics_service_cache_and_offset() {
        let db = Arc::new(Database::open_in_memory().unwrap());
        let service = LyricsService::new(db.clone());

        let record = CachedLyricsRecord {
            cache_key: compute_cache_key("Aimer", "Brave Shine", 230),
            title: "Brave Shine".to_string(),
            artist: "Aimer".to_string(),
            album: Some("Fate/stay night".to_string()),
            duration_sec: 230,
            source: "lrclib".to_string(),
            synced_lyrics: Some("[00:10.00]Hidari te ni wa sabita kioku".to_string()),
            parsed_json: None,
            plain_lyrics: Some("Hidari te ni wa sabita kioku".to_string()),
            is_instrumental: false,
            user_offset_ms: 0,
            cached_at: 1000,
            expires_at: Some(2000000000),
        };
        db.store_cached_lyrics(&record).unwrap();

        // Query service (offline)
        let res = service
            .get_lyrics("Brave Shine", "Aimer", Some("Fate/stay night"), Some(230), false)
            .await
            .unwrap();
        assert_eq!(res.source, "cache");
        assert_eq!(res.lines.len(), 1);
        assert_eq!(res.lines[0].ms, 10000);
        assert_eq!(res.lines[0].text, "Hidari te ni wa sabita kioku");

        // Set offset
        assert!(service.set_offset(&res.cache_key, -500).unwrap());

        // Re-query: lines must reflect updated offset
        let res2 = service
            .get_lyrics("Brave Shine", "Aimer", Some("Fate/stay night"), Some(230), false)
            .await
            .unwrap();
        assert_eq!(res2.user_offset_ms, -500);
        assert_eq!(res2.lines[0].ms, 9500);

        // Clear cache
        let cleared = service.clear_cache().unwrap();
        assert_eq!(cleared, 1);
    }
}
