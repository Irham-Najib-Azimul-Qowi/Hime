use hime_core::{CoreServer, JobSupervisor};
use hime_ipc::framing::{Frame, MsgType};
use hime_ipc::protocol::{
    JsonRpcRequest, JsonRpcResponse, POLICY_VIOLATION, REPLAY_DETECTED, UNAUTHORIZED,
};
use hime_ipc::transport::memory::create_memory_duplex;
use hime_security::csprng::generate_token_256;
use hime_security::dpapi::{SecretVault, WindowsDpapiVault};
use hime_security::redaction::redact_sensitive_text;
use hime_storage::Database;
use std::sync::Arc;

#[tokio::test]
async fn test_end_to_end_ipc_ping_roundtrip() {
    let storage = Arc::new(Database::open_in_memory().unwrap());
    let token = generate_token_256().unwrap();
    let server = Arc::new(CoreServer::new(token.clone(), storage));

    let (mut client_stream, server_stream) = create_memory_duplex(1024 * 1024);

    let server_task = tokio::spawn(async move {
        server.handle_connection(server_stream).await;
    });

    // Client sends system.ping
    let req = JsonRpcRequest::new("req-1", "system.ping", None);
    let frame = Frame::new(MsgType::Request, req.to_bytes().unwrap()).unwrap();
    frame.write_to(&mut client_stream).await.unwrap();

    // Client reads response
    let resp_frame = Frame::read_from(&mut client_stream).await.unwrap();
    assert_eq!(resp_frame.header.msg_type, MsgType::Response);

    let resp = JsonRpcResponse::from_slice(&resp_frame.payload).unwrap();
    assert_eq!(resp.id, serde_json::json!("req-1"));
    assert!(resp.error.is_none());

    let result = resp.result.unwrap();
    assert_eq!(result.get("status").unwrap(), "pong");
    assert_eq!(result.get("core_version").unwrap(), "0.1.0");

    drop(client_stream);
    let _ = server_task.await;
}

#[tokio::test]
async fn test_handshake_authentication_and_anti_replay() {
    let storage = Arc::new(Database::open_in_memory().unwrap());
    let valid_token = generate_token_256().unwrap();
    let server = Arc::new(CoreServer::new(valid_token.clone(), storage));

    let (mut client_stream, server_stream) = create_memory_duplex(1024 * 1024);

    tokio::spawn(async move {
        server.handle_connection(server_stream).await;
    });

    // 1. Handshake with invalid token
    let bad_req = JsonRpcRequest::new(
        "auth-1",
        "system.handshake",
        Some(serde_json::json!({
            "auth_token": "wrong-token-123",
            "client_id": "test-client",
            "nonce": 1
        })),
    );
    let f1 = Frame::new(MsgType::Request, bad_req.to_bytes().unwrap()).unwrap();
    f1.write_to(&mut client_stream).await.unwrap();

    let resp_f1 = Frame::read_from(&mut client_stream).await.unwrap();
    let resp1 = JsonRpcResponse::from_slice(&resp_f1.payload).unwrap();
    assert!(resp1.error.is_some());
    assert_eq!(resp1.error.unwrap().code, UNAUTHORIZED);

    // 2. Handshake with valid token & nonce 1
    let good_req = JsonRpcRequest::new(
        "auth-2",
        "system.handshake",
        Some(serde_json::json!({
            "auth_token": valid_token,
            "client_id": "test-client",
            "nonce": 1
        })),
    );
    let f2 = Frame::new(MsgType::Request, good_req.to_bytes().unwrap()).unwrap();
    f2.write_to(&mut client_stream).await.unwrap();

    let resp_f2 = Frame::read_from(&mut client_stream).await.unwrap();
    let resp2 = JsonRpcResponse::from_slice(&resp_f2.payload).unwrap();
    assert!(resp2.error.is_none());
    let res = resp2.result.unwrap();
    assert!(res.get("session_id").is_some());
    assert!(res.get("reconnect_ticket").is_some());

    // 3. Replay attack: resending request with same nonce 1
    let replay_req = JsonRpcRequest::new(
        "auth-3",
        "system.handshake",
        Some(serde_json::json!({
            "auth_token": valid_token,
            "client_id": "test-client",
            "nonce": 1
        })),
    );
    let f3 = Frame::new(MsgType::Request, replay_req.to_bytes().unwrap()).unwrap();
    f3.write_to(&mut client_stream).await.unwrap();

    let resp_f3 = Frame::read_from(&mut client_stream).await.unwrap();
    let resp3 = JsonRpcResponse::from_slice(&resp_f3.payload).unwrap();
    assert!(resp3.error.is_some());
    assert_eq!(resp3.error.unwrap().code, REPLAY_DETECTED);
}

#[tokio::test]
async fn test_policy_enforcement_prevents_system_directory_access() {
    let storage = Arc::new(Database::open_in_memory().unwrap());
    let token = generate_token_256().unwrap();
    let server = Arc::new(CoreServer::new(token, storage));

    let (mut client_stream, server_stream) = create_memory_duplex(1024 * 1024);

    tokio::spawn(async move {
        server.handle_connection(server_stream).await;
    });

    // Attempt to write to C:\Windows\System32
    let req = JsonRpcRequest::new(
        "tool-1",
        "tool.execute",
        Some(serde_json::json!({
            "action_type": "file_write",
            "path": r"C:\Windows\System32\malicious.dll"
        })),
    );
    let frame = Frame::new(MsgType::Request, req.to_bytes().unwrap()).unwrap();
    frame.write_to(&mut client_stream).await.unwrap();

    let resp_frame = Frame::read_from(&mut client_stream).await.unwrap();
    let resp = JsonRpcResponse::from_slice(&resp_frame.payload).unwrap();
    assert!(resp.error.is_some());
    assert_eq!(resp.error.unwrap().code, POLICY_VIOLATION);
}

#[test]
#[cfg(windows)]
fn test_supervisor_job_object_spawn_and_lifecycle() {
    let supervisor = JobSupervisor::new(None).expect("Supervisor creation must succeed");
    let pid = supervisor
        .spawn_child("integration-test-worker", "cmd.exe /c ping -n 5 127.0.0.1")
        .expect("Process spawn under Job Object must succeed");
    assert!(pid > 0);

    let terminated = supervisor.terminate_child("integration-test-worker");
    assert!(terminated);
}

#[test]
#[cfg(windows)]
fn test_dpapi_secret_vault_roundtrip() {
    let vault = WindowsDpapiVault::new();
    let secret_key = b"sk-proj-super-secret-production-key-12345";
    let encrypted = vault.protect(secret_key).unwrap();
    assert_ne!(encrypted, secret_key);

    let decrypted = vault.unprotect(&encrypted).unwrap();
    assert_eq!(decrypted, secret_key);
}

#[test]
fn test_secret_redaction_filters_keys_and_cot() {
    let raw = "Log: Connected with sk-12345678901234567890abcdef! <think>Secret CoT</think>";
    let redacted = redact_sensitive_text(raw);
    assert!(!redacted.contains("sk-1234567890"));
    assert!(!redacted.contains("Secret CoT"));
    assert!(redacted.contains("[REDACTED_OPENAI_KEY]"));
    assert!(redacted.contains("[PRIVATE_COT_FILTERED]"));
}

#[tokio::test]
#[cfg(windows)]
async fn test_live_named_pipe_roundtrip() {
    use hime_ipc::transport::windows_pipe::{connect_pipe_client, create_pipe_server};

    let test_pipe_name = r"\\.\pipe\hime-test-live-pipe";
    let storage = Arc::new(Database::open_in_memory().unwrap());
    let token = generate_token_256().unwrap();
    let server = Arc::new(CoreServer::new(token.clone(), storage));

    let pipe_server = create_pipe_server(test_pipe_name, true).unwrap();
    let server_clone = server.clone();

    let server_task = tokio::spawn(async move {
        pipe_server.connect().await.unwrap();
        server_clone.handle_connection(pipe_server).await;
    });

    let mut client = connect_pipe_client(test_pipe_name).await.unwrap();

    // Send system.ping over real Windows Named Pipe
    let req = JsonRpcRequest::new("pipe-req-1", "system.ping", None);
    let frame = Frame::new(MsgType::Request, req.to_bytes().unwrap()).unwrap();
    frame.write_to(&mut client).await.unwrap();

    // Read response over real Windows Named Pipe
    let resp_frame = Frame::read_from(&mut client).await.unwrap();
    let resp = JsonRpcResponse::from_slice(&resp_frame.payload).unwrap();
    assert_eq!(resp.id, serde_json::json!("pipe-req-1"));
    assert!(resp.error.is_none());
    assert_eq!(resp.result.unwrap().get("status").unwrap(), "pong");

    // Send system.shutdown
    let shutdown_req = JsonRpcRequest::new("pipe-req-2", "system.shutdown", None);
    let shutdown_frame = Frame::new(MsgType::Request, shutdown_req.to_bytes().unwrap()).unwrap();
    shutdown_frame.write_to(&mut client).await.unwrap();

    let shutdown_resp_frame = Frame::read_from(&mut client).await.unwrap();
    let shutdown_resp = JsonRpcResponse::from_slice(&shutdown_resp_frame.payload).unwrap();
    assert_eq!(
        shutdown_resp.result.unwrap().get("status").unwrap(),
        "shutting_down"
    );

    drop(client);
    let _ = server_task.await;
}
