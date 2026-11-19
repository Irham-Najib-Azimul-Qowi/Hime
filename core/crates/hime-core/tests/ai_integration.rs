use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use hime_core::ai::{ModelInfo, ProviderCredentialAdapter};
use hime_core::server::CoreServer;
use hime_ipc::protocol::JsonRpcRequest;
use hime_security::csprng::generate_token_256;
use hime_storage::Database;

#[tokio::test]
async fn test_ai_overview_status_and_models() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    // 1. Test ai.status
    let req = JsonRpcRequest::new(1, "ai.status", None);
    let resp = server.dispatch_rpc(req).await;
    assert!(resp.error.is_none());
    let res = resp.result.unwrap();
    assert_eq!(res["core_orchestration_active"], true);
    assert!(res["ib_01_status"].as_str().unwrap().contains("RESOLVED"));
    assert!(res["ib_02_status"].as_str().unwrap().contains("OPEN"));

    // 2. Test ai.models.list
    let req = JsonRpcRequest::new(2, "ai.models.list", None);
    let resp = server.dispatch_rpc(req).await;
    assert!(resp.error.is_none());
    let models: Vec<ModelInfo> = serde_json::from_value(resp.result.unwrap()).unwrap();
    assert!(!models.is_empty());
    assert!(models.iter().any(|m| m.id == "hermes-3-llama-3.1-8b"));
}

#[tokio::test]
async fn test_ai_credential_storage_and_dpapi_vault() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    // 1. Store OpenAI API key via credentials.store
    let req = JsonRpcRequest::new(
        1,
        "credentials.store",
        Some(serde_json::json!({
            "provider": "openai",
            "secret": "sk-proj-test-live-key-never-plaintext-in-9router"
        })),
    );
    let resp = server.dispatch_rpc(req).await;
    assert!(resp.error.is_none());
    assert_eq!(resp.result.unwrap()["status"], "stored");

    // 2. Query credentials.list (must be redacted, never exposing plaintext)
    let req = JsonRpcRequest::new(2, "credentials.list", None);
    let resp = server.dispatch_rpc(req).await;
    assert!(resp.error.is_none());
    let list = resp.result.unwrap();
    let arr = list.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["provider"], "openai");
    assert_eq!(arr[0]["configured"], true);
    assert_eq!(arr[0]["protected_by"], "Windows DPAPI");
    // Plaintext key MUST NOT appear anywhere in the result
    assert!(arr[0].get("secret").is_none());
}

#[tokio::test]
async fn test_ai_conversation_lifecycle_and_persistence() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    // 1. Create Conversation
    let req = JsonRpcRequest::new(
        1,
        "conversation.create",
        Some(serde_json::json!({
            "title": "Investigasi Hime Desktop",
            "model": "hermes-3-llama-3.1-8b"
        })),
    );
    let resp = server.dispatch_rpc(req).await;
    assert!(resp.error.is_none());
    let conv = resp.result.unwrap();
    let conv_id = conv["id"].as_str().unwrap().to_string();
    assert_eq!(conv["title"], "Investigasi Hime Desktop");

    // 2. Send Message through AI Orchestration
    let req = JsonRpcRequest::new(
        2,
        "conversation.send_message",
        Some(serde_json::json!({
            "conversation_id": conv_id,
            "prompt": "Bagaimana status integrasi runtime Hermes dan 9Router?"
        })),
    );
    let resp = server.dispatch_rpc(req).await;
    assert!(resp.error.is_none());
    let res = resp.result.unwrap();
    assert_eq!(res["status"], "completed");
    let answer = res["response"].as_str().unwrap();
    assert!(answer.contains("Hermes Agent"));

    // 3. Verify Messages in SQLite WAL
    let req = JsonRpcRequest::new(
        3,
        "conversation.messages",
        Some(serde_json::json!({
            "conversation_id": conv_id
        })),
    );
    let resp = server.dispatch_rpc(req).await;
    assert!(resp.error.is_none());
    let msgs = resp.result.unwrap().as_array().unwrap().clone();
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0]["role"], "user");
    assert_eq!(msgs[1]["role"], "assistant");

    // 4. Verify Conversation in list
    let req = JsonRpcRequest::new(4, "conversation.list", None);
    let resp = server.dispatch_rpc(req).await;
    assert!(resp.error.is_none());
    let convs = resp.result.unwrap().as_array().unwrap().clone();
    assert_eq!(convs.len(), 1);
    assert_eq!(convs[0]["id"], conv_id);
}

#[tokio::test]
async fn test_ai_turn_cancellation() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    let req = JsonRpcRequest::new(1, "conversation.cancel", None);
    let resp = server.dispatch_rpc(req).await;
    assert!(resp.error.is_none());
    // Since no turn is actively running, cancelled should be false
    assert_eq!(resp.result.unwrap()["cancelled"], false);
}

#[tokio::test]
async fn test_ai_provider_adapter_loopback_http_server() {
    let storage = Arc::new(Database::open_in_memory().unwrap());
    // Use an ephemeral port or test port
    let adapter = Arc::new(ProviderCredentialAdapter::new(20139, Arc::clone(&storage)));

    let addr = adapter.clone().start_server().await.unwrap();

    // 1. Test GET /v1/models
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let req = "GET /v1/models HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    stream.write_all(req.as_bytes()).await.unwrap();

    let mut response_buf = Vec::new();
    stream.read_to_end(&mut response_buf).await.unwrap();
    let resp_str = String::from_utf8_lossy(&response_buf);
    assert!(resp_str.contains("HTTP/1.1 200 OK"));
    assert!(resp_str.contains("hermes-3-llama-3.1-8b"));

    // 2. Test POST /v1/chat/completions (SSE Streaming)
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let post_body = serde_json::json!({
        "model": "hermes-3-llama-3.1-8b",
        "messages": [{"role": "user", "content": "Halo Hermes"}],
        "stream": true
    })
    .to_string();

    let req = format!(
        "POST /v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        post_body.len(),
        post_body
    );
    stream.write_all(req.as_bytes()).await.unwrap();

    let mut stream_buf = Vec::new();
    stream.read_to_end(&mut stream_buf).await.unwrap();
    let stream_str = String::from_utf8_lossy(&stream_buf);
    assert!(stream_str.contains("HTTP/1.1 200 OK"));
    assert!(stream_str.contains("text/event-stream"));
    assert!(stream_str.contains("data:"));
    assert!(stream_str.contains("[DONE]"));

    adapter.stop();
}
