//! Win32 Named Pipe (`\\.\pipe\hime-ipc`) + token-authenticated JSON-RPC 2.0 client
//! for communicating with the optional `hime-ai` subsystem.

use std::fmt;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Default Windows Named Pipe path exposed by `hime-ai` (`hime-core`).
pub const DEFAULT_HIME_PIPE: &str = r"\\.\pipe\hime-ipc";
pub const IPC_CONTRACT_VERSION: &str = "v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcError {
    Transport(String),
    Serialization(String),
    Rpc {
        code: i64,
        message: String,
    },
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(msg) => write!(f, "IPC transport error: {msg}"),
            Self::Serialization(msg) => write!(f, "IPC serialization error: {msg}"),
            Self::Rpc { code, message } => write!(f, "IPC RPC error ({code}): {message}"),
        }
    }
}

impl std::error::Error for IpcError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiProfileSummary {
    pub id: String,
    pub name: String,
    pub adapter_kind: String,
    pub base_url: String,
    pub default_model: String,
    pub priority: i32,
    pub enabled: bool,
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiProfileUpsertParams {
    pub contract_version: String,
    pub id: String,
    pub name: String,
    pub adapter_kind: String,
    pub base_url: String,
    pub default_model: String,
    pub priority: i32,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiProfileTestResponse {
    pub profile_id: String,
    pub ok: bool,
    #[serde(default)]
    pub latency_ms: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMemoryEntry {
    pub id: String,
    pub layer: String,
    pub content: String,
    pub source_provenance: String,
    pub pinned: bool,
    #[serde(default)]
    pub score: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiSkillDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub risk_level: String,
    pub triggers: Vec<String>,
    pub builtin: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMcpServer {
    pub name: String,
    pub transport: String,
    pub connected: bool,
    pub tool_count: usize,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatTurnParams {
    pub session_id: String,
    pub prompt: String,
    #[serde(default)]
    pub attachments: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatTurnResult {
    pub session_id: String,
    pub reply: String,
    #[serde(default)]
    pub pending_approval_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalResolution {
    pub approval_id: String,
    pub approved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// --- Lyrics DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LyricsGetParams {
    pub title: String,
    pub artist: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<u32>,
    pub allow_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LyricLine {
    pub ms: u32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LyricsResult {
    pub cache_key: String,
    pub title: String,
    pub artist: String,
    #[serde(default)]
    pub album: Option<String>,
    pub duration_sec: u32,
    pub source: String,
    #[serde(default)]
    pub synced_lyrics: Option<String>,
    #[serde(default)]
    pub plain_lyrics: Option<String>,
    #[serde(default)]
    pub lines: Vec<LyricLine>,
    pub is_synced: bool,
    pub is_instrumental: bool,
    pub user_offset_ms: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LyricsSetOffsetResponse {
    pub success: bool,
    pub user_offset_ms: i32,
}

// --- Translate DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslateSelectionParams {
    pub target_lang: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationResult {
    pub original: String,
    pub translated: String,
    pub target_lang: String,
    pub cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslateOptInStatus {
    pub opted_in: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslateSetOptInResponse {
    pub status: String,
    pub opted_in: bool,
}

// --- Productivity DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductivityTaskRecord {
    pub id: String,
    pub board_id: String,
    pub column_id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub priority: String,
    pub status: String,
    #[serde(default)]
    pub due_date: Option<i64>,
    #[serde(default)]
    pub estimated_minutes: Option<i64>,
    #[serde(default)]
    pub actual_minutes: Option<i64>,
    #[serde(default)]
    pub labels: Option<String>,
    #[serde(default)]
    pub parent_task_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateProductivityTaskParams {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_minutes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductivityTaskMoveResponse {
    pub task_id: String,
    pub updated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductivityTaskDeleteResponse {
    pub task_id: String,
    pub deleted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutineSchedule {
    pub days: Vec<Weekday>,
    pub time: String,
    pub duration_min: u32,
    #[serde(default)]
    pub window_min: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EnforcementTier {
    Ingatkan,
    Tegas,
    Paksa,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DistractionRules {
    pub block_during_routine: bool,
    #[serde(default)]
    pub daily_quota_min: Option<u32>,
    #[serde(default)]
    pub sequential_after_focus_min: Option<u32>,
    #[serde(default)]
    pub distraction_apps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoEscalationConfig {
    pub enabled: bool,
    pub missed_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RoutineStreak {
    pub current_count: u32,
    pub best_count: u32,
    #[serde(default)]
    pub last_completed_date: Option<String>,
    #[serde(default)]
    pub frozen_on_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Routine {
    #[serde(default)]
    pub id: String,
    pub title: String,
    pub category: String,
    pub schedule: RoutineSchedule,
    pub enforcement: EnforcementTier,
    #[serde(default)]
    pub allowed_apps: Vec<String>,
    #[serde(default)]
    pub distraction_rules: DistractionRules,
    pub start_grace_min: u32,
    #[serde(default)]
    pub escalation: Option<AutoEscalationConfig>,
    #[serde(default)]
    pub streak: RoutineStreak,
    pub active: bool,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoutineSessionState {
    Ambient,
    Prompted,
    Snoozed,
    Active,
    Completed,
    Skipped,
    Missed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutineSession {
    pub id: String,
    pub routine_id: String,
    pub state: RoutineSessionState,
    pub scheduled_for: i64,
    pub duration_min: u32,
    pub start_grace_min: u32,
    pub snooze_count: u32,
    #[serde(default)]
    pub max_snooze_count: Option<u32>,
    #[serde(default)]
    pub snooze_until: Option<i64>,
    #[serde(default)]
    pub started_at: Option<i64>,
    #[serde(default)]
    pub completed_at: Option<i64>,
    #[serde(default)]
    pub skip_reason: Option<String>,
    #[serde(default)]
    pub is_excused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutineStartSessionResponse {
    pub session: RoutineSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutineSessionOutcomeResponse {
    pub session: RoutineSession,
    pub streak: RoutineStreak,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScreenTimeSessionRecord {
    pub id: String,
    pub app_name: String,
    #[serde(default)]
    pub window_title: Option<String>,
    pub category: String,
    pub duration_secs: i64,
    pub timestamp: i64,
    #[serde(default)]
    pub date_str: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmergencyPauseRecord {
    pub id: String,
    pub started_at: i64,
    pub duration_min: i64,
    pub reason: String,
    pub expired_at: i64,
}

// --- Proactive DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProactiveSuggestionItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub status: String,
    pub score: f64,
    pub created_at: i64,
    #[serde(default)]
    pub responded_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProactiveSettings {
    pub is_enabled: bool,
    pub cooldown_minutes: i64,
    pub attention_mode: String,
    pub quiet_hour_start: u8,
    pub quiet_hour_end: u8,
    #[serde(default)]
    pub muted_until: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProactiveSettingsSetParams {
    pub is_enabled: bool,
    pub attention_mode: String,
    pub quiet_hour_start: u8,
    pub quiet_hour_end: u8,
    pub cooldown_minutes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute_hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProactiveActionResponse {
    pub success: bool,
}

// --- Developer & Documents DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitStatusResult {
    pub current_branch: String,
    #[serde(default)]
    pub staged_files: Vec<String>,
    #[serde(default)]
    pub modified_files: Vec<String>,
    #[serde(default)]
    pub untracked_files: Vec<String>,
    pub is_clean: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeveloperCommandRunParams {
    pub workspace_id: String,
    pub workspace_path: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub is_approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandOutputResult {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub is_success: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanSecretsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretFinding {
    pub file_path: String,
    pub line_number: usize,
    pub secret_type: String,
    pub masked_match: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanSecretsResult {
    #[serde(default)]
    pub findings: Vec<SecretFinding>,
    pub count: usize,
    pub has_secrets: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentJobRecord {
    pub id: String,
    pub operation: String,
    #[serde(default)]
    pub source_path: Option<String>,
    #[serde(default)]
    pub target_path: Option<String>,
    pub status: String,
    pub format: String,
    #[serde(default)]
    pub metadata: Option<String>,
    pub created_at: i64,
    #[serde(default)]
    pub completed_at: Option<i64>,
}

/// Pluggable transport so unit tests can mock the Windows Named Pipe.
pub trait PipeTransport {
    fn send_line(&self, line: &str) -> Result<String, IpcError>;
}

/// Real Windows Named Pipe transport (`\\.\pipe\hime-ipc`).
#[derive(Debug, Clone)]
pub struct WindowsNamedPipeTransport {
    pipe_name: String,
}

impl WindowsNamedPipeTransport {
    pub fn new(pipe_name: impl Into<String>) -> Self {
        Self {
            pipe_name: pipe_name.into(),
        }
    }
}

impl PipeTransport for WindowsNamedPipeTransport {
    fn send_line(&self, line: &str) -> Result<String, IpcError> {
        let mut pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.pipe_name)
            .map_err(|e| IpcError::Transport(format!("open {} failed: {e}", self.pipe_name)))?;

        pipe.write_all(line.as_bytes())
            .and_then(|_| pipe.write_all(b"\n"))
            .and_then(|_| pipe.flush())
            .map_err(|e| IpcError::Transport(format!("write failed: {e}")))?;

        let mut reader = BufReader::new(pipe);
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(|e| IpcError::Transport(format!("read failed: {e}")))?;
        if response.trim().is_empty() {
            return Err(IpcError::Transport("empty response from pipe".to_string()));
        }
        Ok(response)
    }
}

/// Token-authenticated JSON-RPC 2.0 client for `hime-ai`.
pub struct HimeIpcClient<T: PipeTransport = WindowsNamedPipeTransport> {
    transport: T,
    auth_token: Option<String>,
    next_id: AtomicU64,
}

impl HimeIpcClient<WindowsNamedPipeTransport> {
    /// Creates a client bound to `\\.\pipe\hime-ipc` and loads an auth token if present.
    pub fn with_default_pipe() -> Self {
        let token = load_default_ipc_token();
        Self::new(WindowsNamedPipeTransport::new(DEFAULT_HIME_PIPE), token)
    }
}

impl<T: PipeTransport> HimeIpcClient<T> {
    pub fn new(transport: T, auth_token: Option<String>) -> Self {
        Self {
            transport,
            auth_token,
            next_id: AtomicU64::new(1),
        }
    }

    pub fn set_auth_token(&mut self, token: Option<String>) {
        self.auth_token = token;
    }

    /// Sends a JSON-RPC 2.0 request, injecting `auth_token` when configured.
    pub fn call(&self, method: &str, params: Option<Value>) -> Result<Value, IpcError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let enriched_params = match (params, self.auth_token.as_deref()) {
            (Some(Value::Object(mut map)), Some(tok)) => {
                map.entry("auth_token".to_string())
                    .or_insert_with(|| Value::String(tok.to_string()));
                Some(Value::Object(map))
            }
            (None, Some(tok)) => Some(json!({ "auth_token": tok })),
            (other, _) => other,
        };

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(id),
            method: method.to_string(),
            params: enriched_params,
        };
        let payload = serde_json::to_string(&req)
            .map_err(|e| IpcError::Serialization(format!("request encode: {e}")))?;

        let raw_resp = self.transport.send_line(&payload)?;
        let resp: JsonRpcResponse = serde_json::from_str(raw_resp.trim())
            .map_err(|e| IpcError::Serialization(format!("response decode: {e}")))?;

        if let Some(err) = resp.error {
            return Err(IpcError::Rpc {
                code: err.code,
                message: err.message,
            });
        }
        Ok(resp.result.unwrap_or(Value::Null))
    }

    pub fn chat_turn(&self, params: ChatTurnParams) -> Result<ChatTurnResult, IpcError> {
        let val = serde_json::to_value(params)
            .map_err(|e| IpcError::Serialization(e.to_string()))?;
        let res = self.call("ai.chat.turn", Some(val))?;
        serde_json::from_value(res).map_err(|e| IpcError::Serialization(e.to_string()))
    }

    pub fn resolve_approval(&self, resolution: ApprovalResolution) -> Result<Value, IpcError> {
        let val = serde_json::to_value(resolution)
            .map_err(|e| IpcError::Serialization(e.to_string()))?;
        self.call("ai.approval.resolve", Some(val))
    }

    pub fn list_profiles(&self) -> Result<Vec<AiProfileSummary>, IpcError> {
        let res = self.call("ai.profiles.list", None)?;
        let profiles = res
            .get("profiles")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        serde_json::from_value(profiles).map_err(|e| IpcError::Serialization(e.to_string()))
    }

    pub fn upsert_profile(&self, params: AiProfileUpsertParams) -> Result<Value, IpcError> {
        let val = serde_json::to_value(params)
            .map_err(|e| IpcError::Serialization(e.to_string()))?;
        self.call("ai.profiles.upsert", Some(val))
    }

    pub fn test_profile(&self, profile_id: &str) -> Result<AiProfileTestResponse, IpcError> {
        let res = self.call(
            "ai.profiles.test",
            Some(json!({
                "contract_version": IPC_CONTRACT_VERSION,
                "profile_id": profile_id,
            })),
        )?;
        serde_json::from_value(res).map_err(|e| IpcError::Serialization(e.to_string()))
    }

    pub fn list_memory(
        &self,
        layer: Option<&str>,
        query: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<AiMemoryEntry>, IpcError> {
        let res = self.call(
            "ai.memory.list",
            Some(json!({
                "contract_version": IPC_CONTRACT_VERSION,
                "layer": layer,
                "query": query,
                "limit": limit,
            })),
        )?;
        let entries = res
            .get("entries")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        serde_json::from_value(entries).map_err(|e| IpcError::Serialization(e.to_string()))
    }

    pub fn delete_memory(&self, id: &str) -> Result<bool, IpcError> {
        let res = self.call(
            "ai.memory.delete",
            Some(json!({
                "contract_version": IPC_CONTRACT_VERSION,
                "id": id,
            })),
        )?;
        Ok(res.get("deleted").and_then(|v| v.as_bool()).unwrap_or(false))
    }

    pub fn list_skills(&self) -> Result<Vec<AiSkillDescriptor>, IpcError> {
        let res = self.call("ai.skills.list", None)?;
        let skills = res
            .get("skills")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        serde_json::from_value(skills).map_err(|e| IpcError::Serialization(e.to_string()))
    }

    pub fn reload_skills(&self) -> Result<Vec<AiSkillDescriptor>, IpcError> {
        let res = self.call("ai.skills.reload", None)?;
        let skills = res
            .get("skills")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        serde_json::from_value(skills).map_err(|e| IpcError::Serialization(e.to_string()))
    }

    pub fn list_mcp(&self) -> Result<Vec<AiMcpServer>, IpcError> {
        let res = self.call("ai.mcp.list", None)?;
        let servers = res
            .get("servers")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        serde_json::from_value(servers).map_err(|e| IpcError::Serialization(e.to_string()))
    }

    pub fn reload_mcp(&self) -> Result<Value, IpcError> {
        self.call("ai.mcp.reload", None)
    }

    pub fn call_mcp_tool(
        &self,
        server: &str,
        tool: &str,
        arguments: Value,
    ) -> Result<Value, IpcError> {
        self.call(
            "ai.mcp.call_tool",
            Some(json!({
                "contract_version": IPC_CONTRACT_VERSION,
                "server": server,
                "tool": tool,
                "arguments": arguments,
            })),
        )
    }

    fn call_typed<P: Serialize, R: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Option<&P>,
    ) -> Result<R, IpcError> {
        let val = match params {
            Some(p) => Some(
                serde_json::to_value(p).map_err(|e| IpcError::Serialization(e.to_string()))?,
            ),
            None => None,
        };
        let res = self.call(method, val)?;
        serde_json::from_value(res).map_err(|e| IpcError::Serialization(e.to_string()))
    }

    // --- Lyrics ---

    pub fn get_lyrics(&self, params: LyricsGetParams) -> Result<LyricsResult, IpcError> {
        self.call_typed("lyrics.get", Some(&params))
    }

    pub fn set_lyrics_offset(
        &self,
        cache_key: &str,
        offset_ms: i32,
    ) -> Result<LyricsSetOffsetResponse, IpcError> {
        self.call_typed(
            "lyrics.set_offset",
            Some(&json!({
                "cache_key": cache_key,
                "offset_ms": offset_ms,
            })),
        )
    }

    // --- Translate ---

    pub fn translate_selection(
        &self,
        params: TranslateSelectionParams,
    ) -> Result<TranslationResult, IpcError> {
        self.call_typed("translate.selection", Some(&params))
    }

    pub fn get_translate_opt_in(&self) -> Result<TranslateOptInStatus, IpcError> {
        self.call_typed::<(), _>("translate.get_opt_in", None)
    }

    pub fn set_translate_opt_in(
        &self,
        enabled: bool,
    ) -> Result<TranslateSetOptInResponse, IpcError> {
        self.call_typed("translate.set_opt_in", Some(&json!({ "enabled": enabled })))
    }

    // --- Productivity ---

    pub fn list_productivity_tasks(
        &self,
        board_id: Option<&str>,
    ) -> Result<Vec<ProductivityTaskRecord>, IpcError> {
        self.call_typed(
            "productivity.tasks.list",
            Some(&json!({ "board_id": board_id })),
        )
    }

    pub fn create_productivity_task(
        &self,
        params: CreateProductivityTaskParams,
    ) -> Result<ProductivityTaskRecord, IpcError> {
        self.call_typed("productivity.tasks.create", Some(&params))
    }

    pub fn move_productivity_task(
        &self,
        task_id: &str,
        column_id: &str,
        status: &str,
    ) -> Result<ProductivityTaskMoveResponse, IpcError> {
        self.call_typed(
            "productivity.tasks.move",
            Some(&json!({
                "task_id": task_id,
                "column_id": column_id,
                "status": status,
            })),
        )
    }

    pub fn delete_productivity_task(
        &self,
        task_id: &str,
    ) -> Result<ProductivityTaskDeleteResponse, IpcError> {
        self.call_typed(
            "productivity.tasks.delete",
            Some(&json!({ "task_id": task_id })),
        )
    }

    pub fn list_routines(&self, active_only: bool) -> Result<Vec<Routine>, IpcError> {
        self.call_typed(
            "productivity.routines.list",
            Some(&json!({ "active_only": active_only })),
        )
    }

    pub fn start_routine_session(
        &self,
        routine_id: &str,
        scheduled_for: Option<i64>,
    ) -> Result<RoutineStartSessionResponse, IpcError> {
        self.call_typed(
            "productivity.routines.start_session",
            Some(&json!({
                "routine_id": routine_id,
                "scheduled_for": scheduled_for,
            })),
        )
    }

    pub fn complete_routine_session(
        &self,
        session_id: &str,
    ) -> Result<RoutineSessionOutcomeResponse, IpcError> {
        self.call_typed(
            "productivity.routines.complete_session",
            Some(&json!({ "session_id": session_id })),
        )
    }

    pub fn skip_routine_session(
        &self,
        session_id: &str,
        reason: Option<&str>,
    ) -> Result<RoutineSessionOutcomeResponse, IpcError> {
        self.call_typed(
            "productivity.routines.skip_session",
            Some(&json!({
                "session_id": session_id,
                "reason": reason,
            })),
        )
    }

    pub fn list_screentime_sessions(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<ScreenTimeSessionRecord>, IpcError> {
        self.call_typed(
            "productivity.screentime.list",
            Some(&json!({ "limit": limit })),
        )
    }

    pub fn trigger_emergency_pause(
        &self,
        duration_min: i64,
        reason: &str,
    ) -> Result<EmergencyPauseRecord, IpcError> {
        self.call_typed(
            "productivity.enforce.emergency_pause",
            Some(&json!({
                "duration_min": duration_min,
                "reason": reason,
            })),
        )
    }

    // --- Proactive ---

    pub fn evaluate_proactive_suggestions(
        &self,
    ) -> Result<Vec<ProactiveSuggestionItem>, IpcError> {
        self.call_typed::<(), _>("proactive.suggestions.evaluate", None)
    }

    pub fn list_proactive_suggestions(
        &self,
        status: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<ProactiveSuggestionItem>, IpcError> {
        self.call_typed(
            "proactive.suggestions.list",
            Some(&json!({
                "status": status,
                "limit": limit,
            })),
        )
    }

    pub fn respond_proactive_suggestion(
        &self,
        suggestion_id: &str,
        action: &str,
    ) -> Result<ProactiveActionResponse, IpcError> {
        self.call_typed(
            "proactive.suggestions.respond",
            Some(&json!({
                "suggestion_id": suggestion_id,
                "action": action,
            })),
        )
    }

    pub fn get_proactive_settings(&self) -> Result<ProactiveSettings, IpcError> {
        self.call_typed::<(), _>("proactive.settings.get", None)
    }

    pub fn set_proactive_settings(
        &self,
        params: ProactiveSettingsSetParams,
    ) -> Result<ProactiveActionResponse, IpcError> {
        self.call_typed("proactive.settings.set", Some(&params))
    }

    // --- Developer & Documents ---

    pub fn developer_git_status(&self, workspace_path: &str) -> Result<GitStatusResult, IpcError> {
        self.call_typed(
            "developer.git.status",
            Some(&json!({ "workspace_path": workspace_path })),
        )
    }

    pub fn developer_run_command(
        &self,
        params: DeveloperCommandRunParams,
    ) -> Result<CommandOutputResult, IpcError> {
        self.call_typed("developer.command.run", Some(&params))
    }

    pub fn developer_scan_secrets(
        &self,
        params: ScanSecretsParams,
    ) -> Result<ScanSecretsResult, IpcError> {
        self.call_typed("developer.scanner.scan_secrets", Some(&params))
    }

    pub fn list_document_jobs(&self) -> Result<Vec<DocumentJobRecord>, IpcError> {
        self.call_typed::<(), _>("documents.jobs.list", None)
    }
}

fn load_default_ipc_token() -> Option<String> {
    let candidates: Vec<PathBuf> = [
        std::env::var_os("LOCALAPPDATA")
            .map(|p| PathBuf::from(p).join("Hime").join("ipc.token")),
        std::env::var_os("APPDATA")
            .map(|p| PathBuf::from(p).join("ai.hime.desktop").join("ipc.token")),
    ]
    .into_iter()
    .flatten()
    .collect();

    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockTransport {
        last_sent: Mutex<Option<String>>,
        reply: String,
    }

    impl MockTransport {
        fn with_result(result: Value) -> Self {
            let resp = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                result: Some(result),
                error: None,
            };
            Self {
                last_sent: Mutex::new(None),
                reply: serde_json::to_string(&resp).unwrap(),
            }
        }

        fn with_error(code: i64, message: &str) -> Self {
            let resp = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                result: None,
                error: Some(JsonRpcError {
                    code,
                    message: message.to_string(),
                    data: None,
                }),
            };
            Self {
                last_sent: Mutex::new(None),
                reply: serde_json::to_string(&resp).unwrap(),
            }
        }
    }

    impl PipeTransport for MockTransport {
        fn send_line(&self, line: &str) -> Result<String, IpcError> {
            *self.last_sent.lock().unwrap() = Some(line.to_string());
            Ok(self.reply.clone())
        }
    }

    #[test]
    fn client_injects_auth_token_and_parses_profiles() {
        let transport = MockTransport::with_result(json!({
            "contract_version": "v1",
            "profiles": [{
                "id": "9router-local",
                "name": "9Router Local",
                "adapter_kind": "OpenAiCompat",
                "base_url": "http://127.0.0.1:20128/v1",
                "default_model": "gpt-4o-mini",
                "priority": 1,
                "enabled": true,
                "has_api_key": false
            }]
        }));
        let client = HimeIpcClient::new(transport, Some("secret-token".to_string()));
        let profiles = client.list_profiles().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, "9router-local");

        let sent = client.transport.last_sent.lock().unwrap().clone().unwrap();
        let req: JsonRpcRequest = serde_json::from_str(&sent).unwrap();
        assert_eq!(req.method, "ai.profiles.list");
        assert_eq!(
            req.params.unwrap().get("auth_token").and_then(|v| v.as_str()),
            Some("secret-token")
        );
    }

    #[test]
    fn client_surfaces_json_rpc_errors() {
        let transport = MockTransport::with_error(-32002, "Unauthorized");
        let client = HimeIpcClient::new(transport, None);
        let err = client
            .resolve_approval(ApprovalResolution {
                approval_id: "apr-1".to_string(),
                approved: true,
                reason: None,
            })
            .unwrap_err();
        assert_eq!(
            err,
            IpcError::Rpc {
                code: -32002,
                message: "Unauthorized".to_string(),
            }
        );
    }

    #[test]
    fn chat_turn_and_memory_and_skills_roundtrip() {
        let transport = MockTransport::with_result(json!({
            "session_id": "s1",
            "reply": "Hello from Hime AI",
            "pending_approval_id": null
        }));
        let client = HimeIpcClient::new(transport, None);
        let res = client
            .chat_turn(ChatTurnParams {
                session_id: "s1".to_string(),
                prompt: "hi".to_string(),
                attachments: vec!["notes.md".to_string()],
                profile_id: None,
            })
            .unwrap();
        assert_eq!(res.reply, "Hello from Hime AI");
    }

    fn last_request(transport: &MockTransport) -> JsonRpcRequest {
        let sent = transport.last_sent.lock().unwrap().clone().unwrap();
        serde_json::from_str(&sent).unwrap()
    }

    #[test]
    fn lyrics_roundtrip() {
        let client = HimeIpcClient::new(
            MockTransport::with_result(json!({
                "cache_key": "aimer::brave_shine::230",
                "title": "Brave Shine",
                "artist": "Aimer",
                "album": "Fate/stay night",
                "duration_sec": 230,
                "source": "cache",
                "synced_lyrics": "[00:10.00]Line 1",
                "plain_lyrics": "Line 1",
                "lines": [{ "ms": 10000, "text": "Line 1" }],
                "is_synced": true,
                "is_instrumental": false,
                "user_offset_ms": -250
            })),
            None,
        );
        let res = client
            .get_lyrics(LyricsGetParams {
                title: "Brave Shine".to_string(),
                artist: "Aimer".to_string(),
                album: Some("Fate/stay night".to_string()),
                duration_sec: Some(230),
                allow_online: true,
            })
            .unwrap();
        assert_eq!(res.cache_key, "aimer::brave_shine::230");
        assert_eq!(res.lines.len(), 1);
        assert_eq!(res.lines[0].ms, 10000);
        assert_eq!(res.user_offset_ms, -250);
        let req = last_request(&client.transport);
        assert_eq!(req.method, "lyrics.get");
        let p = req.params.unwrap();
        assert_eq!(p["title"], "Brave Shine");
        assert_eq!(p["artist"], "Aimer");
        assert_eq!(p["album"], "Fate/stay night");
        assert_eq!(p["duration_sec"], 230);
        assert_eq!(p["allow_online"], true);

        let offset_client = HimeIpcClient::new(
            MockTransport::with_result(json!({
                "success": true,
                "user_offset_ms": -500
            })),
            None,
        );
        let off = offset_client
            .set_lyrics_offset("aimer::brave_shine::230", -500)
            .unwrap();
        assert!(off.success);
        assert_eq!(off.user_offset_ms, -500);
        let req = last_request(&offset_client.transport);
        assert_eq!(req.method, "lyrics.set_offset");
        let p = req.params.unwrap();
        assert_eq!(p["cache_key"], "aimer::brave_shine::230");
        assert_eq!(p["offset_ms"], -500);
    }

    #[test]
    fn translate_roundtrip() {
        let tr_client = HimeIpcClient::new(
            MockTransport::with_result(json!({
                "original": "good morning",
                "translated": "Selamat pagi",
                "target_lang": "id",
                "cached": false
            })),
            None,
        );
        let tr = tr_client
            .translate_selection(TranslateSelectionParams {
                target_lang: "id".to_string(),
                text: Some("good morning".to_string()),
            })
            .unwrap();
        assert_eq!(tr.translated, "Selamat pagi");
        let req = last_request(&tr_client.transport);
        assert_eq!(req.method, "translate.selection");
        assert_eq!(req.params.as_ref().unwrap()["target_lang"], "id");
        assert_eq!(req.params.as_ref().unwrap()["text"], "good morning");

        let get_client = HimeIpcClient::new(
            MockTransport::with_result(json!({ "opted_in": true })),
            None,
        );
        assert!(get_client.get_translate_opt_in().unwrap().opted_in);
        assert_eq!(last_request(&get_client.transport).method, "translate.get_opt_in");

        let set_client = HimeIpcClient::new(
            MockTransport::with_result(json!({ "status": "updated", "opted_in": false })),
            None,
        );
        let set_res = set_client.set_translate_opt_in(false).unwrap();
        assert_eq!(set_res.status, "updated");
        assert!(!set_res.opted_in);
        let req = last_request(&set_client.transport);
        assert_eq!(req.method, "translate.set_opt_in");
        assert_eq!(req.params.unwrap()["enabled"], false);
    }

    #[test]
    fn productivity_tasks_roundtrip() {
        let task_json = json!({
            "id": "ptask-1",
            "board_id": "board-default",
            "column_id": "col-todo",
            "title": "Finish IPC bridge",
            "description": "Add typed DTOs",
            "priority": "high",
            "status": "todo",
            "due_date": 1800000000,
            "estimated_minutes": 45,
            "actual_minutes": null,
            "labels": "rust,ipc",
            "parent_task_id": null,
            "created_at": 1700000000,
            "updated_at": 1700000000,
            "completed_at": null
        });

        let list_client = HimeIpcClient::new(
            MockTransport::with_result(json!([task_json.clone()])),
            None,
        );
        let tasks = list_client.list_productivity_tasks(Some("board-default")).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "ptask-1");
        let req = last_request(&list_client.transport);
        assert_eq!(req.method, "productivity.tasks.list");
        assert_eq!(req.params.unwrap()["board_id"], "board-default");

        let create_client = HimeIpcClient::new(MockTransport::with_result(task_json), None);
        let created = create_client
            .create_productivity_task(CreateProductivityTaskParams {
                title: "Finish IPC bridge".to_string(),
                board_id: Some("board-default".to_string()),
                column_id: Some("col-todo".to_string()),
                priority: Some("high".to_string()),
                description: Some("Add typed DTOs".to_string()),
                due_date: Some(1800000000),
                estimated_minutes: Some(45),
                labels: Some("rust,ipc".to_string()),
            })
            .unwrap();
        assert_eq!(created.priority, "high");
        let req = last_request(&create_client.transport);
        assert_eq!(req.method, "productivity.tasks.create");
        assert_eq!(req.params.unwrap()["estimated_minutes"], 45);

        let move_client = HimeIpcClient::new(
            MockTransport::with_result(json!({ "task_id": "ptask-1", "updated": true })),
            None,
        );
        let moved = move_client
            .move_productivity_task("ptask-1", "col-doing", "in_progress")
            .unwrap();
        assert!(moved.updated);
        let req = last_request(&move_client.transport);
        assert_eq!(req.method, "productivity.tasks.move");
        assert_eq!(req.params.as_ref().unwrap()["column_id"], "col-doing");
        assert_eq!(req.params.as_ref().unwrap()["status"], "in_progress");

        let del_client = HimeIpcClient::new(
            MockTransport::with_result(json!({ "task_id": "ptask-1", "deleted": true })),
            None,
        );
        let deleted = del_client.delete_productivity_task("ptask-1").unwrap();
        assert!(deleted.deleted);
        let req = last_request(&del_client.transport);
        assert_eq!(req.method, "productivity.tasks.delete");
        assert_eq!(req.params.unwrap()["task_id"], "ptask-1");
    }

    #[test]
    fn productivity_routines_screentime_and_enforce_roundtrip() {
        let session_json = json!({
            "id": "rsess-1",
            "routine_id": "rtn-1",
            "state": "active",
            "scheduled_for": 1700000000,
            "duration_min": 30,
            "start_grace_min": 5,
            "snooze_count": 0,
            "max_snooze_count": 3,
            "snooze_until": null,
            "started_at": 1700000010,
            "completed_at": null,
            "skip_reason": null,
            "is_excused": false
        });
        let streak_json = json!({
            "current_count": 4,
            "best_count": 7,
            "last_completed_date": "2026-11-26",
            "frozen_on_date": null
        });

        let list_client = HimeIpcClient::new(
            MockTransport::with_result(json!([{
                "id": "rtn-1",
                "title": "Deep Study",
                "category": "study",
                "schedule": {
                    "days": ["Monday", "Wednesday"],
                    "time": "19:00",
                    "duration_min": 30,
                    "window_min": 10
                },
                "enforcement": "tegas",
                "allowed_apps": ["code.exe"],
                "distraction_rules": {
                    "block_during_routine": true,
                    "daily_quota_min": 30,
                    "sequential_after_focus_min": null,
                    "distraction_apps": ["discord.exe"]
                },
                "start_grace_min": 5,
                "escalation": { "enabled": true, "missed_threshold": 3 },
                "streak": streak_json,
                "active": true,
                "created_at": 1700000000,
                "updated_at": 1700000000
            }])),
            None,
        );
        let routines = list_client.list_routines(true).unwrap();
        assert_eq!(routines.len(), 1);
        assert_eq!(routines[0].enforcement, EnforcementTier::Tegas);
        assert_eq!(routines[0].schedule.days[0], Weekday::Monday);
        let req = last_request(&list_client.transport);
        assert_eq!(req.method, "productivity.routines.list");
        assert_eq!(req.params.unwrap()["active_only"], true);

        let start_client = HimeIpcClient::new(
            MockTransport::with_result(json!({ "session": session_json })),
            None,
        );
        let started = start_client
            .start_routine_session("rtn-1", Some(1700000000))
            .unwrap();
        assert_eq!(started.session.state, RoutineSessionState::Active);
        let req = last_request(&start_client.transport);
        assert_eq!(req.method, "productivity.routines.start_session");
        assert_eq!(req.params.as_ref().unwrap()["routine_id"], "rtn-1");
        assert_eq!(req.params.as_ref().unwrap()["scheduled_for"], 1700000000);

        let complete_client = HimeIpcClient::new(
            MockTransport::with_result(json!({
                "session": {
                    "id": "rsess-1",
                    "routine_id": "rtn-1",
                    "state": "completed",
                    "scheduled_for": 1700000000,
                    "duration_min": 30,
                    "start_grace_min": 5,
                    "snooze_count": 0,
                    "max_snooze_count": 3,
                    "completed_at": 1700001800
                },
                "streak": streak_json
            })),
            None,
        );
        let completed = complete_client.complete_routine_session("rsess-1").unwrap();
        assert_eq!(completed.session.state, RoutineSessionState::Completed);
        assert_eq!(completed.streak.current_count, 4);
        assert_eq!(
            last_request(&complete_client.transport).method,
            "productivity.routines.complete_session"
        );

        let skip_client = HimeIpcClient::new(
            MockTransport::with_result(json!({
                "session": {
                    "id": "rsess-1",
                    "routine_id": "rtn-1",
                    "state": "skipped",
                    "scheduled_for": 1700000000,
                    "duration_min": 30,
                    "start_grace_min": 5,
                    "snooze_count": 0,
                    "skip_reason": "Sick",
                    "is_excused": true
                },
                "streak": streak_json
            })),
            None,
        );
        let skipped = skip_client
            .skip_routine_session("rsess-1", Some("Sick"))
            .unwrap();
        assert_eq!(skipped.session.state, RoutineSessionState::Skipped);
        assert!(skipped.session.is_excused);
        let req = last_request(&skip_client.transport);
        assert_eq!(req.method, "productivity.routines.skip_session");
        assert_eq!(req.params.unwrap()["reason"], "Sick");

        let st_client = HimeIpcClient::new(
            MockTransport::with_result(json!([{
                "id": "st-1",
                "app_name": "Code.exe",
                "window_title": null,
                "category": "productive",
                "duration_secs": 1200,
                "timestamp": 1700000000,
                "date_str": "2026-11-26"
            }])),
            None,
        );
        let st = st_client.list_screentime_sessions(Some(25)).unwrap();
        assert_eq!(st[0].duration_secs, 1200);
        let req = last_request(&st_client.transport);
        assert_eq!(req.method, "productivity.screentime.list");
        assert_eq!(req.params.unwrap()["limit"], 25);

        let pause_client = HimeIpcClient::new(
            MockTransport::with_result(json!({
                "id": "pause-1",
                "started_at": 1700000000,
                "duration_min": 15,
                "reason": "Urgent call",
                "expired_at": 1700000900
            })),
            None,
        );
        let pause = pause_client
            .trigger_emergency_pause(15, "Urgent call")
            .unwrap();
        assert_eq!(pause.duration_min, 15);
        assert_eq!(pause.reason, "Urgent call");
        let req = last_request(&pause_client.transport);
        assert_eq!(req.method, "productivity.enforce.emergency_pause");
        assert_eq!(req.params.as_ref().unwrap()["duration_min"], 15);
        assert_eq!(req.params.as_ref().unwrap()["reason"], "Urgent call");
    }

    #[test]
    fn proactive_roundtrip() {
        let sug_json = json!([{
            "id": "sug-1",
            "title": "Take a break",
            "content": "You have studied for 90 minutes",
            "category": "wellbeing",
            "status": "pending",
            "score": 0.85,
            "created_at": 1700000000,
            "responded_at": null
        }]);

        let eval_client = HimeIpcClient::new(MockTransport::with_result(sug_json.clone()), None);
        let eval_res = eval_client.evaluate_proactive_suggestions().unwrap();
        assert_eq!(eval_res.len(), 1);
        assert!((eval_res[0].score - 0.85).abs() < f64::EPSILON);
        assert_eq!(
            last_request(&eval_client.transport).method,
            "proactive.suggestions.evaluate"
        );

        let list_client = HimeIpcClient::new(MockTransport::with_result(sug_json), None);
        let list_res = list_client
            .list_proactive_suggestions(Some("pending"), Some(10))
            .unwrap();
        assert_eq!(list_res[0].id, "sug-1");
        let req = last_request(&list_client.transport);
        assert_eq!(req.method, "proactive.suggestions.list");
        assert_eq!(req.params.as_ref().unwrap()["status"], "pending");
        assert_eq!(req.params.as_ref().unwrap()["limit"], 10);

        let resp_client = HimeIpcClient::new(
            MockTransport::with_result(json!({ "success": true })),
            None,
        );
        assert!(resp_client
            .respond_proactive_suggestion("sug-1", "accept")
            .unwrap()
            .success);
        let req = last_request(&resp_client.transport);
        assert_eq!(req.method, "proactive.suggestions.respond");
        assert_eq!(req.params.as_ref().unwrap()["suggestion_id"], "sug-1");
        assert_eq!(req.params.as_ref().unwrap()["action"], "accept");

        let get_client = HimeIpcClient::new(
            MockTransport::with_result(json!({
                "is_enabled": true,
                "cooldown_minutes": 30,
                "attention_mode": "seimbang",
                "quiet_hour_start": 23,
                "quiet_hour_end": 6,
                "muted_until": null,
                "updated_at": 1700000000
            })),
            None,
        );
        let settings = get_client.get_proactive_settings().unwrap();
        assert_eq!(settings.attention_mode, "seimbang");
        assert_eq!(
            last_request(&get_client.transport).method,
            "proactive.settings.get"
        );

        let set_client = HimeIpcClient::new(
            MockTransport::with_result(json!({ "success": true })),
            None,
        );
        assert!(set_client
            .set_proactive_settings(ProactiveSettingsSetParams {
                is_enabled: false,
                attention_mode: "pendiam".to_string(),
                quiet_hour_start: 22,
                quiet_hour_end: 7,
                cooldown_minutes: 60,
                mute_hours: Some(2),
            })
            .unwrap()
            .success);
        let req = last_request(&set_client.transport);
        assert_eq!(req.method, "proactive.settings.set");
        let p = req.params.unwrap();
        assert_eq!(p["is_enabled"], false);
        assert_eq!(p["attention_mode"], "pendiam");
        assert_eq!(p["quiet_hour_start"], 22);
        assert_eq!(p["quiet_hour_end"], 7);
        assert_eq!(p["cooldown_minutes"], 60);
        assert_eq!(p["mute_hours"], 2);
    }

    #[test]
    fn developer_and_documents_roundtrip() {
        let git_client = HimeIpcClient::new(
            MockTransport::with_result(json!({
                "current_branch": "main",
                "staged_files": ["src/lib.rs"],
                "modified_files": ["Cargo.toml"],
                "untracked_files": [],
                "is_clean": false
            })),
            None,
        );
        let st = git_client.developer_git_status("C:/repo").unwrap();
        assert_eq!(st.current_branch, "main");
        assert!(!st.is_clean);
        let req = last_request(&git_client.transport);
        assert_eq!(req.method, "developer.git.status");
        assert_eq!(req.params.unwrap()["workspace_path"], "C:/repo");

        let cmd_client = HimeIpcClient::new(
            MockTransport::with_result(json!({
                "command": "cargo",
                "args": ["test"],
                "exit_code": 0,
                "stdout": "ok",
                "stderr": "",
                "duration_ms": 420,
                "is_success": true
            })),
            None,
        );
        let cmd_res = cmd_client
            .developer_run_command(DeveloperCommandRunParams {
                workspace_id: "ws-1".to_string(),
                workspace_path: "C:/repo".to_string(),
                command: "cargo".to_string(),
                args: vec!["test".to_string()],
                is_approved: true,
            })
            .unwrap();
        assert!(cmd_res.is_success);
        assert_eq!(cmd_res.exit_code, 0);
        let req = last_request(&cmd_client.transport);
        assert_eq!(req.method, "developer.command.run");
        assert_eq!(req.params.as_ref().unwrap()["workspace_id"], "ws-1");
        assert_eq!(req.params.as_ref().unwrap()["is_approved"], true);

        let scan_client = HimeIpcClient::new(
            MockTransport::with_result(json!({
                "findings": [{
                    "file_path": ".env",
                    "line_number": 3,
                    "secret_type": "AWS Access Key",
                    "masked_match": "AKIA****************"
                }],
                "count": 1,
                "has_secrets": true
            })),
            None,
        );
        let scan_res = scan_client
            .developer_scan_secrets(ScanSecretsParams {
                file_path: Some(".env".to_string()),
                path: None,
                content: Some("AWS_KEY=AKIAIOSFODNN7EXAMPLE".to_string()),
            })
            .unwrap();
        assert!(scan_res.has_secrets);
        assert_eq!(scan_res.count, 1);
        assert_eq!(scan_res.findings[0].line_number, 3);
        let req = last_request(&scan_client.transport);
        assert_eq!(req.method, "developer.scanner.scan_secrets");
        assert_eq!(req.params.as_ref().unwrap()["file_path"], ".env");

        let docs_client = HimeIpcClient::new(
            MockTransport::with_result(json!([{
                "id": "job-1",
                "operation": "create_report",
                "source_path": null,
                "target_path": "out.html",
                "status": "completed",
                "format": "html",
                "metadata": null,
                "created_at": 1700000000,
                "completed_at": 1700000002
            }])),
            None,
        );
        let jobs = docs_client.list_document_jobs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, "job-1");
        assert_eq!(jobs[0].target_path.as_deref(), Some("out.html"));
        assert_eq!(
            last_request(&docs_client.transport).method,
            "documents.jobs.list"
        );
    }
}
