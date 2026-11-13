use regex::Regex;
use std::sync::LazyLock;

static PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // OpenAI API keys
        (
            Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(),
            "[REDACTED_OPENAI_KEY]",
        ),
        // Anthropic API keys
        (
            Regex::new(r"sk-ant-[a-zA-Z0-9_\-]{20,}").unwrap(),
            "[REDACTED_ANTHROPIC_KEY]",
        ),
        // Google API keys
        (
            Regex::new(r"AIza[0-9A-Za-z\-_]{35}").unwrap(),
            "[REDACTED_GOOGLE_KEY]",
        ),
        // Bearer tokens
        (
            Regex::new(r"(?i)bearer\s+[a-zA-Z0-9_\-\.\+/=]{16,}").unwrap(),
            "Bearer [REDACTED_BEARER_TOKEN]",
        ),
        // Private Chain of Thought (<think>...</think>)
        (
            Regex::new(r"(?s)<think>.*?</think>").unwrap(),
            "[PRIVATE_COT_FILTERED]",
        ),
        (
            Regex::new(r"(?s)<thought>.*?</thought>").unwrap(),
            "[PRIVATE_COT_FILTERED]",
        ),
    ]
});

/// Redacts sensitive information from log statements or text displays.
pub fn redact_sensitive_text(input: &str) -> String {
    let mut sanitized = input.to_string();
    for (regex, replacement) in PATTERNS.iter() {
        sanitized = regex.replace_all(&sanitized, *replacement).to_string();
    }
    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction_api_keys() {
        let text = "Connecting with sk-proj1234567890abcdef1234567890 to backend";
        let redacted = redact_sensitive_text(text);
        assert!(!redacted.contains("sk-proj1234567890"));
        assert!(redacted.contains("[REDACTED_OPENAI_KEY]"));
    }

    #[test]
    fn test_redaction_chain_of_thought() {
        let text = "Visible answer.<think>This is internal secret CoT that should never be shown</think> Final answer.";
        let redacted = redact_sensitive_text(text);
        assert!(!redacted.contains("internal secret CoT"));
        assert!(redacted.contains("[PRIVATE_COT_FILTERED]"));
    }
}
