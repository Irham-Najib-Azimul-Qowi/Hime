use hime_core::productivity::accountability::AccountabilityManager;
use hime_core::productivity::kanban::KanbanManager;
use hime_core::productivity::scheduler::StudyScheduler;
use hime_core::server::CoreServer;
use hime_ipc::protocol::JsonRpcRequest;
use hime_storage::{Database, ScreenTimeSessionRecord};
use std::sync::Arc;

#[test]
fn test_natural_language_task_parser_indonesian_and_english() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let km = KanbanManager::new(db);

    // 1. Indonesian urgent task with deadline
    let (title1, priority1, due1) = km.parse_task_natural_language("Saya punya tugas makalah metode penelitian deadline Jumat penting");
    assert!(title1.contains("makalah metode penelitian"));
    assert_eq!(priority1, "high");
    assert!(due1.is_some());

    // 2. Critical bug fix tomorrow
    let (title2, priority2, due2) = km.parse_task_natural_language("Tolong buat tugas hotfix memory leak kritis besok");
    assert!(title2.contains("hotfix memory leak"));
    assert_eq!(priority2, "critical");
    assert!(due2.is_some());

    // 3. Relaxed low-priority task
    let (title3, priority3, due3) = km.parse_task_natural_language("Catat tugas baca buku desain UI santai");
    assert!(title3.contains("baca buku desain UI"));
    assert_eq!(priority3, "low");
    assert!(due3.is_none());
}

#[test]
fn test_subtask_breakdown_proposals() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let km = KanbanManager::new(db);

    // Academic paper breakdown
    let academic_subtasks = km.propose_subtasks("Penulisan Bab 2 Tinjauan Pustaka Skripsi");
    assert_eq!(academic_subtasks.len(), 4);
    assert!(academic_subtasks.iter().any(|(t, _)| t.contains("literatur")));

    // Software engineering breakdown
    let coding_subtasks = km.propose_subtasks("Implementasi fitur autentikasi biometrik");
    assert_eq!(coding_subtasks.len(), 4);
    assert!(coding_subtasks.iter().any(|(t, _)| t.contains("test")));
}

#[test]
fn test_accountability_modes_and_screen_time_nudge() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let am = AccountabilityManager::new(db.clone());

    // 1. Initial default mode is balanced
    let init_settings = am.get_settings().unwrap();
    assert_eq!(init_settings.mode, "balanced");
    assert!(!init_settings.screen_time_opt_in);

    // 2. Check-in in balanced mode
    let greeting = am.generate_startup_checkin(3).unwrap();
    assert_eq!(greeting.mode, "balanced");
    assert_eq!(greeting.priority_tasks_count, 3);
    assert!(greeting.can_snooze); // Non-punitive: Hime menagih, bukan mengunci

    // 3. Switch to Strict mode with screen time opt-in
    let strict_settings = am.set_mode("strict").unwrap();
    assert_eq!(strict_settings.mode, "strict");
    let opted_in = am.set_screen_time_opt_in(true).unwrap();
    assert!(opted_in.screen_time_opt_in);

    // 4. Distraction nudge evaluation
    let sessions = vec![
        ScreenTimeSessionRecord {
            id: "s1".to_string(),
            app_name: "YouTube.exe".to_string(),
            window_title: Some("Cat Videos".to_string()),
            category: "distracting".to_string(),
            duration_secs: 700, // > 10 min threshold in strict mode
            timestamp: 1000,
            date_str: None,
        },
    ];

    let nudge = am.evaluate_distraction_nudge(&sessions, Some("task-target-01".to_string())).unwrap();
    assert!(nudge.triggered);
    assert_eq!(nudge.actionable_task_id, Some("task-target-01".to_string()));
    assert!(nudge.message.contains("fokus"));
}

#[test]
fn test_study_scheduler_deterministic_collision_avoidance() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let scheduler = StudyScheduler::new(db.clone());

    // Session 1: 10:00 to 11:30 (90 min)
    let s1 = scheduler.plan_study_session("sched-1", "Aljabar Linear", "Math", 36000, 90, "high").unwrap();
    assert_eq!(s1.start_time, 36000);
    assert_eq!(s1.end_time, 36000 + (90 * 60));

    // Session 2: Tries to schedule at 10:30 (overlap) -> automatically shifted after s1 + 10 min break
    let s2 = scheduler.plan_study_session("sched-2", "Pemrograman Web", "CS", 37800, 60, "medium").unwrap();
    assert!(s2.start_time >= s1.end_time);
    assert_eq!(s2.start_time, s1.end_time + 600); // 10 min break
}

#[tokio::test]
async fn test_productivity_core_server_rpc_endpoints() {
    let server = CoreServer::new_in_memory("test-token".to_string()).unwrap();

    // 1. Get board & columns
    let req_board = JsonRpcRequest::new("rpc-board", "productivity.kanban.board.get", None);
    let resp_board = server.dispatch_rpc(req_board).await;
    assert!(resp_board.error.is_none());
    let board_val = resp_board.result.unwrap();
    assert!(board_val["board"]["id"].is_string());
    assert!(!board_val["columns"].as_array().unwrap().is_empty());

    // 2. Create task via RPC
    let req_create = JsonRpcRequest::new(
        "rpc-create",
        "productivity.tasks.create",
        Some(serde_json::json!({
            "title": "Analisis Kompleksitas Algoritma",
            "priority": "high",
            "estimated_minutes": 90
        })),
    );
    let resp_create = server.dispatch_rpc(req_create).await;
    assert!(resp_create.error.is_none());
    let task = resp_create.result.unwrap();
    let task_id = task["id"].as_str().unwrap();

    // 3. Move task to In Progress
    let req_move = JsonRpcRequest::new(
        "rpc-move",
        "productivity.tasks.move",
        Some(serde_json::json!({
            "task_id": task_id,
            "column_id": "col-in-progress",
            "status": "in_progress"
        })),
    );
    let resp_move = server.dispatch_rpc(req_move).await;
    assert!(resp_move.error.is_none());
    assert_eq!(resp_move.result.unwrap()["updated"], true);

    // 4. Set accountability mode
    let req_mode = JsonRpcRequest::new(
        "rpc-mode",
        "productivity.accountability.set_mode",
        Some(serde_json::json!({ "mode": "strict" })),
    );
    let resp_mode = server.dispatch_rpc(req_mode).await;
    assert!(resp_mode.error.is_none());
    assert_eq!(resp_mode.result.unwrap()["mode"], "strict");
}
