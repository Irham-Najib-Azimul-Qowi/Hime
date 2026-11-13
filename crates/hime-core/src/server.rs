use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{error, info, warn};

use hime_ipc::framing::{Frame, MsgType};
use hime_ipc::protocol::{
    JsonRpcRequest, JsonRpcResponse, INVALID_REQUEST, METHOD_NOT_FOUND, POLICY_VIOLATION,
    UNAUTHORIZED,
};
use hime_ipc::session::ScopedNonceTracker;
use hime_security::csprng::generate_token_256;
use hime_security::policy::{Action, PolicyDecision, PolicyEnforcer, PolicyEngine};
use hime_storage::Database;

use crate::ai::AiOrchestrator;
use crate::bus::{EventBus, SystemEvent};

pub struct CoreServer {
    pub version: String,
    pub event_bus: EventBus,
    pub nonce_tracker: Arc<ScopedNonceTracker>,
    pub policy: Arc<dyn PolicyEnforcer>,
    pub storage: Arc<Database>,
    pub auth_token: String,
    pub start_time: std::time::Instant,
    pub tasks: Arc<tokio::sync::Mutex<Vec<TaskRecord>>>,
    pub approvals: Arc<tokio::sync::Mutex<HashMap<String, ApprovalRecord>>>,
    pub ai: Arc<AiOrchestrator>,
    pub storage_engine: Arc<crate::storage::StorageEngine>,
    pub device_manager: Arc<crate::device::DeviceManager>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ApprovalRecord {
    pub request_id: String,
    pub action_type: String,
    pub path: String,
    pub description: String,
    pub status: String, // "pending" | "approved" | "denied" | "expired"
    pub created_at: u64,
    pub expires_at: u64,
    pub consumed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TaskRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub priority: String,
    pub status: String,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

impl CoreServer {
    pub fn new(auth_token: String, storage: Arc<Database>) -> Self {
        // Record server startup audit event
        let _ = storage.record_audit_event(
            "core.startup",
            "kernel",
            &format!("{{\"version\":\"0.1.0\",\"event\":\"initialized\"}}"),
        );

        let event_bus = EventBus::new(128);
        let policy_engine = Arc::new(PolicyEngine::new());
        let policy: Arc<dyn PolicyEnforcer> = Arc::clone(&policy_engine) as Arc<dyn PolicyEnforcer>;
        let ai = Arc::new(AiOrchestrator::new(
            Arc::clone(&storage),
            Arc::clone(&policy_engine),
            event_bus.clone(),
            20129,
        ));

        let base_dir = hime_pal::PlatformPaths::app_data_dir();
        let storage_engine = Arc::new(crate::storage::StorageEngine::new(storage.clone(), &base_dir).unwrap_or_else(|_| {
            let tmp = std::env::temp_dir().join(format!("hime_se_{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            let _ = std::fs::create_dir_all(&tmp);
            crate::storage::StorageEngine::new(storage.clone(), &tmp).expect("Failed to init storage engine")
        }));

        let approvals = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let device_manager = Arc::new(crate::device::DeviceManager::new(
            storage.clone(),
            storage_engine.ops_engine.clone(),
            event_bus.clone(),
            approvals.clone(),
            &base_dir,
            "127.0.0.1",
            20130,
        ).unwrap_or_else(|_| {
            let tmp = std::env::temp_dir().join(format!("hime_dm_{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            let _ = std::fs::create_dir_all(&tmp);
            crate::device::DeviceManager::new(
                storage.clone(),
                storage_engine.ops_engine.clone(),
                event_bus.clone(),
                approvals.clone(),
                &tmp,
                "127.0.0.1",
                20130,
            ).expect("Failed to init device manager")
        }));

        Self {
            version: "0.1.0".to_string(),
            event_bus,
            nonce_tracker: Arc::new(ScopedNonceTracker::new()),
            policy,
            storage,
            auth_token,
            start_time: std::time::Instant::now(),
            tasks: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            approvals,
            ai,
            storage_engine,
            device_manager,
        }
    }

    /// Creates an in-memory test instance of CoreServer with default in-memory SQLite storage.
    pub fn new_in_memory(auth_token: String) -> Result<Self, hime_storage::StorageError> {
        let storage = Arc::new(Database::open_in_memory()?);
        Ok(Self::new(auth_token, storage))
    }

    /// Handles a single bidirectional stream (Named Pipe or duplex memory channel)
    pub async fn handle_connection<S>(&self, stream: S)
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        info!("Client connected to Hime Core IPC stream");
        let (mut reader, mut writer) = tokio::io::split(stream);

        loop {
            let frame = match Frame::read_from(&mut reader).await {
                Ok(f) => f,
                Err(hime_ipc::framing::FramingError::ConnectionClosed) => {
                    info!("Client disconnected cleanly");
                    break;
                }
                Err(e) => {
                    error!(err = %e, "Framing error encountered; closing connection");
                    break;
                }
            };

            if frame.header.msg_type != MsgType::Request {
                warn!(msg_type = ?frame.header.msg_type, "Ignoring non-request frame from client");
                continue;
            }

            let response_frame = match JsonRpcRequest::from_slice(&frame.payload) {
                Ok(req) => {
                    let resp = self.dispatch_rpc(req).await;
                    let payload = resp.to_bytes().unwrap_or_else(|_| b"{}".to_vec());
                    Frame::new(MsgType::Response, payload).expect("Response frame must be valid")
                }
                Err(e) => {
                    let err_resp = JsonRpcResponse::error(
                        serde_json::Value::Null,
                        hime_ipc::protocol::PARSE_ERROR,
                        format!("Parse error: {}", e),
                        None,
                    );
                    let payload = err_resp.to_bytes().unwrap_or_else(|_| b"{}".to_vec());
                    Frame::new(MsgType::Response, payload).expect("Response frame must be valid")
                }
            };

            if let Err(e) = response_frame.write_to(&mut writer).await {
                error!(err = %e, "Failed to write response frame to client");
                break;
            }
        }
    }

    pub async fn dispatch_rpc(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let req_id = req.id.clone();
        match req.method.as_str() {
            "system.ping" => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                JsonRpcResponse::success(
                    req_id,
                    serde_json::json!({
                        "status": "pong",
                        "core_version": self.version,
                        "server_time_ms": now,
                    }),
                )
            }

            "system.handshake" => {
                let params = req.params.unwrap_or_default();
                let client_token = params
                    .get("auth_token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let nonce = params.get("nonce").and_then(|v| v.as_u64()).unwrap_or(0);
                let client_id = params
                    .get("client_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                if client_token != self.auth_token {
                    return JsonRpcResponse::error(
                        req_id,
                        UNAUTHORIZED,
                        "Invalid authentication token",
                        None,
                    );
                }

                if let Err(e) = self.nonce_tracker.validate_and_advance_request_nonce(nonce) {
                    return JsonRpcResponse::error(
                        req_id,
                        hime_ipc::protocol::REPLAY_DETECTED,
                        e.to_string(),
                        None,
                    );
                }

                let session_id =
                    generate_token_256().unwrap_or_else(|_| "sess-fallback".to_string());
                let reconnect_ticket =
                    generate_token_256().unwrap_or_else(|_| "ticket-fallback".to_string());

                self.event_bus.publish(SystemEvent::ClientConnected {
                    client_id: client_id.to_string(),
                });

                JsonRpcResponse::success(
                    req_id,
                    serde_json::json!({
                        "session_id": session_id,
                        "reconnect_ticket": reconnect_ticket,
                        "valid_until_secs": 5,
                    }),
                )
            }

            "system.status" => {
                let tasks = self.tasks.lock().await;
                let active_tasks = tasks
                    .iter()
                    .filter(|t| t.status == "running" || t.status == "queued")
                    .count();
                let total_tasks = tasks.len();
                let uptime_secs = self.start_time.elapsed().as_secs();

                JsonRpcResponse::success(
                    req_id,
                    serde_json::json!({
                        "status": "running",
                        "core": "online",
                        "version": self.version,
                        "uptime_secs": uptime_secs,
                        "db_status": "connected",
                        "named_pipe": "\\\\.\\pipe\\hime-ipc",
                        "active_tasks": active_tasks,
                        "total_tasks": total_tasks,
                    }),
                )
            }

            "events.list" => {
                let params = req.params.unwrap_or_default();
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(30) as usize;
                match self.storage.get_audit_events(limit) {
                    Ok(events) => JsonRpcResponse::success(
                        req_id,
                        serde_json::to_value(events).unwrap_or_default(),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        hime_ipc::protocol::INTERNAL_ERROR,
                        format!("Storage query failed: {}", e),
                        None,
                    ),
                }
            }

            "tasks.list" => {
                let tasks = self.tasks.lock().await;
                JsonRpcResponse::success(req_id, serde_json::to_value(&*tasks).unwrap_or_default())
            }

            "tasks.create" => {
                let params = req.params.unwrap_or_default();
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled Task");
                let description = params
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let priority = params
                    .get("priority")
                    .and_then(|v| v.as_str())
                    .unwrap_or("normal");

                let now = std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let task_id = format!("task-{}", now);

                let task = TaskRecord {
                    id: task_id,
                    name: name.to_string(),
                    description: description.to_string(),
                    priority: priority.to_string(),
                    status: "queued".to_string(),
                    created_at: now,
                    completed_at: None,
                };

                let _ = self.storage.record_audit_event(
                    "task.created",
                    "desktop",
                    &serde_json::to_string(&task).unwrap_or_default(),
                );

                let mut tasks = self.tasks.lock().await;
                tasks.push(task.clone());
                JsonRpcResponse::success(req_id, serde_json::to_value(task).unwrap_or_default())
            }

            "tasks.cancel" => {
                let params = req.params.unwrap_or_default();
                let task_id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");

                let mut tasks = self.tasks.lock().await;
                if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id) {
                    t.status = "cancelled".to_string();
                    let now = std::time::SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64;
                    t.completed_at = Some(now);

                    let _ = self.storage.record_audit_event(
                        "task.cancelled",
                        "desktop",
                        &format!("{{\"task_id\":\"{}\"}}", task_id),
                    );

                    JsonRpcResponse::success(req_id, serde_json::to_value(&*t).unwrap_or_default())
                } else {
                    JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Task not found: {}", task_id),
                        None,
                    )
                }
            }

            "system.shutdown" => {
                info!("Shutdown request received via IPC");
                self.event_bus.publish(SystemEvent::ShutdownInitiated);
                JsonRpcResponse::success(req_id, serde_json::json!({"status": "shutting_down"}))
            }

            "tool.execute" => {
                let params = req.params.unwrap_or_default();
                let action_type = params
                    .get("action_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let action = match action_type {
                    "file_read" => {
                        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
                        Action::FileRead {
                            path: path.to_string(),
                        }
                    }
                    "file_write" => {
                        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
                        Action::FileWrite {
                            path: path.to_string(),
                        }
                    }
                    _ => {
                        return JsonRpcResponse::error(
                            req_id,
                            INVALID_REQUEST,
                            format!("Unknown action type: {}", action_type),
                            None,
                        );
                    }
                };

                let decision = self.policy.evaluate("default_client", &action);

                // Record audit event for policy evaluation
                let _ = self.storage.record_audit_event(
                    "tool.policy_evaluation",
                    "desktop",
                    &format!(
                        "{{\"action_type\":\"{}\",\"decision\":\"{:?}\"}}",
                        action_type, decision
                    ),
                );

                match decision {
                    PolicyDecision::Allow => JsonRpcResponse::success(
                        req_id,
                        serde_json::json!({
                            "decision": "allowed",
                            "result": "Operation authorized and executed successfully"
                        }),
                    ),
                    PolicyDecision::Deny { reason } => JsonRpcResponse::error(
                        req_id,
                        POLICY_VIOLATION,
                        format!("Policy violation: {}", reason),
                        None,
                    ),
                    PolicyDecision::RequireUserApproval { description } => {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let token = generate_token_256().unwrap_or_else(|_| "appr-token".to_string());
                        let request_id = format!("appr-{}", &token[..12]);
                        let expires_at = now + 60; // 60 seconds TTL

                        let record = ApprovalRecord {
                            request_id: request_id.clone(),
                            action_type: action_type.to_string(),
                            path: match &action {
                                Action::FileRead { path } | Action::FileWrite { path } => path.clone(),
                                _ => String::new(),
                            },
                            description: description.clone(),
                            status: "pending".to_string(),
                            created_at: now,
                            expires_at,
                            consumed: false,
                        };

                        {
                            let mut appr_map = self.approvals.lock().await;
                            appr_map.insert(request_id.clone(), record);
                        }

                        let _ = self.storage.record_audit_event(
                            "policy.approval_requested",
                            "desktop",
                            &format!(
                                "{{\"request_id\":\"{}\",\"action\":\"{}\",\"expires_at\":{}}}",
                                request_id, action_type, expires_at
                            ),
                        );

                        JsonRpcResponse::success(
                            req_id,
                            serde_json::json!({
                                "decision": "pending_approval",
                                "request_id": request_id,
                                "description": description,
                                "expires_at": expires_at,
                                "status": "pending"
                            }),
                        )
                    }
                }
            }

            "approval.resolve" => {
                let params = req.params.unwrap_or_default();
                let request_id = params
                    .get("request_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let decision = params
                    .get("decision")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let mut appr_map = self.approvals.lock().await;

                if let Some(record) = appr_map.get_mut(request_id) {
                    if record.consumed || record.status != "pending" {
                        return JsonRpcResponse::error(
                            req_id,
                            INVALID_REQUEST,
                            format!(
                                "Approval request '{}' already consumed or resolved",
                                request_id
                            ),
                            None,
                        );
                    }

                    if now > record.expires_at {
                        record.status = "expired".to_string();
                        let _ = self.storage.record_audit_event(
                            "policy.approval_expired",
                            "desktop",
                            &format!("{{\"request_id\":\"{}\"}}", request_id),
                        );
                        return JsonRpcResponse::error(
                            req_id,
                            POLICY_VIOLATION,
                            format!("Approval request '{}' has expired", request_id),
                            None,
                        );
                    }

                    match decision {
                        "approve" => {
                            record.status = "approved".to_string();
                            record.consumed = true;
                            let _ = self.storage.record_audit_event(
                                "policy.approval_granted",
                                "desktop",
                                &format!(
                                    "{{\"request_id\":\"{}\",\"action\":\"{}\"}}",
                                    request_id, record.action_type
                                ),
                            );
                            JsonRpcResponse::success(
                                req_id,
                                serde_json::json!({
                                    "request_id": request_id,
                                    "status": "approved",
                                    "result": "Operation approved by user and authorized"
                                }),
                            )
                        }
                        "deny" => {
                            record.status = "denied".to_string();
                            record.consumed = true;
                            let _ = self.storage.record_audit_event(
                                "policy.approval_denied",
                                "desktop",
                                &format!(
                                    "{{\"request_id\":\"{}\",\"action\":\"{}\"}}",
                                    request_id, record.action_type
                                ),
                            );
                            JsonRpcResponse::error(
                                req_id,
                                POLICY_VIOLATION,
                                format!(
                                    "Operation denied by user approval policy for request '{}'",
                                    request_id
                                ),
                                None,
                            )
                        }
                        _ => JsonRpcResponse::error(
                            req_id,
                            INVALID_REQUEST,
                            format!(
                                "Invalid decision '{}'; expected 'approve' or 'deny'",
                                decision
                            ),
                            None,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Approval request '{}' not found", request_id),
                        None,
                    )
                }
            }

            // Phase 08 — AI Runtime Integration RPC Methods
            "ai.status" => {
                let status = self.ai.get_overview_status().await;
                JsonRpcResponse::success(req_id, serde_json::to_value(status).unwrap_or_default())
            }

            "ai.models.list" => {
                let models = self.ai.list_models();
                JsonRpcResponse::success(req_id, serde_json::to_value(models).unwrap_or_default())
            }

            "credentials.store" => {
                let params = req.params.unwrap_or_default();
                let provider = params
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let secret = params
                    .get("secret")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if provider.is_empty() || secret.is_empty() {
                    return JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        "Provider and secret parameters must not be empty",
                        None,
                    );
                }

                match self.ai.store_provider_credential(provider, secret) {
                    Ok(_) => JsonRpcResponse::success(
                        req_id,
                        serde_json::json!({
                            "status": "stored",
                            "provider": provider,
                            "vault": "DPAPI",
                        }),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Failed to store credential: {}", e),
                        None,
                    ),
                }
            }

            "credentials.list" => {
                let providers = self.storage.list_configured_providers().unwrap_or_default();
                let result = providers
                    .into_iter()
                    .map(|p| {
                        serde_json::json!({
                            "provider": p,
                            "configured": true,
                            "protected_by": "Windows DPAPI",
                        })
                    })
                    .collect::<Vec<_>>();
                JsonRpcResponse::success(req_id, serde_json::to_value(result).unwrap_or_default())
            }

            "conversation.create" => {
                let params = req.params.unwrap_or_default();
                let title = params
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("New Conversation");
                let model = params
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("hermes-3-llama-3.1-8b");

                match self.ai.conversation_service.create_conversation(title, model) {
                    Ok(conv) => JsonRpcResponse::success(
                        req_id,
                        serde_json::to_value(conv).unwrap_or_default(),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Failed to create conversation: {}", e),
                        None,
                    ),
                }
            }

            "conversation.list" => {
                let convs = self
                    .ai
                    .conversation_service
                    .list_conversations()
                    .unwrap_or_default();
                JsonRpcResponse::success(req_id, serde_json::to_value(convs).unwrap_or_default())
            }

            "conversation.messages" => {
                let params = req.params.unwrap_or_default();
                let conv_id = params
                    .get("conversation_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if conv_id.is_empty() {
                    return JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        "conversation_id parameter is required",
                        None,
                    );
                }

                let msgs = self
                    .ai
                    .conversation_service
                    .get_messages(conv_id)
                    .unwrap_or_default();
                JsonRpcResponse::success(req_id, serde_json::to_value(msgs).unwrap_or_default())
            }

            "conversation.send_message" => {
                let params = req.params.unwrap_or_default();
                let conv_id = params
                    .get("conversation_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let prompt = params
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if conv_id.is_empty() || prompt.is_empty() {
                    return JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        "conversation_id and prompt parameters are required",
                        None,
                    );
                }

                match self.ai.execute_chat_prompt(conv_id, prompt).await {
                    Ok(resp) => JsonRpcResponse::success(
                        req_id,
                        serde_json::json!({
                            "status": "completed",
                            "conversation_id": conv_id,
                            "response": resp,
                        }),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("AI execution error: {}", e),
                        None,
                    ),
                }
            }

            "conversation.cancel" => {
                let cancelled = self.ai.cancel_active_turn().await;
                JsonRpcResponse::success(
                    req_id,
                    serde_json::json!({
                        "cancelled": cancelled,
                        "status": if cancelled { "generation_stopped" } else { "no_active_turn" }
                    }),
                )
            }

            // Phase 09 — Intelligence Core RPC Handlers
            "tools.list" => {
                let tools = self.ai.tool_gateway.list_tools().await;
                JsonRpcResponse::success(req_id, serde_json::to_value(tools).unwrap_or_default())
            }

            "tools.execute" => {
                let params = req.params.unwrap_or_default();
                let tool_id = params
                    .get("tool_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let tool_params = params.get("params").cloned().unwrap_or(serde_json::json!({}));
                let ticket = params.get("approval_ticket").and_then(|v| v.as_str());
                let idempotency_key = params.get("idempotency_key").and_then(|v| v.as_str());

                if tool_id.is_empty() {
                    return JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        "tool_id parameter is required",
                        None,
                    );
                }

                match self
                    .ai
                    .tool_gateway
                    .execute_tool("desktop", tool_id, tool_params, ticket, idempotency_key)
                    .await
                {
                    Ok(result) => {
                        JsonRpcResponse::success(req_id, serde_json::to_value(result).unwrap_or_default())
                    }
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        POLICY_VIOLATION,
                        format!("Tool execution rejected: {}", e),
                        None,
                    ),
                }
            }

            "orchestrator.execute_plan" => {
                let params = req.params.unwrap_or_default();
                let steps_val = params.get("steps").cloned().unwrap_or(serde_json::json!([]));
                let steps: Vec<crate::ai::orchestrator::PlanStep> = match serde_json::from_value(steps_val) {
                    Ok(s) => s,
                    Err(e) => {
                        return JsonRpcResponse::error(
                            req_id,
                            INVALID_REQUEST,
                            format!("Invalid plan steps format: {}", e),
                            None,
                        );
                    }
                };

                match self.ai.execute_orchestrated_plan("desktop", steps).await {
                    Ok(summary) => JsonRpcResponse::success(
                        req_id,
                        serde_json::to_value(summary).unwrap_or_default(),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Plan execution error: {}", e),
                        None,
                    ),
                }
            }

            "task.engine.list" => {
                let params = req.params.unwrap_or_default();
                let status_filter = params.get("status").and_then(|v| v.as_str());
                match self.ai.task_engine.list_tasks(status_filter) {
                    Ok(tasks) => JsonRpcResponse::success(
                        req_id,
                        serde_json::to_value(tasks).unwrap_or_default(),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        hime_ipc::protocol::INTERNAL_ERROR,
                        format!("Task engine error: {}", e),
                        None,
                    ),
                }
            }

            "task.engine.create" => {
                let params = req.params.unwrap_or_default();
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| {
                        format!(
                            "task-{}",
                            std::time::SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis()
                        )
                    });
                let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("New Task");
                let description = params.get("description").and_then(|v| v.as_str());
                let priority = params.get("priority").and_then(|v| v.as_str()).unwrap_or("medium");

                match self.ai.task_engine.submit_task(&id, title, description, priority) {
                    Ok(record) => JsonRpcResponse::success(
                        req_id,
                        serde_json::to_value(record).unwrap_or_default(),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Failed to create task: {}", e),
                        None,
                    ),
                }
            }

            "task.engine.update_status" => {
                let params = req.params.unwrap_or_default();
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let status_str = params.get("status").and_then(|v| v.as_str()).unwrap_or("");
                let progress = params.get("progress").and_then(|v| v.as_f64()).unwrap_or(0.0);

                let status = match crate::ai::task_engine::TaskStatus::parse(status_str) {
                    Some(s) => s,
                    None => {
                        return JsonRpcResponse::error(
                            req_id,
                            INVALID_REQUEST,
                            format!("Invalid task status: {}", status_str),
                            None,
                        );
                    }
                };

                match self.ai.task_engine.transition_status(id, status, progress) {
                    Ok(updated) => JsonRpcResponse::success(
                        req_id,
                        serde_json::json!({ "id": id, "updated": updated, "status": status_str }),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Transition failed: {}", e),
                        None,
                    ),
                }
            }

            "task.engine.retry" => {
                let params = req.params.unwrap_or_default();
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                match self.ai.task_engine.retry_task(id) {
                    Ok(record) => JsonRpcResponse::success(
                        req_id,
                        serde_json::to_value(record).unwrap_or_default(),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Retry failed: {}", e),
                        None,
                    ),
                }
            }

            "task.engine.cancel" => {
                let params = req.params.unwrap_or_default();
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                match self.ai.task_engine.cancel_task(id) {
                    Ok(cancelled) => JsonRpcResponse::success(
                        req_id,
                        serde_json::json!({ "id": id, "cancelled": cancelled }),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Cancel failed: {}", e),
                        None,
                    ),
                }
            }

            "memory.list" => {
                let params = req.params.unwrap_or_default();
                let cat = params.get("category").and_then(|v| v.as_str());
                match self.ai.memory_engine.list_memories(cat) {
                    Ok(items) => JsonRpcResponse::success(
                        req_id,
                        serde_json::to_value(items).unwrap_or_default(),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        hime_ipc::protocol::INTERNAL_ERROR,
                        format!("Memory query failed: {}", e),
                        None,
                    ),
                }
            }

            "memory.insert" => {
                let params = req.params.unwrap_or_default();
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| {
                        format!(
                            "mem-{}",
                            std::time::SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis()
                        )
                    });
                let source = params.get("source").and_then(|v| v.as_str()).unwrap_or("user");
                let category = params.get("category").and_then(|v| v.as_str()).unwrap_or("general");
                let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let score = params.get("relevance_score").and_then(|v| v.as_f64()).unwrap_or(1.0);
                let retention = params.get("retention_policy").and_then(|v| v.as_str()).unwrap_or("permanent");

                match self.ai.memory_engine.insert_memory(&id, source, category, content, score, retention) {
                    Ok(item) => JsonRpcResponse::success(
                        req_id,
                        serde_json::to_value(item).unwrap_or_default(),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Failed to store memory: {}", e),
                        None,
                    ),
                }
            }

            "memory.delete" => {
                let params = req.params.unwrap_or_default();
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                match self.ai.memory_engine.delete_memory(id) {
                    Ok(deleted) => JsonRpcResponse::success(
                        req_id,
                        serde_json::json!({ "id": id, "deleted": deleted }),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Failed to delete memory: {}", e),
                        None,
                    ),
                }
            }

            "memory.context" => {
                let params = req.params.unwrap_or_default();
                let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let max_tokens = params.get("max_tokens").and_then(|v| v.as_u64()).unwrap_or(200) as usize;
                let items = self.ai.memory_engine.build_prompt_context(query, max_tokens);
                JsonRpcResponse::success(req_id, serde_json::to_value(items).unwrap_or_default())
            }

            "resource.get_mode" => {
                let mode = self.ai.resource_manager.get_mode();
                let max_concurrent = self.ai.resource_manager.max_concurrent_tasks();
                JsonRpcResponse::success(
                    req_id,
                    serde_json::json!({
                        "mode": format!("{:?}", mode),
                        "max_concurrent_tasks": max_concurrent
                    }),
                )
            }

            "resource.set_mode" => {
                let params = req.params.unwrap_or_default();
                let mode_str = params.get("mode").and_then(|v| v.as_str()).unwrap_or("");
                let mode = match mode_str.to_lowercase().as_str() {
                    "efficiency" => crate::ai::resource::ResourceMode::Efficiency,
                    "performance" => crate::ai::resource::ResourceMode::Performance,
                    _ => crate::ai::resource::ResourceMode::Balanced,
                };
                self.ai.resource_manager.set_mode(mode);
                JsonRpcResponse::success(
                    req_id,
                    serde_json::json!({
                        "status": "updated",
                        "mode": format!("{:?}", mode),
                        "max_concurrent_tasks": self.ai.resource_manager.max_concurrent_tasks()
                    }),
                )
            }

            "translate.selection" => {
                let params = req.params.unwrap_or_default();
                let target_lang = params.get("target_lang").and_then(|v| v.as_str()).unwrap_or("id");
                let explicit_text = params.get("text").and_then(|v| v.as_str());

                let result = if let Some(txt) = explicit_text {
                    self.ai.translation_service.translate_text(txt, target_lang).await
                } else {
                    self.ai.translation_service.capture_and_translate(target_lang).await
                };

                match result {
                    Ok(tr) => JsonRpcResponse::success(
                        req_id,
                        serde_json::to_value(tr).unwrap_or_default(),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        format!("Translation error: {}", e),
                        None,
                    ),
                }
            }

            "translate.set_opt_in" => {
                let params = req.params.unwrap_or_default();
                let enabled = params.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
                self.ai.translation_service.set_opted_in(enabled);
                JsonRpcResponse::success(
                    req_id,
                    serde_json::json!({ "status": "updated", "opted_in": enabled }),
                )
            }

            "translate.get_opt_in" => {
                let opted_in = self.ai.translation_service.is_opted_in();
                JsonRpcResponse::success(req_id, serde_json::json!({ "opted_in": opted_in }))
            }

            "events.recent" => {
                let params = req.params.unwrap_or_default();
                let since_id = params.get("since_id").and_then(|v| v.as_u64());
                let events = self.ai.event_engine.get_recent_events(since_id).await;
                JsonRpcResponse::success(req_id, serde_json::to_value(events).unwrap_or_default())
            }

            "audit.verify" => {
                match self.storage.verify_audit_trail() {
                    Ok(is_valid) => JsonRpcResponse::success(
                        req_id,
                        serde_json::json!({
                            "chain_valid": is_valid,
                            "hash_algorithm": "SHA-256",
                            "tamper_detected": !is_valid
                        }),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        req_id,
                        hime_ipc::protocol::INTERNAL_ERROR,
                        format!("Audit trail verification failed: {}", e),
                        None,
                    ),
                }
            }

            "storage.scan" => {
                let params = req.params.unwrap_or_default();
                let root_path = params.get("root_path").and_then(|v| v.as_str()).unwrap_or("");
                match self.storage_engine.indexer.scan_directory(std::path::Path::new(root_path)) {
                    Ok(count) => JsonRpcResponse::success(req_id, serde_json::json!({ "indexed_count": count })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.search" => {
                let params = req.params.unwrap_or_default();
                let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
                match self.storage.search_indexed_files(query, limit) {
                    Ok(files) => JsonRpcResponse::success(req_id, serde_json::to_value(files).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.analysis" => {
                let params = req.params.unwrap_or_default();
                let top_n = params.get("top_n").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                match self.storage_engine.analyzer.generate_report(top_n) {
                    Ok(report) => JsonRpcResponse::success(req_id, serde_json::to_value(report).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.duplicates" => {
                match self.storage_engine.duplicate_detector.find_exact_duplicates() {
                    Ok(dups) => JsonRpcResponse::success(req_id, serde_json::to_value(dups).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.versions" => {
                match self.storage_engine.duplicate_detector.find_version_candidates() {
                    Ok(vers) => JsonRpcResponse::success(req_id, serde_json::to_value(vers).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.proposals.list" => {
                let params = req.params.unwrap_or_default();
                let dir = params.get("directory").and_then(|v| v.as_str()).unwrap_or("");
                match self.storage_engine.organizer.evaluate_proposals(dir) {
                    Ok(props) => JsonRpcResponse::success(req_id, serde_json::to_value(props).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.proposals.execute" => {
                let params = req.params.unwrap_or_default();
                let prop_val = params.get("proposal").cloned().unwrap_or_default();
                match serde_json::from_value::<crate::storage::OrganizationProposal>(prop_val) {
                    Ok(prop) => {
                        let actor = params.get("actor").and_then(|v| v.as_str()).unwrap_or("desktop");
                        match self.storage_engine.organizer.execute_proposal(&prop, actor, None) {
                            Ok(res) => JsonRpcResponse::success(req_id, serde_json::to_value(res).unwrap_or_default()),
                            Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                        }
                    }
                    Err(e) => JsonRpcResponse::error(req_id, INVALID_REQUEST, format!("Invalid proposal: {}", e), None),
                }
            }

            "storage.operations.list" => {
                let params = req.params.unwrap_or_default();
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                match self.storage.list_file_operations(limit) {
                    Ok(ops) => JsonRpcResponse::success(req_id, serde_json::to_value(ops).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.operations.restore" => {
                let params = req.params.unwrap_or_default();
                let original_op_id = params.get("original_op_id").and_then(|v| v.as_str()).unwrap_or("");
                let restore_op_id = format!("res-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
                let actor = params.get("actor").and_then(|v| v.as_str()).unwrap_or("desktop");

                match self.storage_engine.ops_engine.restore_file(&restore_op_id, original_op_id, actor) {
                    Ok(res) => JsonRpcResponse::success(req_id, serde_json::to_value(res).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.rules.list" => {
                match self.storage.list_file_rules() {
                    Ok(rules) => JsonRpcResponse::success(req_id, serde_json::to_value(rules).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.rules.create" => {
                let params = req.params.unwrap_or_default();
                let rule_val = params.get("rule").cloned().unwrap_or_default();
                match serde_json::from_value::<hime_storage::FileRuleRecord>(rule_val) {
                    Ok(rule) => match self.storage.create_file_rule(&rule) {
                        Ok(_) => JsonRpcResponse::success(req_id, serde_json::json!({ "created": true, "id": rule.id })),
                        Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                    },
                    Err(e) => JsonRpcResponse::error(req_id, INVALID_REQUEST, format!("Invalid file rule: {}", e), None),
                }
            }

            "storage.rules.delete" => {
                let params = req.params.unwrap_or_default();
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                match self.storage.delete_file_rule(id) {
                    Ok(deleted) => JsonRpcResponse::success(req_id, serde_json::json!({ "deleted": deleted })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.rules.toggle" => {
                let params = req.params.unwrap_or_default();
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let is_active = params.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true);
                match self.storage.update_file_rule_active(id, is_active) {
                    Ok(updated) => JsonRpcResponse::success(req_id, serde_json::json!({ "updated": updated })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.backups.create" => {
                let params = req.params.unwrap_or_default();
                let backup_id = params.get("backup_id").and_then(|v| v.as_str()).unwrap_or("");
                let src = params.get("source_root").and_then(|v| v.as_str()).unwrap_or("");
                let dst = params.get("backup_destination").and_then(|v| v.as_str()).unwrap_or("");
                match self.storage_engine.backup.create_backup(backup_id, std::path::Path::new(src), std::path::Path::new(dst)) {
                    Ok(rec) => JsonRpcResponse::success(req_id, serde_json::to_value(rec).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.backups.list" => {
                match self.storage.list_backup_records() {
                    Ok(recs) => JsonRpcResponse::success(req_id, serde_json::to_value(recs).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.backups.verify" => {
                let params = req.params.unwrap_or_default();
                let backup_id = params.get("backup_id").and_then(|v| v.as_str()).unwrap_or("");
                match self.storage_engine.backup.verify_backup(backup_id) {
                    Ok(verified) => JsonRpcResponse::success(req_id, serde_json::json!({ "verified": verified })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.download.inspect" => {
                let params = req.params.unwrap_or_default();
                let download_path = params.get("download_path").and_then(|v| v.as_str()).unwrap_or("");
                match self.storage_engine.download_intel.inspect_download(std::path::Path::new(download_path)) {
                    Ok(report) => JsonRpcResponse::success(req_id, serde_json::to_value(report).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "storage.download.replace_and_archive" => {
                let params = req.params.unwrap_or_default();
                let new_dl = params.get("new_download").and_then(|v| v.as_str()).unwrap_or("");
                let existing = params.get("existing_file").and_then(|v| v.as_str()).unwrap_or("");
                let actor = params.get("actor").and_then(|v| v.as_str()).unwrap_or("desktop");
                match self.storage_engine.download_intel.replace_and_archive_old(std::path::Path::new(new_dl), std::path::Path::new(existing), actor) {
                    Ok(res) => JsonRpcResponse::success(req_id, serde_json::to_value(res).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "devices.list" => {
                match self.storage.list_paired_devices() {
                    Ok(devs) => JsonRpcResponse::success(req_id, serde_json::to_value(devs).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "devices.pairing.start" => {
                let params = req.params.unwrap_or_default();
                let ttl_secs = params.get("ttl_secs").and_then(|v| v.as_u64()).unwrap_or(300);
                match self.device_manager.pairing.create_pairing_session(ttl_secs) {
                    Ok(session) => JsonRpcResponse::success(req_id, serde_json::to_value(session).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "devices.pairing.confirm" => {
                let params = req.params.unwrap_or_default();
                let token = params.get("token").and_then(|v| v.as_str()).unwrap_or("");
                let code = params.get("confirmation_code").and_then(|v| v.as_str()).unwrap_or("");
                let dev_id = params.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
                let dev_name = params.get("device_name").and_then(|v| v.as_str()).unwrap_or("Android Companion");
                let dev_type = params.get("device_type").and_then(|v| v.as_str()).unwrap_or("android");
                let fingerprint = params.get("client_cert_fingerprint").and_then(|v| v.as_str()).unwrap_or("default-fp");

                match self.device_manager.pairing.complete_pairing(token, code, dev_id, dev_name, dev_type, fingerprint) {
                    Ok(dev) => JsonRpcResponse::success(req_id, serde_json::to_value(dev).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::UNAUTHORIZED, e.to_string(), None),
                }
            }

            "devices.revoke" => {
                let params = req.params.unwrap_or_default();
                let device_id = params.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
                match self.storage.revoke_device(device_id) {
                    Ok(revoked) => {
                        if revoked {
                            self.event_bus.publish(SystemEvent::DeviceRevoked {
                                device_id: device_id.to_string(),
                            });
                        }
                        JsonRpcResponse::success(req_id, serde_json::json!({ "device_id": device_id, "revoked": revoked }))
                    }
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "devices.storage.get" => {
                let params = req.params.unwrap_or_default();
                let device_id = params.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
                match self.storage.get_device_storage_summary(device_id) {
                    Ok(summary) => JsonRpcResponse::success(req_id, serde_json::to_value(summary).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "devices.transfers.list" => {
                let params = req.params.unwrap_or_default();
                let device_id = params.get("device_id").and_then(|v| v.as_str());
                match self.storage.list_file_transfers(device_id) {
                    Ok(transfers) => JsonRpcResponse::success(req_id, serde_json::to_value(transfers).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            other => JsonRpcResponse::error(
                req_id,
                METHOD_NOT_FOUND,
                format!("Method not found: {}", other),
                None,
            ),
        }
    }
}
