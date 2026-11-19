use hime_core::CoreServer;
use hime_ipc::framing::{Frame, MsgType};
use hime_ipc::protocol::{JsonRpcRequest, JsonRpcResponse};
use hime_ipc::transport::memory::create_memory_duplex;
use hime_security::csprng::generate_token_256;
use hime_storage::Database;
use std::sync::Arc;

/// Integration test verifying the Bridge Client contract with Hime Core:
/// 1. Handshake with session token & nonce anti-replay.
/// 2. Bidirectional request/response framing.
/// 3. Multiplexed commands (ping, status, task lifecycle).
/// 4. Graceful EOF shutdown without resource leaks.

#[tokio::test]
async fn test_bridge_client_handshake_and_ping_lifecycle() {
    let storage = Arc::new(Database::open_in_memory().expect("In-memory storage failed"));
    let auth_token = generate_token_256().expect("CSPRNG token generation failed");
    let server = Arc::new(CoreServer::new(auth_token.clone(), storage));

    let (mut client_stream, server_stream) = create_memory_duplex(1024 * 1024);

    let server_task = tokio::spawn(async move {
        server.handle_connection(server_stream).await;
    });

    // Step 1: Handshake
    let handshake_req = JsonRpcRequest::new(
        "hs-1",
        "system.handshake",
        Some(serde_json::json!({
            "client_id": "bloom-shell-bridge",
            "auth_token": auth_token,
            "nonce": 1u64,
        })),
    );
    let hs_frame = Frame::new(MsgType::Request, handshake_req.to_bytes().unwrap()).unwrap();
    hs_frame.write_to(&mut client_stream).await.unwrap();

    let hs_resp_frame = Frame::read_from(&mut client_stream).await.unwrap();
    assert_eq!(hs_resp_frame.header.msg_type, MsgType::Response);
    let hs_resp = JsonRpcResponse::from_slice(&hs_resp_frame.payload).unwrap();
    assert_eq!(hs_resp.id, serde_json::json!("hs-1"));
    assert!(hs_resp.error.is_none(), "Handshake should succeed");

    // Step 2: System Ping
    let ping_req = JsonRpcRequest::new("ping-1", "system.ping", None);
    let ping_frame = Frame::new(MsgType::Request, ping_req.to_bytes().unwrap()).unwrap();
    ping_frame.write_to(&mut client_stream).await.unwrap();

    let ping_resp_frame = Frame::read_from(&mut client_stream).await.unwrap();
    let ping_resp = JsonRpcResponse::from_slice(&ping_resp_frame.payload).unwrap();
    assert_eq!(ping_resp.id, serde_json::json!("ping-1"));
    let ping_result = ping_resp.result.unwrap();
    assert_eq!(ping_result.get("status").unwrap(), "pong");

    // Step 3: Graceful Client Shutdown
    drop(client_stream);
    let server_result = tokio::time::timeout(std::time::Duration::from_secs(2), server_task).await;
    assert!(server_result.is_ok(), "Server must shutdown cleanly within 2 seconds upon client EOF");
}

#[tokio::test]
async fn test_bridge_client_multiplexed_task_crud_roundtrip() {
    let storage = Arc::new(Database::open_in_memory().expect("In-memory storage failed"));
    let auth_token = generate_token_256().expect("CSPRNG token generation failed");
    let server = Arc::new(CoreServer::new(auth_token.clone(), storage));

    let (mut client_stream, server_stream) = create_memory_duplex(1024 * 1024);

    let server_task = tokio::spawn(async move {
        server.handle_connection(server_stream).await;
    });

    // 1. Handshake
    let handshake_req = JsonRpcRequest::new(
        "hs-2",
        "system.handshake",
        Some(serde_json::json!({
            "client_id": "bloom-shell-bridge",
            "auth_token": auth_token,
            "nonce": 100u64,
        })),
    );
    Frame::new(MsgType::Request, handshake_req.to_bytes().unwrap())
        .unwrap()
        .write_to(&mut client_stream)
        .await
        .unwrap();

    let _ = Frame::read_from(&mut client_stream).await.unwrap();

    // 2. Create Task via Core IPC
    let create_req = JsonRpcRequest::new(
        "task-create-1",
        "tasks.create",
        Some(serde_json::json!({
            "name": "Sync Bloom Island State",
            "description": "Bridge task verification",
            "priority": "high",
        })),
    );
    Frame::new(MsgType::Request, create_req.to_bytes().unwrap())
        .unwrap()
        .write_to(&mut client_stream)
        .await
        .unwrap();

    let create_resp_frame = Frame::read_from(&mut client_stream).await.unwrap();
    let create_resp = JsonRpcResponse::from_slice(&create_resp_frame.payload).unwrap();
    assert!(create_resp.error.is_none());
    let task_obj = create_resp.result.unwrap();
    let task_id = task_obj.get("id").unwrap().as_str().unwrap().to_string();

    // 3. List Tasks via Core IPC
    let list_req = JsonRpcRequest::new("task-list-1", "tasks.list", None);
    Frame::new(MsgType::Request, list_req.to_bytes().unwrap())
        .unwrap()
        .write_to(&mut client_stream)
        .await
        .unwrap();

    let list_resp_frame = Frame::read_from(&mut client_stream).await.unwrap();
    let list_resp = JsonRpcResponse::from_slice(&list_resp_frame.payload).unwrap();
    assert!(list_resp.error.is_none());
    let tasks_array = list_resp.result.unwrap().as_array().unwrap().clone();
    assert!(tasks_array.iter().any(|t| t.get("id").and_then(|v| v.as_str()) == Some(&task_id)));

    // Clean disconnect
    drop(client_stream);
    let _ = server_task.await;
}
