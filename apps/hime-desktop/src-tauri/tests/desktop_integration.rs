use hime_core::CoreServer;
use hime_desktop::{
    create_conversation_logic, create_task_logic, execute_tool_logic, get_ai_models_logic,
    get_ai_status_logic, get_system_status_logic, ping_core_logic, resolve_approval_logic,
    send_conversation_message_logic, store_credential_logic,
};
#[cfg(windows)]
use hime_ipc::transport::windows_pipe::create_pipe_server;
use std::sync::Arc;

#[tokio::test]
#[cfg(windows)]
async fn test_desktop_to_core_live_ipc_roundtrip() {
    let test_pipe_name = r"\\.\pipe\hime-test-desktop-live";
    let server = Arc::new(CoreServer::new_in_memory("desktop-dev-token".to_string()).unwrap());

    let pipe_server =
        create_pipe_server(test_pipe_name, true).expect("Named pipe server creation must succeed");

    let server_clone = server.clone();
    let server_task = tokio::spawn(async move {
        let pipe = pipe_server;
        pipe.connect().await.unwrap();
        server_clone.handle_connection(pipe).await;
    });

    // 1. Desktop UI invokes ping_core_logic
    let ping_res = ping_core_logic(test_pipe_name).await;
    assert!(ping_res.is_ok(), "Desktop ping to Core must succeed");
    let ping_val = ping_res.unwrap();
    assert!(ping_val.success);
    assert_eq!(
        ping_val.core_response.unwrap().get("status").unwrap(),
        "pong"
    );

    let _ = server_task.await;
}

#[tokio::test]
#[cfg(windows)]
async fn test_desktop_handles_core_unavailable_gracefully() {
    let nonexistent_pipe = r"\\.\pipe\hime-nonexistent-pipe-9999";
    // Core is not running on this pipe. Desktop logic must return a descriptive error without crashing.
    let res = ping_core_logic(nonexistent_pipe).await;
    assert!(res.is_err(), "Must return Err when Core is unavailable");
    let err_msg = res.unwrap_err();
    assert!(
        err_msg.contains("Hime Core is unavailable"),
        "Error message must clearly inform that Core is unavailable, got: {}",
        err_msg
    );
}

#[tokio::test]
#[cfg(windows)]
async fn test_desktop_tool_execution_policy_boundary() {
    let test_pipe_name = r"\\.\pipe\hime-test-desktop-policy";
    let server = Arc::new(CoreServer::new_in_memory("desktop-dev-token".to_string()).unwrap());

    let pipe_server =
        create_pipe_server(test_pipe_name, true).expect("Named pipe server creation must succeed");

    let server_clone = server.clone();
    tokio::spawn(async move {
        let pipe = pipe_server;
        pipe.connect().await.unwrap();
        server_clone.handle_connection(pipe).await;
    });

    // Policy violation: write to C:\Windows\System32
    let res = execute_tool_logic(
        test_pipe_name,
        "file_write",
        r"C:\Windows\System32\blocked.dll",
    )
    .await;
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.contains("Policy Denial / Error"));
}

#[tokio::test]
#[cfg(windows)]
async fn test_desktop_system_status_flow() {
    let test_pipe_name = r"\\.\pipe\hime-test-desktop-status";
    let server = Arc::new(CoreServer::new_in_memory("desktop-dev-token".to_string()).unwrap());

    let pipe_server =
        create_pipe_server(test_pipe_name, true).expect("Named pipe server creation must succeed");

    let server_clone = server.clone();
    tokio::spawn(async move {
        let pipe = pipe_server;
        pipe.connect().await.unwrap();
        server_clone.handle_connection(pipe).await;
    });

    // Query system status
    let status_res = get_system_status_logic(test_pipe_name).await;
    assert!(status_res.is_ok(), "Querying system status must succeed");
    let status_val = status_res.unwrap();
    assert_eq!(status_val.get("core").unwrap(), "online");
    assert_eq!(status_val.get("status").unwrap(), "running");
}

#[tokio::test]
#[cfg(windows)]
async fn test_desktop_tasks_and_events_flow() {
    let test_pipe_name = r"\\.\pipe\hime-test-desktop-tasks";
    let server = Arc::new(CoreServer::new_in_memory("desktop-dev-token".to_string()).unwrap());

    let pipe_server =
        create_pipe_server(test_pipe_name, true).expect("Named pipe server creation must succeed");

    let server_clone = server.clone();
    tokio::spawn(async move {
        let pipe = pipe_server;
        pipe.connect().await.unwrap();
        server_clone.handle_connection(pipe).await;
    });

    // Create a task
    let task_res = create_task_logic(
        test_pipe_name,
        "Inspect File System",
        "Perform audit on workspace",
        "high",
    )
    .await;
    assert!(task_res.is_ok(), "Creating task must succeed");
    let task_val = task_res.unwrap();
    assert_eq!(task_val.get("name").unwrap(), "Inspect File System");
    assert_eq!(task_val.get("priority").unwrap(), "high");
}

#[tokio::test]
#[cfg(windows)]
async fn test_desktop_permission_approval_security_lifecycle() {
    let server = Arc::new(CoreServer::new_in_memory("desktop-dev-token".to_string()).unwrap());

    // 1. Tool execution requiring user approval (file_write to non-system dir)
    let pipe_1 = r"\\.\pipe\hime-test-appr-1";
    let pipe_server_1 = create_pipe_server(pipe_1, true).expect("Pipe 1 must succeed");
    let server_clone_1 = server.clone();
    tokio::spawn(async move {
        pipe_server_1.connect().await.unwrap();
        server_clone_1.handle_connection(pipe_server_1).await;
    });

    let exec_res = execute_tool_logic(pipe_1, "file_write", "workspace/artifact.log").await;
    assert!(exec_res.is_ok(), "Tool execution should trigger pending approval");
    let val = exec_res.unwrap();
    assert_eq!(val.get("decision").unwrap(), "pending_approval");
    let request_id = val.get("request_id").unwrap().as_str().unwrap().to_string();
    assert!(request_id.starts_with("appr-"));

    // 2. Non-existent approval resolution must fail-closed
    let pipe_2 = r"\\.\pipe\hime-test-appr-2";
    let pipe_server_2 = create_pipe_server(pipe_2, true).expect("Pipe 2 must succeed");
    let server_clone_2 = server.clone();
    tokio::spawn(async move {
        pipe_server_2.connect().await.unwrap();
        server_clone_2.handle_connection(pipe_server_2).await;
    });

    let fake_res = resolve_approval_logic(pipe_2, "appr-nonexistent-9999", "approve").await;
    assert!(fake_res.is_err());
    assert!(fake_res.unwrap_err().contains("not found"));

    // 3. User denies approval request
    let pipe_3 = r"\\.\pipe\hime-test-appr-3";
    let pipe_server_3 = create_pipe_server(pipe_3, true).expect("Pipe 3 must succeed");
    let server_clone_3 = server.clone();
    tokio::spawn(async move {
        pipe_server_3.connect().await.unwrap();
        server_clone_3.handle_connection(pipe_server_3).await;
    });

    let deny_res = resolve_approval_logic(pipe_3, &request_id, "deny").await;
    assert!(deny_res.is_err(), "Denial must reject operation");
    assert!(deny_res.unwrap_err().contains("Operation denied by user approval policy"));

    // 4. Denied ticket cannot be reused / approved again (Nonce / Single-use)
    let pipe_4 = r"\\.\pipe\hime-test-appr-4";
    let pipe_server_4 = create_pipe_server(pipe_4, true).expect("Pipe 4 must succeed");
    let server_clone_4 = server.clone();
    tokio::spawn(async move {
        pipe_server_4.connect().await.unwrap();
        server_clone_4.handle_connection(pipe_server_4).await;
    });

    let reuse_res = resolve_approval_logic(pipe_4, &request_id, "approve").await;
    assert!(reuse_res.is_err(), "Consumed ticket cannot be re-approved");
    assert!(reuse_res.unwrap_err().contains("already consumed or resolved"));
}

#[tokio::test]
#[cfg(windows)]
async fn test_desktop_ai_runtime_integration_flow() {
    let test_pipe_name = r"\\.\pipe\hime-test-desktop-ai";
    let server = Arc::new(CoreServer::new_in_memory("ai-test-token".to_string()).unwrap());

    // 1. Check AI Status via Desktop logic
    let pipe_server_1 = create_pipe_server(test_pipe_name, true).unwrap();
    let s1 = server.clone();
    tokio::spawn(async move {
        pipe_server_1.connect().await.unwrap();
        s1.handle_connection(pipe_server_1).await;
    });

    let status_res = get_ai_status_logic(test_pipe_name).await;
    assert!(status_res.is_ok());
    let status_val = status_res.unwrap();
    assert_eq!(status_val.get("core_orchestration_active").unwrap(), true);

    // 2. Discover Models via Desktop logic
    let pipe_2 = r"\\.\pipe\hime-test-desktop-ai-2";
    let pipe_server_2 = create_pipe_server(pipe_2, true).unwrap();
    let s2 = server.clone();
    tokio::spawn(async move {
        pipe_server_2.connect().await.unwrap();
        s2.handle_connection(pipe_server_2).await;
    });

    let models_res = get_ai_models_logic(pipe_2).await;
    assert!(models_res.is_ok());
    let models_arr = models_res.unwrap();
    assert!(!models_arr.is_empty());

    // 3. Store Credential (IB-01 DPAPI Protected) via Desktop logic
    let pipe_3 = r"\\.\pipe\hime-test-desktop-ai-3";
    let pipe_server_3 = create_pipe_server(pipe_3, true).unwrap();
    let s3 = server.clone();
    tokio::spawn(async move {
        pipe_server_3.connect().await.unwrap();
        s3.handle_connection(pipe_server_3).await;
    });

    let cred_res = store_credential_logic(pipe_3, "openai", "sk-proj-test-live-key").await;
    assert!(cred_res.is_ok());
    assert_eq!(cred_res.unwrap().get("status").unwrap(), "stored");

    // 4. Create and Chat in Conversation via Desktop logic
    let pipe_4 = r"\\.\pipe\hime-test-desktop-ai-4";
    let pipe_server_4 = create_pipe_server(pipe_4, true).unwrap();
    let s4 = server.clone();
    tokio::spawn(async move {
        pipe_server_4.connect().await.unwrap();
        s4.handle_connection(pipe_server_4).await;
    });

    let conv_res = create_conversation_logic(pipe_4, "Testing AI Chat", "hermes-3-llama-3.1-8b").await;
    assert!(conv_res.is_ok());
    let conv_id = conv_res.unwrap().get("id").unwrap().as_str().unwrap().to_string();

    let pipe_5 = r"\\.\pipe\hime-test-desktop-ai-5";
    let pipe_server_5 = create_pipe_server(pipe_5, true).unwrap();
    let s5 = server.clone();
    tokio::spawn(async move {
        pipe_server_5.connect().await.unwrap();
        s5.handle_connection(pipe_server_5).await;
    });

    let msg_res = send_conversation_message_logic(pipe_5, &conv_id, "Tes respon Hermes").await;
    assert!(msg_res.is_ok());
    let msg_val = msg_res.unwrap();
    assert_eq!(msg_val.get("status").unwrap(), "completed");
    assert!(msg_val.get("response").unwrap().as_str().unwrap().contains("Hermes Agent"));
}
