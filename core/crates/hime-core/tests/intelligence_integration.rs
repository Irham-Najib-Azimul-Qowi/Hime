use hime_core::server::CoreServer;
use hime_ipc::protocol::JsonRpcRequest;
use hime_security::csprng::generate_token_256;

#[tokio::test]
async fn test_intelligence_e2e_plan_execution_and_audit_hash_chain() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    // 1. Execute an orchestrated plan with 2 steps
    let steps = serde_json::json!([
        {
            "step_id": "step-01",
            "description": "Probe host environment",
            "tool_id": "system.info",
            "params": {}
        },
        {
            "step_id": "step-02",
            "description": "Create task for processing",
            "tool_id": "tasks.create",
            "params": {
                "title": "Automated Batch Task",
                "priority": "high"
            }
        }
    ]);

    let req = JsonRpcRequest::new(
        1,
        "orchestrator.execute_plan",
        Some(serde_json::json!({ "steps": steps })),
    );
    let plan_resp = server.dispatch_rpc(req).await;

    assert!(plan_resp.error.is_none());
    let summary = plan_resp.result.unwrap();
    assert_eq!(summary["total_steps"], 2);
    assert_eq!(summary["executed_steps"], 2);
    assert_eq!(summary["success"], true);

    // 2. Verify that the audit trail's SHA-256 hash chain is intact and valid
    let req_audit = JsonRpcRequest::new(2, "audit.verify", None);
    let verify_resp = server.dispatch_rpc(req_audit).await;
    assert!(verify_resp.error.is_none());
    let audit_status = verify_resp.result.unwrap();
    assert_eq!(audit_status["chain_valid"], true);
    assert_eq!(audit_status["tamper_detected"], false);
}

#[tokio::test]
async fn test_intelligence_e2e_loop_detection_fails_safely() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    // 3 identical calls to trigger quality control loop detection
    let looping_steps = serde_json::json!([
        {
            "step_id": "loop-1",
            "description": "Loop call",
            "tool_id": "system.info",
            "params": {}
        },
        {
            "step_id": "loop-2",
            "description": "Loop call",
            "tool_id": "system.info",
            "params": {}
        },
        {
            "step_id": "loop-3",
            "description": "Loop call",
            "tool_id": "system.info",
            "params": {}
        }
    ]);

    let req = JsonRpcRequest::new(
        1,
        "orchestrator.execute_plan",
        Some(serde_json::json!({ "steps": looping_steps })),
    );
    let resp = server.dispatch_rpc(req).await;

    assert!(resp.error.is_none());
    let summary = resp.result.unwrap();
    assert_eq!(summary["success"], false);
    assert_eq!(summary["loop_detected"], true);
    assert!(summary["synthesized_output"]
        .as_str()
        .unwrap()
        .contains("infinite loop"));
}

#[tokio::test]
async fn test_intelligence_e2e_tool_gateway_security_path_traversal() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    // Traversal attack
    let req = JsonRpcRequest::new(
        1,
        "tools.execute",
        Some(serde_json::json!({
            "tool_id": "files.stat",
            "params": {
                "path": "projects/../../../Windows/System32"
            }
        })),
    );
    let resp = server.dispatch_rpc(req).await;

    assert!(resp.error.is_none());
    let res = resp.result.unwrap();
    assert_eq!(res["success"], false);
    assert!(res["error"]
        .as_str()
        .unwrap()
        .contains("Security validation rejected"));
}

#[tokio::test]
async fn test_intelligence_e2e_task_engine_lifecycle() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    // 1. Create task
    let req_create = JsonRpcRequest::new(
        1,
        "task.engine.create",
        Some(serde_json::json!({
            "id": "task-e2e-01",
            "title": "Index workspace documents",
            "priority": "high"
        })),
    );
    let create_resp = server.dispatch_rpc(req_create).await;
    assert!(create_resp.error.is_none());
    let task = create_resp.result.unwrap();
    assert_eq!(task["id"], "task-e2e-01");
    assert_eq!(task["status"], "queued");

    // 2. Transition status to running
    let req_run = JsonRpcRequest::new(
        2,
        "task.engine.update_status",
        Some(serde_json::json!({
            "id": "task-e2e-01",
            "status": "running",
            "progress": 0.4
        })),
    );
    let run_resp = server.dispatch_rpc(req_run).await;
    assert!(run_resp.error.is_none());

    // 3. Transition status to succeeded
    let req_succ = JsonRpcRequest::new(
        3,
        "task.engine.update_status",
        Some(serde_json::json!({
            "id": "task-e2e-01",
            "status": "succeeded",
            "progress": 1.0
        })),
    );
    let succ_resp = server.dispatch_rpc(req_succ).await;
    assert!(succ_resp.error.is_none());

    // 4. List tasks with filter
    let req_list = JsonRpcRequest::new(
        4,
        "task.engine.list",
        Some(serde_json::json!({ "status": "succeeded" })),
    );
    let list_resp = server.dispatch_rpc(req_list).await;
    assert!(list_resp.error.is_none());
    let tasks = list_resp.result.unwrap().as_array().unwrap().clone();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"], "task-e2e-01");
}

#[tokio::test]
async fn test_intelligence_e2e_event_engine_rolling_buffer() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    // Emit events via server's AI event engine
    server
        .ai
        .event_engine
        .emit("test.event.one", serde_json::json!({"n": 1}), None)
        .await;
    server
        .ai
        .event_engine
        .emit("test.event.two", serde_json::json!({"n": 2}), None)
        .await;

    // Fetch recent events
    let req = JsonRpcRequest::new(1, "events.recent", None);
    let resp = server.dispatch_rpc(req).await;
    assert!(resp.error.is_none());
    let events = resp.result.unwrap().as_array().unwrap().clone();
    assert!(events.len() >= 2);
    assert_eq!(events[events.len() - 2]["event_type"], "test.event.one");
    assert_eq!(events[events.len() - 1]["event_type"], "test.event.two");
}

#[tokio::test]
async fn test_intelligence_e2e_memory_provenance_and_token_budget() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    // 1. Insert memory
    let req_ins = JsonRpcRequest::new(
        1,
        "memory.insert",
        Some(serde_json::json!({
            "id": "mem-e2e-01",
            "source": "conversation",
            "category": "user_preference",
            "content": "User prefers concise answers in Indonesian language",
            "relevance_score": 0.95,
            "retention_policy": "permanent"
        })),
    );
    let ins_resp = server.dispatch_rpc(req_ins).await;
    assert!(ins_resp.error.is_none());

    // 2. Query context with token budget limit
    let req_ctx = JsonRpcRequest::new(
        2,
        "memory.context",
        Some(serde_json::json!({
            "query": "Indonesian",
            "max_tokens": 100
        })),
    );
    let ctx_resp = server.dispatch_rpc(req_ctx).await;
    assert!(ctx_resp.error.is_none());
    let items = ctx_resp.result.unwrap().as_array().unwrap().clone();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "mem-e2e-01");

    // 3. Delete memory
    let req_del = JsonRpcRequest::new(
        3,
        "memory.delete",
        Some(serde_json::json!({ "id": "mem-e2e-01" })),
    );
    let del_resp = server.dispatch_rpc(req_del).await;
    assert!(del_resp.error.is_none());
    assert_eq!(del_resp.result.unwrap()["deleted"], true);
}

#[tokio::test]
async fn test_intelligence_e2e_translation_opt_in_and_privacy() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    // 1. Check opt-in status initially true (or configurable)
    let req_opt = JsonRpcRequest::new(1, "translate.get_opt_in", None);
    let get_opt = server.dispatch_rpc(req_opt).await;
    assert!(get_opt.error.is_none());

    // 2. Translate safe text
    let req_trans = JsonRpcRequest::new(
        2,
        "translate.selection",
        Some(serde_json::json!({
            "text": "hello",
            "target_lang": "id"
        })),
    );
    let trans_resp = server.dispatch_rpc(req_trans).await;
    assert!(trans_resp.error.is_none());
    assert_eq!(trans_resp.result.unwrap()["translated"], "Halo");

    // 3. Attempt to translate sensitive text containing API key
    let req_sens = JsonRpcRequest::new(
        3,
        "translate.selection",
        Some(serde_json::json!({
            "text": "Bearer sk-1234567890abcdef1234567890abcdef",
            "target_lang": "id"
        })),
    );
    let sens_resp = server.dispatch_rpc(req_sens).await;
    assert!(sens_resp.error.is_some());
    assert!(sens_resp
        .error
        .unwrap()
        .message
        .contains("sensitive credentials"));
}

#[tokio::test]
async fn test_intelligence_e2e_resource_mode_adaptation() {
    let auth_token = generate_token_256().unwrap();
    let server = CoreServer::new_in_memory(auth_token).unwrap();

    // 1. Initial Balanced mode
    let req_initial = JsonRpcRequest::new(1, "resource.get_mode", None);
    let initial = server.dispatch_rpc(req_initial).await;
    assert!(initial.error.is_none());
    assert_eq!(initial.result.unwrap()["mode"], "Balanced");

    // 2. Set to Efficiency
    let req_eff = JsonRpcRequest::new(
        2,
        "resource.set_mode",
        Some(serde_json::json!({ "mode": "efficiency" })),
    );
    let eff = server.dispatch_rpc(req_eff).await;
    assert!(eff.error.is_none());
    assert_eq!(eff.result.unwrap()["max_concurrent_tasks"], 1);

    // 3. Set to Performance
    let req_perf = JsonRpcRequest::new(
        3,
        "resource.set_mode",
        Some(serde_json::json!({ "mode": "performance" })),
    );
    let perf = server.dispatch_rpc(req_perf).await;
    assert!(perf.error.is_none());
    assert_eq!(perf.result.unwrap()["max_concurrent_tasks"], 4);
}
