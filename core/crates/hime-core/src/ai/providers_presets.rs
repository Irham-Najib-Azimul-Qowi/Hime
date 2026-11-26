use serde::{Deserialize, Serialize};

/// Supported wire protocol adapters in Hime's Adaptive AI Model Gateway (ADR-017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProtocolKind {
    /// OpenAI-compatible `/v1/chat/completions` & `/v1/models` (`Authorization: Bearer <key>`).
    OpenAiCompat,
    /// Google Gemini Native `v1beta/models/{model}:generateContent` & `streamGenerateContent` (`x-goog-api-key`).
    GeminiNative,
    /// Anthropic Messages API `/v1/messages` (`x-api-key`, `anthropic-version`).
    AnthropicCompat,
}

impl ProtocolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtocolKind::OpenAiCompat => "openai-compat",
            ProtocolKind::GeminiNative => "gemini-native",
            ProtocolKind::AnthropicCompat => "anthropic-compat",
        }
    }
}

/// Static/externalizable preset descriptor for a supported AI provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderPreset {
    pub id: String,
    pub display_name: String,
    pub protocol: ProtocolKind,
    pub default_base_url: String,
    pub default_model: String,
    pub requires_api_key: bool,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
}

/// Ordered failover profile entry for routing AI inference requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorityProfile {
    pub profile_id: String,
    pub provider_id: String,
    pub protocol: ProtocolKind,
    pub base_url: String,
    pub model: String,
    /// Lower value = higher priority (1 is tried before 2).
    pub priority: u32,
    pub enabled: bool,
    pub timeout_secs: u64,
    /// Optional DPAPI vault key identifier. Defaults to `provider_id` when `None`.
    pub secret_key_id: Option<String>,
}

impl PriorityProfile {
    pub fn new(
        profile_id: impl Into<String>,
        provider_id: impl Into<String>,
        protocol: ProtocolKind,
        base_url: impl Into<String>,
        model: impl Into<String>,
        priority: u32,
    ) -> Self {
        Self {
            profile_id: profile_id.into(),
            provider_id: provider_id.into(),
            protocol,
            base_url: base_url.into(),
            model: model.into(),
            priority,
            enabled: true,
            timeout_secs: 30,
            secret_key_id: None,
        }
    }

    pub fn from_preset(preset: &ProviderPreset, profile_id: impl Into<String>, priority: u32) -> Self {
        Self {
            profile_id: profile_id.into(),
            provider_id: preset.id.clone(),
            protocol: preset.protocol,
            base_url: preset.default_base_url.clone(),
            model: preset.default_model.clone(),
            priority,
            enabled: true,
            timeout_secs: 30,
            secret_key_id: Some(preset.id.clone()),
        }
    }

    pub fn effective_secret_key_id(&self) -> &str {
        self.secret_key_id.as_deref().unwrap_or(&self.provider_id)
    }
}

/// Returns the built-in registry of all 11 supported provider presets.
pub fn builtin_provider_presets() -> Vec<ProviderPreset> {
    vec![
        ProviderPreset {
            id: "9router-local".to_string(),
            display_name: "9Router (Local Loopback)".to_string(),
            protocol: ProtocolKind::OpenAiCompat,
            default_base_url: "http://127.0.0.1:20128/v1".to_string(),
            default_model: "hime-combo".to_string(),
            requires_api_key: false,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
        },
        ProviderPreset {
            id: "9router-remote".to_string(),
            display_name: "9Router (Remote Instance)".to_string(),
            protocol: ProtocolKind::OpenAiCompat,
            default_base_url: "https://127.0.0.1:20128/v1".to_string(),
            default_model: "hime-combo".to_string(),
            requires_api_key: true,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
        },
        ProviderPreset {
            id: "gemini".to_string(),
            display_name: "Google Gemini (AI Studio)".to_string(),
            protocol: ProtocolKind::GeminiNative,
            default_base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            default_model: "gemini-2.5-flash".to_string(),
            requires_api_key: true,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
        },
        ProviderPreset {
            id: "openai".to_string(),
            display_name: "OpenAI".to_string(),
            protocol: ProtocolKind::OpenAiCompat,
            default_base_url: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-4o".to_string(),
            requires_api_key: true,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
        },
        ProviderPreset {
            id: "groq".to_string(),
            display_name: "Groq Cloud".to_string(),
            protocol: ProtocolKind::OpenAiCompat,
            default_base_url: "https://api.groq.com/openai/v1".to_string(),
            default_model: "llama-3.3-70b-versatile".to_string(),
            requires_api_key: true,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: false,
        },
        ProviderPreset {
            id: "openrouter".to_string(),
            display_name: "OpenRouter".to_string(),
            protocol: ProtocolKind::OpenAiCompat,
            default_base_url: "https://openrouter.ai/api/v1".to_string(),
            default_model: "deepseek/deepseek-chat".to_string(),
            requires_api_key: true,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
        },
        ProviderPreset {
            id: "nvidia".to_string(),
            display_name: "NVIDIA NIM".to_string(),
            protocol: ProtocolKind::OpenAiCompat,
            default_base_url: "https://integrate.api.nvidia.com/v1".to_string(),
            default_model: "meta/llama-3.1-70b-instruct".to_string(),
            requires_api_key: true,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: false,
        },
        ProviderPreset {
            id: "cloudflare-ai".to_string(),
            display_name: "Cloudflare Workers AI".to_string(),
            protocol: ProtocolKind::OpenAiCompat,
            default_base_url: "https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1"
                .to_string(),
            default_model: "@cf/meta/llama-3.1-8b-instruct".to_string(),
            requires_api_key: true,
            supports_streaming: true,
            supports_tools: false,
            supports_vision: false,
        },
        ProviderPreset {
            id: "anthropic".to_string(),
            display_name: "Anthropic Claude".to_string(),
            protocol: ProtocolKind::AnthropicCompat,
            default_base_url: "https://api.anthropic.com/v1".to_string(),
            default_model: "claude-3-5-sonnet-latest".to_string(),
            requires_api_key: true,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
        },
        ProviderPreset {
            id: "ollama".to_string(),
            display_name: "Ollama (Local)".to_string(),
            protocol: ProtocolKind::OpenAiCompat,
            default_base_url: "http://127.0.0.1:11434/v1".to_string(),
            default_model: "llama3.2".to_string(),
            requires_api_key: false,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
        },
        ProviderPreset {
            id: "custom".to_string(),
            display_name: "Custom OpenAI-Compatible Endpoint".to_string(),
            protocol: ProtocolKind::OpenAiCompat,
            default_base_url: "http://127.0.0.1:8080/v1".to_string(),
            default_model: "default".to_string(),
            requires_api_key: false,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: false,
        },
    ]
}

/// Looks up a built-in provider preset by its ID (case-insensitive).
pub fn find_provider_preset(provider_id: &str) -> Option<ProviderPreset> {
    let needle = provider_id.trim().to_lowercase();
    builtin_provider_presets()
        .into_iter()
        .find(|p| p.id == needle)
}

/// Returns the default base URL for a given provider preset ID.
pub fn default_base_url(provider_id: &str) -> Option<String> {
    find_provider_preset(provider_id).map(|p| p.default_base_url)
}

/// Returns a sorted list of enabled `PriorityProfile`s (ascending by `priority`).
pub fn active_failover_chain(profiles: &[PriorityProfile]) -> Vec<PriorityProfile> {
    let mut active: Vec<PriorityProfile> = profiles.iter().filter(|p| p.enabled).cloned().collect();
    active.sort_by_key(|p| p.priority);
    active
}

/// Builds the default priority failover chain (9Router local -> Ollama local).
pub fn default_priority_chain() -> Vec<PriorityProfile> {
    let mut chain = Vec::new();
    if let Some(p) = find_provider_preset("9router-local") {
        chain.push(PriorityProfile::from_preset(&p, "default-9router-local", 1));
    }
    if let Some(p) = find_provider_preset("ollama") {
        chain.push(PriorityProfile::from_preset(&p, "default-ollama-local", 2));
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_eleven_presets_registered() {
        let presets = builtin_provider_presets();
        let expected_ids = [
            "9router-local",
            "9router-remote",
            "gemini",
            "openai",
            "groq",
            "openrouter",
            "nvidia",
            "cloudflare-ai",
            "anthropic",
            "ollama",
            "custom",
        ];

        assert_eq!(presets.len(), expected_ids.len());
        for id in expected_ids {
            let found = find_provider_preset(id);
            assert!(found.is_some(), "Missing preset for {}", id);
            assert!(!found.unwrap().default_base_url.is_empty());
        }
    }

    #[test]
    fn test_protocol_kinds_assigned_correctly() {
        assert_eq!(
            find_provider_preset("9router-local").unwrap().protocol,
            ProtocolKind::OpenAiCompat
        );
        assert_eq!(
            find_provider_preset("gemini").unwrap().protocol,
            ProtocolKind::GeminiNative
        );
        assert_eq!(
            find_provider_preset("anthropic").unwrap().protocol,
            ProtocolKind::AnthropicCompat
        );
        assert_eq!(
            default_base_url("gemini").as_deref(),
            Some("https://generativelanguage.googleapis.com/v1beta")
        );
    }

    #[test]
    fn test_active_failover_chain_filters_and_sorts() {
        let mut p1 = PriorityProfile::new(
            "p1",
            "openai",
            ProtocolKind::OpenAiCompat,
            "https://api.openai.com/v1",
            "gpt-4o",
            30,
        );
        p1.enabled = true;

        let mut p2 = PriorityProfile::new(
            "p2",
            "gemini",
            ProtocolKind::GeminiNative,
            "https://generativelanguage.googleapis.com/v1beta",
            "gemini-2.5-flash",
            10,
        );
        p2.enabled = true;

        let mut p3 = PriorityProfile::new(
            "p3",
            "anthropic",
            ProtocolKind::AnthropicCompat,
            "https://api.anthropic.com/v1",
            "claude-3-5-sonnet-latest",
            5,
        );
        p3.enabled = false;

        let chain = active_failover_chain(&[p1, p2, p3]);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].profile_id, "p2");
        assert_eq!(chain[1].profile_id, "p1");
    }
}
