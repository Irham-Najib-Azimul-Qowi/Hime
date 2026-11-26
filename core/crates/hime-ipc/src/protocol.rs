use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;
pub const REPLAY_DETECTED: i64 = -32001;
pub const UNAUTHORIZED: i64 = -32002;
pub const POLICY_VIOLATION: i64 = -32003;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: impl Into<Value>, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(slice)
    }
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

impl JsonRpcResponse {
    pub fn success(id: impl Into<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            result: Some(result),
            error: None,
        }
    }

    pub fn error(
        id: impl Into<Value>,
        code: i64,
        message: impl Into<String>,
        data: Option<Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data,
            }),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(slice)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcNotification {
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(slice)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamChunkPayload {
    pub chunk_seq: u64,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestMetadata {
    pub nonce: u64,
    pub timestamp: u64,
}

pub const IPC_CONTRACT_VERSION: &str = "v1";

fn default_contract_version() -> String {
    IPC_CONTRACT_VERSION.to_string()
}

pub mod methods {
    pub const AI_PROFILES_LIST: &str = "ai.profiles.list";
    pub const AI_PROFILES_UPSERT: &str = "ai.profiles.upsert";
    pub const AI_PROFILES_TEST: &str = "ai.profiles.test";

    pub const AI_MEMORY_LIST: &str = "ai.memory.list";
    pub const AI_MEMORY_DELETE: &str = "ai.memory.delete";

    pub const AI_SKILLS_LIST: &str = "ai.skills.list";
    pub const AI_SKILLS_RELOAD: &str = "ai.skills.reload";

    pub const AI_MCP_LIST: &str = "ai.mcp.list";
    pub const AI_MCP_RELOAD: &str = "ai.mcp.reload";
    pub const AI_MCP_CALL_TOOL: &str = "ai.mcp.call_tool";

    pub const MODULE_STATUS: &str = "module.status";
    pub const MODULE_INSTALL: &str = "module.install";
    pub const MODULE_ROLLBACK: &str = "module.rollback";
}

// --- AI Provider Profiles (v1) ---

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
pub struct AiProfilesListResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub profiles: Vec<AiProfileSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiProfileUpsertParams {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub id: String,
    pub name: String,
    pub adapter_kind: String,
    pub base_url: String,
    pub default_model: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiProfileTestParams {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub profile_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiProfileTestResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub profile_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    pub message: String,
}

// --- Self-Learning Memory (v1) ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AiMemoryListParams {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMemoryEntryDto {
    pub id: String,
    pub layer: String,
    pub content: String,
    pub source_provenance: String,
    pub pinned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMemoryListResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub entries: Vec<AiMemoryEntryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMemoryDeleteParams {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMemoryDeleteResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub id: String,
    pub deleted: bool,
}

// --- Dynamic Skills (v1) ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiSkillDescriptorDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub risk_level: String,
    pub triggers: Vec<String>,
    pub builtin: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiSkillsListResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub skills: Vec<AiSkillDescriptorDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiSkillsReloadResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub reloaded_count: usize,
    pub skills: Vec<AiSkillDescriptorDto>,
}

// --- MCP Servers (v1) ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMcpServerDto {
    pub name: String,
    pub transport: String,
    pub connected: bool,
    pub tool_count: usize,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMcpListResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub servers: Vec<AiMcpServerDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMcpReloadResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub reloaded_servers: usize,
    pub total_tools: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMcpCallToolParams {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub server: String,
    pub tool: String,
    #[serde(default)]
    pub arguments: Value,
    #[serde(default)]
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMcpCallToolResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub server: String,
    pub tool: String,
    pub ok: bool,
    pub output: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// --- Modular Installer (v1) ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleStatusParams {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub module_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleStatusResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub module_id: String,
    pub installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_path: Option<String>,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModulePackageManifestDto {
    pub version: String,
    pub download_url: String,
    pub sha256: String,
    pub min_shell_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleInstallParams {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub module_id: String,
    pub manifest: ModulePackageManifestDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleInstallResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub module_id: String,
    pub installed_version: String,
    pub active_path: String,
    pub rolled_back: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleRollbackParams {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub module_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleRollbackResponse {
    #[serde(default = "default_contract_version")]
    pub contract_version: String,
    pub module_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restored_version: Option<String>,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = JsonRpcRequest::new(
            "req-1",
            "system.ping",
            Some(serde_json::json!({"client": "hime-desktop"})),
        );
        let bytes = req.to_bytes().unwrap();
        let parsed = JsonRpcRequest::from_slice(&bytes).unwrap();
        assert_eq!(req, parsed);
    }

    #[test]
    fn test_response_success() {
        let resp = JsonRpcResponse::success("req-1", serde_json::json!({"status": "pong"}));
        let bytes = resp.to_bytes().unwrap();
        let parsed = JsonRpcResponse::from_slice(&bytes).unwrap();
        assert_eq!(resp, parsed);
        assert!(parsed.error.is_none());
    }

    #[test]
    fn test_response_error() {
        let resp = JsonRpcResponse::error("req-1", METHOD_NOT_FOUND, "Method not found", None);
        let bytes = resp.to_bytes().unwrap();
        let parsed = JsonRpcResponse::from_slice(&bytes).unwrap();
        assert_eq!(resp, parsed);
        assert_eq!(parsed.error.unwrap().code, METHOD_NOT_FOUND);
    }

    #[test]
    fn test_adr017_ai_profiles_contract_roundtrip() {
        let upsert = AiProfileUpsertParams {
            contract_version: IPC_CONTRACT_VERSION.to_string(),
            id: "gemini-main".to_string(),
            name: "Google Gemini".to_string(),
            adapter_kind: "gemini-native".to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            default_model: "gemini-2.5-flash".to_string(),
            priority: 1,
            enabled: true,
            api_key: Some("AIza-test".to_string()),
        };
        let req = JsonRpcRequest::new(
            "req-prof",
            methods::AI_PROFILES_UPSERT,
            Some(serde_json::to_value(&upsert).unwrap()),
        );
        let bytes = req.to_bytes().unwrap();
        let parsed = JsonRpcRequest::from_slice(&bytes).unwrap();
        let parsed_params: AiProfileUpsertParams =
            serde_json::from_value(parsed.params.unwrap()).unwrap();
        assert_eq!(upsert, parsed_params);
        assert_eq!(parsed.method, "ai.profiles.upsert");
    }

    #[test]
    fn test_adr017_memory_skills_mcp_contract_roundtrip() {
        let mem_req = AiMemoryListParams {
            contract_version: IPC_CONTRACT_VERSION.to_string(),
            layer: Some("procedural".to_string()),
            query: Some("terminal".to_string()),
            limit: Some(10),
        };
        let mem_val = serde_json::to_value(&mem_req).unwrap();
        assert_eq!(serde_json::from_value::<AiMemoryListParams>(mem_val).unwrap(), mem_req);

        let skills_resp = AiSkillsReloadResponse {
            contract_version: IPC_CONTRACT_VERSION.to_string(),
            reloaded_count: 1,
            skills: vec![AiSkillDescriptorDto {
                id: "task-reminder".to_string(),
                name: "Task Reminder".to_string(),
                description: "Manages deadlines".to_string(),
                risk_level: "low".to_string(),
                triggers: vec!["remind".to_string()],
                builtin: true,
                enabled: true,
            }],
        };
        let skills_val = serde_json::to_value(&skills_resp).unwrap();
        assert_eq!(
            serde_json::from_value::<AiSkillsReloadResponse>(skills_val).unwrap(),
            skills_resp
        );

        let mcp_call = AiMcpCallToolParams {
            contract_version: IPC_CONTRACT_VERSION.to_string(),
            server: "filesystem".to_string(),
            tool: "read_file".to_string(),
            arguments: serde_json::json!({"path": "README.md"}),
            approved: true,
        };
        let mcp_val = serde_json::to_value(&mcp_call).unwrap();
        assert_eq!(
            serde_json::from_value::<AiMcpCallToolParams>(mcp_val).unwrap(),
            mcp_call
        );
    }

    #[test]
    fn test_adr017_module_installer_contract_roundtrip() {
        let install_params = ModuleInstallParams {
            contract_version: IPC_CONTRACT_VERSION.to_string(),
            module_id: "hime-ai".to_string(),
            manifest: ModulePackageManifestDto {
                version: "0.2.0".to_string(),
                download_url: "https://github.com/example/hime/releases/download/v0.2.0/hime-ai.bin"
                    .to_string(),
                sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string(),
                min_shell_version: "0.1.0".to_string(),
            },
            shell_version: Some("0.1.0".to_string()),
        };
        let req = JsonRpcRequest::new(
            "req-mod",
            methods::MODULE_INSTALL,
            Some(serde_json::to_value(&install_params).unwrap()),
        );
        let bytes = req.to_bytes().unwrap();
        let parsed = JsonRpcRequest::from_slice(&bytes).unwrap();
        let parsed_params: ModuleInstallParams =
            serde_json::from_value(parsed.params.unwrap()).unwrap();
        assert_eq!(install_params, parsed_params);

        let rollback_resp = ModuleRollbackResponse {
            contract_version: IPC_CONTRACT_VERSION.to_string(),
            module_id: "hime-ai".to_string(),
            restored_version: Some("0.1.0".to_string()),
            status: "rolled_back".to_string(),
        };
        let rb_val = serde_json::to_value(&rollback_resp).unwrap();
        assert_eq!(
            serde_json::from_value::<ModuleRollbackResponse>(rb_val).unwrap(),
            rollback_resp
        );
    }
}
