use hime_core::ai::resource::{ResourceManager, ResourceMode, SaveMode};
use hime_core::server::CoreServer;
use hime_ipc::framing::{Frame, MsgType};
use hime_ipc::protocol::{JsonRpcRequest, JsonRpcResponse};
use hime_storage::Database;

#[tokio::test]
async fn test_resource_manager_unit_invariants() {
    let rm = ResourceManager::new(ResourceMode::Balanced);

    // 1. Initial defaults
    assert_eq!(rm.get_mode(), ResourceMode::Balanced);
    assert_eq!(rm.get_save_mode(), SaveMode::Auto);
    assert_eq!(rm.ai_idle_timeout_seconds(), 600); // 10 minutes normal
    assert_eq!(rm.target_budget_mb(), 350); // <= 350 MB normal idle target

    // 2. Mode Hemat activation
    rm.set_save_mode(SaveMode::On);
    assert!(rm.is_save_mode_active());
    assert_eq!(rm.ai_idle_timeout_seconds(), 180); // 3 minutes Mode Hemat
    assert_eq!(rm.target_budget_mb(), 250); // <= 250 MB Mode Hemat target
    assert_eq!(rm.max_concurrent_tasks(), 1);

    // 3. AI runtime stop and free
    rm.touch_ai_activity();
    assert_eq!(rm.get_ai_runtime_status(), "running");
    rm.mark_ai_idle();
    assert_eq!(rm.get_ai_runtime_status(), "idle");
    assert!(rm.stop_ai_runtime());
    assert_eq!(rm.get_ai_runtime_status(), "stopped");
    assert_eq!(rm.get_ai_idle_seconds(), 0);

    // 4. Working set calculation adheres to budget
    let metrics = rm.get_process_metrics();
    assert!(metrics.within_budget);
    assert!(metrics.total_working_set_mb <= 250.0);
}

#[tokio::test]
async fn test_sqlite_cache_tuning_pragmas_verification() {
    let db = Database::open_in_memory().unwrap();
    let (cache_size, mmap_size) = db.get_resource_tuning_pragmas().unwrap();

    assert_eq!(cache_size, -2000, "cache_size must be -2000 (~2MB)");
    assert_eq!(mmap_size, 0, "mmap_size must be 0");
}

#[tokio::test]
async fn test_resource_rpc_endpoints() {
    let server = CoreServer::new_in_memory("token-resource-test".to_string()).unwrap();

    let (client_stream, server_stream) = tokio::io::duplex(8192);
    tokio::spawn(async move {
        server.handle_connection(server_stream).await;
    });

    let (mut client_reader, mut client_writer) = tokio::io::split(client_stream);

    // 1. Query resource metrics
    let req1 = JsonRpcRequest::new("req-res-1", "resource.metrics.get", None);
    let payload1 = req1.to_bytes().unwrap();
    Frame::new(MsgType::Request, payload1)
        .unwrap()
        .write_to(&mut client_writer)
        .await
        .unwrap();

    let resp_frame1 = Frame::read_from(&mut client_reader).await.unwrap();
    let resp1 = JsonRpcResponse::from_slice(&resp_frame1.payload).unwrap();
    assert!(resp1.result.is_some());
    let res1 = resp1.result.unwrap();
    assert!(res1["processes"].is_array());
    assert!(res1["total_working_set_mb"].as_f64().unwrap() <= 350.0);
    assert_eq!(res1["target_budget_mb"], 350);
    assert_eq!(res1["within_budget"], true);

    // 2. Set Save Mode to 'on' (Mode Hemat)
    let req2 = JsonRpcRequest::new(
        "req-res-2",
        "resource.save_mode.set",
        Some(serde_json::json!({
            "mode": "on"
        })),
    );
    let payload2 = req2.to_bytes().unwrap();
    Frame::new(MsgType::Request, payload2)
        .unwrap()
        .write_to(&mut client_writer)
        .await
        .unwrap();

    let resp_frame2 = Frame::read_from(&mut client_reader).await.unwrap();
    let resp2 = JsonRpcResponse::from_slice(&resp_frame2.payload).unwrap();
    assert!(resp2.result.is_some());
    let res2 = resp2.result.unwrap();
    assert_eq!(res2["save_mode"], "on");
    assert_eq!(res2["is_active"], true);
    assert_eq!(res2["ai_idle_timeout_seconds"], 180);
    assert_eq!(res2["target_budget_mb"], 250);

    // 3. Query metrics again under Mode Hemat
    let req3 = JsonRpcRequest::new("req-res-3", "resource.metrics.get", None);
    let payload3 = req3.to_bytes().unwrap();
    Frame::new(MsgType::Request, payload3)
        .unwrap()
        .write_to(&mut client_writer)
        .await
        .unwrap();

    let resp_frame3 = Frame::read_from(&mut client_reader).await.unwrap();
    let resp3 = JsonRpcResponse::from_slice(&resp_frame3.payload).unwrap();
    assert!(resp3.result.is_some());
    let res3 = resp3.result.unwrap();
    assert_eq!(res3["save_mode"], "on");
    assert_eq!(res3["target_budget_mb"], 250);
    assert!(res3["total_working_set_mb"].as_f64().unwrap() <= 250.0);
    assert_eq!(res3["within_budget"], true);

    // 4. Stop AI Runtime
    let req4 = JsonRpcRequest::new("req-res-4", "resource.ai_runtime.stop", None);
    let payload4 = req4.to_bytes().unwrap();
    Frame::new(MsgType::Request, payload4)
        .unwrap()
        .write_to(&mut client_writer)
        .await
        .unwrap();

    let resp_frame4 = Frame::read_from(&mut client_reader).await.unwrap();
    let resp4 = JsonRpcResponse::from_slice(&resp_frame4.payload).unwrap();
    assert!(resp4.result.is_some());
    let res4 = resp4.result.unwrap();
    assert_eq!(res4["success"], true);
    assert_eq!(res4["status"], "stopped");
    assert_eq!(res4["freed"], true);
}
