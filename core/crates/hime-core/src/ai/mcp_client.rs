use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

use crate::supervisor::JobSupervisor;

/// Default MCP protocol version supported by Hime Core.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Returns the default MCP servers configuration path:
/// `%APPDATA%\ai.hime.desktop\mcp_servers.json` on Windows,
/// or `~/.config/ai.hime.desktop/mcp_servers.json` on Unix.
pub fn default_mcp_config_path() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata)
                .join("ai.hime.desktop")
                .join("mcp_servers.json");
        }
    }
    #[cfg(not(windows))]
    {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg)
                .join("ai.hime.desktop")
                .join("mcp_servers.json");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join(".config")
                .join("ai.hime.desktop")
                .join("mcp_servers.json");
        }
    }
    hime_pal::PlatformPaths::app_data_dir()
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("ai.hime.desktop")
        .join("mcp_servers.json")
}

/// Transport kind for an MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpTransportKind {
    Stdio,
    Http,
    Sse,
}

impl std::fmt::Display for McpTransportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpTransportKind::Stdio => write!(f, "stdio"),
            McpTransportKind::Http => write!(f, "http"),
            McpTransportKind::Sse => write!(f, "sse"),
        }
    }
}

/// Root configuration file structure for `mcp_servers.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpServersConfig {
    #[serde(rename = "mcpServers", alias = "servers", default)]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

/// Per-server configuration entry in `mcp_servers.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpServerConfig {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub transport: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub disabled: bool,
    /// Tool names explicitly marked safe by user configuration (downgrades from High risk).
    #[serde(default)]
    pub safe_tools: Vec<String>,
}

impl McpServerConfig {
    /// Resolves the transport kind from explicit `transport` or inferred fields.
    pub fn resolved_transport(&self) -> McpTransportKind {
        if let Some(ref t) = self.transport {
            match t.trim().to_lowercase().as_str() {
                "sse" => return McpTransportKind::Sse,
                "http" | "streamable-http" | "streamable_http" => return McpTransportKind::Http,
                "stdio" => return McpTransportKind::Stdio,
                _ => {}
            }
        }
        if self.command.is_some() {
            McpTransportKind::Stdio
        } else if let Some(ref url) = self.url {
            if url.trim_end_matches('/').ends_with("/sse") {
                McpTransportKind::Sse
            } else {
                McpTransportKind::Http
            }
        } else {
            McpTransportKind::Stdio
        }
    }

    pub fn is_tool_safe(&self, tool_name: &str) -> bool {
        self.safe_tools.iter().any(|t| t == "*" || t == tool_name)
    }
}

/// JSON-RPC 2.0 request / notification payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC 2.0 response payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

/// MCP Tool descriptor returned by `tools/list`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpToolDescriptor {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "inputSchema", default = "default_object_schema")]
    pub input_schema: Value,
}

fn default_object_schema() -> Value {
    serde_json::json!({ "type": "object", "properties": {} })
}

/// MCP Resource descriptor returned by `resources/list`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpResourceDescriptor {
    pub uri: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "mimeType", default)]
    pub mime_type: Option<String>,
}

/// MCP Prompt descriptor returned by `prompts/list`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpPromptDescriptor {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub arguments: Vec<Value>,
}

/// Runtime status summary for an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerStatus {
    pub name: String,
    pub transport: String,
    pub connected: bool,
    pub tool_count: usize,
    pub resource_count: usize,
    pub prompt_count: usize,
    pub pid: Option<u32>,
    pub last_error: Option<String>,
}

// ============================================================================
// JSON-RPC 2.0 Message Builders & Parsers
// ============================================================================

pub fn build_initialize_request(id: u64) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(Value::from(id)),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {
                "roots": { "listChanged": false },
                "sampling": {}
            },
            "clientInfo": {
                "name": "hime-core",
                "version": env!("CARGO_PKG_VERSION")
            }
        })),
    }
}

pub fn build_initialized_notification() -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: None,
        method: "notifications/initialized".to_string(),
        params: Some(serde_json::json!({})),
    }
}

pub fn build_tools_list_request(id: u64) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(Value::from(id)),
        method: "tools/list".to_string(),
        params: Some(serde_json::json!({})),
    }
}

pub fn build_tools_call_request(id: u64, name: &str, arguments: Value) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(Value::from(id)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": name,
            "arguments": arguments
        })),
    }
}

pub fn build_resources_list_request(id: u64) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(Value::from(id)),
        method: "resources/list".to_string(),
        params: Some(serde_json::json!({})),
    }
}

pub fn build_prompts_list_request(id: u64) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(Value::from(id)),
        method: "prompts/list".to_string(),
        params: Some(serde_json::json!({})),
    }
}

/// Parses a raw JSON-RPC 2.0 response line or HTTP body (supports plain JSON and SSE `data:` lines).
pub fn parse_jsonrpc_response(raw: &str) -> Result<JsonRpcResponse, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Empty JSON-RPC response".to_string());
    }

    // Direct JSON object
    if trimmed.starts_with('{') {
        let resp: JsonRpcResponse = serde_json::from_str(trimmed)
            .map_err(|e| format!("Invalid JSON-RPC response: {}", e))?;
        if resp.jsonrpc != "2.0" {
            return Err(format!("Unsupported JSON-RPC version: {}", resp.jsonrpc));
        }
        return Ok(resp);
    }

    // SSE format: look for `data: {...}` lines
    for line in trimmed.lines() {
        let line = line.trim();
        if let Some(payload) = line.strip_prefix("data:") {
            let payload = payload.trim();
            if payload.starts_with('{') {
                if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(payload) {
                    if resp.jsonrpc == "2.0" {
                        return Ok(resp);
                    }
                }
            }
        }
    }

    Err("No valid JSON-RPC 2.0 payload found in response".to_string())
}

pub fn parse_tools_list_result(val: &Value) -> Result<Vec<McpToolDescriptor>, String> {
    let tools_val = val
        .get("tools")
        .ok_or_else(|| "MCP tools/list result missing 'tools' array".to_string())?;
    serde_json::from_value(tools_val.clone())
        .map_err(|e| format!("Failed to parse MCP tools list: {}", e))
}

pub fn parse_resources_list_result(val: &Value) -> Result<Vec<McpResourceDescriptor>, String> {
    let res_val = val
        .get("resources")
        .ok_or_else(|| "MCP resources/list result missing 'resources' array".to_string())?;
    serde_json::from_value(res_val.clone())
        .map_err(|e| format!("Failed to parse MCP resources list: {}", e))
}

pub fn parse_prompts_list_result(val: &Value) -> Result<Vec<McpPromptDescriptor>, String> {
    let prompts_val = val
        .get("prompts")
        .ok_or_else(|| "MCP prompts/list result missing 'prompts' array".to_string())?;
    serde_json::from_value(prompts_val.clone())
        .map_err(|e| format!("Failed to parse MCP prompts list: {}", e))
}

/// Formats a tool ID in `mcp.<server>.<tool>` convention.
pub fn format_mcp_tool_id(server: &str, tool: &str) -> String {
    format!("mcp.{}.{}", server, tool)
}

/// Parses a `mcp.<server>.<tool>` identifier into `(server, tool)`.
pub fn parse_mcp_tool_id(tool_id: &str) -> Option<(&str, &str)> {
    let rest = tool_id.strip_prefix("mcp.")?;
    let dot_idx = rest.find('.')?;
    let server = &rest[..dot_idx];
    let tool = &rest[dot_idx + 1..];
    if server.is_empty() || tool.is_empty() {
        return None;
    }
    Some((server, tool))
}

// ============================================================================
// Win32 Job Object Guard for MCP Stdio Child Processes
// ============================================================================

/// Holds a dedicated Win32 Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`
/// so every spawned MCP stdio child process is guaranteed to terminate on
/// emergency pause (`Ctrl+Alt+Shift+P`) or core shutdown.
struct McpJobObject {
    #[cfg(windows)]
    handle: windows::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
unsafe impl Send for McpJobObject {}
#[cfg(windows)]
unsafe impl Sync for McpJobObject {}

impl McpJobObject {
    fn new() -> Result<Self, String> {
        #[cfg(windows)]
        {
            use windows::core::PWSTR;
            use windows::Win32::Foundation::CloseHandle;
            use windows::Win32::System::JobObjects::{
                CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
                JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            };

            let job = unsafe { CreateJobObjectW(None, PWSTR::null()) }
                .map_err(|e| format!("CreateJobObjectW failed: {}", e))?;

            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let set_res = unsafe {
                SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const std::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            };

            if let Err(e) = set_res {
                unsafe {
                    let _ = CloseHandle(job);
                }
                return Err(format!("SetInformationJobObject failed: {}", e));
            }

            Ok(Self { handle: job })
        }
        #[cfg(not(windows))]
        {
            Ok(Self {})
        }
    }

    fn assign_child(&self, child: &Child) -> Result<(), String> {
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::HANDLE;
            use windows::Win32::System::JobObjects::AssignProcessToJobObject;

            let raw = child
                .raw_handle()
                .ok_or_else(|| "Child process has no raw Windows handle".to_string())?;
            unsafe { AssignProcessToJobObject(self.handle, HANDLE(raw as *mut std::ffi::c_void)) }
                .map_err(|e| format!("AssignProcessToJobObject failed: {}", e))?;
        }
        #[cfg(not(windows))]
        {
            let _ = child;
        }
        Ok(())
    }
}

impl Drop for McpJobObject {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            use windows::Win32::Foundation::CloseHandle;
            let _ = CloseHandle(self.handle);
        }
    }
}

// ============================================================================
// Active Transport Session
// ============================================================================

struct StdioTransport {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

enum McpTransport {
    Stdio(Mutex<StdioTransport>),
    Http {
        url: String,
        headers: HashMap<String, String>,
        client: HttpClient,
    },
}

struct McpServerSession {
    config: McpServerConfig,
    transport_kind: McpTransportKind,
    transport: McpTransport,
    tools: Vec<McpToolDescriptor>,
    resources: Vec<McpResourceDescriptor>,
    prompts: Vec<McpPromptDescriptor>,
    connected: bool,
    pid: Option<u32>,
    last_error: Option<String>,
}

impl McpServerSession {
    async fn send_request(&self, req: &JsonRpcRequest) -> Result<JsonRpcResponse, String> {
        match &self.transport {
            McpTransport::Stdio(guard) => {
                let mut io = guard.lock().await;
                let mut line = serde_json::to_string(req)
                    .map_err(|e| format!("Serialize JSON-RPC error: {}", e))?;
                line.push('\n');
                io.stdin
                    .write_all(line.as_bytes())
                    .await
                    .map_err(|e| format!("MCP stdio write failed: {}", e))?;
                io.stdin
                    .flush()
                    .await
                    .map_err(|e| format!("MCP stdio flush failed: {}", e))?;

                // Read lines until we get a matching JSON-RPC response (skipping server notifications)
                let expected_id = req.id.clone();
                let timeout_dur = std::time::Duration::from_secs(15);
                tokio::time::timeout(timeout_dur, async {
                    let mut buf = String::new();
                    loop {
                        buf.clear();
                        let n = io
                            .stdout
                            .read_line(&mut buf)
                            .await
                            .map_err(|e| format!("MCP stdio read failed: {}", e))?;
                        if n == 0 {
                            return Err("MCP stdio server closed stdout unexpectedly".to_string());
                        }
                        let trimmed = buf.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        if let Ok(resp) = parse_jsonrpc_response(trimmed) {
                            if resp.id == expected_id || resp.id.is_some() {
                                return Ok(resp);
                            }
                        }
                    }
                })
                .await
                .map_err(|_| "MCP stdio request timed out".to_string())?
            }
            McpTransport::Http {
                url,
                headers,
                client,
            } => {
                let mut builder = client
                    .post(url)
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json, text/event-stream")
                    .json(req);

                for (k, v) in headers {
                    builder = builder.header(k.as_str(), v.as_str());
                }

                let resp = builder
                    .send()
                    .await
                    .map_err(|e| format!("MCP HTTP request failed: {}", e))?;
                let status = resp.status();
                let body = resp
                    .text()
                    .await
                    .map_err(|e| format!("MCP HTTP body read failed: {}", e))?;

                if !status.is_success() {
                    return Err(format!("MCP HTTP error {}: {}", status, body));
                }

                parse_jsonrpc_response(&body)
            }
        }
    }

    async fn send_notification(&self, req: &JsonRpcRequest) -> Result<(), String> {
        match &self.transport {
            McpTransport::Stdio(guard) => {
                let mut io = guard.lock().await;
                let mut line = serde_json::to_string(req)
                    .map_err(|e| format!("Serialize JSON-RPC notification error: {}", e))?;
                line.push('\n');
                io.stdin
                    .write_all(line.as_bytes())
                    .await
                    .map_err(|e| format!("MCP stdio write failed: {}", e))?;
                io.stdin
                    .flush()
                    .await
                    .map_err(|e| format!("MCP stdio flush failed: {}", e))?;
                Ok(())
            }
            McpTransport::Http {
                url,
                headers,
                client,
            } => {
                let mut builder = client
                    .post(url)
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json, text/event-stream")
                    .json(req);
                for (k, v) in headers {
                    builder = builder.header(k.as_str(), v.as_str());
                }
                let _ = builder.send().await;
                Ok(())
            }
        }
    }

    async fn kill_if_stdio(&self) {
        if let McpTransport::Stdio(guard) = &self.transport {
            let mut io = guard.lock().await;
            let _ = io.child.kill().await;
        }
    }
}

// ============================================================================
// MCP Client Host
// ============================================================================

/// Host manager for Model Context Protocol (MCP) servers (`stdio`, `http`, `sse`).
pub struct McpClientHost {
    config_path: PathBuf,
    sessions: RwLock<HashMap<String, Arc<McpServerSession>>>,
    next_id: AtomicU64,
    job_object: RwLock<Option<McpJobObject>>,
    supervisor: Option<Arc<JobSupervisor>>,
    http_client: HttpClient,
}

impl Default for McpClientHost {
    fn default() -> Self {
        Self::new()
    }
}

impl McpClientHost {
    pub fn new() -> Self {
        Self::with_config_path(default_mcp_config_path())
    }

    pub fn with_config_path(config_path: PathBuf) -> Self {
        let job_object = McpJobObject::new().ok();
        let http_client = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| HttpClient::new());

        Self {
            config_path,
            sessions: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            job_object: RwLock::new(job_object),
            supervisor: None,
            http_client,
        }
    }

    pub fn with_supervisor(mut self, supervisor: Arc<JobSupervisor>) -> Self {
        self.supervisor = Some(supervisor);
        self
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    fn alloc_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Loads `mcp_servers.json` from disk. Returns empty config if the file does not exist.
    pub async fn load_config(&self) -> Result<McpServersConfig, String> {
        if !self.config_path.exists() {
            debug!(path = %self.config_path.display(), "MCP config file not found; using empty config");
            return Ok(McpServersConfig::default());
        }
        let raw = tokio::fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| format!("Failed to read MCP config '{}': {}", self.config_path.display(), e))?;
        serde_json::from_str::<McpServersConfig>(&raw)
            .map_err(|e| format!("Failed to parse MCP config JSON: {}", e))
    }

    /// Terminates existing stdio servers, reloads `mcp_servers.json`, and connects all enabled servers.
    pub async fn reload_servers(&self) -> Result<Vec<McpServerStatus>, String> {
        self.shutdown().await;
        let config = self.load_config().await?;

        // Re-create job object if it was dropped during emergency_pause_kill
        {
            let mut job_guard = self.job_object.write().await;
            if job_guard.is_none() {
                *job_guard = McpJobObject::new().ok();
            }
        }

        let mut statuses = Vec::new();
        for (name, srv_cfg) in config.mcp_servers {
            if srv_cfg.disabled {
                statuses.push(McpServerStatus {
                    name,
                    transport: srv_cfg.resolved_transport().to_string(),
                    connected: false,
                    tool_count: 0,
                    resource_count: 0,
                    prompt_count: 0,
                    pid: None,
                    last_error: Some("Disabled in configuration".to_string()),
                });
                continue;
            }

            let status = self.connect_server(&name, srv_cfg).await;
            statuses.push(status);
        }

        Ok(statuses)
    }

    async fn connect_server(&self, name: &str, cfg: McpServerConfig) -> McpServerStatus {
        let kind = cfg.resolved_transport();
        let transport_res: Result<(McpTransport, Option<u32>), String> = match kind {
            McpTransportKind::Stdio => self.spawn_stdio_transport(name, &cfg).await,
            McpTransportKind::Http | McpTransportKind::Sse => {
                if let Some(ref url) = cfg.url {
                    Ok((
                        McpTransport::Http {
                            url: url.clone(),
                            headers: cfg.headers.clone(),
                            client: self.http_client.clone(),
                        },
                        None,
                    ))
                } else {
                    Err(format!("MCP server '{}' missing 'url' for {} transport", name, kind))
                }
            }
        };

        let (transport, pid) = match transport_res {
            Ok(v) => v,
            Err(err) => {
                warn!(server = %name, error = %err, "Failed to start MCP server transport");
                return McpServerStatus {
                    name: name.to_string(),
                    transport: kind.to_string(),
                    connected: false,
                    tool_count: 0,
                    resource_count: 0,
                    prompt_count: 0,
                    pid: None,
                    last_error: Some(err),
                };
            }
        };

        let mut session = McpServerSession {
            config: cfg,
            transport_kind: kind,
            transport,
            tools: Vec::new(),
            resources: Vec::new(),
            prompts: Vec::new(),
            connected: false,
            pid,
            last_error: None,
        };

        // MCP Handshake: initialize -> notifications/initialized -> tools/list, resources/list, prompts/list
        let init_req = build_initialize_request(self.alloc_id());
        match session.send_request(&init_req).await {
            Ok(resp) if resp.error.is_none() => {
                let notif = build_initialized_notification();
                let _ = session.send_notification(&notif).await;
                session.connected = true;

                // Discover tools
                let tools_req = build_tools_list_request(self.alloc_id());
                if let Ok(t_resp) = session.send_request(&tools_req).await {
                    if let Some(res) = t_resp.result {
                        if let Ok(tools) = parse_tools_list_result(&res) {
                            session.tools = tools;
                        }
                    }
                }

                // Discover resources (optional capability)
                let res_req = build_resources_list_request(self.alloc_id());
                if let Ok(r_resp) = session.send_request(&res_req).await {
                    if let Some(res) = r_resp.result {
                        if let Ok(resources) = parse_resources_list_result(&res) {
                            session.resources = resources;
                        }
                    }
                }

                // Discover prompts (optional capability)
                let prompts_req = build_prompts_list_request(self.alloc_id());
                if let Ok(p_resp) = session.send_request(&prompts_req).await {
                    if let Some(res) = p_resp.result {
                        if let Ok(prompts) = parse_prompts_list_result(&res) {
                            session.prompts = prompts;
                        }
                    }
                }

                info!(
                    server = %name,
                    transport = %kind,
                    tools = session.tools.len(),
                    resources = session.resources.len(),
                    prompts = session.prompts.len(),
                    "MCP server connected and initialized"
                );
            }
            Ok(resp) => {
                let err_msg = resp
                    .error
                    .map(|e| format!("MCP initialize error {}: {}", e.code, e.message))
                    .unwrap_or_else(|| "Unknown MCP initialize error".to_string());
                session.last_error = Some(err_msg);
            }
            Err(err) => {
                session.last_error = Some(err);
            }
        }

        let status = McpServerStatus {
            name: name.to_string(),
            transport: kind.to_string(),
            connected: session.connected,
            tool_count: session.tools.len(),
            resource_count: session.resources.len(),
            prompt_count: session.prompts.len(),
            pid: session.pid,
            last_error: session.last_error.clone(),
        };

        let mut map = self.sessions.write().await;
        map.insert(name.to_string(), Arc::new(session));
        status
    }

    async fn spawn_stdio_transport(
        &self,
        name: &str,
        cfg: &McpServerConfig,
    ) -> Result<(McpTransport, Option<u32>), String> {
        let command = cfg
            .command
            .as_deref()
            .filter(|c| !c.trim().is_empty())
            .ok_or_else(|| format!("MCP stdio server '{}' missing 'command'", name))?;

        let mut cmd = Command::new(command);
        cmd.args(&cfg.args)
            .envs(&cfg.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP stdio server '{}': {}", name, e))?;

        // Fail-closed: bind child process to Win32 Job Object so it dies on emergency pause / shutdown
        let job_guard = self.job_object.read().await;
        if let Some(ref job) = *job_guard {
            if let Err(err) = job.assign_child(&child) {
                let _ = child.kill().await;
                return Err(format!(
                    "MCP stdio server '{}' failed Job Object assignment (fail-closed): {}",
                    name, err
                ));
            }
        }

        let pid = child.id();
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to capture MCP stdio stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture MCP stdio stdout".to_string())?;

        Ok((
            McpTransport::Stdio(Mutex::new(StdioTransport {
                child,
                stdin,
                stdout: BufReader::new(stdout),
            })),
            pid,
        ))
    }

    /// Lists runtime status of all loaded MCP servers.
    pub async fn list_servers(&self) -> Vec<McpServerStatus> {
        let map = self.sessions.read().await;
        map.iter()
            .map(|(name, s)| McpServerStatus {
                name: name.clone(),
                transport: s.transport_kind.to_string(),
                connected: s.connected,
                tool_count: s.tools.len(),
                resource_count: s.resources.len(),
                prompt_count: s.prompts.len(),
                pid: s.pid,
                last_error: s.last_error.clone(),
            })
            .collect()
    }

    /// Lists all discovered tools across all connected MCP servers as `(server_name, descriptor, is_safe)`.
    pub async fn list_all_tools(&self) -> Vec<(String, McpToolDescriptor, bool)> {
        let map = self.sessions.read().await;
        let mut out = Vec::new();
        for (server_name, session) in map.iter() {
            if !session.connected {
                continue;
            }
            for tool in &session.tools {
                let is_safe = session.config.is_tool_safe(&tool.name);
                out.push((server_name.clone(), tool.clone(), is_safe));
            }
        }
        out
    }

    /// Executes `tools/call` on a connected MCP server.
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value, String> {
        let session = {
            let map = self.sessions.read().await;
            map.get(server_name).cloned()
        }
        .ok_or_else(|| format!("MCP server '{}' is not registered", server_name))?;

        if !session.connected {
            return Err(format!("MCP server '{}' is not connected", server_name));
        }

        let req = build_tools_call_request(self.alloc_id(), tool_name, arguments);
        let resp = session.send_request(&req).await?;
        if let Some(err) = resp.error {
            return Err(format!("MCP tool error {}: {}", err.code, err.message));
        }
        Ok(resp.result.unwrap_or(Value::Null))
    }

    /// Lists resources from a connected MCP server (via cached or fresh `resources/list`).
    pub async fn list_resources(
        &self,
        server_name: &str,
    ) -> Result<Vec<McpResourceDescriptor>, String> {
        let session = {
            let map = self.sessions.read().await;
            map.get(server_name).cloned()
        }
        .ok_or_else(|| format!("MCP server '{}' is not registered", server_name))?;

        if !session.connected {
            return Err(format!("MCP server '{}' is not connected", server_name));
        }

        let req = build_resources_list_request(self.alloc_id());
        let resp = session.send_request(&req).await?;
        if let Some(err) = resp.error {
            return Err(format!("MCP resources/list error {}: {}", err.code, err.message));
        }
        let val = resp.result.unwrap_or_else(|| serde_json::json!({ "resources": [] }));
        parse_resources_list_result(&val)
    }

    /// Lists prompts from a connected MCP server.
    pub async fn list_prompts(
        &self,
        server_name: &str,
    ) -> Result<Vec<McpPromptDescriptor>, String> {
        let session = {
            let map = self.sessions.read().await;
            map.get(server_name).cloned()
        }
        .ok_or_else(|| format!("MCP server '{}' is not registered", server_name))?;

        if !session.connected {
            return Err(format!("MCP server '{}' is not connected", server_name));
        }

        let req = build_prompts_list_request(self.alloc_id());
        let resp = session.send_request(&req).await?;
        if let Some(err) = resp.error {
            return Err(format!("MCP prompts/list error {}: {}", err.code, err.message));
        }
        let val = resp.result.unwrap_or_else(|| serde_json::json!({ "prompts": [] }));
        parse_prompts_list_result(&val)
    }

    /// Immediately terminates all stdio child processes (used for Emergency Pause `Ctrl+Alt+Shift+P`).
    pub async fn emergency_pause_kill(&self) {
        warn!("Emergency pause triggered: terminating all MCP stdio child processes");
        self.shutdown().await;
        // Dropping the Job Object closes the handle with KILL_ON_JOB_CLOSE
        let mut job_guard = self.job_object.write().await;
        *job_guard = None;
    }

    /// Gracefully shuts down all active MCP server sessions and kills stdio children.
    pub async fn shutdown(&self) {
        let sessions: Vec<Arc<McpServerSession>> = {
            let mut map = self.sessions.write().await;
            map.drain().map(|(_, s)| s).collect()
        };
        for session in sessions {
            session.kill_if_stdio().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_config_parsing_stdio_and_http_sse() {
        let json = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "C:\\temp"],
                    "env": { "NODE_ENV": "production" },
                    "safe_tools": ["read_file"]
                },
                "remote-http": {
                    "url": "http://127.0.0.1:8080/mcp",
                    "transport": "http",
                    "headers": { "Authorization": "Bearer test-token" }
                },
                "remote-sse": {
                    "url": "http://127.0.0.1:9090/sse"
                }
            }
        }"#;

        let cfg: McpServersConfig = serde_json::from_str(json).expect("valid mcp_servers.json");
        assert_eq!(cfg.mcp_servers.len(), 3);

        let fs = &cfg.mcp_servers["filesystem"];
        assert_eq!(fs.resolved_transport(), McpTransportKind::Stdio);
        assert_eq!(fs.command.as_deref(), Some("npx"));
        assert!(fs.is_tool_safe("read_file"));
        assert!(!fs.is_tool_safe("write_file"));

        let http = &cfg.mcp_servers["remote-http"];
        assert_eq!(http.resolved_transport(), McpTransportKind::Http);

        let sse = &cfg.mcp_servers["remote-sse"];
        assert_eq!(sse.resolved_transport(), McpTransportKind::Sse);
    }

    #[test]
    fn test_mcp_jsonrpc_builders_and_parsers() {
        let init = build_initialize_request(1);
        assert_eq!(init.jsonrpc, "2.0");
        assert_eq!(init.method, "initialize");
        assert_eq!(init.id, Some(Value::from(1)));

        let notif = build_initialized_notification();
        assert_eq!(notif.method, "notifications/initialized");
        assert_eq!(notif.id, None);

        let call = build_tools_call_request(42, "search", serde_json::json!({ "q": "hime" }));
        assert_eq!(call.method, "tools/call");
        assert_eq!(call.params.as_ref().unwrap()["name"], "search");

        // Plain JSON response
        let raw_json = r#"{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"echo","description":"Echo text","inputSchema":{"type":"object"}}]}}"#;
        let parsed = parse_jsonrpc_response(raw_json).expect("plain JSON parses");
        let tools = parse_tools_list_result(parsed.result.as_ref().unwrap()).unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "echo");

        // SSE response
        let raw_sse = "event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":3,\"result\":{\"resources\":[{\"uri\":\"file:///a.txt\",\"name\":\"a.txt\"}]}}\n\n";
        let sse_parsed = parse_jsonrpc_response(raw_sse).expect("SSE JSON-RPC parses");
        let resources = parse_resources_list_result(sse_parsed.result.as_ref().unwrap()).unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].uri, "file:///a.txt");

        // Prompts response
        let prompts_val = serde_json::json!({
            "prompts": [{ "name": "summarize", "description": "Summarize doc", "arguments": [] }]
        });
        let prompts = parse_prompts_list_result(&prompts_val).unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "summarize");
    }

    #[test]
    fn test_mcp_tool_id_routing_format() {
        let id = format_mcp_tool_id("github", "create_issue");
        assert_eq!(id, "mcp.github.create_issue");
        assert_eq!(parse_mcp_tool_id(&id), Some(("github", "create_issue")));
        assert_eq!(parse_mcp_tool_id("tasks.create"), None);
        assert_eq!(parse_mcp_tool_id("mcp.onlyserver"), None);
    }

    #[tokio::test]
    async fn test_mcp_client_host_missing_config_returns_empty() {
        let tmp = std::env::temp_dir().join(format!("nonexistent_mcp_{}.json", std::process::id()));
        let host = McpClientHost::with_config_path(tmp);
        let statuses = host.reload_servers().await.expect("missing config is ok");
        assert!(statuses.is_empty());
        assert!(host.list_all_tools().await.is_empty());
    }
}
