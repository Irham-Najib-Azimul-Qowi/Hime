use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use hime_pal::selection::SelectionReader;
use hime_security::redaction::redact_api_keys;

use crate::bus::{EventBus, SystemEvent};

/// Translation Service powering Island Intelligence selected-text translation (UR-16).
///
/// Implements:
/// - Explicit opt-in gate (defaults to disabled for user privacy)
/// - Sensitive text detection (blocks API keys, private keys, passwords)
/// - Clipboard-restoring selection capture via Hime PAL
/// - Local translation cache
pub struct TranslationService {
    is_opted_in: AtomicBool,
    cache: Arc<RwLock<HashMap<String, String>>>,
    event_bus: EventBus,
}

impl TranslationService {
    pub fn new(event_bus: EventBus, opted_in: bool) -> Self {
        Self {
            is_opted_in: AtomicBool::new(opted_in),
            cache: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
        }
    }

    pub fn is_opted_in(&self) -> bool {
        self.is_opted_in.load(Ordering::Relaxed)
    }

    pub fn set_opted_in(&self, enabled: bool) {
        self.is_opted_in.store(enabled, Ordering::Relaxed);
        info!(enabled = enabled, "Translation opt-in preference changed");
    }

    /// Captures the current system selection via PAL and translates it.
    pub async fn capture_and_translate(&self, target_lang: &str) -> Result<TranslationResult, String> {
        if !self.is_opted_in() {
            return Err("Text translation is an opt-in privacy feature and is currently disabled".to_string());
        }

        let selected = SelectionReader::get_selected_text()
            .map_err(|e| format!("Failed to capture selected text from active window: {}", e))?;

        self.translate_text(&selected, target_lang).await
    }

    /// Translates raw text with privacy checks, caching, and redaction.
    pub async fn translate_text(
        &self,
        raw_text: &str,
        target_lang: &str,
    ) -> Result<TranslationResult, String> {
        let trimmed = raw_text.trim();
        if trimmed.is_empty() {
            return Err("Selected text is empty".to_string());
        }

        // Privacy check 1: Sensitive text detection (API keys, RSA keys, auth tokens)
        if self.is_sensitive_text(trimmed) {
            warn!("Rejected translation of text containing sensitive credentials/keys");
            return Err("Text appears to contain sensitive credentials or passwords and was blocked from cloud translation".to_string());
        }

        let cache_key = format!("{}:{}", target_lang, trimmed);
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(TranslationResult {
                    original: trimmed.to_string(),
                    translated: cached.clone(),
                    target_lang: target_lang.to_string(),
                    cached: true,
                });
            }
        }

        // Execute translation (Indonesian dictionary mock / model call)
        let translated = Self::perform_translation(trimmed, target_lang);

        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, translated.clone());
        }

        let _ = self.event_bus.publish(SystemEvent::AgentStateChanged {
            new_state: "selection_translated".to_string(),
        });

        Ok(TranslationResult {
            original: trimmed.to_string(),
            translated,
            target_lang: target_lang.to_string(),
            cached: false,
        })
    }

    fn is_sensitive_text(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        if lower.contains("begin rsa private key")
            || lower.contains("begin openssh private key")
            || lower.contains("password=")
            || lower.contains("bearer ")
        {
            return true;
        }

        // Check if redacted version differs (i.e. contains API key)
        let redacted = redact_api_keys(text);
        redacted != text
    }

    fn perform_translation(text: &str, target_lang: &str) -> String {
        if target_lang == "id" {
            match text.trim().to_lowercase().as_str() {
                "hello" => "Halo".to_string(),
                "world" => "Dunia".to_string(),
                "good morning" => "Selamat pagi".to_string(),
                "thank you" => "Terima kasih".to_string(),
                "cancel" => "Batal".to_string(),
                "confirm" => "Konfirmasi".to_string(),
                _ => format!("[ID: {}]", text),
            }
        } else {
            format!("[{}: {}]", target_lang, text)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranslationResult {
    pub original: String,
    pub translated: String,
    pub target_lang: String,
    pub cached: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_translation_opt_in_gate() {
        let bus = EventBus::new(128);
        // Opt-in false by default
        let service = TranslationService::new(bus, false);
        assert!(!service.is_opted_in());

        let _res = service.translate_text("hello", "id").await;
        // Even raw translate_text blocks if sensitive, but here capture_and_translate checks opt_in
        let capture_res = service.capture_and_translate("id").await;
        assert!(capture_res.is_err());
        assert!(capture_res.unwrap_err().contains("opt-in"));
    }

    #[tokio::test]
    async fn test_translation_sensitive_text_blocked() {
        let bus = EventBus::new(128);
        let service = TranslationService::new(bus, true);

        // OpenAI API Key pattern
        let sensitive = "My secret token is sk-1234567890abcdef1234567890abcdef";
        let res = service.translate_text(sensitive, "id").await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("sensitive credentials"));
    }

    #[tokio::test]
    async fn test_translation_caching() {
        let bus = EventBus::new(128);
        let service = TranslationService::new(bus, true);

        let res1 = service.translate_text("hello", "id").await.unwrap();
        assert_eq!(res1.translated, "Halo");
        assert!(!res1.cached);

        let res2 = service.translate_text("hello", "id").await.unwrap();
        assert_eq!(res2.translated, "Halo");
        assert!(res2.cached);
    }
}
