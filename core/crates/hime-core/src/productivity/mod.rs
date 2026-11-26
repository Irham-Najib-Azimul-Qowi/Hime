pub mod accountability;
pub mod kanban;
pub mod scheduler;

pub use accountability::{AccountabilityManager, CheckinGreeting, NudgeRecommendation};
pub use kanban::KanbanManager;
pub use scheduler::StudyScheduler;

use crate::enforce::EnforcementManager;
use crate::reminders::ReminderScheduler;
use crate::routines::RoutineManager;
use hime_storage::Database;
use std::sync::Arc;

pub struct ProductivityEngine {
    pub db: Arc<Database>,
    pub kanban: Arc<KanbanManager>,
    pub accountability: Arc<AccountabilityManager>,
    pub scheduler: Arc<StudyScheduler>,
    pub routines: Arc<RoutineManager>,
    pub enforce: Arc<EnforcementManager>,
    pub reminders: Arc<ReminderScheduler>,
}

impl ProductivityEngine {
    pub fn new(db: Arc<Database>) -> Self {
        let kanban = Arc::new(KanbanManager::new(db.clone()));
        let accountability = Arc::new(AccountabilityManager::new(db.clone()));
        let scheduler = Arc::new(StudyScheduler::new(db.clone()));
        let routines = Arc::new(RoutineManager::new(db.clone()));
        let enforce = Arc::new(EnforcementManager::new(db.clone()));
        let reminders = Arc::new(ReminderScheduler::new(db.clone(), routines.clone(), enforce.clone()));

        // Ensure default board is initialized
        let _ = kanban.ensure_default_board();

        Self {
            db,
            kanban,
            accountability,
            scheduler,
            routines,
            enforce,
            reminders,
        }
    }
}

