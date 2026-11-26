use crate::enforce::models::Commitment;
use crate::routines::models::Routine;
use crate::routines::schedule::calculate_next_routine_start;
use crate::routines::session::{RoutineSession, RoutineSessionState};
use hime_storage::EmergencyPauseRecord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NextWakeEventKind {
    RoutineAmbientPeek { routine_id: String },
    RoutinePrompt { routine_id: String },
    RoutineGraceExpired { routine_id: String, session_id: Option<String> },
    RoutineSnoozeExpired { session_id: String },
    RoutineSessionCompleted { session_id: String },
    CommitmentWeakeningExpired { commitment_id: String },
    EmergencyPauseExpired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextWakeEvent {
    pub timestamp: i64,
    pub kind: NextWakeEventKind,
}

/// Calculates the single coalesced next wake event across all routines, active sessions,
/// commitments, and emergency pause states.
///
/// This avoids running multiple polling loops or independent timers, fulfilling the
/// single next-timer engine constraint and surviving sleep/DST shifts.
pub fn calculate_coalesced_next_wake(
    routines: &[Routine],
    sessions: &[RoutineSession],
    commitments: &[Commitment],
    active_pause: Option<&EmergencyPauseRecord>,
    now_secs: i64,
) -> Option<NextWakeEvent> {
    let mut candidates: Vec<NextWakeEvent> = Vec::new();

    // 1. Routine schedules
    for routine in routines {
        if !routine.active {
            continue;
        }

        let grace_secs = (routine.start_grace_min as i64) * 60;
        if let Some(start_time) = calculate_next_routine_start(&routine.schedule, now_secs, grace_secs) {
            if start_time > now_secs {
                // Ambient peek at T - 5 min
                let ambient_time = start_time - 300;
                if ambient_time >= now_secs {
                    candidates.push(NextWakeEvent {
                        timestamp: ambient_time,
                        kind: NextWakeEventKind::RoutineAmbientPeek {
                            routine_id: routine.id.clone(),
                        },
                    });
                }

                // Routine prompt at T
                candidates.push(NextWakeEvent {
                    timestamp: start_time,
                    kind: NextWakeEventKind::RoutinePrompt {
                        routine_id: routine.id.clone(),
                    },
                });
            } else {
                // Currently within start grace window
                let grace_expire = start_time + grace_secs;
                if grace_expire >= now_secs {
                    candidates.push(NextWakeEvent {
                        timestamp: grace_expire,
                        kind: NextWakeEventKind::RoutineGraceExpired {
                            routine_id: routine.id.clone(),
                            session_id: None,
                        },
                    });
                }
            }
        }
    }

    // 2. Active in-memory sessions
    for session in sessions {
        match session.state {
            RoutineSessionState::Snoozed => {
                if let Some(wake) = session.snooze_until {
                    if wake >= now_secs {
                        candidates.push(NextWakeEvent {
                            timestamp: wake,
                            kind: NextWakeEventKind::RoutineSnoozeExpired {
                                session_id: session.id.clone(),
                            },
                        });
                    }
                }
            }
            RoutineSessionState::Active => {
                if let Some(started) = session.started_at {
                    let end_time = started + (session.duration_min as i64) * 60;
                    if end_time >= now_secs {
                        candidates.push(NextWakeEvent {
                            timestamp: end_time,
                            kind: NextWakeEventKind::RoutineSessionCompleted {
                                session_id: session.id.clone(),
                            },
                        });
                    }
                }
            }
            RoutineSessionState::Prompted => {
                let grace_expire = session.scheduled_for + (session.start_grace_min as i64) * 60;
                if grace_expire >= now_secs {
                    candidates.push(NextWakeEvent {
                        timestamp: grace_expire,
                        kind: NextWakeEventKind::RoutineGraceExpired {
                            routine_id: session.routine_id.clone(),
                            session_id: Some(session.id.clone()),
                        },
                    });
                }
            }
            _ => {}
        }
    }

    // 3. Commitments weakening delays
    for commitment in commitments {
        if commitment.active {
            if let Some(pending_until) = commitment.pending_weakening_until {
                if pending_until >= now_secs {
                    candidates.push(NextWakeEvent {
                        timestamp: pending_until,
                        kind: NextWakeEventKind::CommitmentWeakeningExpired {
                            commitment_id: commitment.id.clone(),
                        },
                    });
                }
            }
        }
    }

    // 4. Emergency pause expiry
    if let Some(pause) = active_pause {
        if pause.expired_at >= now_secs {
            candidates.push(NextWakeEvent {
                timestamp: pause.expired_at,
                kind: NextWakeEventKind::EmergencyPauseExpired,
            });
        }
    }

    // Return the earliest upcoming wake event
    candidates.into_iter().min_by_key(|e| e.timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routines::models::{EnforcementTier, RoutineSchedule, Weekday};

    #[test]
    fn test_coalesced_next_wake_selects_earliest() {
        let routine = Routine {
            id: "r1".to_string(),
            title: "Belajar".to_string(),
            category: "belajar".to_string(),
            schedule: RoutineSchedule {
                days: vec![Weekday::Thursday],
                time: "10:00".to_string(),
                duration_min: 30,
                window_min: None,
            },
            enforcement: EnforcementTier::Ingatkan,
            allowed_apps: vec![],
            distraction_rules: Default::default(),
            start_grace_min: 5,
            escalation: None,
            streak: Default::default(),
            active: true,
            created_at: 0,
            updated_at: 0,
        };

        // 1970-01-01 is Thursday. 10:00 = 36000.
        // At 30000 (08:20), ambient peek is 36000 - 300 = 35700.
        let next = calculate_coalesced_next_wake(&[routine], &[], &[], None, 30000).unwrap();
        assert_eq!(next.timestamp, 35700);
        assert_eq!(next.kind, NextWakeEventKind::RoutineAmbientPeek { routine_id: "r1".to_string() });
    }

    #[test]
    fn test_coalesced_next_wake_emergency_pause_earlier() {
        let pause = EmergencyPauseRecord {
            id: "p1".to_string(),
            started_at: 1000,
            duration_min: 15,
            reason: "Sakit".to_string(),
            expired_at: 1900,
        };

        let next = calculate_coalesced_next_wake(&[], &[], &[], Some(&pause), 1200).unwrap();
        assert_eq!(next.timestamp, 1900);
        assert_eq!(next.kind, NextWakeEventKind::EmergencyPauseExpired);
    }
}
