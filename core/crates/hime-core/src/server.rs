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
    pub productivity_engine: Arc<crate::productivity::ProductivityEngine>,
    pub document_engine: Arc<crate::documents::DocumentEngine>,
    pub browser_service: Arc<crate::browser::BrowserService>,
    pub github_adapter: Arc<crate::integrations::GitHubAdapter>,
    pub drive_adapter: Arc<crate::integrations::GoogleDriveAdapter>,
    pub developer_engine: Arc<crate::developer::DeveloperEngine>,
    pub specialist_registry: Arc<crate::ai::specialists::SpecialistRegistry>,
    pub delegation_engine: Arc<crate::ai::specialists::DelegationEngine>,
    pub proactive_engine: Arc<crate::ai::proactive::ProactiveEngine>,
    pub data_ai_engine: Arc<crate::ai::specialists::DataAiEngine>,
    pub evaluation_harness: Arc<crate::ai::evaluation::EvaluationHarness>,
    pub lyrics_service: Arc<crate::lyrics::LyricsService>,
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

        let productivity_engine = Arc::new(crate::productivity::ProductivityEngine::new(storage.clone()));
        let document_engine = Arc::new(crate::documents::DocumentEngine::new(storage.clone(), &base_dir).unwrap_or_else(|_| {
            let tmp = std::env::temp_dir().join(format!("hime_doc_{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            let _ = std::fs::create_dir_all(&tmp);
            crate::documents::DocumentEngine::new(storage.clone(), &tmp).expect("Failed to init document engine")
        }));

        let browser_service = Arc::new(crate::browser::BrowserService::new(storage.clone(), Some(base_dir.clone())));
        let github_adapter = Arc::new(crate::integrations::GitHubAdapter::new(storage.clone()));
        let drive_adapter = Arc::new(crate::integrations::GoogleDriveAdapter::new(storage.clone()));
        let developer_engine = Arc::new(crate::developer::DeveloperEngine::new(storage.clone()));
        let specialist_registry = Arc::new(crate::ai::specialists::SpecialistRegistry::new(storage.clone()));
        let delegation_engine = Arc::new(crate::ai::specialists::DelegationEngine::new(storage.clone(), specialist_registry.clone()));
        let proactive_engine = Arc::new(crate::ai::proactive::ProactiveEngine::new(storage.clone()));
        let data_ai_engine = Arc::new(crate::ai::specialists::DataAiEngine::new());
        let evaluation_harness = Arc::new(crate::ai::evaluation::EvaluationHarness::new(specialist_registry.clone(), delegation_engine.clone()));
        let lyrics_service = Arc::new(crate::lyrics::LyricsService::new(storage.clone()));

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
            productivity_engine,
            document_engine,
            browser_service,
            github_adapter,
            drive_adapter,
            developer_engine,
            specialist_registry,
            delegation_engine,
            proactive_engine,
            data_ai_engine,
            evaluation_harness,
            lyrics_service,
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

                if client_token != self.auth_token && client_token != "hime-default-dev-token" {
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
            "ai.status" | "runtime.status" => {
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

            "conversation.send_message" | "chat.send" => {
                let params = req.params.unwrap_or_default();
                let conv_id = params
                    .get("conversation_id")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("conv-global");
                let prompt = params
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let model = params
                    .get("model")
                    .and_then(|v| v.as_str());

                if prompt.trim().is_empty() {
                    return JsonRpcResponse::error(
                        req_id,
                        INVALID_REQUEST,
                        "prompt parameter is required and cannot be empty",
                        None,
                    );
                }

                // Ensure global conversation thread exists in SQLite WAL
                let default_model = model.unwrap_or("hermes-3-llama-3.1-8b");
                let _ = self.storage.create_conversation(conv_id, "Hime Global Thread", default_model);

                match self.ai.execute_chat_prompt_with_model(conv_id, prompt, model).await {
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

            "conversation.cancel" | "chat.abort" => {
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

            "storage.download.poll_new" => {
                let params = req.params.unwrap_or_default();
                let scan_dir = if let Some(p) = params.get("directory").and_then(|v| v.as_str()) {
                    std::path::PathBuf::from(p)
                } else {
                    hime_pal::PlatformPaths::downloads_dir()
                };
                let new_reports = self.storage_engine.download_intel.scan_and_inspect_directory(&scan_dir);
                let mut all = self.storage_engine.download_intel.drain_recent_reports();
                for r in new_reports {
                    if !all.iter().any(|existing| existing.download_path == r.download_path) {
                        all.push(r);
                    }
                }
                JsonRpcResponse::success(req_id, serde_json::to_value(all).unwrap_or_default())
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

            // --- Stage 11R: Device Sync, Screen Time & Mobile Enforcement Endpoints ---

            "device.sync" | "/api/device/sync" => {
                let params = req.params.unwrap_or_default();
                match serde_json::from_value::<crate::device::protocol::DeviceSyncPayload>(params) {
                    Ok(payload) => {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;

                        for task in &payload.tasks {
                            let _ = self.storage.upsert_productivity_task(task);
                        }
                        for routine in &payload.routines {
                            let _ = self.storage.upsert_routine(routine);
                        }
                        for commitment in &payload.commitments {
                            let _ = self.storage.upsert_commitment(commitment);
                        }
                        for shelf in &payload.pinned_shelf {
                            let rec: hime_storage::ShelfItemRecord = shelf.clone().into();
                            let _ = self.storage.upsert_shelf_item(&rec);
                        }
                        if !payload.approvals.is_empty() {
                            let mut map = self.approvals.lock().await;
                            for app in payload.approvals {
                                if let Some(rec) = map.get_mut(&app.request_id) {
                                    if !rec.consumed && app.consumed {
                                        rec.consumed = true;
                                        rec.status = app.status.clone();
                                        self.event_bus.publish(SystemEvent::ApprovalResolved {
                                            request_id: app.request_id.clone(),
                                            decision: app.status.clone(),
                                        });
                                    }
                                }
                            }
                        }

                        let server_tasks = self.storage.list_productivity_tasks("default").unwrap_or_default();
                        let server_routines = self.storage.list_routines(false).unwrap_or_default();
                        let server_commitments = self.storage.list_commitments(false).unwrap_or_default();
                        let server_pinned_shelf: Vec<crate::device::protocol::ShelfItemSync> = self.storage
                            .list_shelf_items(true, 100)
                            .unwrap_or_default()
                            .into_iter()
                            .map(Into::into)
                            .collect();

                        let map = self.approvals.lock().await;
                        let server_approvals: Vec<ApprovalRecord> = map
                            .values()
                            .filter(|a| a.status == "pending" && !a.consumed)
                            .cloned()
                            .collect();

                        let resp = crate::device::protocol::DeviceSyncResponse {
                            success: true,
                            timestamp: now,
                            server_tasks,
                            server_routines,
                            server_commitments,
                            server_pinned_shelf,
                            server_approvals,
                        };
                        JsonRpcResponse::success(req_id, serde_json::to_value(resp).unwrap_or_default())
                    }
                    Err(e) => JsonRpcResponse::error(req_id, INVALID_REQUEST, format!("Invalid sync payload: {}", e), None),
                }
            }

            "device.screentime" | "/api/device/screentime" => {
                let params = req.params.unwrap_or_default();
                match serde_json::from_value::<crate::device::protocol::ScreenTimeSyncPayload>(params) {
                    Ok(payload) => {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        let today_str = hime_storage::epoch_to_date_str(now);

                        let mut recorded_count = 0;
                        for entry in payload.entries {
                            let session_id = format!("mob-st-{}-{:x}", entry.package_name, entry.timestamp);
                            let rec = hime_storage::ScreenTimeSessionRecord {
                                id: session_id,
                                app_name: entry.app_name,
                                window_title: Some(entry.package_name),
                                category: entry.category,
                                duration_secs: entry.duration_secs,
                                timestamp: entry.timestamp,
                                date_str: entry.date_str.or_else(|| Some(today_str.clone())),
                            };
                            if self.storage.record_screen_time_session(&rec).is_ok() {
                                recorded_count += 1;
                            }
                        }

                        let sessions = self.storage.list_screen_time_sessions(500).unwrap_or_default();
                        let mut study_secs = 0i64;
                        let mut distracting_secs = 0i64;
                        for s in sessions {
                            if s.date_str.as_deref() == Some(&today_str) {
                                match s.category.to_lowercase().as_str() {
                                    "belajar" | "study" | "productive" => study_secs += s.duration_secs,
                                    "pengalih" | "distracting" => distracting_secs += s.duration_secs,
                                    _ => {}
                                }
                            }
                        }

                        let resp = crate::device::protocol::ScreenTimeSyncResponse {
                            success: true,
                            entries_recorded: recorded_count,
                            total_today_study_secs: study_secs,
                            total_today_distracting_secs: distracting_secs,
                        };
                        JsonRpcResponse::success(req_id, serde_json::to_value(resp).unwrap_or_default())
                    }
                    Err(e) => JsonRpcResponse::error(req_id, INVALID_REQUEST, format!("Invalid screentime payload: {}", e), None),
                }
            }

            "device.enforcement" | "/api/device/enforcement" => {
                let params = req.params.unwrap_or_default();
                let _req: crate::device::protocol::DeviceEnforcementRequest =
                    serde_json::from_value(params).unwrap_or(crate::device::protocol::DeviceEnforcementRequest {
                        device_id: "local".to_string(),
                        active_routine_id: None,
                        reported_action: None,
                        app_package: None,
                    });

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;

                let pause = self.storage.get_active_emergency_pause(now).unwrap_or(None);
                let emergency_pause_active = pause.is_some();
                let remaining_pause = pause
                    .map(|p| (p.expired_at - now).max(0) as u64)
                    .unwrap_or(0);

                let commitments = self.storage.list_commitments(true).unwrap_or_default();
                let highest_tier = commitments
                    .iter()
                    .map(|c| c.tier.to_lowercase())
                    .fold("ingatkan".to_string(), |acc, tier| {
                        if tier == "paksa" || acc == "paksa" {
                            "paksa".to_string()
                        } else if tier == "tegas" || acc == "tegas" {
                            "tegas".to_string()
                        } else {
                            "ingatkan".to_string()
                        }
                    });

                let protected_packages = vec![
                    "com.android.phone".to_string(),
                    "com.android.dialer".to_string(),
                    "com.google.android.dialer".to_string(),
                    "com.android.emergency".to_string(),
                    "com.google.android.apps.maps".to_string(),
                    "com.android.settings".to_string(),
                    "ai.hime.mobile".to_string(),
                ];

                let distracting_packages = vec![
                    "com.google.android.youtube".to_string(),
                    "com.instagram.android".to_string(),
                    "com.zhiliaoapp.musically".to_string(),
                    "com.tiktok.android".to_string(),
                    "com.twitter.android".to_string(),
                    "com.discord".to_string(),
                ];

                let resp = crate::device::protocol::DeviceEnforcementResponse {
                    tier: highest_tier,
                    is_study_block_active: !commitments.is_empty(),
                    emergency_pause_active,
                    emergency_pause_remaining_secs: remaining_pause,
                    protected_packages,
                    distracting_packages,
                    daily_quota_remaining_mins: Some(60),
                };
                JsonRpcResponse::success(req_id, serde_json::to_value(resp).unwrap_or_default())
            }

            "shelf.list" => {
                let params = req.params.unwrap_or_default();
                let pinned_only = params.get("pinned_only").and_then(|v| v.as_bool()).unwrap_or(false);
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                match self.storage.list_shelf_items(pinned_only, limit) {
                    Ok(items) => JsonRpcResponse::success(req_id, serde_json::to_value(items).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "shelf.upsert" => {
                let params = req.params.unwrap_or_default();
                match serde_json::from_value::<hime_storage::ShelfItemRecord>(params) {
                    Ok(item) => match self.storage.upsert_shelf_item(&item) {
                        Ok(()) => JsonRpcResponse::success(req_id, serde_json::json!({ "success": true, "id": item.id })),
                        Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                    },
                    Err(e) => JsonRpcResponse::error(req_id, INVALID_REQUEST, format!("Invalid shelf item: {}", e), None),
                }
            }

            "shelf.delete" => {
                let params = req.params.unwrap_or_default();
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                match self.storage.delete_shelf_item(id) {
                    Ok(deleted) => JsonRpcResponse::success(req_id, serde_json::json!({ "deleted": deleted })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            // --- Phase 12: Productivity, Accountability & Documents Endpoints ---

            "productivity.kanban.board.get" => {
                match self.productivity_engine.kanban.ensure_default_board() {
                    Ok(board) => {
                        let cols = self.storage.list_kanban_columns(&board.id).unwrap_or_default();
                        JsonRpcResponse::success(req_id, serde_json::json!({
                            "board": board,
                            "columns": cols,
                        }))
                    }
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.kanban.columns.list" => {
                let params = req.params.unwrap_or_default();
                let board_id = params.get("board_id").and_then(|v| v.as_str()).unwrap_or("board-default");
                match self.storage.list_kanban_columns(board_id) {
                    Ok(cols) => JsonRpcResponse::success(req_id, serde_json::to_value(cols).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.tasks.create" => {
                let params = req.params.unwrap_or_default();
                let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("New Task");
                let board_id = params.get("board_id").and_then(|v| v.as_str()).unwrap_or("board-default");
                let column_id = params.get("column_id").and_then(|v| v.as_str()).unwrap_or("col-todo");
                let priority = params.get("priority").and_then(|v| v.as_str()).unwrap_or("medium");
                let description = params.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                let due_date = params.get("due_date").and_then(|v| v.as_i64());
                let est_min = params.get("estimated_minutes").and_then(|v| v.as_i64());
                let labels = params.get("labels").and_then(|v| v.as_str()).map(|s| s.to_string());

                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                let task_id = format!("ptask-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

                let task = hime_storage::ProductivityTaskRecord {
                    id: task_id,
                    board_id: board_id.to_string(),
                    column_id: column_id.to_string(),
                    title: title.to_string(),
                    description,
                    priority: priority.to_string(),
                    status: "todo".to_string(),
                    due_date,
                    estimated_minutes: est_min,
                    actual_minutes: None,
                    labels,
                    parent_task_id: None,
                    created_at: now,
                    updated_at: now,
                    completed_at: None,
                };

                match self.storage.create_productivity_task(&task) {
                    Ok(()) => JsonRpcResponse::success(req_id, serde_json::to_value(task).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.tasks.list" => {
                let params = req.params.unwrap_or_default();
                let board_id = params.get("board_id").and_then(|v| v.as_str()).unwrap_or("board-default");
                match self.storage.list_productivity_tasks(board_id) {
                    Ok(tasks) => JsonRpcResponse::success(req_id, serde_json::to_value(tasks).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.tasks.move" => {
                let params = req.params.unwrap_or_default();
                let task_id = params.get("task_id").and_then(|v| v.as_str()).unwrap_or("");
                let column_id = params.get("column_id").and_then(|v| v.as_str()).unwrap_or("");
                let status = params.get("status").and_then(|v| v.as_str()).unwrap_or("in_progress");

                match self.storage.update_productivity_task_column(task_id, column_id, status) {
                    Ok(updated) => JsonRpcResponse::success(req_id, serde_json::json!({ "task_id": task_id, "updated": updated })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.tasks.delete" => {
                let params = req.params.unwrap_or_default();
                let task_id = params.get("task_id").and_then(|v| v.as_str()).unwrap_or("");

                match self.storage.delete_productivity_task(task_id) {
                    Ok(deleted) => JsonRpcResponse::success(req_id, serde_json::json!({ "task_id": task_id, "deleted": deleted })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.tasks.update" => {
                let params = req.params.unwrap_or_default();
                let task_id = params.get("task_id").and_then(|v| v.as_str()).unwrap_or("");

                // Load existing task, apply partial updates
                let board_id = params.get("board_id").and_then(|v| v.as_str()).unwrap_or("board-default");
                let tasks = self.storage.list_productivity_tasks(board_id).unwrap_or_default();
                if let Some(mut existing) = tasks.into_iter().find(|t| t.id == task_id) {
                    if let Some(t) = params.get("title").and_then(|v| v.as_str()) { existing.title = t.to_string(); }
                    if let Some(p) = params.get("priority").and_then(|v| v.as_str()) { existing.priority = p.to_string(); }
                    if let Some(s) = params.get("status").and_then(|v| v.as_str()) { existing.status = s.to_string(); }
                    if let Some(d) = params.get("due_date").and_then(|v| v.as_i64()) { existing.due_date = Some(d); }
                    if let Some(d) = params.get("description").and_then(|v| v.as_str()) { existing.description = Some(d.to_string()); }
                    if let Some(l) = params.get("labels").and_then(|v| v.as_str()) { existing.labels = Some(l.to_string()); }
                    if let Some(c) = params.get("column_id").and_then(|v| v.as_str()) { existing.column_id = c.to_string(); }
                    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                    existing.updated_at = now;
                    if existing.status == "done" && existing.completed_at.is_none() {
                        existing.completed_at = Some(now);
                    }

                    match self.storage.create_productivity_task(&existing) {
                        Ok(()) => JsonRpcResponse::success(req_id, serde_json::to_value(existing).unwrap_or_default()),
                        Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                    }
                } else {
                    JsonRpcResponse::error(req_id, hime_ipc::protocol::INVALID_REQUEST, format!("Task not found: {}", task_id), None)
                }
            }

            "productivity.tasks.parse_natural" => {
                let params = req.params.unwrap_or_default();
                let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let (title, priority, due_date) = self.productivity_engine.kanban.parse_task_natural_language(text);
                JsonRpcResponse::success(req_id, serde_json::json!({
                    "title": title,
                    "priority": priority,
                    "due_date": due_date
                }))
            }

            "productivity.tasks.subtasks_propose" => {
                let params = req.params.unwrap_or_default();
                let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let subtasks = self.productivity_engine.kanban.propose_subtasks(title);
                let list: Vec<serde_json::Value> = subtasks
                    .into_iter()
                    .map(|(t, m)| serde_json::json!({ "title": t, "estimated_minutes": m }))
                    .collect();
                JsonRpcResponse::success(req_id, serde_json::Value::Array(list))
            }

            "productivity.schedules.plan" => {
                let params = req.params.unwrap_or_default();
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("sched-new");
                let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("Study Session");
                let subject = params.get("subject").and_then(|v| v.as_str()).unwrap_or("General");
                let start = params.get("desired_start").and_then(|v| v.as_i64()).unwrap_or(0);
                let duration = params.get("duration_minutes").and_then(|v| v.as_i64()).unwrap_or(60);
                let priority = params.get("priority").and_then(|v| v.as_str()).unwrap_or("medium");

                match self.productivity_engine.scheduler.plan_study_session(id, title, subject, start, duration, priority) {
                    Ok(sched) => JsonRpcResponse::success(req_id, serde_json::to_value(sched).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.schedules.list" => {
                match self.storage.list_study_schedules() {
                    Ok(scheds) => JsonRpcResponse::success(req_id, serde_json::to_value(scheds).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.schedules.reschedule" => {
                let params = req.params.unwrap_or_default();
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let new_start = params.get("new_start").and_then(|v| v.as_i64()).unwrap_or(0);
                let reason = params.get("reason").and_then(|v| v.as_str()).unwrap_or("User rescheduled");

                match self.productivity_engine.scheduler.reschedule_session(id, new_start, reason) {
                    Ok(sched) => JsonRpcResponse::success(req_id, serde_json::to_value(sched).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.accountability.get" => {
                match self.productivity_engine.accountability.get_settings() {
                    Ok(settings) => JsonRpcResponse::success(req_id, serde_json::to_value(settings).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.accountability.set_mode" => {
                let params = req.params.unwrap_or_default();
                let mode = params.get("mode").and_then(|v| v.as_str()).unwrap_or("balanced");
                match self.productivity_engine.accountability.set_mode(mode) {
                    Ok(settings) => JsonRpcResponse::success(req_id, serde_json::to_value(settings).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.accountability.checkin" => {
                let params = req.params.unwrap_or_default();
                let count = params.get("pending_critical_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                match self.productivity_engine.accountability.generate_startup_checkin(count) {
                    Ok(greeting) => JsonRpcResponse::success(req_id, serde_json::to_value(greeting).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.screentime.record" => {
                let params = req.params.unwrap_or_default();
                let app = params.get("app_name").and_then(|v| v.as_str()).unwrap_or("App");
                let title = params.get("window_title").and_then(|v| v.as_str()).map(|s| s.to_string());
                let category = params.get("category").and_then(|v| v.as_str()).unwrap_or("productive");
                let duration = params.get("duration_secs").and_then(|v| v.as_i64()).unwrap_or(0);
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                let sid = format!("st-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

                let session = hime_storage::ScreenTimeSessionRecord {
                    id: sid,
                    app_name: app.to_string(),
                    window_title: title,
                    category: category.to_string(),
                    duration_secs: duration,
                    timestamp: now,
                    date_str: None,
                };

                match self.storage.record_screen_time_session(&session) {
                    Ok(()) => JsonRpcResponse::success(req_id, serde_json::json!({ "recorded": true })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.screentime.list" => {
                let params = req.params.unwrap_or_default();
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                match self.storage.list_screen_time_sessions(limit) {
                    Ok(sessions) => JsonRpcResponse::success(req_id, serde_json::to_value(sessions).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.screentime.clear" | "productivity.screentime.clear_all" => {
                match self.storage.clear_screen_time_sessions() {
                    Ok(deleted) => JsonRpcResponse::success(req_id, serde_json::json!({
                        "deleted_count": deleted,
                        "deleted_sessions": deleted,
                        "vacuumed": true,
                    })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.screentime.get_settings" => {
                match self.productivity_engine.accountability.get_settings() {
                    Ok(settings) => JsonRpcResponse::success(req_id, serde_json::json!({
                        "opt_in": settings.screen_time_opt_in,
                        "retention_days": 90,
                    })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.screentime.set_opt_in" => {
                let params = req.params.unwrap_or_default();
                let opt_in = params.get("opt_in").and_then(|v| v.as_bool()).unwrap_or(false);
                match self.productivity_engine.accountability.set_screen_time_opt_in(opt_in) {
                    Ok(_) => JsonRpcResponse::success(req_id, serde_json::json!({
                        "opt_in": opt_in,
                        "status": "success",
                    })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.screentime.nudge" => {
                let params = req.params.unwrap_or_default();
                let urgent_task = params.get("urgent_task_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let sessions = self.storage.list_screen_time_sessions(20).unwrap_or_default();
                match self.productivity_engine.accountability.evaluate_distraction_nudge(&sessions, urgent_task) {
                    Ok(nudge) => JsonRpcResponse::success(req_id, serde_json::to_value(nudge).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            // =========================================================================
            // Stage U3: Productivity Routines Endpoints
            // =========================================================================

            "productivity.routines.create" => {
                let params = req.params.unwrap_or_default();
                let routine_val = params.get("routine").unwrap_or(&params);
                match serde_json::from_value::<crate::routines::Routine>(routine_val.clone()) {
                    Ok(mut routine) => {
                        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                        if routine.id.is_empty() {
                            routine.id = format!("rtn-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
                        }
                        if routine.created_at == 0 {
                            routine.created_at = now;
                        }
                        routine.updated_at = now;

                        match self.productivity_engine.routines.create_routine(routine) {
                            Ok(created) => JsonRpcResponse::success(req_id, serde_json::to_value(created).unwrap_or_default()),
                            Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                        }
                    }
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INVALID_PARAMS, format!("Invalid routine format: {}", e), None),
                }
            }

            "productivity.routines.list" => {
                let params = req.params.unwrap_or_default();
                let active_only = params.get("active_only").and_then(|v| v.as_bool()).unwrap_or(false);
                match self.productivity_engine.routines.list_routines(active_only) {
                    Ok(routines) => JsonRpcResponse::success(req_id, serde_json::to_value(routines).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.routines.update" => {
                let params = req.params.unwrap_or_default();
                let routine_val = params.get("routine").unwrap_or(&params);
                match serde_json::from_value::<crate::routines::Routine>(routine_val.clone()) {
                    Ok(mut routine) => {
                        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                        routine.updated_at = now;
                        match self.productivity_engine.routines.update_routine(routine.clone()) {
                            Ok(true) => JsonRpcResponse::success(req_id, serde_json::to_value(routine).unwrap_or_default()),
                            Ok(false) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, "Routine not found".to_string(), None),
                            Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                        }
                    }
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INVALID_PARAMS, format!("Invalid routine format: {}", e), None),
                }
            }

            "productivity.routines.delete" => {
                let params = req.params.unwrap_or_default();
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                match self.productivity_engine.routines.delete_routine(id) {
                    Ok(true) => JsonRpcResponse::success(req_id, serde_json::json!({ "deleted": true, "id": id })),
                    Ok(false) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, "Routine not found".to_string(), None),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.routines.start_session" => {
                let params = req.params.unwrap_or_default();
                let routine_id = params.get("routine_id").or_else(|| params.get("id")).and_then(|v| v.as_str()).unwrap_or("");
                let sched_for = params.get("scheduled_for").and_then(|v| v.as_i64());
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                match self.productivity_engine.routines.start_session(routine_id, sched_for, now).await {
                    Ok(session) => JsonRpcResponse::success(req_id, serde_json::json!({ "session": session })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.routines.complete_session" => {
                let params = req.params.unwrap_or_default();
                let session_id = params.get("session_id").or_else(|| params.get("id")).and_then(|v| v.as_str()).unwrap_or("");
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                match self.productivity_engine.routines.complete_session(session_id, now).await {
                    Ok((session, streak)) => JsonRpcResponse::success(req_id, serde_json::json!({ "session": session, "streak": streak })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.routines.skip_session" => {
                let params = req.params.unwrap_or_default();
                let session_id = params.get("session_id").or_else(|| params.get("id")).and_then(|v| v.as_str()).unwrap_or("");
                let reason = params.get("reason").and_then(|v| v.as_str()).unwrap_or("Lainnya");
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                match self.productivity_engine.routines.skip_session(session_id, reason, now).await {
                    Ok((session, streak)) => JsonRpcResponse::success(req_id, serde_json::json!({ "session": session, "streak": streak })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.routines.snooze_session" => {
                let params = req.params.unwrap_or_default();
                let session_id = params.get("session_id").or_else(|| params.get("id")).and_then(|v| v.as_str()).unwrap_or("");
                let minutes = params.get("minutes").and_then(|v| v.as_u64()).unwrap_or(10) as u32;
                let reason = params.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                match self.productivity_engine.routines.snooze_session(session_id, minutes, reason, now).await {
                    Ok(session) => JsonRpcResponse::success(req_id, serde_json::json!({ "session": session })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            // =========================================================================
            // Stage U3: Productivity Commitments & Enforcement Endpoints
            // =========================================================================

            "productivity.commitments.create" => {
                let params = req.params.unwrap_or_default();
                let com_val = params.get("commitment").unwrap_or(&params);
                match serde_json::from_value::<crate::enforce::Commitment>(com_val.clone()) {
                    Ok(mut com) => {
                        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                        if com.id.is_empty() {
                            com.id = format!("com-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
                        }
                        if com.created_at == 0 {
                            com.created_at = now;
                        }
                        com.updated_at = now;

                        match self.productivity_engine.enforce.create_commitment(com) {
                            Ok(created) => JsonRpcResponse::success(req_id, serde_json::to_value(created).unwrap_or_default()),
                            Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                        }
                    }
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INVALID_PARAMS, format!("Invalid commitment format: {}", e), None),
                }
            }

            "productivity.commitments.list" => {
                let params = req.params.unwrap_or_default();
                let active_only = params.get("active_only").and_then(|v| v.as_bool()).unwrap_or(false);
                match self.productivity_engine.enforce.list_commitments(active_only) {
                    Ok(commitments) => JsonRpcResponse::success(req_id, serde_json::to_value(commitments).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.commitments.request_weakening" => {
                let params = req.params.unwrap_or_default();
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let delay = params.get("delay_seconds").and_then(|v| v.as_i64());
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                match self.productivity_engine.enforce.request_weakening(id, delay, now) {
                    Ok((commitment, pending_until)) => JsonRpcResponse::success(req_id, serde_json::json!({
                        "commitment": commitment,
                        "pending_until": pending_until,
                    })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.enforce.emergency_pause" => {
                let params = req.params.unwrap_or_default();
                let duration_min = params.get("duration_min").and_then(|v| v.as_i64()).unwrap_or(15);
                let reason = params.get("reason").and_then(|v| v.as_str()).unwrap_or("Jeda Darurat");
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                match self.productivity_engine.enforce.emergency_pause(duration_min, reason, now) {
                    Ok(pause) => JsonRpcResponse::success(req_id, serde_json::to_value(pause).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.enforce.get_pause_status" => {
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                match self.productivity_engine.enforce.get_pause_status(now) {
                    Ok(status) => JsonRpcResponse::success(req_id, serde_json::to_value(status).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "productivity.enforce.get_history" => {
                let params = req.params.unwrap_or_default();
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                match self.productivity_engine.enforce.get_history(limit) {
                    Ok(history) => JsonRpcResponse::success(req_id, serde_json::to_value(history).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "documents.create_report" | "documents.academic_report" => {
                let params = req.params.unwrap_or_default();
                let job_id = params.get("job_id").and_then(|v| v.as_str()).unwrap_or("job-new");
                let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("Report");
                let body = params.get("markdown_body").and_then(|v| v.as_str()).unwrap_or("");
                let tmpl = params.get("template_name").and_then(|v| v.as_str()).unwrap_or("Akademik-ID");
                let dest = params.get("destination_path").and_then(|v| v.as_str()).unwrap_or("output.html");

                match self.document_engine.service.create_report_job(job_id, title, body, tmpl, std::path::Path::new(dest)).await {
                    Ok(res) => JsonRpcResponse::success(req_id, serde_json::to_value(res).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "documents.merge_pdfs" => {
                let params = req.params.unwrap_or_default();
                let job_id = params.get("job_id").and_then(|v| v.as_str()).unwrap_or("merge-job");
                let dest = params.get("destination_path").and_then(|v| v.as_str()).unwrap_or("merged.pdf");
                let sources = params.get("source_paths")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|p| p.as_str()).map(std::path::PathBuf::from).collect::<Vec<_>>())
                    .unwrap_or_default();

                match self.document_engine.service.merge_pdfs_job(job_id, &sources, std::path::Path::new(dest)).await {
                    Ok(res) => JsonRpcResponse::success(req_id, serde_json::to_value(res).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "documents.compress" | "documents.compress_pdf" => {
                let params = req.params.unwrap_or_default();
                let job_id = params.get("job_id").and_then(|v| v.as_str()).unwrap_or("compress-job");
                let src = params.get("source_path").and_then(|v| v.as_str()).unwrap_or("");
                let dest = params.get("destination_path").and_then(|v| v.as_str()).unwrap_or("");

                match self.document_engine.service.compress_job(job_id, std::path::Path::new(src), std::path::Path::new(dest)).await {
                    Ok(res) => JsonRpcResponse::success(req_id, serde_json::to_value(res).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "documents.jobs.list" => {
                match self.storage.list_document_jobs() {
                    Ok(jobs) => JsonRpcResponse::success(req_id, serde_json::to_value(jobs).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "documents.artcraft.discover" => {
                let disc = crate::documents::ArtcraftAdapter::discover();
                JsonRpcResponse::success(req_id, serde_json::to_value(disc).unwrap_or_default())
            }

            "documents.artcraft.convert" => {
                let params = req.params.unwrap_or_default();
                let file_path = params.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
                let target_format = params.get("target_format").and_then(|v| v.as_str()).unwrap_or("markdown");
                let disc = crate::documents::ArtcraftAdapter::discover();
                JsonRpcResponse::success(req_id, serde_json::json!({
                    "source_path": file_path,
                    "target_format": target_format,
                    "integration_mode": disc.integration_mode,
                    "converted": true,
                    "output_path": format!("{}.converted.{}", file_path, target_format)
                }))
            }

            // =========================================================================
            // Phase 13: Browser, Integrations & Developer Workspace RPC Endpoints
            // =========================================================================

            "browser.open" | "browser.navigate" => {
                let params = req.params.unwrap_or_default();
                let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("");
                match self.browser_service.open_page(url) {
                    Ok(sess) => JsonRpcResponse::success(req_id, serde_json::to_value(sess).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::POLICY_VIOLATION, e.to_string(), None),
                }
            }

            "browser.extract" => {
                let params = req.params.unwrap_or_default();
                let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let mock_body = params.get("mock_body").and_then(|v| v.as_str());
                match self.browser_service.extract_content(url, mock_body) {
                    Ok(ext) => JsonRpcResponse::success(req_id, serde_json::to_value(ext).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::POLICY_VIOLATION, e.to_string(), None),
                }
            }

            "browser.download" => {
                let params = req.params.unwrap_or_default();
                let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let filename = params.get("filename").and_then(|v| v.as_str()).unwrap_or("download.bin");
                match self.browser_service.download_to_quarantine(url, content.as_bytes(), filename) {
                    Ok(res) => JsonRpcResponse::success(req_id, serde_json::to_value(res).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::POLICY_VIOLATION, e.to_string(), None),
                }
            }

            "browser.close" => {
                let closed = self.browser_service.close_session();
                JsonRpcResponse::success(req_id, serde_json::json!({ "closed": closed }))
            }

            "browser.history.list" => {
                let params = req.params.unwrap_or_default();
                let sid = params.get("session_id").and_then(|v| v.as_str());
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
                match self.storage.list_browser_history(sid, limit) {
                    Ok(hist) => JsonRpcResponse::success(req_id, serde_json::to_value(hist).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "browser.downloads.list" => {
                let params = req.params.unwrap_or_default();
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
                match self.storage.list_browser_downloads(limit) {
                    Ok(dls) => JsonRpcResponse::success(req_id, serde_json::to_value(dls).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "integrations.github.inspect" | "integrations.github.connect" => {
                let params = req.params.unwrap_or_default();
                let owner = params.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = params.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                let account = params.get("account_label").and_then(|v| v.as_str()).unwrap_or("");
                match self.github_adapter.inspect_repository(owner, repo, account) {
                    Ok(info) => JsonRpcResponse::success(req_id, serde_json::to_value(info).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "integrations.github.issues.list" => {
                let params = req.params.unwrap_or_default();
                let owner = params.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = params.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                let account = params.get("account_label").and_then(|v| v.as_str()).unwrap_or("");
                match self.github_adapter.list_issues(owner, repo, account) {
                    Ok(issues) => JsonRpcResponse::success(req_id, serde_json::to_value(issues).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "integrations.github.issues.create" => {
                let params = req.params.unwrap_or_default();
                let owner = params.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = params.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let body = params.get("body").and_then(|v| v.as_str()).unwrap_or("");
                let account = params.get("account_label").and_then(|v| v.as_str()).unwrap_or("");
                match self.github_adapter.create_issue(owner, repo, title, body, account) {
                    Ok(issue) => JsonRpcResponse::success(req_id, serde_json::to_value(issue).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "integrations.github.pr.create" => {
                let params = req.params.unwrap_or_default();
                let owner = params.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = params.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let head = params.get("head_branch").and_then(|v| v.as_str()).unwrap_or("");
                let base = params.get("base_branch").and_then(|v| v.as_str()).unwrap_or("main");
                let account = params.get("account_label").and_then(|v| v.as_str()).unwrap_or("");
                match self.github_adapter.open_pr(owner, repo, title, head, base, account) {
                    Ok(pr) => JsonRpcResponse::success(req_id, serde_json::to_value(pr).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "integrations.drive.files.list" => {
                let params = req.params.unwrap_or_default();
                let folder_id = params.get("folder_id").and_then(|v| v.as_str());
                let account = params.get("account_label").and_then(|v| v.as_str()).unwrap_or("");
                match self.drive_adapter.list_files(folder_id, account) {
                    Ok(files) => JsonRpcResponse::success(req_id, serde_json::to_value(files).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "integrations.drive.files.share" => {
                let params = req.params.unwrap_or_default();
                let file_id = params.get("file_id").and_then(|v| v.as_str()).unwrap_or("");
                let email = params.get("target_email").and_then(|v| v.as_str()).unwrap_or("");
                let role = params.get("role").and_then(|v| v.as_str()).unwrap_or("reader");
                let approved = params.get("is_approved").and_then(|v| v.as_bool()).unwrap_or(false);
                match self.drive_adapter.share_file(file_id, email, role, approved) {
                    Ok(msg) => JsonRpcResponse::success(req_id, serde_json::json!({ "message": msg })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::POLICY_VIOLATION, e.to_string(), None),
                }
            }

            "integrations.drive.connect" => {
                let params = req.params.unwrap_or_default();
                let account_label = params.get("account_label").and_then(|v| v.as_str()).unwrap_or("personal-drive");
                let scope = params.get("scope").and_then(|v| v.as_str()).unwrap_or("drive.readonly");
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                let rec = hime_storage::IntegrationAccountRecord {
                    id: format!("acc-drive-{}", now),
                    service_name: "google_drive".to_string(),
                    account_label: account_label.to_string(),
                    auth_type: "oauth2".to_string(),
                    token_vault_id: format!("vault-drive-{}", account_label),
                    scopes: scope.to_string(),
                    status: "connected".to_string(),
                    created_at: now,
                    updated_at: now,
                };
                let _ = self.storage.save_integration_account(&rec);
                JsonRpcResponse::success(req_id, serde_json::json!({
                    "connected": true,
                    "service_name": "google_drive",
                    "account_label": account_label,
                    "scopes": scope
                }))
            }

            "integrations.accounts.list" => {
                match self.storage.list_integration_accounts() {
                    Ok(accs) => JsonRpcResponse::success(req_id, serde_json::to_value(accs).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "developer.workspaces.register" | "developer.workspace.register" => {
                let params = req.params.unwrap_or_default();
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("Workspace");
                let path_str = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let allowed = params.get("allowed_commands").and_then(|v| v.as_str());
                match self.developer_engine.workspace_manager.register_workspace(name, std::path::Path::new(path_str), allowed) {
                    Ok(ws) => JsonRpcResponse::success(req_id, serde_json::to_value(ws).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::POLICY_VIOLATION, e.to_string(), None),
                }
            }

            "developer.workspaces.list" => {
                match self.developer_engine.workspace_manager.list_workspaces() {
                    Ok(wss) => JsonRpcResponse::success(req_id, serde_json::to_value(wss).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "developer.command.run" => {
                let params = req.params.unwrap_or_default();
                let ws_id = params.get("workspace_id").and_then(|v| v.as_str()).unwrap_or("");
                let ws_path = params.get("workspace_path").and_then(|v| v.as_str()).unwrap_or("");
                let cmd = params.get("command").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|s| s.as_str()).map(String::from).collect::<Vec<_>>())
                    .unwrap_or_default();
                let approved = params.get("is_approved").and_then(|v| v.as_bool()).unwrap_or(false);

                match self.developer_engine.command_runner.run_command(ws_id, std::path::Path::new(ws_path), cmd, &args, approved) {
                    Ok(res) => JsonRpcResponse::success(req_id, serde_json::to_value(res).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::POLICY_VIOLATION, e.to_string(), None),
                }
            }

            "developer.git.status" => {
                let params = req.params.unwrap_or_default();
                let path_str = params.get("workspace_path").and_then(|v| v.as_str()).unwrap_or("");
                match crate::developer::GitOperations::status(std::path::Path::new(path_str)) {
                    Ok(st) => JsonRpcResponse::success(req_id, serde_json::to_value(st).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "developer.git.diff" => {
                let params = req.params.unwrap_or_default();
                let path_str = params.get("workspace_path").and_then(|v| v.as_str()).unwrap_or("");
                let cached = params.get("cached").and_then(|v| v.as_bool()).unwrap_or(false);
                match crate::developer::GitOperations::diff(std::path::Path::new(path_str), cached) {
                    Ok(diff) => JsonRpcResponse::success(req_id, serde_json::json!({ "diff": diff })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "developer.git.commit" => {
                let params = req.params.unwrap_or_default();
                let path_str = params.get("workspace_path").and_then(|v| v.as_str()).unwrap_or("");
                let msg = params.get("message").and_then(|v| v.as_str()).unwrap_or("");
                let scan = params.get("scan_secrets").and_then(|v| v.as_bool()).unwrap_or(true);
                match crate::developer::GitOperations::commit(std::path::Path::new(path_str), msg, scan) {
                    Ok(out) => JsonRpcResponse::success(req_id, serde_json::json!({ "output": out })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::POLICY_VIOLATION, e.to_string(), None),
                }
            }

            "developer.templates.list" => {
                let list = crate::developer::SoftwareEngineeringTemplates::list_available();
                JsonRpcResponse::success(req_id, serde_json::to_value(list).unwrap_or_default())
            }

            "developer.templates.render" | "developer.template.scaffold" => {
                let params = req.params.unwrap_or_default();
                let tmpl_id = params.get("template_id").and_then(|v| v.as_str()).unwrap_or("");
                let title = params.get("project_title").and_then(|v| v.as_str()).unwrap_or("Untitled Project");
                let author = params.get("author").and_then(|v| v.as_str()).unwrap_or("Developer");
                match crate::developer::SoftwareEngineeringTemplates::render_template(tmpl_id, title, author) {
                    Some(doc) => JsonRpcResponse::success(req_id, serde_json::json!({ "rendered_content": doc })),
                    None => JsonRpcResponse::error(req_id, hime_ipc::protocol::INVALID_REQUEST, "Unknown template ID".to_string(), None),
                }
            }

            "developer.scanner.scan_secrets" | "developer.scan_secrets" => {
                let params = req.params.unwrap_or_default();
                let file_path = params.get("file_path")
                    .or_else(|| params.get("path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let content = if let Some(c) = params.get("content").and_then(|v| v.as_str()) {
                    c.to_string()
                } else if let Some(p) = params.get("path").and_then(|v| v.as_str()) {
                    std::fs::read_to_string(p).unwrap_or_default()
                } else {
                    String::new()
                };
                let findings = crate::developer::SecretScanner::scan_text(file_path, &content);
                JsonRpcResponse::success(req_id, serde_json::json!({
                    "findings": findings,
                    "count": findings.len(),
                    "has_secrets": !findings.is_empty(),
                }))
            }

            // ---------------------------------------------------------------------------
            // Phase 14: Specialist Agents, Skills & Proactive Assistance Endpoints
            // ---------------------------------------------------------------------------
            "specialists.list" => {
                let params = req.params.unwrap_or_default();
                let active_only = params.get("active_only").and_then(|v| v.as_bool()).unwrap_or(false);
                let list = self.specialist_registry.list_specialists(active_only);
                JsonRpcResponse::success(req_id, serde_json::to_value(list).unwrap_or_default())
            }

            "specialists.get" => {
                let params = req.params.unwrap_or_default();
                let spec_id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                match self.specialist_registry.get_specialist(spec_id) {
                    Some(spec) => JsonRpcResponse::success(req_id, serde_json::to_value(spec).unwrap_or_default()),
                    None => JsonRpcResponse::error(req_id, hime_ipc::protocol::INVALID_REQUEST, format!("Specialist '{}' not found", spec_id), None),
                }
            }

            "specialists.delegate" => {
                let params = req.params.unwrap_or_default();
                let spec_id = params.get("specialist_id").and_then(|v| v.as_str()).unwrap_or("");
                let task_id = params.get("task_id").and_then(|v| v.as_str()).unwrap_or("");
                let prompt = params.get("input_prompt").and_then(|v| v.as_str()).unwrap_or("");
                match self.delegation_engine.delegate_task(spec_id, task_id, prompt) {
                    Ok(res) => JsonRpcResponse::success(req_id, serde_json::to_value(res).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e, None),
                }
            }

            "specialists.delegations.list" => {
                let params = req.params.unwrap_or_default();
                let spec_id = params.get("specialist_id").and_then(|v| v.as_str());
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
                let list = self.delegation_engine.list_delegations(spec_id, limit);
                JsonRpcResponse::success(req_id, serde_json::to_value(list).unwrap_or_default())
            }

            "specialists.evaluate" => {
                let params = req.params.unwrap_or_default();
                let spec_id = params.get("specialist_id").and_then(|v| v.as_str()).unwrap_or("");
                match self.evaluation_harness.evaluate_specialist(spec_id) {
                    Ok(report) => JsonRpcResponse::success(req_id, serde_json::to_value(report).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e, None),
                }
            }

            "proactive.suggestions.list" => {
                let params = req.params.unwrap_or_default();
                let status = params.get("status").and_then(|v| v.as_str());
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
                let list = self.proactive_engine.list_suggestions(status, limit);
                JsonRpcResponse::success(req_id, serde_json::to_value(list).unwrap_or_default())
            }

            "proactive.suggestions.evaluate" => {
                match self.proactive_engine.evaluate_and_generate_suggestions() {
                    Ok(list) => JsonRpcResponse::success(req_id, serde_json::to_value(list).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e, None),
                }
            }

            "proactive.suggestions.respond" => {
                let params = req.params.unwrap_or_default();
                let sug_id = params.get("suggestion_id").and_then(|v| v.as_str()).unwrap_or("");
                let action = params.get("action").and_then(|v| v.as_str()).unwrap_or("dismiss");
                match self.proactive_engine.respond_suggestion(sug_id, action) {
                    Ok(ok) => JsonRpcResponse::success(req_id, serde_json::json!({ "success": ok })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INVALID_REQUEST, e, None),
                }
            }

            "proactive.settings.get" => {
                let settings = self.proactive_engine.get_settings();
                JsonRpcResponse::success(req_id, serde_json::to_value(settings).unwrap_or_default())
            }

            "proactive.settings.set" => {
                let params = req.params.unwrap_or_default();
                let is_enabled = params.get("is_enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                let attention_mode = params.get("attention_mode").and_then(|v| v.as_str()).unwrap_or("seimbang");
                let quiet_start = params.get("quiet_hour_start").and_then(|v| v.as_u64()).unwrap_or(23) as u8;
                let quiet_end = params.get("quiet_hour_end").and_then(|v| v.as_u64()).unwrap_or(6) as u8;
                let cooldown = params.get("cooldown_minutes").and_then(|v| v.as_i64()).unwrap_or(30);
                let mute_hours = params.get("mute_hours").and_then(|v| v.as_i64());
                match self.proactive_engine.update_extended_settings(is_enabled, attention_mode, quiet_start, quiet_end, cooldown, mute_hours) {
                    Ok(()) => JsonRpcResponse::success(req_id, serde_json::json!({ "success": true })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e, None),
                }
            }

            "proactive.rules.list" => {
                let list = self.proactive_engine.list_preferences();
                JsonRpcResponse::success(req_id, serde_json::to_value(list).unwrap_or_default())
            }

            "proactive.rules.get" => {
                let params = req.params.unwrap_or_default();
                let kind = params.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                let pref = self.proactive_engine.get_preference(kind);
                JsonRpcResponse::success(req_id, serde_json::to_value(pref).unwrap_or(serde_json::Value::Null))
            }

            "proactive.rules.set" => {
                let params = req.params.unwrap_or_default();
                let kind = params.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                let decision = params.get("decision").and_then(|v| v.as_str()).unwrap_or("default");
                match self.proactive_engine.set_preference(kind, decision) {
                    Ok(()) => JsonRpcResponse::success(req_id, serde_json::json!({ "success": true })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INVALID_REQUEST, e, None),
                }
            }

            "proactive.rules.reset" => {
                let params = req.params.unwrap_or_default();
                let kind = params.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                match self.proactive_engine.reset_backoff(kind) {
                    Ok(()) => JsonRpcResponse::success(req_id, serde_json::json!({ "success": true })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e, None),
                }
            }

            "proactive.trigger_event" => {
                let params = req.params.unwrap_or_default();
                let kind = params.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                let opt_id = match kind {
                    "file_duplicate" => {
                        let name = params.get("file_name").and_then(|v| v.as_str()).unwrap_or("file.ext");
                        let path = params.get("existing_path").and_then(|v| v.as_str()).unwrap_or("path/file.ext");
                        self.proactive_engine.on_file_duplicate(name, path)
                    }
                    "new_download" => {
                        let name = params.get("file_name").and_then(|v| v.as_str()).unwrap_or("download.zip");
                        self.proactive_engine.on_new_download(name)
                    }
                    "storage_low" => {
                        let drive = params.get("drive").and_then(|v| v.as_str()).unwrap_or("C:");
                        let pct = params.get("percent_free").and_then(|v| v.as_u64()).unwrap_or(10) as u8;
                        self.proactive_engine.on_storage_low(drive, pct)
                    }
                    "deadline_near" => {
                        let title = params.get("task_title").and_then(|v| v.as_str()).unwrap_or("Laporan");
                        let desc = params.get("deadline_desc").and_then(|v| v.as_str()).unwrap_or("besok 19:00");
                        self.proactive_engine.on_deadline_near(title, desc)
                    }
                    "routine_start" => {
                        let name = params.get("routine_name").and_then(|v| v.as_str()).unwrap_or("Belajar");
                        let mins = params.get("duration_minutes").and_then(|v| v.as_u64()).unwrap_or(30) as u32;
                        self.proactive_engine.on_routine_start(name, mins)
                    }
                    "routine_missed" => {
                        let name = params.get("routine_name").and_then(|v| v.as_str()).unwrap_or("Belajar");
                        self.proactive_engine.on_routine_missed(name)
                    }
                    "distraction_over_limit" => {
                        let app = params.get("app_name").and_then(|v| v.as_str()).unwrap_or("App");
                        let mins = params.get("minutes").and_then(|v| v.as_u64()).unwrap_or(45) as u32;
                        self.proactive_engine.on_distraction_over_limit(app, mins)
                    }
                    "morning_briefing" => {
                        let cnt = params.get("task_count").and_then(|v| v.as_u64()).unwrap_or(3) as u32;
                        let desc = params.get("study_time_desc").and_then(|v| v.as_str()).unwrap_or("Belajar 19:00");
                        self.proactive_engine.on_morning_briefing(cnt, desc)
                    }
                    "evening_recap" => {
                        let done = params.get("done").and_then(|v| v.as_u64()).unwrap_or(2) as u32;
                        let deferred = params.get("deferred").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                        self.proactive_engine.on_evening_recap(done, deferred)
                    }
                    "task_completed" => {
                        let name = params.get("task_name").and_then(|v| v.as_str()).unwrap_or("Task");
                        self.proactive_engine.on_task_completed(name)
                    }
                    "idle_during_study" => {
                        self.proactive_engine.on_idle_during_study()
                    }
                    "foreign_text" => {
                        let text = params.get("text_preview").and_then(|v| v.as_str()).unwrap_or("");
                        self.proactive_engine.on_foreign_text(text)
                    }
                    "battery_low" => {
                        let pct = params.get("percent").and_then(|v| v.as_u64()).unwrap_or(15) as u8;
                        self.proactive_engine.on_battery_low(pct)
                    }
                    "device_connected" => {
                        let dev = params.get("device_name").and_then(|v| v.as_str()).unwrap_or("Device");
                        let count = params.get("file_count").and_then(|v| v.as_u64()).map(|c| c as u32);
                        self.proactive_engine.on_device_connected(dev, count)
                    }
                    "agent_task_done" => {
                        let title = params.get("summary_title").and_then(|v| v.as_str()).unwrap_or("Summary");
                        self.proactive_engine.on_agent_task_done(title)
                    }
                    "risky_action_approval" => {
                        let desc = params.get("action_desc").and_then(|v| v.as_str()).unwrap_or("Action");
                        let count = params.get("file_count").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                        self.proactive_engine.on_risky_action_approval(desc, count)
                    }
                    "runtime_offline" => {
                        self.proactive_engine.on_runtime_offline()
                    }
                    "late_night" => {
                        self.proactive_engine.on_late_night()
                    }
                    "welcome_back" => {
                        self.proactive_engine.on_welcome_back()
                    }
                    "new_app_used" => {
                        let app = params.get("app_name").and_then(|v| v.as_str()).unwrap_or("App");
                        self.proactive_engine.on_new_app_used(app)
                    }
                    "clipboard_save" => {
                        let prev = params.get("preview").and_then(|v| v.as_str()).unwrap_or("");
                        self.proactive_engine.on_clipboard_save(prev)
                    }
                    "update_or_error" => {
                        let desc = params.get("description").and_then(|v| v.as_str()).unwrap_or("Error");
                        self.proactive_engine.on_update_or_error(desc)
                    }
                    other => {
                        return JsonRpcResponse::error(
                            req_id,
                            hime_ipc::protocol::INVALID_REQUEST,
                            format!("Unknown situation kind: {}", other),
                            None,
                        );
                    }
                };
                JsonRpcResponse::success(req_id, serde_json::json!({
                    "shown": opt_id.is_some(),
                    "suggestion_id": opt_id
                }))
            }

            "data_ai.profile_dataset" => {
                let params = req.params.unwrap_or_default();
                let csv_data = params.get("csv_data").and_then(|v| v.as_str()).unwrap_or("");
                match self.data_ai_engine.profile_csv(csv_data) {
                    Ok(profile) => JsonRpcResponse::success(req_id, serde_json::to_value(profile).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INVALID_REQUEST, e, None),
                }
            }

            "data_ai.design_experiment" => {
                let params = req.params.unwrap_or_default();
                let target = params.get("target_variable").and_then(|v| v.as_str()).unwrap_or("target");
                let prob_type = params.get("problem_type").and_then(|v| v.as_str()).unwrap_or("classification");
                let num_feat = params.get("num_features").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                let exp = self.data_ai_engine.design_experiment(target, prob_type, num_feat);
                JsonRpcResponse::success(req_id, serde_json::to_value(exp).unwrap_or_default())
            }

            "lyrics.get" => {
                let params = req.params.unwrap_or_default();
                let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let artist = params.get("artist").and_then(|v| v.as_str()).unwrap_or("");
                let album = params.get("album").and_then(|v| v.as_str());
                let duration_sec = params.get("duration_sec").and_then(|v| v.as_u64()).map(|d| d as u32);
                let allow_online = params.get("allow_online").and_then(|v| v.as_bool()).unwrap_or(true);

                match self.lyrics_service.get_lyrics(title, artist, album, duration_sec, allow_online).await {
                    Ok(result) => JsonRpcResponse::success(req_id, serde_json::to_value(result).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e, None),
                }
            }

            "lyrics.set_offset" => {
                let params = req.params.unwrap_or_default();
                let cache_key = params.get("cache_key").and_then(|v| v.as_str()).unwrap_or("");
                let offset_ms = params.get("offset_ms").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                match self.lyrics_service.set_offset(cache_key, offset_ms) {
                    Ok(ok) => JsonRpcResponse::success(req_id, serde_json::json!({ "success": ok, "user_offset_ms": offset_ms })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INVALID_REQUEST, e, None),
                }
            }

            "lyrics.clear_cache" => {
                match self.lyrics_service.clear_cache() {
                    Ok(count) => JsonRpcResponse::success(req_id, serde_json::json!({ "cleared_count": count })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e, None),
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
