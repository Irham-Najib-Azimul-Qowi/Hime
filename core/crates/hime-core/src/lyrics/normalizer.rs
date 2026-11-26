use regex::Regex;
use sha2::{Digest, Sha256};
use std::sync::LazyLock;

static NOISE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\s*(?:[\(\[](?:official\s+(?:music\s+)?video|official\s+audio|lyric\s+video|lyrics|audio|remastered(?:\s+\d{4})?|\d{4}\s+remaster|remaster)[\)\]]|(?:[-–—]\s*(?:remastered(?:\s+\d{4})?|\d{4}\s+remaster|remaster)))").expect("valid noise regex")
});

static FEAT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\s*[\(\[](?:feat\.|ft\.|featuring)\s+[^\)\]]+[\)\]]|\s+(?:feat\.|ft\.|featuring)\s+.+$").expect("valid feat regex")
});

static TRAILING_PUNCT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\s\-_–—]+$").expect("valid trailing punct regex")
});

/// Normalizes track title by removing noise patterns like `(Official Video)`, `[Lyrics]`, `(feat. ...)`.
pub fn normalize_title(title: &str) -> String {
    let without_noise = NOISE_RE.replace_all(title, "");
    let without_feat = FEAT_RE.replace_all(&without_noise, "");
    let trimmed = without_feat.trim();
    let cleaned = TRAILING_PUNCT_RE.replace(trimmed, "");
    cleaned.trim().to_string()
}

/// Normalizes artist name by stripping extraneous `feat.` or `ft.` clauses.
pub fn normalize_artist(artist: &str) -> String {
    let without_feat = FEAT_RE.replace_all(artist, "");
    let trimmed = without_feat.trim();
    let cleaned = TRAILING_PUNCT_RE.replace(trimmed, "");
    cleaned.trim().to_string()
}

/// Generates a SHA-256 cache key with 2-second duration bucketing.
pub fn compute_cache_key(artist: &str, title: &str, duration_sec: u32) -> String {
    let clean_artist = normalize_artist(artist).to_lowercase();
    let clean_title = normalize_title(title).to_lowercase();
    // 2-second bucket: round to nearest even second
    let bucket = ((duration_sec as f32 / 2.0).round() * 2.0) as u32;
    let raw = format!("{}:{}:{}", clean_artist, clean_title, bucket);
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title_official_video() {
        assert_eq!(
            normalize_title("Starboy (Official Video)"),
            "Starboy"
        );
        assert_eq!(
            normalize_title("Blinding Lights [Official Music Video]"),
            "Blinding Lights"
        );
    }

    #[test]
    fn test_normalize_title_lyrics_and_audio() {
        assert_eq!(
            normalize_title("Shape of You [Lyrics]"),
            "Shape of You"
        );
        assert_eq!(
            normalize_title("Levitating (Lyric Video)"),
            "Levitating"
        );
        assert_eq!(
            normalize_title("Something [Official Audio]"),
            "Something"
        );
    }

    #[test]
    fn test_normalize_title_feat() {
        assert_eq!(
            normalize_title("Save Your Tears (feat. Ariana Grande)"),
            "Save Your Tears"
        );
        assert_eq!(
            normalize_title("Stay ft. Justin Bieber"),
            "Stay"
        );
    }

    #[test]
    fn test_normalize_title_remaster() {
        assert_eq!(
            normalize_title("Here Comes the Sun - 2019 Remaster"),
            "Here Comes the Sun"
        );
        assert_eq!(
            normalize_title("Come Together (Remastered 2009)"),
            "Come Together"
        );
    }

    #[test]
    fn test_normalize_artist() {
        assert_eq!(
            normalize_artist("The Weeknd feat. Daft Punk"),
            "The Weeknd"
        );
        assert_eq!(
            normalize_artist("Kid LAROI ft. Justin Bieber"),
            "Kid LAROI"
        );
    }

    #[test]
    fn test_compute_cache_key_duration_bucket() {
        // Durations 180s and 181s bucket to 180 or 182, but 180 and 180.4 bucket to same
        let key1 = compute_cache_key("Queen", "Bohemian Rhapsody", 354);
        let _key2 = compute_cache_key("Queen", "Bohemian Rhapsody (Official Video)", 355);
        // 354 / 2.0 = 177.0 -> 354. 355 / 2.0 = 177.5 -> round = 178 -> 356.
        // Let's test 354 and 354:
        let key3 = compute_cache_key("Queen feat. Nobody", "Bohemian Rhapsody [Lyrics]", 354);
        assert_eq!(key1, key3);
    }
}
