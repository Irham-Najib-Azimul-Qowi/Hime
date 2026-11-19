use std::sync::Arc;
use hime_core::ai::specialists::{DataAiEngine, DelegationEngine, SpecialistRegistry};
use hime_core::ai::proactive::ProactiveEngine;
use hime_core::ai::evaluation::EvaluationHarness;
use hime_core::server::CoreServer;
use hime_ipc::framing::{Frame, MsgType};
use hime_ipc::protocol::{JsonRpcRequest, JsonRpcResponse};
use hime_storage::Database;

#[test]
fn test_specialist_registry_initialization_and_tool_allowlist() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let registry = Arc::new(SpecialistRegistry::new(db.clone()));

    let specialists = registry.list_specialists(false);
    assert!(specialists.len() >= 6, "Expected at least 6 registered domain specialists, got {}", specialists.len());

    let se_spec = registry.get_specialist("specialist.software_engineer").expect("SE specialist exists");
    assert_eq!(se_spec.domain, "developer");
    assert!(se_spec.is_active);
    assert!(se_spec.allowed_tools.contains(&"dev.run_command".to_string()));

    let delegation = DelegationEngine::new(db, registry);
    assert!(delegation.is_tool_allowed_for_specialist("specialist.software_engineer", "dev.run_command"));
    assert!(delegation.is_tool_allowed_for_specialist("specialist.software_engineer", "dev.git_status"));
    // Unallowlisted tool must be blocked
    assert!(!delegation.is_tool_allowed_for_specialist("specialist.software_engineer", "storage.format_disk"));
    assert!(!delegation.is_tool_allowed_for_specialist("specialist.software_engineer", "unauthorized.tool"));
}

#[test]
fn test_delegation_engine_lifecycle_and_evidence_check() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let registry = Arc::new(SpecialistRegistry::new(db.clone()));
    let delegation = DelegationEngine::new(db.clone(), registry);

    let res = delegation.delegate_task(
        "specialist.software_engineer",
        "task-test-01",
        "Periksa batas direktori dan jalankan pengujian",
    ).expect("Delegation succeeded");

    assert_eq!(res.status, "completed");
    assert_eq!(res.specialist_id, "specialist.software_engineer");
    assert_eq!(res.task_id, "task-test-01");
    assert!(!res.evidence.is_empty(), "Evidence checklist must not be empty");
    assert!(res.tokens_used > 0);

    // Verify stored delegation record
    let history = delegation.list_delegations(Some("specialist.software_engineer"), 10);
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].delegation_id, res.delegation_id);
}

#[test]
fn test_data_ai_profiler_and_anti_leakage_experiment() {
    let engine = DataAiEngine::new();

    let csv_sample = "age,income,education,default\n25,45000,bachelor,0\n38,82000,master,0\n,28000,highschool,1\n52,110000,phd,0\n41,,bachelor,0";
    let profile = engine.profile_csv(csv_sample).expect("CSV profiling succeeded");

    assert_eq!(profile.row_count, 5);
    assert_eq!(profile.column_count, 4);
    assert_eq!(profile.columns.len(), 4);

    let age_col = profile.columns.iter().find(|c| c.name == "age").expect("age column exists");
    assert_eq!(age_col.inferred_type, "numeric");
    assert_eq!(age_col.missing_count, 1);
    assert!((age_col.missing_percentage - 20.0).abs() < 0.1);
    assert_eq!(age_col.min, Some(25.0));
    assert_eq!(age_col.max, Some(52.0));

    // Experiment design
    let exp = engine.design_experiment("default", "classification", 4);
    assert_eq!(exp.target_variable, "default");
    assert_eq!(exp.problem_type, "classification");
    assert!(exp.validation_strategy.contains("Stratified 5-Fold"));
    assert!(exp.uncertainty_and_limitations.contains("confidence intervals"));
    assert!(exp.execution_constraints.contains("Intel i5"));
}

#[test]
fn test_proactive_engine_opt_in_cooldown_and_response() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let proactive = ProactiveEngine::new(db);

    let initial_settings = proactive.get_settings();
    assert!(initial_settings.is_enabled);
    assert_eq!(initial_settings.cooldown_minutes, 30);

    // Generate suggestions
    let suggestions = proactive.evaluate_and_generate_suggestions().expect("Evaluation succeeded");
    assert_eq!(suggestions.len(), 1);
    let sug_id = suggestions[0].id.clone();

    // Respond to suggestion
    let ok = proactive.respond_suggestion(&sug_id, "accept").expect("Response processed");
    assert!(ok);

    let accepted = proactive.list_suggestions(Some("accepted"), 10);
    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted[0].id, sug_id);

    // Mute test
    proactive.update_settings(true, 30, Some(2)).expect("Mute updated");
    let after_mute = proactive.evaluate_and_generate_suggestions().expect("Evaluation while muted succeeded");
    assert!(after_mute.is_empty(), "Must not generate new suggestions while muted");
}

#[test]
fn test_evaluation_harness_scoring() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let registry = Arc::new(SpecialistRegistry::new(db.clone()));
    let delegation = Arc::new(DelegationEngine::new(db, registry.clone()));
    let harness = EvaluationHarness::new(registry.clone(), delegation);

    let report_se = harness.evaluate_specialist("specialist.software_engineer").expect("SE evaluation passed");
    assert_eq!(report_se.scenarios_evaluated, 2);
    assert_eq!(report_se.scenarios_passed, 2);
    assert_eq!(report_se.pass_rate, 100.0);
    assert!(report_se.overall_score >= 95.0);

    let report_data = harness.evaluate_specialist("specialist.data_ai").expect("Data AI evaluation passed");
    assert_eq!(report_data.scenarios_evaluated, 2);
    assert_eq!(report_data.scenarios_passed, 2);
    assert!(report_data.overall_score >= 90.0);

    // Verify registry updated score
    let se_updated = registry.get_specialist("specialist.software_engineer").unwrap();
    assert_eq!(se_updated.evaluation_score, report_se.overall_score);
}

#[tokio::test]
async fn test_core_server_phase14_rpc_endpoints() {
    let server = CoreServer::new_in_memory("test-token-phase14".to_string()).unwrap();
    let (client_stream, server_stream) = tokio::io::duplex(4096);

    tokio::spawn(async move {
        server.handle_connection(server_stream).await;
    });

    let (mut client_reader, mut client_writer) = tokio::io::split(client_stream);

    // 1. specialists.list
    let req1 = JsonRpcRequest::new("rpc-1", "specialists.list", Some(serde_json::json!({ "active_only": true })));
    let p1 = req1.to_bytes().unwrap();
    Frame::new(MsgType::Request, p1).unwrap().write_to(&mut client_writer).await.unwrap();

    let f1 = Frame::read_from(&mut client_reader).await.unwrap();
    let r1 = JsonRpcResponse::from_slice(&f1.payload).unwrap();
    assert!(r1.error.is_none());
    let specs = r1.result.unwrap();
    assert!(specs.as_array().unwrap().len() >= 6);

    // 2. specialists.delegate
    let req2 = JsonRpcRequest::new("rpc-2", "specialists.delegate", Some(serde_json::json!({
        "specialist_id": "specialist.software_engineer",
        "task_id": "task-rpc-01",
        "input_prompt": "Audit codebase boundaries"
    })));
    let p2 = req2.to_bytes().unwrap();
    Frame::new(MsgType::Request, p2).unwrap().write_to(&mut client_writer).await.unwrap();

    let f2 = Frame::read_from(&mut client_reader).await.unwrap();
    let r2 = JsonRpcResponse::from_slice(&f2.payload).unwrap();
    assert!(r2.error.is_none());
    let del_res = r2.result.unwrap();
    assert_eq!(del_res["status"], "completed");

    // 3. data_ai.profile_dataset
    let req3 = JsonRpcRequest::new("rpc-3", "data_ai.profile_dataset", Some(serde_json::json!({
        "csv_data": "colA,colB\n10,20\n30,40\n"
    })));
    let p3 = req3.to_bytes().unwrap();
    Frame::new(MsgType::Request, p3).unwrap().write_to(&mut client_writer).await.unwrap();

    let f3 = Frame::read_from(&mut client_reader).await.unwrap();
    let r3 = JsonRpcResponse::from_slice(&f3.payload).unwrap();
    assert!(r3.error.is_none());
    let prof_res = r3.result.unwrap();
    assert_eq!(prof_res["column_count"], 2);
    assert_eq!(prof_res["row_count"], 2);

    // 4. proactive.suggestions.list
    let req4 = JsonRpcRequest::new("rpc-4", "proactive.suggestions.list", None);
    let p4 = req4.to_bytes().unwrap();
    Frame::new(MsgType::Request, p4).unwrap().write_to(&mut client_writer).await.unwrap();

    let f4 = Frame::read_from(&mut client_reader).await.unwrap();
    let r4 = JsonRpcResponse::from_slice(&f4.payload).unwrap();
    assert!(r4.error.is_none());
}
