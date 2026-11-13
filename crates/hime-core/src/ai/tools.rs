use sha2::Digest;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use hime_security::policy::{Action, PolicyDecision, PolicyEngine, RiskLevel};
use hime_security::redaction::{redact_api_keys, redact_chain_of_thought};
use hime_storage::Database;

use crate::bus::{EventBus, SystemEvent};

/// Tool contract definition in Hime Core.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    pub id: String,
    pub version: String,
    pub domain: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
    pub risk_level: RiskLevel,
    pub idempotent: bool,
    pub timeout_ms: u64,
}

/// Execution output of a tool invocation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
    pub tool_id: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub idempotency_key: Option<String>,
    pub cached: bool,
    pub requires_approval: bool,
    pub approval_ticket: Option<String>,
}

/// Central Tool Gateway for Hime Core.
///
/// Implements strict Policy Enforcement (PEP), parameter validation, approval workflows,
/// idempotency deduplication, output sanitization (SEC-09), and hash-chained audit logging.
pub struct ToolGateway {
    tools: Arc<RwLock<HashMap<String, ToolDefinition>>>,
    storage: Arc<Database>,
    policy: Arc<PolicyEngine>,
    event_bus: EventBus,
    ops_engine: Arc<crate::storage::operations::SafeFileOperationsEngine>,
}

impl ToolGateway {
    pub fn new(storage: Arc<Database>, policy: Arc<PolicyEngine>, event_bus: EventBus) -> Self {
        let mut map = HashMap::new();
        for t in Self::create_default_tools() {
            map.insert(t.id.clone(), t);
        }

        let base_dir = hime_pal::PlatformPaths::app_data_dir();
        let ops_engine = Arc::new(crate::storage::operations::SafeFileOperationsEngine::new(storage.clone(), &base_dir).unwrap_or_else(|_| {
            let tmp = std::env::temp_dir().join(format!("hime_ops_{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            let _ = std::fs::create_dir_all(&tmp);
            crate::storage::operations::SafeFileOperationsEngine::new(storage.clone(), &tmp).expect("Failed to init fallback storage ops engine")
        }));

        Self {
            tools: Arc::new(RwLock::new(map)),
            storage,
            policy,
            event_bus,
            ops_engine,
        }
    }

    pub fn with_ops_engine(
        storage: Arc<Database>,
        policy: Arc<PolicyEngine>,
        event_bus: EventBus,
        ops_engine: Arc<crate::storage::operations::SafeFileOperationsEngine>,
    ) -> Self {
        let mut map = HashMap::new();
        for t in Self::create_default_tools() {
            map.insert(t.id.clone(), t);
        }

        Self {
            tools: Arc::new(RwLock::new(map)),
            storage,
            policy,
            event_bus,
            ops_engine,
        }
    }

    fn create_default_tools() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                id: "tasks.create".to_string(),
                version: "1.0.0".to_string(),
                domain: "tasks".to_string(),
                description: "Create a new planned task in the Hime task engine".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["title"],
                    "properties": {
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "priority": { "type": "string", "enum": ["low", "medium", "high"] }
                    }
                }),
                risk_level: RiskLevel::L1CreateSafe,
                idempotent: false,
                timeout_ms: 5000,
            },
            ToolDefinition {
                id: "tasks.list".to_string(),
                version: "1.0.0".to_string(),
                domain: "tasks".to_string(),
                description: "List tasks managed by Hime Task Engine".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": { "type": "string" }
                    }
                }),
                risk_level: RiskLevel::L0ReadOnly,
                idempotent: true,
                timeout_ms: 3000,
            },
            ToolDefinition {
                id: "tasks.update".to_string(),
                version: "1.0.0".to_string(),
                domain: "tasks".to_string(),
                description: "Update the status or progress of an existing task".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["id", "status"],
                    "properties": {
                        "id": { "type": "string" },
                        "status": { "type": "string" },
                        "progress": { "type": "number" }
                    }
                }),
                risk_level: RiskLevel::L2Modify,
                idempotent: true,
                timeout_ms: 5000,
            },
            ToolDefinition {
                id: "system.info".to_string(),
                version: "1.0.0".to_string(),
                domain: "system".to_string(),
                description: "Retrieve read-only information about the host environment".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
                risk_level: RiskLevel::L0ReadOnly,
                idempotent: true,
                timeout_ms: 2000,
            },
            ToolDefinition {
                id: "files.stat".to_string(),
                version: "1.0.0".to_string(),
                domain: "files".to_string(),
                description: "Inspect file or directory metadata within allowed security paths".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["path"],
                    "properties": {
                        "path": { "type": "string" }
                    }
                }),
                risk_level: RiskLevel::L0ReadOnly,
                idempotent: true,
                timeout_ms: 3000,
            },
            ToolDefinition {
                id: "notify.send".to_string(),
                version: "1.0.0".to_string(),
                domain: "notify".to_string(),
                description: "Dispatch a non-intrusive desktop notification alert to the user".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["message"],
                    "properties": {
                        "title": { "type": "string" },
                        "message": { "type": "string" },
                        "level": { "type": "string", "enum": ["info", "warning", "error"] }
                    }
                }),
                risk_level: RiskLevel::L1CreateSafe,
                idempotent: false,
                timeout_ms: 2000,
            },
            ToolDefinition {
                id: "translate.selection".to_string(),
                version: "1.0.0".to_string(),
                domain: "translation".to_string(),
                description: "Translate selected or provided text into Indonesian (or specified target language)".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["text"],
                    "properties": {
                        "text": { "type": "string" },
                        "target_lang": { "type": "string" }
                    }
                }),
                risk_level: RiskLevel::L0ReadOnly,
                idempotent: true,
                timeout_ms: 10000,
            },
            ToolDefinition {
                id: "storage.search_files".to_string(),
                version: "1.0.0".to_string(),
                domain: "storage".to_string(),
                description: "Search indexed files by query string or keyword in name/path".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "integer" }
                    }
                }),
                risk_level: RiskLevel::L0ReadOnly,
                idempotent: true,
                timeout_ms: 5000,
            },
            ToolDefinition {
                id: "storage.inspect_file".to_string(),
                version: "1.0.0".to_string(),
                domain: "storage".to_string(),
                description: "Inspect full indexed metadata and content hash of a specific file".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["path"],
                    "properties": {
                        "path": { "type": "string" }
                    }
                }),
                risk_level: RiskLevel::L0ReadOnly,
                idempotent: true,
                timeout_ms: 3000,
            },
            ToolDefinition {
                id: "storage.analyze".to_string(),
                version: "1.0.0".to_string(),
                domain: "storage".to_string(),
                description: "Retrieve disk usage breakdown, largest files, and cleanable candidates".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "top_n": { "type": "integer" }
                    }
                }),
                risk_level: RiskLevel::L0ReadOnly,
                idempotent: true,
                timeout_ms: 5000,
            },
            ToolDefinition {
                id: "storage.find_duplicates".to_string(),
                version: "1.0.0".to_string(),
                domain: "storage".to_string(),
                description: "Find duplicate file groups matching identical SHA-256 hashes".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
                risk_level: RiskLevel::L0ReadOnly,
                idempotent: true,
                timeout_ms: 5000,
            },
            ToolDefinition {
                id: "storage.propose_move".to_string(),
                version: "1.0.0".to_string(),
                domain: "storage".to_string(),
                description: "Create a proposal to safely reorganize or move a file according to naming rules".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["source_path", "target_dir"],
                    "properties": {
                        "source_path": { "type": "string" },
                        "target_dir": { "type": "string" },
                        "naming_template": { "type": "string" }
                    }
                }),
                risk_level: RiskLevel::L1CreateSafe,
                idempotent: true,
                timeout_ms: 3000,
            },
            ToolDefinition {
                id: "storage.execute_move".to_string(),
                version: "1.0.0".to_string(),
                domain: "storage".to_string(),
                description: "Execute an atomic safe file move (staging, hash verification, Hime Recycle, journal)".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["source_path", "destination_path"],
                    "properties": {
                        "source_path": { "type": "string" },
                        "destination_path": { "type": "string" }
                    }
                }),
                risk_level: RiskLevel::L2Modify,
                idempotent: false,
                timeout_ms: 10000,
            },
            ToolDefinition {
                id: "storage.delete_file".to_string(),
                version: "1.0.0".to_string(),
                domain: "storage".to_string(),
                description: "Safely move a file to Hime Recycle (reversibly) with required approval".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["path"],
                    "properties": {
                        "path": { "type": "string" }
                    }
                }),
                risk_level: RiskLevel::L3Destructive,
                idempotent: false,
                timeout_ms: 5000,
            },
            ToolDefinition {
                id: "storage.restore_file".to_string(),
                version: "1.0.0".to_string(),
                domain: "storage".to_string(),
                description: "Restore a previously moved or deleted file from Hime Recycle using operation ID".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["operation_id"],
                    "properties": {
                        "operation_id": { "type": "string" }
                    }
                }),
                risk_level: RiskLevel::L2Modify,
                idempotent: false,
                timeout_ms: 5000,
            },
            ToolDefinition {
                id: "storage.handle_new_download".to_string(),
                version: "1.0.0".to_string(),
                domain: "storage".to_string(),
                description: "Inspect a newly downloaded file for completion (.crdownload/.part), duplicate detection, or version candidates".to_string(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "required": ["download_path"],
                    "properties": {
                        "download_path": { "type": "string" }
                    }
                }),
                risk_level: RiskLevel::L0ReadOnly,
                idempotent: true,
                timeout_ms: 5000,
            },
        ]
    }

    pub async fn register_tool(&self, def: ToolDefinition) {
        let mut map = self.tools.write().await;
        map.insert(def.id.clone(), def);
    }

    pub async fn list_tools(&self) -> Vec<ToolDefinition> {
        let map = self.tools.read().await;
        map.values().cloned().collect()
    }

    pub async fn get_tool(&self, tool_id: &str) -> Option<ToolDefinition> {
        let map = self.tools.read().await;
        map.get(tool_id).cloned()
    }

    /// Primary Tool Execution Pipeline:
    /// 1. Tool registration & schema validation
    /// 2. Idempotency cache check
    /// 3. Policy evaluation (PEP)
    /// 4. Approval check
    /// 5. Execution in domain service
    /// 6. Output sanitization (SEC-09)
    /// 7. Hash-chained audit logging
    /// 8. Idempotency recording
    pub async fn execute_tool(
        &self,
        client_id: &str,
        tool_id: &str,
        params: serde_json::Value,
        approval_ticket: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<ToolResult, String> {
        // Step 1: Tool presence
        let def = match self.get_tool(tool_id).await {
            Some(t) => t,
            None => {
                return Ok(ToolResult {
                    tool_id: tool_id.to_string(),
                    success: false,
                    output: serde_json::json!({}),
                    error: Some(format!("Tool '{}' not found in registry", tool_id)),
                    idempotency_key: idempotency_key.map(|s| s.to_string()),
                    cached: false,
                    requires_approval: false,
                    approval_ticket: None,
                });
            }
        };

        // Step 2: Idempotency check
        if def.idempotent {
            if let Some(key) = idempotency_key {
                if let Ok(Some(cached_exec)) = self.storage.get_tool_execution(key) {
                    if let Ok(cached_json) = serde_json::from_str::<serde_json::Value>(&cached_exec.result) {
                        return Ok(ToolResult {
                            tool_id: tool_id.to_string(),
                            success: cached_exec.status == "completed",
                            output: cached_json,
                            error: None,
                            idempotency_key: Some(key.to_string()),
                            cached: true,
                            requires_approval: false,
                            approval_ticket: None,
                        });
                    }
                }
            }
        }

        // Step 3: Policy evaluation
        let action = Action::ToolCall {
            tool_id: tool_id.to_string(),
            domain: def.domain.clone(),
            level: def.risk_level,
            payload: params.to_string(),
        };

        let decision = {
            use hime_security::policy::PolicyEnforcer;
            self.policy.evaluate(client_id, &action)
        };

        match decision {
            PolicyDecision::Deny { reason } => {
                warn!(tool_id = %tool_id, reason = %reason, "Tool execution denied by policy");
                let _ = self.storage.record_audit_event(
                    "tool.denied",
                    client_id,
                    &format!("{{\"tool\":\"{}\",\"reason\":\"{}\"}}", tool_id, reason),
                );
                return Ok(ToolResult {
                    tool_id: tool_id.to_string(),
                    success: false,
                    output: serde_json::json!({}),
                    error: Some(format!("Policy violation: {}", reason)),
                    idempotency_key: idempotency_key.map(|s| s.to_string()),
                    cached: false,
                    requires_approval: false,
                    approval_ticket: None,
                });
            }
            PolicyDecision::RequireUserApproval { description } => {
                // Check if ticket is presented
                let approved = approval_ticket.map(|t| !t.trim().is_empty()).unwrap_or(false);
                if !approved {
                    let ticket_id = format!("ticket-{}", uuid_simple());
                    info!(tool_id = %tool_id, ticket = %ticket_id, "Tool execution requires user approval");
                    return Ok(ToolResult {
                        tool_id: tool_id.to_string(),
                        success: false,
                        output: serde_json::json!({ "description": description }),
                        error: Some("Operation requires user approval".to_string()),
                        idempotency_key: idempotency_key.map(|s| s.to_string()),
                        cached: false,
                        requires_approval: true,
                        approval_ticket: Some(ticket_id),
                    });
                }
            }
            PolicyDecision::Allow => {}
        }

        // Step 4: Dispatch tool execution
        let exec_result = self.dispatch_tool_logic(client_id, tool_id, &params).await;

        let (success, raw_output, err_msg) = match exec_result {
            Ok(val) => (true, val, None),
            Err(e) => (false, serde_json::json!({}), Some(e)),
        };

        // Step 5: Output sanitization (SEC-09: strip CoT & secrets)
        let sanitized_output = self.sanitize_json_value(raw_output);

        // Step 6: Chained audit logging
        let audit_payload = serde_json::json!({
            "tool": tool_id,
            "success": success,
            "params": params,
        })
        .to_string();
        let _ = self.storage.record_audit_event("tool.executed", client_id, &audit_payload);

        // Step 7: Idempotency recording
        if def.idempotent {
            if let Some(key) = idempotency_key {
                let status_str = if success { "completed" } else { "failed" };
                let _ = self.storage.record_tool_execution(
                    key,
                    tool_id,
                    &format!("{:x}", sha2::Sha256::digest(params.to_string().as_bytes())),
                    &sanitized_output.to_string(),
                    status_str,
                );
            }
        }

        Ok(ToolResult {
            tool_id: tool_id.to_string(),
            success,
            output: sanitized_output,
            error: err_msg,
            idempotency_key: idempotency_key.map(|s| s.to_string()),
            cached: false,
            requires_approval: false,
            approval_ticket: None,
        })
    }

    async fn dispatch_tool_logic(
        &self,
        client_id: &str,
        tool_id: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match tool_id {
            "tasks.create" => {
                let title = params["title"]
                    .as_str()
                    .ok_or("Missing required field 'title'")?;
                let description = params["description"].as_str();
                let priority = params["priority"].as_str().unwrap_or("medium");
                let task_id = format!("task-{}", uuid_simple());

                let task = self
                    .storage
                    .create_task(&task_id, title, description, priority, "queued")
                    .map_err(|e| format!("Storage error: {}", e))?;

                let _ = self.event_bus.publish(SystemEvent::TaskUpdated {
                    task_id: task_id.clone(),
                    status: "queued".to_string(),
                });

                serde_json::to_value(&task).map_err(|e| e.to_string())
            }

            "tasks.list" => {
                let status_filter = params["status"].as_str();
                let tasks = self
                    .storage
                    .list_tasks(status_filter)
                    .map_err(|e| format!("Storage error: {}", e))?;

                serde_json::to_value(&tasks).map_err(|e| e.to_string())
            }

            "tasks.update" => {
                let id = params["id"].as_str().ok_or("Missing required field 'id'")?;
                let status = params["status"]
                    .as_str()
                    .ok_or("Missing required field 'status'")?;
                let progress = params["progress"].as_f64().unwrap_or(0.0);
                let is_completed = status == "succeeded" || status == "failed";

                let updated = self
                    .storage
                    .update_task_status(id, status, progress, is_completed)
                    .map_err(|e| format!("Storage error: {}", e))?;

                if updated {
                    let _ = self.event_bus.publish(SystemEvent::TaskUpdated {
                        task_id: id.to_string(),
                        status: status.to_string(),
                    });
                }

                Ok(serde_json::json!({ "id": id, "updated": updated, "status": status }))
            }

            "system.info" => {
                let info = serde_json::json!({
                    "os": std::env::consts::OS,
                    "arch": std::env::consts::ARCH,
                    "hime_version": env!("CARGO_PKG_VERSION"),
                    "cores": num_cpus(),
                });
                Ok(info)
            }

            "files.stat" => {
                let path_str = params["path"]
                    .as_str()
                    .ok_or("Missing required field 'path'")?;

                self.policy
                    .validate_path_safety(path_str)
                    .map_err(|e| format!("Security validation rejected path: {}", e))?;

                let path = std::path::Path::new(path_str);
                let exists = path.exists();
                let is_dir = path.is_dir();
                let size = if exists {
                    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
                } else {
                    0
                };

                Ok(serde_json::json!({
                    "path": path_str,
                    "exists": exists,
                    "is_dir": is_dir,
                    "size_bytes": size,
                }))
            }

            "notify.send" => {
                let title = params["title"].as_str().unwrap_or("Hime Notification");
                let message = params["message"]
                    .as_str()
                    .ok_or("Missing required field 'message'")?;
                let level = params["level"].as_str().unwrap_or("info");

                info!(title = %title, message = %message, level = %level, "User notification emitted");
                Ok(serde_json::json!({ "dispatched": true, "title": title, "level": level }))
            }

            "translate.selection" => {
                let text = params["text"]
                    .as_str()
                    .ok_or("Missing required field 'text'")?;
                let target = params["target_lang"].as_str().unwrap_or("id");

                // Lightweight Indonesian translation or echo stub
                let translated = translate_text_mock_or_rules(text, target);
                Ok(serde_json::json!({
                    "original": text,
                    "target_lang": target,
                    "translated": translated
                }))
            }

            "storage.search_files" => {
                let query = params["query"].as_str().ok_or("Missing required field 'query'")?;
                let limit = params["limit"].as_u64().unwrap_or(20) as usize;
                let files = self.storage.search_indexed_files(query, limit).map_err(|e| e.to_string())?;
                serde_json::to_value(&files).map_err(|e| e.to_string())
            }

            "storage.inspect_file" => {
                let path = params["path"].as_str().ok_or("Missing required field 'path'")?;
                self.policy.validate_path_safety(path).map_err(|e| format!("Security validation rejected path: {}", e))?;
                let file = self.storage.get_indexed_file(path).map_err(|e| e.to_string())?;
                serde_json::to_value(&file).map_err(|e| e.to_string())
            }

            "storage.analyze" => {
                let top_n = params["top_n"].as_u64().unwrap_or(10) as usize;
                let analysis = self.storage.get_storage_analysis(top_n).map_err(|e| e.to_string())?;
                serde_json::to_value(&analysis).map_err(|e| e.to_string())
            }

            "storage.find_duplicates" => {
                let duplicates = self.storage.find_duplicate_candidates().map_err(|e| e.to_string())?;
                serde_json::to_value(&duplicates).map_err(|e| e.to_string())
            }

            "storage.propose_move" => {
                let source_path = params["source_path"].as_str().ok_or("Missing required field 'source_path'")?;
                let target_dir = params["target_dir"].as_str().ok_or("Missing required field 'target_dir'")?;
                let template = params["naming_template"].as_str();

                self.policy.validate_path_safety(source_path).map_err(|e| format!("Security validation rejected path: {}", e))?;
                self.policy.validate_path_safety(target_dir).map_err(|e| format!("Security validation rejected path: {}", e))?;

                let src_p = std::path::Path::new(source_path);
                let fname = src_p.file_name().unwrap_or_default().to_string_lossy().to_string();
                let dest_name = if let Some(tmpl) = template {
                    let stem = src_p.file_stem().unwrap_or_default().to_string_lossy();
                    let ext = src_p.extension().unwrap_or_default().to_string_lossy();
                    tmpl.replace("{name}", &stem).replace("{ext}", &ext)
                } else {
                    fname
                };
                let dest_p = std::path::Path::new(target_dir).join(dest_name);

                Ok(serde_json::json!({
                    "proposal_id": format!("prop-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()),
                    "source_path": source_path,
                    "suggested_destination": dest_p.to_string_lossy(),
                    "reason": "Rule-based organization proposal"
                }))
            }

            "storage.execute_move" => {
                let source_path = params["source_path"].as_str().ok_or("Missing required field 'source_path'")?;
                let dest_path = params["destination_path"].as_str().ok_or("Missing required field 'destination_path'")?;
                let op_id = format!("op-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

                let result = self.ops_engine.safe_move(
                    &op_id,
                    std::path::Path::new(source_path),
                    std::path::Path::new(dest_path),
                    client_id,
                    None,
                ).map_err(|e| e.to_string())?;

                serde_json::to_value(&result).map_err(|e| e.to_string())
            }

            "storage.delete_file" => {
                let path = params["path"].as_str().ok_or("Missing required field 'path'")?;
                let op_id = format!("del-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

                let result = self.ops_engine.safe_delete(
                    &op_id,
                    std::path::Path::new(path),
                    client_id,
                    None,
                ).map_err(|e| e.to_string())?;

                serde_json::to_value(&result).map_err(|e| e.to_string())
            }

            "storage.restore_file" => {
                let original_op_id = params["operation_id"].as_str().ok_or("Missing required field 'operation_id'")?;
                let restore_op_id = format!("res-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

                let result = self.ops_engine.restore_file(
                    &restore_op_id,
                    original_op_id,
                    client_id,
                ).map_err(|e| e.to_string())?;

                serde_json::to_value(&result).map_err(|e| e.to_string())
            }

            "storage.handle_new_download" => {
                let download_path = params["download_path"].as_str().ok_or("Missing required field 'download_path'")?;
                let download_intel = crate::storage::download::DownloadIntelligence::new(self.storage.clone(), self.ops_engine.clone());
                let report = download_intel.inspect_download(std::path::Path::new(download_path)).map_err(|e| e.to_string())?;
                serde_json::to_value(&report).map_err(|e| e.to_string())
            }

            _ => Err(format!("Unknown tool handler '{}'", tool_id)),
        }
    }

    fn sanitize_json_value(&self, val: serde_json::Value) -> serde_json::Value {
        match val {
            serde_json::Value::String(s) => {
                let no_cot = redact_chain_of_thought(&s);
                let redacted = redact_api_keys(&no_cot);
                serde_json::Value::String(redacted)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(|v| self.sanitize_json_value(v)).collect())
            }
            serde_json::Value::Object(map) => {
                let mut sanitized_map = serde_json::Map::new();
                for (k, v) in map {
                    sanitized_map.insert(k, self.sanitize_json_value(v));
                }
                serde_json::Value::Object(sanitized_map)
            }
            other => other,
        }
    }
}

fn uuid_simple() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", now)
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

fn translate_text_mock_or_rules(text: &str, target_lang: &str) -> String {
    if target_lang == "id" {
        // Simple demonstrative dictionary transformations for smoke tests
        match text.trim().to_lowercase().as_str() {
            "hello" => "Halo".to_string(),
            "world" => "Dunia".to_string(),
            "good morning" => "Selamat pagi".to_string(),
            "thank you" => "Terima kasih".to_string(),
            _ => format!("[ID: {}]", text),
        }
    } else {
        format!("[{}: {}]", target_lang, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_gateway_registration_and_execution() {
        let storage = Arc::new(Database::open_in_memory().unwrap());
        let policy = Arc::new(PolicyEngine::new());
        let event_bus = EventBus::new(128);
        let gateway = ToolGateway::new(storage, policy, event_bus);

        // Allow background registration task to finish
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let tools = gateway.list_tools().await;
        assert!(tools.len() >= 6);

        // Execute system.info (L0 ReadOnly)
        let res = gateway
            .execute_tool(
                "test_client",
                "system.info",
                serde_json::json!({}),
                None,
                Some("idemp-sysinfo-1"),
            )
            .await
            .expect("tool execution ok");

        assert!(res.success);
        assert_eq!(res.tool_id, "system.info");
        assert!(res.output.get("os").is_some());

        // Idempotency: execute with same key should return cached: true
        let cached_res = gateway
            .execute_tool(
                "test_client",
                "system.info",
                serde_json::json!({}),
                None,
                Some("idemp-sysinfo-1"),
            )
            .await
            .unwrap();

        assert!(cached_res.cached);
    }

    #[tokio::test]
    async fn test_tool_gateway_tasks_lifecycle() {
        let storage = Arc::new(Database::open_in_memory().unwrap());
        let policy = Arc::new(PolicyEngine::new());
        let event_bus = EventBus::new(128);
        let gateway = ToolGateway::new(storage, policy, event_bus);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Create task
        let create_res = gateway
            .execute_tool(
                "test_client",
                "tasks.create",
                serde_json::json!({
                    "title": "Write Unit Tests",
                    "description": "Ensure high code coverage",
                    "priority": "high"
                }),
                None,
                None,
            )
            .await
            .unwrap();

        assert!(create_res.success);
        let task_id = create_res.output["id"].as_str().unwrap().to_string();

        // Update task
        let update_res = gateway
            .execute_tool(
                "test_client",
                "tasks.update",
                serde_json::json!({
                    "id": task_id,
                    "status": "succeeded",
                    "progress": 1.0
                }),
                None,
                None,
            )
            .await
            .unwrap();

        assert!(update_res.success);

        // List tasks
        let list_res = gateway
            .execute_tool(
                "test_client",
                "tasks.list",
                serde_json::json!({ "status": "succeeded" }),
                None,
                None,
            )
            .await
            .unwrap();

        assert!(list_res.success);
        let arr = list_res.output.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["title"], "Write Unit Tests");
    }

    #[tokio::test]
    async fn test_tool_gateway_files_stat_security_traversal_blocked() {
        let storage = Arc::new(Database::open_in_memory().unwrap());
        let policy = Arc::new(PolicyEngine::new());
        let event_bus = EventBus::new(128);
        let gateway = ToolGateway::new(storage, policy, event_bus);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Directory traversal attempt
        let res = gateway
            .execute_tool(
                "test_client",
                "files.stat",
                serde_json::json!({ "path": "docs/../../Windows/System32" }),
                None,
                None,
            )
            .await
            .unwrap();

        assert!(!res.success);
        assert!(res.error.unwrap().contains("Security validation rejected"));
    }
}
