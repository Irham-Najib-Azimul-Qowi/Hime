use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LyricLine {
    pub ms: u32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedLyrics {
    pub lines: Vec<LyricLine>,
    pub offset_ms: i32,
    pub is_instrumental: bool,
    pub has_sync: bool,
    pub plain_text: Option<String>,
}

static TIMESTAMP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[(\d{1,3}):(\d{2})(?:\.(\d{1,3}))?\]").expect("valid timestamp regex")
});

static OFFSET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[offset:\s*([+-]?\d+)\s*\]").expect("valid offset regex")
});

/// High-performance Rust LRC parser supporting standard centisecond [mm:ss.xx]
/// and millisecond [mm:ss.xxx] tags, multi-timestamp expansion, global [offset] shifting,
/// and chronological sorting.
pub fn parse_lrc(input: &str) -> ParsedLyrics {
    let mut offset_ms: i32 = 0;
    let mut raw_lines: Vec<(i64, String)> = Vec::new();
    let mut has_sync = false;
    let mut is_instrumental = false;

    // Check for instrumental markers
    let lower_input = input.to_lowercase();
    if lower_input.contains("[instrumental]") || lower_input.contains("instrumental.") {
        is_instrumental = true;
    }

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check for offset tag
        if let Some(caps) = OFFSET_RE.captures(trimmed) {
            if let Some(m) = caps.get(1) {
                if let Ok(parsed_offset) = m.as_str().parse::<i32>() {
                    offset_ms = parsed_offset;
                }
            }
            continue;
        }

        // Find all timestamps at the start of the line or embedded
        let mut timestamps: Vec<i64> = Vec::new();
        let mut last_end = 0;

        for caps in TIMESTAMP_RE.captures_iter(trimmed) {
            let full_match = caps.get(0).unwrap();
            // Collect timestamp values
            let minutes: i64 = caps[1].parse().unwrap_or(0);
            let seconds: i64 = caps[2].parse().unwrap_or(0);
            let subsec_str = caps.get(3).map(|m| m.as_str()).unwrap_or("0");

            let millis = match subsec_str.len() {
                1 => subsec_str.parse::<i64>().unwrap_or(0) * 100,
                2 => subsec_str.parse::<i64>().unwrap_or(0) * 10,
                3 => subsec_str.parse::<i64>().unwrap_or(0),
                _ => {
                    let slice = &subsec_str[..3];
                    slice.parse::<i64>().unwrap_or(0)
                }
            };

            let line_ms = (minutes * 60 + seconds) * 1000 + millis;
            timestamps.push(line_ms);
            last_end = full_match.end();
        }

        if !timestamps.is_empty() {
            has_sync = true;
            let lyric_text = trimmed[last_end..].trim().to_string();
            for ts in timestamps {
                raw_lines.push((ts, lyric_text.clone()));
            }
        }
    }

    // Apply offset and clamp to 0
    let mut lines: Vec<LyricLine> = raw_lines
        .into_iter()
        .map(|(ts, text)| {
            let adjusted = ts + offset_ms as i64;
            let final_ms = if adjusted < 0 { 0 } else { adjusted as u32 };
            LyricLine { ms: final_ms, text }
        })
        .collect();

    // Sort chronologically
    lines.sort_by_key(|l| l.ms);

    let plain_text = if !has_sync {
        Some(input.to_string())
    } else {
        None
    };

    ParsedLyrics {
        lines,
        offset_ms,
        is_instrumental,
        has_sync,
        plain_text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_centiseconds() {
        let lrc = "[00:12.34]First line\n[01:05.67]Second line";
        let parsed = parse_lrc(lrc);
        assert!(parsed.has_sync);
        assert_eq!(parsed.lines.len(), 2);
        assert_eq!(parsed.lines[0].ms, 12340);
        assert_eq!(parsed.lines[0].text, "First line");
        assert_eq!(parsed.lines[1].ms, 65670);
        assert_eq!(parsed.lines[1].text, "Second line");
    }

    #[test]
    fn test_parse_milliseconds() {
        let lrc = "[00:01.500]Half second\n[00:02.123]Milliseconds test";
        let parsed = parse_lrc(lrc);
        assert_eq!(parsed.lines.len(), 2);
        assert_eq!(parsed.lines[0].ms, 1500);
        assert_eq!(parsed.lines[1].ms, 2123);
    }

    #[test]
    fn test_multi_timestamp_expansion() {
        let lrc = "[00:10.00][00:20.00]Chorus line";
        let parsed = parse_lrc(lrc);
        assert_eq!(parsed.lines.len(), 2);
        assert_eq!(parsed.lines[0].ms, 10000);
        assert_eq!(parsed.lines[0].text, "Chorus line");
        assert_eq!(parsed.lines[1].ms, 20000);
        assert_eq!(parsed.lines[1].text, "Chorus line");
    }

    #[test]
    fn test_offset_tag_handling() {
        let lrc = "[offset:+500]\n[00:01.000]Shifted line";
        let parsed = parse_lrc(lrc);
        assert_eq!(parsed.offset_ms, 500);
        assert_eq!(parsed.lines[0].ms, 1500);

        let lrc_neg = "[offset:-1500]\n[00:01.000]Negative clamp";
        let parsed_neg = parse_lrc(lrc_neg);
        assert_eq!(parsed_neg.offset_ms, -1500);
        assert_eq!(parsed_neg.lines[0].ms, 0); // clamped
    }

    #[test]
    fn test_interlude_empty_lines() {
        let lrc = "[00:10.00]Vocal\n[00:15.00]\n[00:20.00]Vocal again";
        let parsed = parse_lrc(lrc);
        assert_eq!(parsed.lines.len(), 3);
        assert_eq!(parsed.lines[1].ms, 15000);
        assert_eq!(parsed.lines[1].text, "");
    }

    #[test]
    fn test_unsorted_timestamps_sorting() {
        let lrc = "[00:30.00]Line 3\n[00:10.00]Line 1\n[00:20.00]Line 2";
        let parsed = parse_lrc(lrc);
        assert_eq!(parsed.lines[0].ms, 10000);
        assert_eq!(parsed.lines[0].text, "Line 1");
        assert_eq!(parsed.lines[1].ms, 20000);
        assert_eq!(parsed.lines[1].text, "Line 2");
        assert_eq!(parsed.lines[2].ms, 30000);
        assert_eq!(parsed.lines[2].text, "Line 3");
    }

    #[test]
    fn test_instrumental_detection() {
        let lrc = "[ar: Artist]\n[ti: Title]\n[00:00.00][Instrumental]";
        let parsed = parse_lrc(lrc);
        assert!(parsed.is_instrumental);
    }

    #[test]
    fn test_plain_text_without_sync() {
        let plain = "Hello world\nThis is just plain text\nNo tags";
        let parsed = parse_lrc(plain);
        assert!(!parsed.has_sync);
        assert_eq!(parsed.lines.len(), 0);
        assert!(parsed.plain_text.is_some());
    }
}
