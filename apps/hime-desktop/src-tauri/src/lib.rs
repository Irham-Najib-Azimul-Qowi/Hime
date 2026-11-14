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

pub async fn devices_list_logic(pipe_name: &str) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(pipe_name, "devices.list", None).await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn devices_pairing_start_logic(pipe_name: &str, ttl_secs: Option<u64>) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "devices.pairing.start",
        Some(serde_json::json!({ "ttl_secs": ttl_secs.unwrap_or(300) })),
    )
    .await
}

pub async fn devices_revoke_logic(pipe_name: &str, device_id: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "devices.revoke",
        Some(serde_json::json!({ "device_id": device_id })),
    )
    .await
}

pub async fn devices_storage_get_logic(pipe_name: &str, device_id: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "devices.storage.get",
        Some(serde_json::json!({ "device_id": device_id })),
    )
    .await
}

pub async fn devices_transfers_list_logic(pipe_name: &str, device_id: Option<&str>) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(
        pipe_name,
        "devices.transfers.list",
        Some(serde_json::json!({ "device_id": device_id })),
    )
    .await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

// =========================================================================
// Phase 12 Productivity & Documents Logic
// =========================================================================

pub async fn kanban_board_get_logic(pipe_name: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(pipe_name, "productivity.kanban.board.get", None).await
}

pub async fn kanban_columns_list_logic(pipe_name: &str, board_id: &str) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(
        pipe_name,
        "productivity.kanban.columns.list",
        Some(serde_json::json!({ "board_id": board_id })),
    )
    .await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn productivity_tasks_create_logic(
    pipe_name: &str,
    board_id: &str,
    column_id: &str,
    title: &str,
    description: Option<&str>,
    priority: Option<&str>,
    due_date: Option<i64>,
    tags: Option<&str>,
    subtasks: Option<&str>,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "productivity.tasks.create",
        Some(serde_json::json!({
            "board_id": board_id,
            "column_id": column_id,
            "title": title,
            "description": description,
            "priority": priority.unwrap_or("medium"),
            "due_date": due_date,
            "tags": tags,
            "subtasks": subtasks,
        })),
    )
    .await
}

pub async fn productivity_tasks_list_logic(
    pipe_name: &str,
    board_id: Option<&str>,
    column_id: Option<&str>,
) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(
        pipe_name,
        "productivity.tasks.list",
        Some(serde_json::json!({
            "board_id": board_id,
            "column_id": column_id,
        })),
    )
    .await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn productivity_tasks_move_logic(
    pipe_name: &str,
    id: &str,
    column_id: &str,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "productivity.tasks.move",
        Some(serde_json::json!({ "id": id, "column_id": column_id })),
    )
    .await
}

pub async fn productivity_tasks_parse_natural_logic(
    pipe_name: &str,
    text: &str,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "productivity.tasks.parse_natural",
        Some(serde_json::json!({ "text": text })),
    )
    .await
}

pub async fn productivity_schedules_plan_logic(
    pipe_name: &str,
    title: &str,
    subject: &str,
    start_time: i64,
    duration_minutes: i64,
    priority: Option<&str>,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "productivity.schedules.plan",
        Some(serde_json::json!({
            "title": title,
            "subject": subject,
            "start_time": start_time,
            "duration_minutes": duration_minutes,
            "priority": priority.unwrap_or("medium"),
        })),
    )
    .await
}

pub async fn productivity_schedules_list_logic(
    pipe_name: &str,
    is_completed: Option<bool>,
) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(
        pipe_name,
        "productivity.schedules.list",
        Some(serde_json::json!({ "is_completed": is_completed })),
    )
    .await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn productivity_accountability_get_logic(pipe_name: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(pipe_name, "productivity.accountability.get", None).await
}

pub async fn productivity_accountability_set_mode_logic(
    pipe_name: &str,
    mode: &str,
    opt_in_screen_time: Option<bool>,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "productivity.accountability.set_mode",
        Some(serde_json::json!({
            "mode": mode,
            "opt_in_screen_time": opt_in_screen_time.unwrap_or(false),
        })),
    )
    .await
}

pub async fn productivity_accountability_checkin_logic(pipe_name: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(pipe_name, "productivity.accountability.checkin", None).await
}

pub async fn documents_create_report_logic(
    pipe_name: &str,
    template_id: &str,
    title: &str,
    author: &str,
    institution: Option<&str>,
    abstract_text: Option<&str>,
    sections: Vec<serde_json::Value>,
    target_format: Option<&str>,
    output_path: Option<&str>,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "documents.create_report",
        Some(serde_json::json!({
            "template_id": template_id,
            "title": title,
            "author": author,
            "institution": institution,
            "abstract_text": abstract_text,
            "sections": sections,
            "target_format": target_format.unwrap_or("docx"),
            "output_path": output_path,
        })),
    )
    .await
}

pub async fn documents_merge_pdfs_logic(
    pipe_name: &str,
    input_paths: Vec<String>,
    output_path: Option<&str>,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "documents.merge_pdfs",
        Some(serde_json::json!({
            "input_paths": input_paths,
            "output_path": output_path,
        })),
    )
    .await
}

pub async fn documents_jobs_list_logic(
    pipe_name: &str,
    limit: Option<usize>,
) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(
        pipe_name,
        "documents.jobs.list",
        Some(serde_json::json!({ "limit": limit.unwrap_or(20) })),
    )
    .await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn documents_artcraft_discover_logic(pipe_name: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(pipe_name, "documents.artcraft.discover", None).await
}

// ---------------------------------------------------------------------------
// Phase 13: Browser, Integrations & Developer Workspace Logic Functions
// ---------------------------------------------------------------------------

pub async fn browser_open_logic(pipe_name: &str, url: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(pipe_name, "browser.open", Some(serde_json::json!({ "url": url }))).await
}

pub async fn browser_extract_logic(pipe_name: &str, session_id: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(pipe_name, "browser.extract", Some(serde_json::json!({ "session_id": session_id }))).await
}

pub async fn browser_download_logic(pipe_name: &str, url: &str, target_path: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "browser.download",
        Some(serde_json::json!({ "url": url, "target_path": target_path })),
    )
    .await
}

pub async fn browser_close_logic(pipe_name: &str, session_id: &str) -> Result<serde_json::Value, String> {
    send_rpc_request(pipe_name, "browser.close", Some(serde_json::json!({ "session_id": session_id }))).await
}

pub async fn browser_history_list_logic(
    pipe_name: &str,
    limit: Option<usize>,
) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(
        pipe_name,
        "browser.history.list",
        Some(serde_json::json!({ "limit": limit.unwrap_or(50) })),
    )
    .await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn browser_downloads_list_logic(
    pipe_name: &str,
    limit: Option<usize>,
) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(
        pipe_name,
        "browser.downloads.list",
        Some(serde_json::json!({ "limit": limit.unwrap_or(50) })),
    )
    .await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn integrations_github_inspect_logic(
    pipe_name: &str,
    owner: &str,
    repo: &str,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "integrations.github.inspect",
        Some(serde_json::json!({ "owner": owner, "repo": repo })),
    )
    .await
}

pub async fn integrations_github_issues_list_logic(
    pipe_name: &str,
    owner: &str,
    repo: &str,
    state: Option<&str>,
) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(
        pipe_name,
        "integrations.github.issues.list",
        Some(serde_json::json!({ "owner": owner, "repo": repo, "state": state.unwrap_or("open") })),
    )
    .await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn integrations_github_issues_create_logic(
    pipe_name: &str,
    owner: &str,
    repo: &str,
    title: &str,
    body: &str,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "integrations.github.issues.create",
        Some(serde_json::json!({ "owner": owner, "repo": repo, "title": title, "body": body })),
    )
    .await
}

pub async fn integrations_github_pr_create_logic(
    pipe_name: &str,
    owner: &str,
    repo: &str,
    title: &str,
    head: &str,
    base: &str,
    body: &str,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "integrations.github.pr.create",
        Some(serde_json::json!({
            "owner": owner,
            "repo": repo,
            "title": title,
            "head": head,
            "base": base,
            "body": body,
        })),
    )
    .await
}

pub async fn integrations_drive_files_list_logic(
    pipe_name: &str,
    parent_id: Option<&str>,
) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(
        pipe_name,
        "integrations.drive.files.list",
        Some(serde_json::json!({ "parent_id": parent_id })),
    )
    .await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn integrations_drive_files_upload_logic(
    pipe_name: &str,
    name: &str,
    local_path: &str,
    mime_type: Option<&str>,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "integrations.drive.files.upload",
        Some(serde_json::json!({
            "name": name,
            "local_path": local_path,
            "mime_type": mime_type.unwrap_or("application/octet-stream"),
        })),
    )
    .await
}

pub async fn integrations_drive_files_share_logic(
    pipe_name: &str,
    file_id: &str,
    email: &str,
    role: Option<&str>,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "integrations.drive.files.share",
        Some(serde_json::json!({
            "file_id": file_id,
            "email": email,
            "role": role.unwrap_or("reader"),
        })),
    )
    .await
}

pub async fn integrations_accounts_list_logic(pipe_name: &str) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(pipe_name, "integrations.accounts.list", None).await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn developer_workspaces_register_logic(
    pipe_name: &str,
    name: &str,
    root_path: &str,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "developer.workspaces.register",
        Some(serde_json::json!({ "name": name, "root_path": root_path })),
    )
    .await
}

pub async fn developer_workspaces_list_logic(pipe_name: &str) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(pipe_name, "developer.workspaces.list", None).await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn developer_command_run_logic(
    pipe_name: &str,
    workspace_id: &str,
    command: &str,
    args: Vec<String>,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "developer.command.run",
        Some(serde_json::json!({
            "workspace_id": workspace_id,
            "command": command,
            "args": args,
        })),
    )
    .await
}

pub async fn developer_git_status_logic(
    pipe_name: &str,
    workspace_id: &str,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "developer.git.status",
        Some(serde_json::json!({ "workspace_id": workspace_id })),
    )
    .await
}

pub async fn developer_git_diff_logic(
    pipe_name: &str,
    workspace_id: &str,
    staged_only: Option<bool>,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "developer.git.diff",
        Some(serde_json::json!({
            "workspace_id": workspace_id,
            "staged_only": staged_only.unwrap_or(false),
        })),
    )
    .await
}

pub async fn developer_git_commit_logic(
    pipe_name: &str,
    workspace_id: &str,
    message: &str,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "developer.git.commit",
        Some(serde_json::json!({
            "workspace_id": workspace_id,
            "message": message,
        })),
    )
    .await
}

pub async fn developer_templates_list_logic(pipe_name: &str) -> Result<Vec<serde_json::Value>, String> {
    let val = send_rpc_request(pipe_name, "developer.templates.list", None).await?;
    Ok(val.as_array().cloned().unwrap_or_default())
}

pub async fn developer_templates_render_logic(
    pipe_name: &str,
    template_id: &str,
    project_name: &str,
    author: &str,
) -> Result<serde_json::Value, String> {
    send_rpc_request(
        pipe_name,
        "developer.templates.render",
        Some(serde_json::json!({
            "template_id": template_id,
            "project_name": project_name,
            "author": author,
        })),
    )
    .await
}
