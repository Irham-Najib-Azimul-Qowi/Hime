pub mod escalation;
pub mod ladder;
pub mod models;
pub mod protected;

pub use escalation::AutoEscalationTracker;
pub use ladder::evaluate_enforcement_ladder;
pub use models::{Commitment, EmergencyPauseStatus, EnforcementAction};
pub use protected::{extract_process_base_name, is_protected_process};

use crate::routines::models::{EnforcementTier, Routine};
use hime_storage::{Database, EmergencyPauseRecord, EnforcementHistoryRecord};
use std::sync::Arc;

pub struct EnforcementManager {
    db: Arc<Database>,
    pub escalation: AutoEscalationTracker,
}

impl EnforcementManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            escalation: AutoEscalationTracker::new(),
        }
    }

    pub fn create_commitment(&self, commitment: Commitment) -> Result<Commitment, anyhow::Error> {
        let rec = commitment.to_storage_record();
        self.db.create_commitment(&rec)?;
        Ok(commitment)
    }

    pub fn get_commitment(&self, id: &str) -> Result<Option<Commitment>, anyhow::Error> {
        let opt_rec = self.db.get_commitment(id)?;
        Ok(opt_rec.map(|r| Commitment::from_storage_record(&r)))
    }

    pub fn list_commitments(&self, active_only: bool) -> Result<Vec<Commitment>, anyhow::Error> {
        let records = self.db.list_commitments(active_only)?;
        Ok(records.into_iter().map(|r| Commitment::from_storage_record(&r)).collect())
    }

    pub fn update_commitment(&self, commitment: Commitment) -> Result<bool, anyhow::Error> {
        let rec = commitment.to_storage_record();
        let ok = self.db.update_commitment(&rec)?;
        Ok(ok)
    }

    pub fn delete_commitment(&self, id: &str) -> Result<bool, anyhow::Error> {
        let ok = self.db.delete_commitment(id)?;
        Ok(ok)
    }

    /// Requests rule weakening for a commitment.
    /// Under Paksa, a cooling-off waiting period (default 10 min = 600s) is enforced before relaxing.
    /// Under Ingatkan or Tegas, weakening applies immediately.
    pub fn request_weakening(
        &self,
        id: &str,
        delay_seconds: Option<i64>,
        now_secs: i64,
    ) -> Result<(Commitment, i64), anyhow::Error> {
        let mut commitment = self
            .get_commitment(id)?
            .ok_or_else(|| anyhow::anyhow!("Commitment '{}' not found", id))?;

        let pending_until = if commitment.tier == EnforcementTier::Paksa {
            let delay = delay_seconds.unwrap_or(600).max(60); // minimum 60s
            now_secs + delay
        } else {
            now_secs // immediate
        };

        commitment.pending_weakening_until = Some(pending_until);
        commitment.updated_at = now_secs;
        self.db.request_commitment_weakening(id, pending_until)?;

        Ok((commitment, pending_until))
    }

    /// Triggers an unblockable Emergency Pause (15, 30, or 60 min) with reason pill.
    pub fn emergency_pause(
        &self,
        duration_min: i64,
        reason: &str,
        now_secs: i64,
    ) -> Result<EmergencyPauseRecord, anyhow::Error> {
        let dur = match duration_min {
            15 | 30 | 60 => duration_min,
            _ => 15, // fallback to standard 15m
        };

        let expired_at = now_secs + dur * 60;
        let pid = format!("epause-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

        let record = EmergencyPauseRecord {
            id: pid,
            started_at: now_secs,
            duration_min: dur,
            reason: reason.trim().to_string(),
            expired_at,
        };

        self.db.record_emergency_pause(&record)?;

        // Also record to enforcement history audit
        let enf_event = EnforcementHistoryRecord {
            id: format!("audit-pause-{}", record.id),
            timestamp: now_secs,
            target_app: "all".to_string(),
            tier: "all".to_string(),
            action_taken: "paused".to_string(),
            details: Some(format!("Emergency pause {}m: {}", dur, reason)),
        };
        let _ = self.db.record_enforcement_event(&enf_event);

        Ok(record)
    }

    /// Queries whether Emergency Pause is currently active and remaining seconds.
    pub fn get_pause_status(&self, now_secs: i64) -> Result<EmergencyPauseStatus, anyhow::Error> {
        let active = self.db.get_active_emergency_pause(now_secs)?;
        if let Some(pause) = active {
            let rem = (pause.expired_at - now_secs).max(0);
            Ok(EmergencyPauseStatus {
                is_paused: true,
                remaining_secs: rem,
                reason: Some(pause.reason),
            })
        } else {
            Ok(EmergencyPauseStatus {
                is_paused: false,
                remaining_secs: 0,
                reason: None,
            })
        }
    }

    /// Evaluates target app against active emergency pause, protected list, active routine, and commitments.
    pub fn evaluate(
        &self,
        target_app: &str,
        active_routine: Option<&Routine>,
        now_secs: i64,
    ) -> Result<EnforcementAction, anyhow::Error> {
        let pause = self.db.get_active_emergency_pause(now_secs)?;
        let commitments = self.list_commitments(true)?;

        let action = evaluate_enforcement_ladder(
            target_app,
            active_routine,
            &commitments,
            pause.as_ref(),
            now_secs,
        );

        Ok(action)
    }

    pub fn record_enforcement_event(
        &self,
        target_app: &str,
        tier: &str,
        action_taken: &str,
        details: Option<&str>,
        now_secs: i64,
    ) -> Result<(), anyhow::Error> {
        let id = format!("enf-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let event = EnforcementHistoryRecord {
            id,
            timestamp: now_secs,
            target_app: target_app.to_string(),
            tier: tier.to_string(),
            action_taken: action_taken.to_string(),
            details: details.map(|s| s.to_string()),
        };
        self.db.record_enforcement_event(&event)?;
        Ok(())
    }

    pub fn get_history(&self, limit: usize) -> Result<Vec<EnforcementHistoryRecord>, anyhow::Error> {
        let hist = self.db.list_enforcement_history(limit)?;
        Ok(hist)
    }
}
