use std::collections::HashMap;
use std::sync::Mutex;
use crate::routines::models::EnforcementTier;

pub struct AutoEscalationTracker {
    missed_counts: Mutex<HashMap<String, u32>>,
}

impl AutoEscalationTracker {
    pub fn new() -> Self {
        Self {
            missed_counts: Mutex::new(HashMap::new()),
        }
    }

    /// Records an ignored reminder for a commitment or routine.
    /// If consecutive ignored reminders reach or exceed `threshold`, returns `Some(EnforcementTier::Tegas)`.
    pub fn record_ignored_reminder(
        &self,
        id: &str,
        current_tier: EnforcementTier,
        threshold: u32,
    ) -> Option<EnforcementTier> {
        let mut map = self.missed_counts.lock().unwrap();
        let count = map.entry(id.to_string()).or_insert(0);
        *count += 1;

        if current_tier == EnforcementTier::Ingatkan && *count >= threshold {
            Some(EnforcementTier::Tegas)
        } else {
            None
        }
    }

    /// Resets the consecutive ignored count (e.g. when user acknowledges or starts session).
    pub fn reset_ignored(&self, id: &str) {
        let mut map = self.missed_counts.lock().unwrap();
        map.remove(id);
    }

    pub fn get_ignored_count(&self, id: &str) -> u32 {
        let map = self.missed_counts.lock().unwrap();
        map.get(id).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_escalation_triggers_at_threshold() {
        let tracker = AutoEscalationTracker::new();
        assert_eq!(tracker.record_ignored_reminder("com-1", EnforcementTier::Ingatkan, 3), None);
        assert_eq!(tracker.record_ignored_reminder("com-1", EnforcementTier::Ingatkan, 3), None);
        // Third missed reminder triggers escalation to Tegas
        assert_eq!(
            tracker.record_ignored_reminder("com-1", EnforcementTier::Ingatkan, 3),
            Some(EnforcementTier::Tegas)
        );

        // Reset
        tracker.reset_ignored("com-1");
        assert_eq!(tracker.get_ignored_count("com-1"), 0);
    }
}
