pub mod wake;

pub use wake::{calculate_coalesced_next_wake, NextWakeEvent, NextWakeEventKind};

use crate::enforce::EnforcementManager;
use crate::routines::RoutineManager;
use hime_storage::Database;
use std::sync::Arc;
use std::time::Duration;

pub struct ReminderScheduler {
    db: Arc<Database>,
    routines: Arc<RoutineManager>,
    enforce: Arc<EnforcementManager>,
}

impl ReminderScheduler {
    pub fn new(
        db: Arc<Database>,
        routines: Arc<RoutineManager>,
        enforce: Arc<EnforcementManager>,
    ) -> Self {
        Self {
            db,
            routines,
            enforce,
        }
    }

    /// Queries the single coalesced next wake event across all active routines,
    /// commitments, sessions, and emergency pauses.
    pub async fn get_next_wake(&self, now_secs: i64) -> Result<Option<NextWakeEvent>, anyhow::Error> {
        let active_routines = self.routines.list_routines(true)?;
        let active_sessions = self.routines.list_sessions().await;
        let active_commitments = self.enforce.list_commitments(true)?;
        let active_pause = self.db.get_active_emergency_pause(now_secs)?;

        let next_wake = calculate_coalesced_next_wake(
            &active_routines,
            &active_sessions,
            &active_commitments,
            active_pause.as_ref(),
            now_secs,
        );

        Ok(next_wake)
    }

    /// Calculates the duration until the next wake event.
    /// If no events are scheduled, returns None.
    pub async fn get_duration_until_next_wake(&self, now_secs: i64) -> Result<Option<Duration>, anyhow::Error> {
        let next = self.get_next_wake(now_secs).await?;
        if let Some(event) = next {
            let diff = (event.timestamp - now_secs).max(0) as u64;
            Ok(Some(Duration::from_secs(diff)))
        } else {
            Ok(None)
        }
    }
}
