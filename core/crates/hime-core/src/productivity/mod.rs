pub mod accountability;
pub mod kanban;
pub mod scheduler;

pub use accountability::{AccountabilityManager, CheckinGreeting, NudgeRecommendation};
pub use kanban::KanbanManager;
pub use scheduler::StudyScheduler;

use hime_storage::Database;
use std::sync::Arc;

pub struct ProductivityEngine {
    pub db: Arc<Database>,
    pub kanban: Arc<KanbanManager>,
    pub accountability: Arc<AccountabilityManager>,
    pub scheduler: Arc<StudyScheduler>,
}

impl ProductivityEngine {
    pub fn new(db: Arc<Database>) -> Self {
        let kanban = Arc::new(KanbanManager::new(db.clone()));
        let accountability = Arc::new(AccountabilityManager::new(db.clone()));
        let scheduler = Arc::new(StudyScheduler::new(db.clone()));

        // Ensure default board is initialized
        let _ = kanban.ensure_default_board();

        Self {
            db,
            kanban,
            accountability,
            scheduler,
        }
    }
}

