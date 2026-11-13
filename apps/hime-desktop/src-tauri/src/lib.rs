use serde::{Deserialize, Serialize};
use tracing::info;

use hime_ipc::framing::{Frame, MsgType};
use hime_ipc::protocol::{JsonRpcRequest, JsonRpcResponse};
#[cfg(windows)]
use hime_ipc::transport::windows_pipe::connect_pipe_client;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PingResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: u64,
    pub core_response: Option<serde_json::Value>,
}

/// Core business logic for pinging Hime Core via IPC.
/// Returns Ok(PingResult) on successful roundtrip, or Err(String) with a descriptive failure message if Core is unavailable or protocol fails.
pub async fn ping_core_logic(pipe_name: &str) -> Result<PingResult, String> {
    info!(pipe = %pipe_name, "Desktop IPC: invoking ping to Hime Core");
    let start = std::time::Instant::now();

    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable on pipe '{}': {}", pipe_name, e))?;

        let req = JsonRpcRequest::new(
            "desktop-ping-1",
            "system.ping",
            Some(serde_json::json!({"source": "hime-desktop-ui"})),
        );
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        let elapsed = start.elapsed().as_millis() as u64;
        Ok(PingResult {
            success: true,
            message: "Successfully communicated with Hime Core over IPC named pipe!".to_string(),
            latency_ms: elapsed,
            core_response: resp.result,
        })
    }

    #[cfg(not(windows))]
    {
        let _ = (pipe_name, start);
        Err("Windows Named Pipe IPC is only supported on Windows platform".to_string())
    }
}

/// Core business logic for executing tool policy checks on Hime Core.
pub async fn execute_tool_logic(
    pipe_name: &str,
    action_type: &str,
    path: &str,
) -> Result<serde_json::Value, String> {
    info!(pipe = %pipe_name, action = %action_type, path = %path, "Desktop IPC: invoking tool execution on Hime Core");

    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable on pipe '{}': {}", pipe_name, e))?;

        let req = JsonRpcRequest::new(
            "desktop-tool-1",
            "tool.execute",
            Some(serde_json::json!({
                "action_type": action_type,
                "path": path,
            })),
        );
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!(
                "Policy Denial / Error (code {}): {}",
                err.code, err.message
            ))
        } else {
            Ok(resp.result.unwrap_or_default())
        }
    }

    #[cfg(not(windows))]
    {
        let _ = (pipe_name, action_type, path);
        Err("Unsupported on this platform".to_string())
    }
}

/// Retrieves system status from Hime Core via IPC.
pub async fn get_system_status_logic(pipe_name: &str) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new("status-1", "system.status", None);
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Core error ({}): {}", err.code, err.message))
        } else {
            Ok(resp.result.unwrap_or_default())
        }
    }

    #[cfg(not(windows))]
    {
        let _ = pipe_name;
        Err("Windows Named Pipe IPC is only supported on Windows".to_string())
    }
}

/// Retrieves audit events from Hime Core SQLite storage via IPC.
pub async fn get_activity_events_logic(
    pipe_name: &str,
    limit: usize,
) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new(
            "events-1",
            "events.list",
            Some(serde_json::json!({ "limit": limit })),
        );
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Core error ({}): {}", err.code, err.message))
        } else {
            let val = resp.result.unwrap_or_default();
            if let Some(arr) = val.as_array() {
                Ok(arr.clone())
            } else {
                Ok(vec![])
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = (pipe_name, limit);
        Err("Windows Named Pipe IPC is only supported on Windows".to_string())
    }
}

/// Creates a new task in Hime Core via IPC.
pub async fn create_task_logic(
    pipe_name: &str,
    name: &str,
    description: &str,
    priority: &str,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new(
            "task-create-1",
            "tasks.create",
            Some(serde_json::json!({
                "name": name,
                "description": description,
                "priority": priority,
            })),
        );
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Core error ({}): {}", err.code, err.message))
        } else {
            Ok(resp.result.unwrap_or_default())
        }
    }

    #[cfg(not(windows))]
    {
        let _ = (pipe_name, name, description, priority);
        Err("Windows Named Pipe IPC is only supported on Windows".to_string())
    }
}

/// Retrieves the current task list from Hime Core via IPC.
pub async fn get_tasks_logic(pipe_name: &str) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new("tasks-list-1", "tasks.list", None);
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Core error ({}): {}", err.code, err.message))
        } else {
            let val = resp.result.unwrap_or_default();
            if let Some(arr) = val.as_array() {
                Ok(arr.clone())
            } else {
                Ok(vec![])
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = pipe_name;
        Err("Windows Named Pipe IPC is only supported on Windows".to_string())
    }
}

/// Cancels a task in Hime Core via IPC.
pub async fn cancel_task_logic(pipe_name: &str, id: &str) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new(
            "task-cancel-1",
            "tasks.cancel",
            Some(serde_json::json!({ "id": id })),
        );
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Core error ({}): {}", err.code, err.message))
        } else {
            Ok(resp.result.unwrap_or_default())
        }
    }

    #[cfg(not(windows))]
    {
        let _ = (pipe_name, id);
        Err("Windows Named Pipe IPC is only supported on Windows".to_string())
    }
}

/// Resolves a pending policy approval request on Hime Core via IPC.
pub async fn resolve_approval_logic(
    pipe_name: &str,
    request_id: &str,
    decision: &str,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new(
            "approval-resolve-1",
            "approval.resolve",
            Some(serde_json::json!({
                "request_id": request_id,
                "decision": decision,
            })),
        );
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!(
                "Policy Resolution Error ({}): {}",
                err.code, err.message
            ))
        } else {
            Ok(resp.result.unwrap_or_default())
        }
    }

    #[cfg(not(windows))]
    {
        let _ = (pipe_name, request_id, decision);
        Err("Windows Named Pipe IPC is only supported on Windows".to_string())
    }
}

// -----------------------------------------------------------------------------
// Phase 08 — AI Runtime Integration Desktop IPC Handlers
// -----------------------------------------------------------------------------

/// Retrieves AI runtime overview status (Core, Hermes, 9Router, IB-01, IB-02) via IPC.
pub async fn get_ai_status_logic(pipe_name: &str) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new("ai-status-1", "ai.status", None);
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("AI Status Error ({}): {}", err.code, err.message))
        } else {
            Ok(resp.result.unwrap_or_default())
        }
    }
    #[cfg(not(windows))]
    {
        let _ = pipe_name;
        Err("Unsupported on this platform".to_string())
    }
}

/// Retrieves the available AI models from Hime Core via IPC.
pub async fn get_ai_models_logic(pipe_name: &str) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new("ai-models-1", "ai.models.list", None);
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Models Error ({}): {}", err.code, err.message))
        } else {
            let val = resp.result.unwrap_or_default();
            if let Some(arr) = val.as_array() {
                Ok(arr.clone())
            } else {
                Ok(vec![])
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = pipe_name;
        Err("Unsupported on this platform".to_string())
    }
}

/// Stores a provider API key in Hime Core's DPAPI vault via IPC (IB-01).
pub async fn store_credential_logic(
    pipe_name: &str,
    provider: &str,
    secret: &str,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new(
            "cred-store-1",
            "credentials.store",
            Some(serde_json::json!({
                "provider": provider,
                "secret": secret
            })),
        );
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Credential Store Error ({}): {}", err.code, err.message))
        } else {
            Ok(resp.result.unwrap_or_default())
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (pipe_name, provider, secret);
        Err("Unsupported on this platform".to_string())
    }
}

/// Lists configured providers from Hime Core via IPC (redacted, no plaintext keys).
pub async fn list_credentials_logic(pipe_name: &str) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new("cred-list-1", "credentials.list", None);
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Credentials List Error ({}): {}", err.code, err.message))
        } else {
            let val = resp.result.unwrap_or_default();
            if let Some(arr) = val.as_array() {
                Ok(arr.clone())
            } else {
                Ok(vec![])
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = pipe_name;
        Err("Unsupported on this platform".to_string())
    }
}

/// Creates a new conversation thread in Hime Core SQLite WAL via IPC.
pub async fn create_conversation_logic(
    pipe_name: &str,
    title: &str,
    model: &str,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new(
            "conv-create-1",
            "conversation.create",
            Some(serde_json::json!({
                "title": title,
                "model": model,
            })),
        );
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Conversation Error ({}): {}", err.code, err.message))
        } else {
            Ok(resp.result.unwrap_or_default())
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (pipe_name, title, model);
        Err("Unsupported on this platform".to_string())
    }
}

/// Lists conversations from Hime Core via IPC.
pub async fn list_conversations_logic(pipe_name: &str) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new("conv-list-1", "conversation.list", None);
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Conversation List Error ({}): {}", err.code, err.message))
        } else {
            let val = resp.result.unwrap_or_default();
            if let Some(arr) = val.as_array() {
                Ok(arr.clone())
            } else {
                Ok(vec![])
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = pipe_name;
        Err("Unsupported on this platform".to_string())
    }
}

/// Retrieves message history for a conversation from Hime Core via IPC.
pub async fn get_conversation_messages_logic(
    pipe_name: &str,
    conversation_id: &str,
) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new(
            "conv-msgs-1",
            "conversation.messages",
            Some(serde_json::json!({
                "conversation_id": conversation_id,
            })),
        );
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Conversation Messages Error ({}): {}", err.code, err.message))
        } else {
            let val = resp.result.unwrap_or_default();
            if let Some(arr) = val.as_array() {
                Ok(arr.clone())
            } else {
                Ok(vec![])
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (pipe_name, conversation_id);
        Err("Unsupported on this platform".to_string())
    }
}

/// Sends a prompt to Hermes Agent via Hime Core AI Orchestration and waits for completion.
pub async fn send_conversation_message_logic(
    pipe_name: &str,
    conversation_id: &str,
    prompt: &str,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new(
            "conv-send-1",
            "conversation.send_message",
            Some(serde_json::json!({
                "conversation_id": conversation_id,
                "prompt": prompt,
            })),
        );
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("AI Execution Error ({}): {}", err.code, err.message))
        } else {
            Ok(resp.result.unwrap_or_default())
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (pipe_name, conversation_id, prompt);
        Err("Unsupported on this platform".to_string())
    }
}

/// Cancels an ongoing AI generation turn via IPC.
pub async fn cancel_conversation_logic(pipe_name: &str) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new("conv-cancel-1", "conversation.cancel", None);
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("Cancel Error ({}): {}", err.code, err.message))
        } else {
            Ok(resp.result.unwrap_or_default())
        }
    }
    #[cfg(not(windows))]
    {
        let _ = pipe_name;
        Err("Unsupported on this platform".to_string())
    }
}

// -----------------------------------------------------------------------------
// Phase 10: Storage Intelligence IPC Logic
// -----------------------------------------------------------------------------

pub async fn send_rpc_request(
    pipe_name: &str,
    method: &str,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        let mut client = connect_pipe_client(pipe_name)
            .await
            .map_err(|e| format!("Hime Core is unavailable: {}", e))?;

        let req = JsonRpcRequest::new(
            format!("req-{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()).as_str(),
            method,
            params,
        );
        let payload = req
            .to_bytes()
            .map_err(|e| format!("Serialization error: {}", e))?;
        let frame =
            Frame::new(MsgType::Request, payload).map_err(|e| format!("Framing error: {}", e))?;

        frame
            .write_to(&mut client)
            .await
            .map_err(|e| format!("IPC write error: {}", e))?;

        let resp_frame = Frame::read_from(&mut client)
            .await
            .map_err(|e| format!("IPC read error: {}", e))?;
        let resp = JsonRpcResponse::from_slice(&resp_frame.payload)
            .map_err(|e| format!("JSON-RPC parse error: {}", e))?;

        if let Some(err) = resp.error {
            Err(format!("IPC Error ({}): {}", err.code, err.message))
        } else {
            Ok(resp.result.unwrap_or_default())
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (pipe_name, method, params);
        Err("Unsupported on this platform".to_string())
    }
}

pub async fn storage_get_analysis_logic(pipe_name: &str, top_n: Option<usize>) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "storage.analysis",
        Some(serde_json::json!({ "top_n": top_n.unwrap_or(10) })),
    )
    .await
}

pub async fn storage_get_duplicates_logic(pipe_name: &str) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(pipe_name, "storage.duplicates", None).await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn storage_get_operations_logic(pipe_name: &str, limit: Option<usize>) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(
        pipe_name,
        "storage.operations.list",
        Some(serde_json::json!({ "limit": limit.unwrap_or(50) })),
    )
    .await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn storage_restore_operation_logic(pipe_name: &str, original_op_id: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "storage.operations.restore",
        Some(serde_json::json!({
            "original_op_id": original_op_id,
            "actor": "desktop_user"
        })),
    )
    .await
}

pub async fn storage_get_rules_logic(pipe_name: &str) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(pipe_name, "storage.rules.list", None).await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn storage_toggle_rule_logic(pipe_name: &str, id: &str, is_active: bool) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "storage.rules.toggle",
        Some(serde_json::json!({ "id": id, "is_active": is_active })),
    )
    .await
}

pub async fn storage_search_logic(pipe_name: &str, query: &str, limit: Option<usize>) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(
        pipe_name,
        "storage.search",
        Some(serde_json::json!({ "query": query, "limit": limit.unwrap_or(20) })),
    )
    .await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn storage_scan_logic(pipe_name: &str, root_path: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "storage.scan",
        Some(serde_json::json!({ "root_path": root_path })),
    )
    .await
}
