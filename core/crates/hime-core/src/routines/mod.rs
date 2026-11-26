pub mod models;
pub mod schedule;
pub mod session;
pub mod streak;

pub use models::{
    AutoEscalationConfig, DistractionRules, EnforcementTier, Routine, RoutineSchedule, RoutineStreak,
    Weekday,
};
pub use schedule::{calculate_next_routine_start, parse_time_hh_mm, weekday_from_epoch};
pub use session::{RoutineSession, RoutineSessionState};
pub use streak::{
    apply_streak_completion, apply_streak_completion_with_schedule, apply_streak_excused_skip,
    apply_streak_miss,
};

use hime_storage::{epoch_to_date_str, Database};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RoutineManager {
    db: Arc<Database>,
    sessions: Arc<Mutex<HashMap<String, RoutineSession>>>,
}

impl RoutineManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_routine(&self, routine: Routine) -> Result<Routine, anyhow::Error> {
        let record = routine.to_storage_record();
        self.db.create_routine(&record)?;
        Ok(routine)
    }

    pub fn get_routine(&self, id: &str) -> Result<Option<Routine>, anyhow::Error> {
        let opt_rec = self.db.get_routine(id)?;
        Ok(opt_rec.map(|r| Routine::from_storage_record(&r)))
    }

    pub fn list_routines(&self, active_only: bool) -> Result<Vec<Routine>, anyhow::Error> {
        let records = self.db.list_routines(active_only)?;
        Ok(records.into_iter().map(|r| Routine::from_storage_record(&r)).collect())
    }

    pub fn update_routine(&self, routine: Routine) -> Result<bool, anyhow::Error> {
        let record = routine.to_storage_record();
        let ok = self.db.update_routine(&record)?;
        Ok(ok)
    }

    pub fn delete_routine(&self, id: &str) -> Result<bool, anyhow::Error> {
        let ok = self.db.delete_routine(id)?;
        Ok(ok)
    }

    pub async fn start_session(
        &self,
        routine_id: &str,
        scheduled_for: Option<i64>,
        now_secs: i64,
    ) -> Result<RoutineSession, anyhow::Error> {
        let routine = self
            .get_routine(routine_id)?
            .ok_or_else(|| anyhow::anyhow!("Routine '{}' not found", routine_id))?;

        let sched_for = scheduled_for.unwrap_or(now_secs);
        let max_snooze = match routine.enforcement {
            EnforcementTier::Tegas => Some(3),
            EnforcementTier::Paksa => Some(2),
            EnforcementTier::Ingatkan => None,
        };

        let mut session = RoutineSession::new(
            routine_id,
            sched_for,
            routine.schedule.duration_min,
            routine.start_grace_min,
            max_snooze,
        );

        session.start(now_secs).map_err(|e| anyhow::anyhow!(e))?;

        let mut lock = self.sessions.lock().await;
        lock.insert(session.id.clone(), session.clone());

        Ok(session)
    }

    pub async fn snooze_session(
        &self,
        session_id: &str,
        minutes: u32,
        reason: Option<String>,
        now_secs: i64,
    ) -> Result<RoutineSession, anyhow::Error> {
        let mut lock = self.sessions.lock().await;
        let session = lock
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session '{}' not found", session_id))?;

        session
            .snooze(minutes, reason, now_secs)
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(session.clone())
    }

    pub async fn skip_session(
        &self,
        session_id: &str,
        reason: &str,
        now_secs: i64,
    ) -> Result<(RoutineSession, RoutineStreak), anyhow::Error> {
        let mut lock = self.sessions.lock().await;
        let session = lock
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session '{}' not found", session_id))?;

        session.skip(reason, now_secs).map_err(|e| anyhow::anyhow!(e))?;
        let session_clone = session.clone();

        let mut routine = self
            .get_routine(&session.routine_id)?
            .ok_or_else(|| anyhow::anyhow!("Routine '{}' not found", session.routine_id))?;

        let today_str = epoch_to_date_str(now_secs);
        if session.is_excused {
            apply_streak_excused_skip(&mut routine.streak, &today_str);
        } else {
            apply_streak_miss(&mut routine.streak);
        }

        self.db.update_routine_streak(
            &routine.id,
            routine.streak.current_count as i64,
            routine.streak.best_count as i64,
            routine.streak.last_completed_date.as_deref(),
            routine.streak.frozen_on_date.as_deref(),
        )?;

        Ok((session_clone, routine.streak))
    }

    pub async fn complete_session(
        &self,
        session_id: &str,
        now_secs: i64,
    ) -> Result<(RoutineSession, RoutineStreak), anyhow::Error> {
        let mut lock = self.sessions.lock().await;
        let session = lock
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session '{}' not found", session_id))?;

        session.complete(now_secs).map_err(|e| anyhow::anyhow!(e))?;
        let session_clone = session.clone();

        let mut routine = self
            .get_routine(&session.routine_id)?
            .ok_or_else(|| anyhow::anyhow!("Routine '{}' not found", session.routine_id))?;

        let today_str = epoch_to_date_str(now_secs);
        apply_streak_completion_with_schedule(&mut routine.streak, &today_str, Some(&routine.schedule));

        self.db.update_routine_streak(
            &routine.id,
            routine.streak.current_count as i64,
            routine.streak.best_count as i64,
            routine.streak.last_completed_date.as_deref(),
            routine.streak.frozen_on_date.as_deref(),
        )?;

        Ok((session_clone, routine.streak))
    }

    pub async fn get_session(&self, session_id: &str) -> Option<RoutineSession> {
        let lock = self.sessions.lock().await;
        lock.get(session_id).cloned()
    }

    pub async fn list_sessions(&self) -> Vec<RoutineSession> {
        let lock = self.sessions.lock().await;
        lock.values().cloned().collect()
    }
}
