use crate::enforce::models::{Commitment, EnforcementAction};
use crate::enforce::protected::{extract_process_base_name, is_protected_process};
use crate::routines::models::{EnforcementTier, Routine};
use hime_storage::EmergencyPauseRecord;

/// Evaluates the enforcement ladder for a target foreground application.
pub fn evaluate_enforcement_ladder(
    target_app: &str,
    active_routine: Option<&Routine>,
    commitments: &[Commitment],
    active_pause: Option<&EmergencyPauseRecord>,
    now_secs: i64,
) -> EnforcementAction {
    // 1. Emergency Pause check — cannot be denied or blocked
    if let Some(pause) = active_pause {
        if pause.expired_at > now_secs {
            return EnforcementAction::Paused {
                remaining_secs: (pause.expired_at - now_secs) as u64,
                reason: pause.reason.clone(),
            };
        }
    }

    // 2. Protected process list check — immutable hard exclusion
    if is_protected_process(target_app) {
        return EnforcementAction::Allowed {
            reason: format!("'{}' is a protected system or development application", target_app),
        };
    }

    let target_base = extract_process_base_name(target_app);
    let app_matches = |candidate: &str| {
        extract_process_base_name(candidate) == target_base
    };

    // 3. Active routine evaluation
    if let Some(routine) = active_routine {
        // If app is allowlisted in the routine, it is unconditionally allowed
        if routine.allowed_apps.iter().any(|app| app_matches(app)) {
            return EnforcementAction::Allowed {
                reason: format!("'{}' is allowlisted for routine '{}'", target_app, routine.title),
            };
        }

        // Check if explicitly configured as a distraction
        let is_distraction = routine
            .distraction_rules
            .distraction_apps
            .iter()
            .any(|app| app_matches(app));

        if is_distraction || routine.distraction_rules.block_during_routine {
            return match routine.enforcement {
                EnforcementTier::Ingatkan => EnforcementAction::WarnGentle {
                    target_app: target_app.to_string(),
                    bubble: format!("Waktunya fokus: {}", routine.title),
                },
                EnforcementTier::Tegas => EnforcementAction::WarnFirm {
                    target_app: target_app.to_string(),
                    bubble: format!("Pengalih terdeteksi: {}", target_app),
                    open_panel: true,
                    reason_required: true,
                },
                EnforcementTier::Paksa => EnforcementAction::EnforceStrict {
                    target_app: target_app.to_string(),
                    countdown_secs: 10,
                },
            };
        }
    }

    // 4. Standalone commitment evaluation
    for commitment in commitments {
        if !commitment.active {
            continue;
        }

        // If pending weakening delay has elapsed, skip enforcement
        if let Some(until) = commitment.pending_weakening_until {
            if now_secs >= until {
                continue;
            }
        }

        if commitment.target_apps.iter().any(|app| app_matches(app)) {
            return match commitment.tier {
                EnforcementTier::Ingatkan => EnforcementAction::WarnGentle {
                    target_app: target_app.to_string(),
                    bubble: format!("Komitmen: {}", commitment.name),
                },
                EnforcementTier::Tegas => EnforcementAction::WarnFirm {
                    target_app: target_app.to_string(),
                    bubble: format!("Komitmen '{}' aktif", commitment.name),
                    open_panel: true,
                    reason_required: true,
                },
                EnforcementTier::Paksa => EnforcementAction::EnforceStrict {
                    target_app: target_app.to_string(),
                    countdown_secs: 10,
                },
            };
        }
    }

    // 5. Default allowed
    EnforcementAction::Allowed {
        reason: format!("'{}' has no active restrictions", target_app),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routines::models::RoutineSchedule;

    #[test]
    fn test_ladder_emergency_pause_overrides_everything() {
        let pause = EmergencyPauseRecord {
            id: "p1".to_string(),
            started_at: 1000,
            duration_min: 30,
            reason: "Sakit".to_string(),
            expired_at: 2800,
        };

        let action = evaluate_enforcement_ladder("steam.exe", None, &[], Some(&pause), 1500);
        assert!(matches!(action, EnforcementAction::Paused { remaining_secs: 1300, ref reason } if reason == "Sakit"));
    }

    #[test]
    fn test_ladder_protected_app_allowed() {
        let action = evaluate_enforcement_ladder("code.exe", None, &[], None, 1000);
        assert!(matches!(action, EnforcementAction::Allowed { .. }));
    }

    #[test]
    fn test_ladder_routine_enforcement_tiers() {
        let mut routine = Routine {
            id: "r1".to_string(),
            title: "Belajar".to_string(),
            category: "belajar".to_string(),
            schedule: RoutineSchedule {
                days: vec![],
                time: "19:00".to_string(),
                duration_min: 30,
                window_min: None,
            },
            enforcement: EnforcementTier::Paksa,
            allowed_apps: vec!["word.exe".to_string()],
            distraction_rules: crate::routines::models::DistractionRules {
                block_during_routine: false,
                daily_quota_min: None,
                sequential_after_focus_min: None,
                distraction_apps: vec!["youtube.exe".to_string()],
            },
            start_grace_min: 5,
            escalation: None,
            streak: Default::default(),
            active: true,
            created_at: 1000,
            updated_at: 1000,
        };

        // Allowed app
        let res = evaluate_enforcement_ladder("word.exe", Some(&routine), &[], None, 1000);
        assert!(matches!(res, EnforcementAction::Allowed { .. }));

        // Distraction under Paksa -> EnforceStrict 10s countdown
        let res2 = evaluate_enforcement_ladder("youtube.exe", Some(&routine), &[], None, 1000);
        assert!(matches!(res2, EnforcementAction::EnforceStrict { countdown_secs: 10, .. }));

        // Change to Tegas -> WarnFirm
        routine.enforcement = EnforcementTier::Tegas;
        let res3 = evaluate_enforcement_ladder("youtube.exe", Some(&routine), &[], None, 1000);
        assert!(matches!(res3, EnforcementAction::WarnFirm { open_panel: true, .. }));

        // Change to Ingatkan -> WarnGentle
        routine.enforcement = EnforcementTier::Ingatkan;
        let res4 = evaluate_enforcement_ladder("youtube.exe", Some(&routine), &[], None, 1000);
        assert!(matches!(res4, EnforcementAction::WarnGentle { .. }));
    }

    #[test]
    fn test_ladder_path_qualified_protected_and_distraction() {
        let paksa_routine = Routine {
            id: "rtn-strict".to_string(),
            title: "Deep Focus".to_string(),
            category: "kerja".to_string(),
            schedule: RoutineSchedule {
                days: vec![],
                time: "09:00".to_string(),
                duration_min: 60,
                window_min: None,
            },
            enforcement: EnforcementTier::Paksa,
            allowed_apps: vec!["code.exe".to_string()],
            distraction_rules: crate::routines::models::DistractionRules {
                block_during_routine: true,
                daily_quota_min: None,
                sequential_after_focus_min: None,
                distraction_apps: vec!["chrome.exe".to_string()],
            },
            start_grace_min: 5,
            escalation: None,
            streak: Default::default(),
            active: true,
            created_at: 1000,
            updated_at: 1000,
        };

        // Path-qualified protected processes must be Allowed even with block_during_routine = true under Paksa
        let res_exp = evaluate_enforcement_ladder(r".\explorer.exe", Some(&paksa_routine), &[], None, 1000);
        assert!(matches!(res_exp, EnforcementAction::Allowed { .. }));

        let res_cmd = evaluate_enforcement_ladder(r"C:\Windows\System32\cmd.exe", Some(&paksa_routine), &[], None, 1000);
        assert!(matches!(res_cmd, EnforcementAction::Allowed { .. }));

        let res_ps = evaluate_enforcement_ladder("./powershell.exe", Some(&paksa_routine), &[], None, 1000);
        assert!(matches!(res_ps, EnforcementAction::Allowed { .. }));

        // Path-qualified distraction must symmetrically match and trigger EnforceStrict under Paksa
        let res_dist = evaluate_enforcement_ladder(r"C:\Program Files\Google\Chrome\Application\chrome.exe", Some(&paksa_routine), &[], None, 1000);
        assert!(matches!(res_dist, EnforcementAction::EnforceStrict { countdown_secs: 10, .. }));

        let res_dist_rel = evaluate_enforcement_ladder(r".\chrome.exe", Some(&paksa_routine), &[], None, 1000);
        assert!(matches!(res_dist_rel, EnforcementAction::EnforceStrict { countdown_secs: 10, .. }));
    }
}
