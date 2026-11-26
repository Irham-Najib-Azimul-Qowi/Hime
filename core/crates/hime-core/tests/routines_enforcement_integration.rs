use hime_core::enforce::{Commitment, EnforcementAction, EnforcementManager};
use hime_core::reminders::{NextWakeEventKind, ReminderScheduler};
use hime_core::routines::models::{
    AutoEscalationConfig, DistractionRules, EnforcementTier, Routine, RoutineSchedule, RoutineStreak,
    Weekday,
};
use hime_core::routines::session::{RoutineSession, RoutineSessionState};
use hime_core::routines::streak::{
    apply_streak_completion, apply_streak_excused_skip, apply_streak_miss, date_str_to_days,
    days_to_date_str,
};
use hime_core::routines::RoutineManager;
use hime_core::server::CoreServer;
use hime_ipc::protocol::JsonRpcRequest;
use hime_storage::Database;
use std::sync::Arc;

#[tokio::test]
async fn test_routine_lifecycle_and_state_machine() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let rm = Arc::new(RoutineManager::new(db));

    let routine = Routine {
        id: "rtn-test-01".to_string(),
        title: "Belajar Rust Systems".to_string(),
        category: "belajar".to_string(),
        schedule: RoutineSchedule {
            days: vec![Weekday::Monday, Weekday::Wednesday, Weekday::Friday],
            time: "19:00".to_string(),
            duration_min: 45,
            window_min: Some(15),
        },
        enforcement: EnforcementTier::Tegas,
        allowed_apps: vec!["Code.exe".to_string()],
        distraction_rules: DistractionRules {
            block_during_routine: true,
            daily_quota_min: Some(60),
            sequential_after_focus_min: None,
            distraction_apps: vec!["YouTube.exe".to_string(), "Discord.exe".to_string()],
        },
        start_grace_min: 5,
        escalation: Some(AutoEscalationConfig {
            enabled: true,
            missed_threshold: 3,
        }),
        streak: RoutineStreak::default(),
        active: true,
        created_at: 1000,
        updated_at: 1000,
    };

    rm.create_routine(routine.clone()).unwrap();

    // 1. Start Session
    let session = rm.start_session("rtn-test-01", Some(2000), 2000).await.unwrap();
    assert_eq!(session.state, RoutineSessionState::Active);
    assert_eq!(session.started_at, Some(2000));
    assert_eq!(session.max_snooze_count, Some(3)); // Tegas caps to 3

    // 2. Complete Session -> increments streak
    let (completed_sess, streak) = rm.complete_session(&session.id, 2000 + 45 * 60).await.unwrap();
    assert_eq!(completed_sess.state, RoutineSessionState::Completed);
    assert_eq!(streak.current_count, 1);
    assert_eq!(streak.best_count, 1);

    // 3. Snooze Session in Tegas (bounded by max_snooze_count)
    let mut sess2 = RoutineSession::new("rtn-test-01", 3000, 45, 5, Some(2));
    assert_eq!(sess2.state, RoutineSessionState::Prompted);

    // First snooze: 10 min with reason pill "Sibuk"
    sess2.snooze(10, Some("Sibuk".to_string()), 3000).unwrap();
    assert_eq!(sess2.state, RoutineSessionState::Snoozed);
    assert_eq!(sess2.snooze_count, 1);
    assert_eq!(sess2.snooze_until, Some(3000 + 600));

    // Second snooze: 10 min
    sess2.snooze(10, Some("Sibuk".to_string()), 3600).unwrap();
    assert_eq!(sess2.snooze_count, 2);

    // Third snooze exceeds cap of 2 -> must be rejected
    let err = sess2.snooze(10, None, 4200);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("Maksimum penundaan"));

    // 4. Start grace expiry -> marks Missed
    let mut sess3 = RoutineSession::new("rtn-test-01", 5000, 30, 5, None);
    // At 5000 + 4 min (within grace): not expired
    assert!(!sess3.check_grace_expiry(5000 + 240));
    assert_eq!(sess3.state, RoutineSessionState::Prompted);

    // At 5000 + 6 min (past grace): expired!
    assert!(sess3.check_grace_expiry(5000 + 360));
    assert_eq!(sess3.state, RoutineSessionState::Missed);
}

#[test]
fn test_streak_engine_dst_and_calendar_boundaries() {
    let mut streak = RoutineStreak::default();

    // Day 1 completion
    apply_streak_completion(&mut streak, "2026-11-20");
    assert_eq!(streak.current_count, 1);
    assert_eq!(streak.best_count, 1);
    assert_eq!(streak.last_completed_date.as_deref(), Some("2026-11-20"));

    // Same day completion does NOT double-increment
    apply_streak_completion(&mut streak, "2026-11-20");
    assert_eq!(streak.current_count, 1);

    // Day 2 consecutive completion
    apply_streak_completion(&mut streak, "2026-11-21");
    assert_eq!(streak.current_count, 2);
    assert_eq!(streak.best_count, 2);

    // Day 3 excused skip (Sakit) -> freezes streak without penalty
    apply_streak_excused_skip(&mut streak, "2026-11-22");
    assert_eq!(streak.current_count, 2);
    assert_eq!(streak.frozen_on_date.as_deref(), Some("2026-11-22"));

    // Day 4 completion resumes consecutive streak: 2 -> 3
    apply_streak_completion(&mut streak, "2026-11-23");
    assert_eq!(streak.current_count, 3);
    assert_eq!(streak.best_count, 3);
    assert_eq!(streak.frozen_on_date, None); // Freeze cleared

    // Day 5 unexcused miss -> resets streak count to 0, best count preserved
    apply_streak_miss(&mut streak);
    assert_eq!(streak.current_count, 0);
    assert_eq!(streak.best_count, 3);

    // Date math across month/year boundary
    let days_dec31 = date_str_to_days("2026-12-31").unwrap();
    let days_jan01 = date_str_to_days("2027-01-01").unwrap();
    assert_eq!(days_jan01 - days_dec31, 1);
    assert_eq!(days_to_date_str(days_jan01), "2027-01-01");
}

#[tokio::test]
async fn test_three_tier_commitment_ladder_and_auto_escalation() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let em = Arc::new(EnforcementManager::new(db));

    // 1. Create commitments across tiers
    let com_ingatkan = Commitment {
        id: "com-ingat-01".to_string(),
        name: "Kurangi Medsos".to_string(),
        tier: EnforcementTier::Ingatkan,
        target_apps: vec!["twitter.exe".to_string(), "reddit.exe".to_string()],
        daily_quota_min: Some(30),
        active: true,
        pending_weakening_until: None,
        created_at: 1000,
        updated_at: 1000,
    };
    em.create_commitment(com_ingatkan).unwrap();

    let com_paksa = Commitment {
        id: "com-paksa-01".to_string(),
        name: "Blokir Game".to_string(),
        tier: EnforcementTier::Paksa,
        target_apps: vec!["steam.exe".to_string()],
        daily_quota_min: None,
        active: true,
        pending_weakening_until: None,
        created_at: 1000,
        updated_at: 1000,
    };
    em.create_commitment(com_paksa).unwrap();

    // 2. Ladder evaluation: Protected app is always Allowed
    let action_protected = em.evaluate("code.exe", None, 1000).unwrap();
    assert!(matches!(action_protected, EnforcementAction::Allowed { .. }));

    let action_system = em.evaluate("explorer.exe", None, 1000).unwrap();
    assert!(matches!(action_system, EnforcementAction::Allowed { .. }));

    // 3. Ladder evaluation: Ingatkan -> WarnGentle
    let action_ingat = em.evaluate("twitter.exe", None, 1000).unwrap();
    assert!(matches!(action_ingat, EnforcementAction::WarnGentle { .. }));

    // 4. Ladder evaluation: Paksa -> EnforceStrict with 10s countdown
    let action_paksa = em.evaluate("steam.exe", None, 1000).unwrap();
    match action_paksa {
        EnforcementAction::EnforceStrict { countdown_secs, target_app } => {
            assert_eq!(countdown_secs, 10);
            assert_eq!(target_app, "steam.exe");
        }
        other => panic!("Expected EnforceStrict, got {:?}", other),
    }

    // 5. Auto-escalation: After 3 ignored reminders, Ingatkan -> Tegas
    assert_eq!(
        em.escalation.record_ignored_reminder("com-ingat-01", EnforcementTier::Ingatkan, 3),
        None
    );
    assert_eq!(
        em.escalation.record_ignored_reminder("com-ingat-01", EnforcementTier::Ingatkan, 3),
        None
    );
    let escalated = em.escalation.record_ignored_reminder("com-ingat-01", EnforcementTier::Ingatkan, 3);
    assert_eq!(escalated, Some(EnforcementTier::Tegas));
}

#[tokio::test]
async fn test_emergency_pause_and_rule_weakening() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let em = Arc::new(EnforcementManager::new(db.clone()));

    // 1. Weakening delay: Paksa commitment requires 10 min cooling-off delay
    let com_strict = Commitment {
        id: "com-strict-01".to_string(),
        name: "Blokir Total".to_string(),
        tier: EnforcementTier::Paksa,
        target_apps: vec!["game.exe".to_string()],
        daily_quota_min: None,
        active: true,
        pending_weakening_until: None,
        created_at: 1000,
        updated_at: 1000,
    };
    em.create_commitment(com_strict).unwrap();

    let (weakened, pending_until) = em.request_weakening("com-strict-01", None, 1000).unwrap();
    assert_eq!(pending_until, 1000 + 600); // 10 min delay default
    assert_eq!(weakened.pending_weakening_until, Some(1600));

    // During the cooling-off delay (now = 1200), Paksa is STILL enforced
    let act_during_delay = em.evaluate("game.exe", None, 1200).unwrap();
    assert!(matches!(act_during_delay, EnforcementAction::EnforceStrict { .. }));

    // After cooling-off delay expires (now = 1605), commitment restriction ceases
    let act_after_delay = em.evaluate("game.exe", None, 1605).unwrap();
    assert!(matches!(act_after_delay, EnforcementAction::Allowed { .. }));

    // 2. Emergency Pause: immediately halts all enforcement and cannot be denied
    let pause = em.emergency_pause(30, "Panggilan Darurat Keluarga", 2000).unwrap();
    assert_eq!(pause.duration_min, 30);
    assert_eq!(pause.expired_at, 2000 + 1800);

    let status = em.get_pause_status(2500).unwrap();
    assert!(status.is_paused);
    assert_eq!(status.remaining_secs, 1300);
    assert_eq!(status.reason.as_deref(), Some("Panggilan Darurat Keluarga"));

    // Under Emergency Pause, even strict distraction evaluates to Paused
    let act_paused = em.evaluate("game.exe", None, 2500).unwrap();
    assert!(matches!(act_paused, EnforcementAction::Paused { remaining_secs: 1300, .. }));

    // When pause expires, status reports false
    let status_expired = em.get_pause_status(3900).unwrap();
    assert!(!status_expired.is_paused);
}

#[tokio::test]
async fn test_single_coalesced_next_wake_timer() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let rm = Arc::new(RoutineManager::new(db.clone()));
    let em = Arc::new(EnforcementManager::new(db.clone()));
    let scheduler = ReminderScheduler::new(db.clone(), rm.clone(), em.clone());

    // Routine at 19:00 Thursday (1970-01-01 was Thursday: 19:00 = 68400s)
    let routine = Routine {
        id: "r-wake-01".to_string(),
        title: "Sesi Malam".to_string(),
        category: "belajar".to_string(),
        schedule: RoutineSchedule {
            days: vec![Weekday::Thursday],
            time: "19:00".to_string(),
            duration_min: 45,
            window_min: None,
        },
        enforcement: EnforcementTier::Tegas,
        allowed_apps: vec![],
        distraction_rules: Default::default(),
        start_grace_min: 5,
        escalation: None,
        streak: Default::default(),
        active: true,
        created_at: 0,
        updated_at: 0,
    };
    rm.create_routine(routine).unwrap();

    // At 60000 (16:40), next wake should be Ambient Peek at 68400 - 300 = 68100 (18:55)
    let wake = scheduler.get_next_wake(60000).await.unwrap().expect("next wake found");
    assert_eq!(wake.timestamp, 68100);
    assert_eq!(wake.kind, NextWakeEventKind::RoutineAmbientPeek { routine_id: "r-wake-01".to_string() });

    // Duration calculation
    let dur = scheduler.get_duration_until_next_wake(60000).await.unwrap().unwrap();
    assert_eq!(dur.as_secs(), 8100);

    // If an earlier emergency pause is placed (expires at 62000), it coalesces to that earlier wake!
    em.emergency_pause(15, "Istirahat sejenak", 61100).unwrap(); // expires at 61100 + 900 = 62000
    let wake_pause = scheduler.get_next_wake(61200).await.unwrap().expect("next wake pause");
    assert_eq!(wake_pause.timestamp, 62000);
    assert_eq!(wake_pause.kind, NextWakeEventKind::EmergencyPauseExpired);
}

#[tokio::test]
async fn test_routines_and_commitments_rpc_endpoints() {
    let server = CoreServer::new_in_memory("test-token".to_string()).unwrap();

    // 1. productivity.routines.create
    let req_create = JsonRpcRequest::new(
        "rpc-rtn-create",
        "productivity.routines.create",
        Some(serde_json::json!({
            "routine": {
                "id": "rtn-rpc-01",
                "title": "Belajar Algoritma",
                "category": "belajar",
                "schedule": {
                    "days": ["Monday", "Tuesday"],
                    "time": "20:00",
                    "duration_min": 60
                },
                "enforcement": "tegas",
                "allowed_apps": ["code.exe"],
                "distraction_rules": {
                    "block_during_routine": true,
                    "distraction_apps": ["discord.exe"]
                },
                "start_grace_min": 5,
                "active": true
            }
        })),
    );
    let resp_create = server.dispatch_rpc(req_create).await;
    assert!(resp_create.error.is_none());
    assert_eq!(resp_create.result.unwrap()["title"], "Belajar Algoritma");

    // 2. productivity.routines.list
    let req_list = JsonRpcRequest::new("rpc-rtn-list", "productivity.routines.list", None);
    let resp_list = server.dispatch_rpc(req_list).await;
    assert!(resp_list.error.is_none());
    let list_arr = resp_list.result.unwrap();
    assert_eq!(list_arr.as_array().unwrap().len(), 1);

    // 3. productivity.routines.start_session
    let req_start = JsonRpcRequest::new(
        "rpc-rtn-start",
        "productivity.routines.start_session",
        Some(serde_json::json!({ "routine_id": "rtn-rpc-01" })),
    );
    let resp_start = server.dispatch_rpc(req_start).await;
    assert!(resp_start.error.is_none());
    let sess_id = resp_start.result.unwrap()["session"]["id"].as_str().unwrap().to_string();

    // 4. productivity.routines.snooze_session
    let req_snooze = JsonRpcRequest::new(
        "rpc-rtn-snooze",
        "productivity.routines.snooze_session",
        Some(serde_json::json!({
            "session_id": sess_id,
            "minutes": 10,
            "reason": "Sibuk ada tamu"
        })),
    );
    // Note: session is currently active, snoozing active returns error
    let resp_snooze = server.dispatch_rpc(req_snooze).await;
    assert!(resp_snooze.error.is_some());

    // 5. productivity.routines.complete_session
    let req_complete = JsonRpcRequest::new(
        "rpc-rtn-complete",
        "productivity.routines.complete_session",
        Some(serde_json::json!({ "session_id": sess_id })),
    );
    let resp_complete = server.dispatch_rpc(req_complete).await;
    assert!(resp_complete.error.is_none());
    let res_val = resp_complete.result.unwrap();
    assert_eq!(res_val["session"]["state"], "completed");
    assert_eq!(res_val["streak"]["current_count"], 1);

    // 6. productivity.commitments.create
    let req_com = JsonRpcRequest::new(
        "rpc-com-create",
        "productivity.commitments.create",
        Some(serde_json::json!({
            "commitment": {
                "id": "com-rpc-01",
                "name": "Fokus Skripsi",
                "tier": "paksa",
                "target_apps": ["steam.exe"],
                "active": true
            }
        })),
    );
    let resp_com = server.dispatch_rpc(req_com).await;
    assert!(resp_com.error.is_none());
    assert_eq!(resp_com.result.unwrap()["name"], "Fokus Skripsi");

    // 7. productivity.commitments.request_weakening
    let req_weaken = JsonRpcRequest::new(
        "rpc-com-weaken",
        "productivity.commitments.request_weakening",
        Some(serde_json::json!({ "id": "com-rpc-01", "delay_seconds": 600 })),
    );
    let resp_weaken = server.dispatch_rpc(req_weaken).await;
    assert!(resp_weaken.error.is_none());
    assert!(resp_weaken.result.unwrap()["pending_until"].as_i64().is_some());

    // 8. productivity.enforce.emergency_pause
    let req_pause = JsonRpcRequest::new(
        "rpc-enf-pause",
        "productivity.enforce.emergency_pause",
        Some(serde_json::json!({ "duration_min": 15, "reason": "Sakit kepala mendadak" })),
    );
    let resp_pause = server.dispatch_rpc(req_pause).await;
    assert!(resp_pause.error.is_none());
    assert_eq!(resp_pause.result.unwrap()["duration_min"], 15);

    // 9. productivity.enforce.get_pause_status
    let req_status = JsonRpcRequest::new("rpc-enf-status", "productivity.enforce.get_pause_status", None);
    let resp_status = server.dispatch_rpc(req_status).await;
    assert!(resp_status.error.is_none());
    assert_eq!(resp_status.result.unwrap()["is_paused"], true);

    // 10. productivity.enforce.get_history
    let req_hist = JsonRpcRequest::new("rpc-enf-hist", "productivity.enforce.get_history", None);
    let resp_hist = server.dispatch_rpc(req_hist).await;
    assert!(resp_hist.error.is_none());
    assert!(!resp_hist.result.unwrap().as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_non_daily_routine_schedule_streak_integration() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let rm = Arc::new(RoutineManager::new(db));

    let routine = Routine {
        id: "rtn-mwf-integration".to_string(),
        title: "Gym Workout Mon-Wed-Fri".to_string(),
        category: "kesehatan".to_string(),
        schedule: RoutineSchedule {
            days: vec![Weekday::Monday, Weekday::Wednesday, Weekday::Friday],
            time: "07:00".to_string(),
            duration_min: 60,
            window_min: None,
        },
        enforcement: EnforcementTier::Tegas,
        allowed_apps: vec![],
        distraction_rules: DistractionRules::default(),
        start_grace_min: 5,
        escalation: None,
        streak: RoutineStreak::default(),
        active: true,
        created_at: 1000,
        updated_at: 1000,
    };
    rm.create_routine(routine).unwrap();

    // Monday 2026-11-02 07:00:00 UTC
    let mon_days = date_str_to_days("2026-11-02").unwrap();
    let mon_ts = mon_days * 86400 + 7 * 3600;

    let sess1 = rm.start_session("rtn-mwf-integration", Some(mon_ts), mon_ts).await.unwrap();
    let (_, streak1) = rm.complete_session(&sess1.id, mon_ts + 3600).await.unwrap();
    assert_eq!(streak1.current_count, 1);
    assert_eq!(streak1.best_count, 1);
    assert_eq!(streak1.last_completed_date.as_deref(), Some("2026-11-02"));

    // Wednesday 2026-11-04 07:00:00 UTC (Tuesday was unscheduled)
    let wed_days = date_str_to_days("2026-11-04").unwrap();
    let wed_ts = wed_days * 86400 + 7 * 3600;

    let sess2 = rm.start_session("rtn-mwf-integration", Some(wed_ts), wed_ts).await.unwrap();
    let (_, streak2) = rm.complete_session(&sess2.id, wed_ts + 3600).await.unwrap();
    assert_eq!(
        streak2.current_count, 2,
        "Completing the next scheduled occurrence of a Mon/Wed/Fri routine must increment streak to 2!"
    );
    assert_eq!(streak2.best_count, 2);
    assert_eq!(streak2.last_completed_date.as_deref(), Some("2026-11-04"));

    // Friday 2026-11-06 07:00:00 UTC (Thursday was unscheduled)
    let fri_days = date_str_to_days("2026-11-06").unwrap();
    let fri_ts = fri_days * 86400 + 7 * 3600;

    let sess3 = rm.start_session("rtn-mwf-integration", Some(fri_ts), fri_ts).await.unwrap();
    let (_, streak3) = rm.complete_session(&sess3.id, fri_ts + 3600).await.unwrap();
    assert_eq!(streak3.current_count, 3);
    assert_eq!(streak3.best_count, 3);
    assert_eq!(streak3.last_completed_date.as_deref(), Some("2026-11-06"));

    // Monday next week 2026-11-09 07:00:00 UTC (weekend was unscheduled)
    let next_mon_days = date_str_to_days("2026-11-09").unwrap();
    let next_mon_ts = next_mon_days * 86400 + 7 * 3600;

    let sess4 = rm.start_session("rtn-mwf-integration", Some(next_mon_ts), next_mon_ts).await.unwrap();
    let (_, streak4) = rm.complete_session(&sess4.id, next_mon_ts + 3600).await.unwrap();
    assert_eq!(streak4.current_count, 4);
    assert_eq!(streak4.best_count, 4);
    assert_eq!(streak4.last_completed_date.as_deref(), Some("2026-11-09"));
}

#[tokio::test]
async fn test_gate10_path_qualified_protected_process_and_distraction_in_ladder() {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let em = Arc::new(EnforcementManager::new(db.clone()));

    let paksa_routine = Routine {
        id: "rtn-gate10-paksa".to_string(),
        title: "Focus Block".to_string(),
        category: "kerja".to_string(),
        schedule: RoutineSchedule {
            days: vec![],
            time: "10:00".to_string(),
            duration_min: 60,
            window_min: None,
        },
        enforcement: EnforcementTier::Paksa,
        allowed_apps: vec!["code.exe".to_string()],
        distraction_rules: DistractionRules {
            block_during_routine: true,
            daily_quota_min: None,
            sequential_after_focus_min: None,
            distraction_apps: vec!["game.exe".to_string()],
        },
        start_grace_min: 5,
        escalation: None,
        streak: RoutineStreak::default(),
        active: true,
        created_at: 1000,
        updated_at: 1000,
    };

    // Protected processes must be Allowed even with relative or absolute paths under Paksa block_during_routine
    let action_explorer = em.evaluate(r".\explorer.exe", Some(&paksa_routine), 1000).unwrap();
    assert!(matches!(action_explorer, EnforcementAction::Allowed { .. }));

    let action_cmd = em.evaluate(r"C:\Windows\System32\cmd.exe", Some(&paksa_routine), 1000).unwrap();
    assert!(matches!(action_cmd, EnforcementAction::Allowed { .. }));

    let action_ps = em.evaluate("./powershell.exe", Some(&paksa_routine), 1000).unwrap();
    assert!(matches!(action_ps, EnforcementAction::Allowed { .. }));

    // Non-protected distraction app with relative path must trigger EnforceStrict
    let action_game = em.evaluate(r".\game.exe", Some(&paksa_routine), 1000).unwrap();
    assert!(matches!(action_game, EnforcementAction::EnforceStrict { countdown_secs: 10, .. }));
}
