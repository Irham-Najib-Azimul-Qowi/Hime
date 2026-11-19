use hime_storage::{Database, StorageError, StudyScheduleRecord};
use std::sync::Arc;

pub struct StudyScheduler {
    db: Arc<Database>,
}

impl StudyScheduler {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Plans deterministic study blocks avoiding overlaps with existing sessions (UR-10).
    pub fn plan_study_session(
        &self,
        id: &str,
        title: &str,
        subject: &str,
        desired_start: i64,
        duration_minutes: i64,
        priority: &str,
    ) -> Result<StudyScheduleRecord, StorageError> {
        let existing = self.db.list_study_schedules()?;
        let duration_secs = duration_minutes * 60;
        let mut start_time = desired_start;
        let mut end_time = start_time + duration_secs;

        // Check for collisions and advance forward if needed
        let mut collision = true;
        while collision {
            collision = false;
            for s in &existing {
                if s.status != "completed" {
                    // Check overlap: max(start1, start2) < min(end1, end2)
                    if (start_time < s.end_time) && (end_time > s.start_time) {
                        // Conflict found: shift start to immediately after existing session + 10 min break
                        start_time = s.end_time + 600;
                        end_time = start_time + duration_secs;
                        collision = true;
                        break;
                    }
                }
            }
        }

        let schedule = StudyScheduleRecord {
            id: id.to_string(),
            title: title.to_string(),
            subject: subject.to_string(),
            start_time,
            end_time,
            priority: priority.to_string(),
            status: "scheduled".to_string(),
            rescheduled_reason: None,
        };

        self.db.create_study_schedule(&schedule)?;
        Ok(schedule)
    }

    /// Reschedules a missed or postponed study session.
    pub fn reschedule_session(
        &self,
        id: &str,
        new_start: i64,
        reason: &str,
    ) -> Result<StudyScheduleRecord, StorageError> {
        let schedules = self.db.list_study_schedules()?;
        let mut target = schedules
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| StorageError::NotFound(format!("Study schedule '{}' not found", id)))?;

        let duration = target.end_time - target.start_time;
        target.start_time = new_start;
        target.end_time = new_start + duration;
        target.status = "rescheduled".to_string();
        target.rescheduled_reason = Some(reason.to_string());

        self.db.create_study_schedule(&target)?;
        Ok(target)
    }
}

