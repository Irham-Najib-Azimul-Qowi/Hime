use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoutineSessionState {
    Ambient,   // T - 5 min peek
    Prompted,  // P1 routine due prompt
    Snoozed,   // Postponed with reason
    Active,    // Currently in focus session
    Completed, // Successfully completed
    Skipped,   // Skipped with reason
    Missed,    // Unexcused abandon or grace expired
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutineSession {
    pub id: String,
    pub routine_id: String,
    pub state: RoutineSessionState,
    pub scheduled_for: i64, // epoch seconds
    pub duration_min: u32,
    pub start_grace_min: u32,
    pub snooze_count: u32,
    pub max_snooze_count: Option<u32>, // Tegas typically caps to 3
    #[serde(default)]
    pub snooze_until: Option<i64>,
    #[serde(default)]
    pub started_at: Option<i64>,
    #[serde(default)]
    pub completed_at: Option<i64>,
    #[serde(default)]
    pub skip_reason: Option<String>,
    #[serde(default)]
    pub is_excused: bool,
}

impl RoutineSession {
    pub fn new(
        routine_id: impl Into<String>,
        scheduled_for: i64,
        duration_min: u32,
        start_grace_min: u32,
        max_snooze_count: Option<u32>,
    ) -> Self {
        let sid = format!("rsess-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        Self {
            id: sid,
            routine_id: routine_id.into(),
            state: RoutineSessionState::Prompted,
            scheduled_for,
            duration_min,
            start_grace_min,
            snooze_count: 0,
            max_snooze_count,
            snooze_until: None,
            started_at: None,
            completed_at: None,
            skip_reason: None,
            is_excused: false,
        }
    }

    /// Transitions session to Active state
    pub fn start(&mut self, now_secs: i64) -> Result<(), &'static str> {
        match self.state {
            RoutineSessionState::Ambient
            | RoutineSessionState::Prompted
            | RoutineSessionState::Snoozed => {
                self.state = RoutineSessionState::Active;
                self.started_at = Some(now_secs);
                self.snooze_until = None;
                Ok(())
            }
            RoutineSessionState::Active => Ok(()), // Already active
            RoutineSessionState::Completed => Err("Session is already completed"),
            RoutineSessionState::Skipped => Err("Cannot start skipped session"),
            RoutineSessionState::Missed => Err("Cannot start missed session"),
        }
    }

    /// Snoozes session start by `minutes` (default 10m).
    /// In Tegas, `max_snooze_count` is enforced.
    pub fn snooze(&mut self, minutes: u32, reason: Option<String>, now_secs: i64) -> Result<(), &'static str> {
        match self.state {
            RoutineSessionState::Prompted | RoutineSessionState::Snoozed => {
                if let Some(max_snoozes) = self.max_snooze_count {
                    if self.snooze_count >= max_snoozes {
                        return Err("Maksimum penundaan tercapai");
                    }
                }
                self.snooze_count += 1;
                let delay_secs = (minutes.max(1) as i64) * 60;
                self.snooze_until = Some(now_secs + delay_secs);
                self.state = RoutineSessionState::Snoozed;
                if let Some(r) = reason {
                    self.skip_reason = Some(r);
                }
                Ok(())
            }
            RoutineSessionState::Active => Err("Cannot snooze an active session"),
            RoutineSessionState::Completed => Err("Session is already completed"),
            RoutineSessionState::Skipped => Err("Cannot snooze skipped session"),
            RoutineSessionState::Missed => Err("Cannot snooze missed session"),
            RoutineSessionState::Ambient => Err("Session has not been prompted yet"),
        }
    }

    /// Skips session with a reason pill: "Sibuk", "Sakit", "Libur", "Lainnya".
    /// If reason is recognized as excused, `is_excused` is set to true.
    pub fn skip(&mut self, reason: &str, _now_secs: i64) -> Result<(), &'static str> {
        if self.state == RoutineSessionState::Completed {
            return Err("Session is already completed");
        }
        let lower = reason.trim().to_lowercase();
        // Excused reasons per specs: Sakit, Libur, Sibuk
        let excused = lower.contains("sakit")
            || lower.contains("libur")
            || lower.contains("holiday")
            || lower.contains("sick")
            || lower.contains("sibuk")
            || lower.contains("izin");

        self.state = RoutineSessionState::Skipped;
        self.skip_reason = Some(reason.to_string());
        self.is_excused = excused;
        Ok(())
    }

    /// Marks the session as completed
    pub fn complete(&mut self, now_secs: i64) -> Result<(), &'static str> {
        match self.state {
            RoutineSessionState::Active
            | RoutineSessionState::Prompted
            | RoutineSessionState::Snoozed => {
                self.state = RoutineSessionState::Completed;
                self.completed_at = Some(now_secs);
                Ok(())
            }
            RoutineSessionState::Completed => Ok(()),
            RoutineSessionState::Skipped => Err("Cannot complete skipped session"),
            RoutineSessionState::Missed => Err("Cannot complete missed session"),
            RoutineSessionState::Ambient => Err("Session has not started yet"),
        }
    }

    /// Evaluates if start grace window has expired without user starting the session
    pub fn check_grace_expiry(&mut self, now_secs: i64) -> bool {
        if self.state == RoutineSessionState::Prompted {
            let grace_secs = (self.start_grace_min as i64) * 60;
            if now_secs > self.scheduled_for + grace_secs {
                self.state = RoutineSessionState::Missed;
                return true;
            }
        }
        false
    }
}
