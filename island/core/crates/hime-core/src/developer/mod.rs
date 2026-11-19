pub mod git_ops;
pub mod runner;
pub mod scanner;
pub mod templates;
pub mod workspace;

pub use git_ops::{GitOpError, GitOperations, GitStatusResult};
pub use runner::{CommandOutputResult, DeveloperCommandRunner, RunnerError};
pub use scanner::{SecretFinding, SecretScanner};
pub use templates::{SeTemplateMetadata, SoftwareEngineeringTemplates};
pub use workspace::{WorkspaceError, WorkspaceManager};

use std::sync::Arc;
use hime_storage::Database;

pub struct DeveloperEngine {
    pub workspace_manager: WorkspaceManager,
    pub command_runner: DeveloperCommandRunner,
}

impl DeveloperEngine {
    pub fn new(storage: Arc<Database>) -> Self {
        Self {
            workspace_manager: WorkspaceManager::new(storage.clone()),
            command_runner: DeveloperCommandRunner::new(storage),
        }
    }
}

