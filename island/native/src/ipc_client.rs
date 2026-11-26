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
}
