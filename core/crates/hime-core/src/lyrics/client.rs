use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::warn;

pub const ALLOWED_HOSTS: &[&str] = &["lrclib.net", "api.lrclib.net"];
pub const USER_AGENT: &str = "HimeIsland/3.10.2";
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
pub const NEGATIVE_CACHE_TTL_SECS: i64 = 6 * 3600; // 6 hours

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LrclibResponse {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub track_name: Option<String>,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub duration: Option<f64>,
    pub instrumental: Option<bool>,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
}

#[derive(Clone)]
pub struct LrclibClient {
    client: reqwest::Client,
    backoff_until: Arc<AtomicI64>,
}

impl Default for LrclibClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LrclibClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(USER_AGENT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            backoff_until: Arc::new(AtomicI64::new(0)),
        }
    }

    /// Verifies if a given URL host is permitted under host allowlist rules.
    pub fn is_host_allowed(url_str: &str) -> bool {
        if let Ok(parsed) = reqwest::Url::parse(url_str) {
            if let Some(host) = parsed.host_str() {
                return ALLOWED_HOSTS.contains(&host);
            }
        }
        false
    }

    fn now_secs() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    /// Checks if client is currently in rate-limit backoff.
    pub fn is_backed_off(&self) -> bool {
        let backoff = self.backoff_until.load(Ordering::Relaxed);
        backoff > Self::now_secs()
    }

    fn apply_backoff(&self, retry_after_secs: Option<u64>) {
        let delay = retry_after_secs.unwrap_or(30) as i64;
        let until = Self::now_secs() + delay;
        self.backoff_until.store(until, Ordering::Relaxed);
        warn!("LRCLIB 429 rate limit reached. Backing off for {} seconds", delay);
    }

    /// Fetches lyrics from LRCLIB using exact matching with fallback to search.
    pub async fn fetch_lyrics(
        &self,
        track: &str,
        artist: &str,
        album: Option<&str>,
        duration_sec: Option<u32>,
    ) -> Result<Option<LrclibResponse>, String> {
        if self.is_backed_off() {
            return Err("Rate limit backoff active for LRCLIB".to_string());
        }

        // 1. Attempt exact GET /api/get with album (if provided)
        let exact_url = "https://lrclib.net/api/get";
        if !Self::is_host_allowed(exact_url) {
            return Err("Host not in allowlist".to_string());
        }

        let mut query_params: Vec<(&str, String)> = vec![
            ("track_name", track.to_string()),
            ("artist_name", artist.to_string()),
        ];
        if let Some(alb) = album {
            if !alb.is_empty() {
                query_params.push(("album_name", alb.to_string()));
            }
        }
        if let Some(dur) = duration_sec {
            if dur > 0 {
                query_params.push(("duration", dur.to_string()));
            }
        }

        let resp_opt = self.execute_get(exact_url, &query_params).await?;
        if resp_opt.is_some() {
            return Ok(resp_opt);
        }

        // 2. If album was provided and failed, retry exact GET without album
        if album.is_some() {
            let mut query_no_album: Vec<(&str, String)> = vec![
                ("track_name", track.to_string()),
                ("artist_name", artist.to_string()),
            ];
            if let Some(dur) = duration_sec {
                if dur > 0 {
                    query_no_album.push(("duration", dur.to_string()));
                }
            }
            let resp_no_album = self.execute_get(exact_url, &query_no_album).await?;
            if resp_no_album.is_some() {
                return Ok(resp_no_album);
            }
        }

        // 3. Fallback search GET /api/search
        let search_url = "https://lrclib.net/api/search";
        let search_params = vec![
            ("track_name", track.to_string()),
            ("artist_name", artist.to_string()),
        ];
        self.execute_search(search_url, &search_params, duration_sec).await
    }

    async fn execute_get(
        &self,
        url: &str,
        query: &[(&str, String)],
    ) -> Result<Option<LrclibResponse>, String> {
        let res = self.client.get(url).query(query).send().await;

        match res {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    match resp.json::<LrclibResponse>().await {
                        Ok(item) => Ok(Some(item)),
                        Err(e) => {
                            warn!("Failed to parse LRCLIB response JSON: {}", e);
                            Ok(None)
                        }
                    }
                } else if status.as_u16() == 404 {
                    Ok(None)
                } else if status.as_u16() == 429 {
                    let retry_after = resp
                        .headers()
                        .get("Retry-After")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok());
                    self.apply_backoff(retry_after);
                    Err("LRCLIB rate limited (429)".to_string())
                } else {
                    warn!("LRCLIB unexpected status code: {}", status);
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("LRCLIB request error: {}", e);
                Ok(None)
            }
        }
    }

    /// Selects the best search result from LRCLIB results.
    ///
    /// Tolerance is expanded to +/- 6.0 seconds. If no item matches within tolerance,
    /// falls back to the first result with synced_lyrics or plain_lyrics instead of returning None.
    pub fn select_best_search_result(
        items: &[LrclibResponse],
        duration_sec: Option<u32>,
    ) -> Option<LrclibResponse> {
        // 1. If duration provided, match within expanded tolerance of +/- 6.0 seconds
        if let Some(target_dur) = duration_sec {
            if target_dur > 0 {
                // Prefer match with synced lyrics within tolerance
                if let Some(item) = items.iter().find(|i| {
                    i.duration.map_or(false, |d| (d - target_dur as f64).abs() <= 6.0)
                        && i.synced_lyrics.is_some()
                }) {
                    return Some(item.clone());
                }

                // Match with plain lyrics or instrumental within tolerance
                if let Some(item) = items.iter().find(|i| {
                    i.duration.map_or(false, |d| (d - target_dur as f64).abs() <= 6.0)
                        && (i.synced_lyrics.is_some()
                            || i.plain_lyrics.is_some()
                            || i.instrumental == Some(true))
                }) {
                    return Some(item.clone());
                }

                // Any match within 6.0s duration tolerance
                if let Some(item) = items.iter().find(|i| {
                    i.duration.map_or(false, |d| (d - target_dur as f64).abs() <= 6.0)
                }) {
                    return Some(item.clone());
                }
            }
        }

        // 2. If no exact/duration match, fallback to first search result with synced_lyrics or plain_lyrics
        items
            .iter()
            .find(|i| i.synced_lyrics.is_some())
            .or_else(|| items.iter().find(|i| i.plain_lyrics.is_some()))
            .or_else(|| items.first())
            .cloned()
    }

    async fn execute_search(
        &self,
        url: &str,
        query: &[(&str, String)],
        duration_sec: Option<u32>,
    ) -> Result<Option<LrclibResponse>, String> {
        let res = self.client.get(url).query(query).send().await;

        match res {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    match resp.json::<Vec<LrclibResponse>>().await {
                        Ok(items) => {
                            let found = Self::select_best_search_result(&items, duration_sec);
                            Ok(found)
                        }
                        Err(e) => {
                            warn!("Failed to parse LRCLIB search results JSON: {}", e);
                            Ok(None)
                        }
                    }
                } else if status.as_u16() == 429 {
                    let retry_after = resp
                        .headers()
                        .get("Retry-After")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok());
                    self.apply_backoff(retry_after);
                    Err("LRCLIB rate limited (429)".to_string())
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("LRCLIB search request error: {}", e);
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_allowlist_check() {
        assert!(LrclibClient::is_host_allowed("https://lrclib.net/api/get"));
        assert!(LrclibClient::is_host_allowed("https://api.lrclib.net/api/search"));
        assert!(!LrclibClient::is_host_allowed("https://evil.com/api/get"));
        assert!(!LrclibClient::is_host_allowed("http://127.0.0.1:8080/hack"));
        assert!(!LrclibClient::is_host_allowed("https://google.com"));
    }

    #[test]
    fn test_backoff_timing() {
        let client = LrclibClient::new();
        assert!(!client.is_backed_off());

        client.apply_backoff(Some(5));
        assert!(client.is_backed_off());
    }

    #[test]
    fn test_select_best_search_result_expanded_tolerance() {
        let items = vec![
            LrclibResponse {
                id: Some(1),
                name: Some("Song 1".to_string()),
                track_name: Some("Song 1".to_string()),
                artist_name: Some("Artist".to_string()),
                album_name: None,
                duration: Some(205.0), // diff is 5.0s, within 6.0s tolerance
                instrumental: Some(false),
                plain_lyrics: Some("Plain lyrics".to_string()),
                synced_lyrics: Some("[00:05.00]Synced lyrics".to_string()),
            },
            LrclibResponse {
                id: Some(2),
                name: Some("Song 2".to_string()),
                track_name: Some("Song 2".to_string()),
                artist_name: Some("Artist".to_string()),
                album_name: None,
                duration: Some(250.0), // diff is 50.0s
                instrumental: Some(false),
                plain_lyrics: Some("Plain lyrics 2".to_string()),
                synced_lyrics: Some("[00:05.00]Synced lyrics 2".to_string()),
            },
        ];

        let matched = LrclibClient::select_best_search_result(&items, Some(200));
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().id, Some(1));
    }

    #[test]
    fn test_select_best_search_result_fallback_when_outside_tolerance() {
        let items = vec![
            LrclibResponse {
                id: Some(10),
                name: Some("Song Diff Duration".to_string()),
                track_name: Some("Song Diff Duration".to_string()),
                artist_name: Some("Artist".to_string()),
                album_name: None,
                duration: Some(300.0), // diff is 100s, outside 6.0s
                instrumental: Some(false),
                plain_lyrics: Some("Plain lyrics fallback".to_string()),
                synced_lyrics: Some("[00:05.00]Synced lyrics fallback".to_string()),
            },
        ];

        // Even though duration diff is 100s, fallback returns first result with synced_lyrics instead of None
        let fallback = LrclibClient::select_best_search_result(&items, Some(200));
        assert!(fallback.is_some());
        assert_eq!(fallback.unwrap().id, Some(10));
    }
}
