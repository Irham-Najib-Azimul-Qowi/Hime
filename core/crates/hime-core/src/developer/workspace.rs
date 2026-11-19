use std::path::{Path, PathBuf};
use std::sync::Arc;
use hime_storage::{Database, DeveloperWorkspaceRecord};
use thiserror::Error;
use tracing::{info, warn};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum WorkspaceError {
    #[error("Workspace path does not exist or is invalid: {0}")]
    InvalidPath(String),

    #[error("Access denied: Path '{0}' is outside registered workspace root '{1}'.")]
    PathBoundaryViolation(String, String),

    #[error("Workspace '{0}' not found.")]
    WorkspaceNotFound(String),

    #[error("Dangerous path contains Alternate Data Stream (ADS) colon: {0}")]
    AdsColonBlocked(String),

    #[error("Database error: {0}")]
    DbError(String),
}

fn strip_unc_prefix(path_str: &str) -> String {
    if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        path_str.to_string()
    }
}

pub struct WorkspaceManager {
    storage: Arc<Database>,
}

impl WorkspaceManager {
    pub fn new(storage: Arc<Database>) -> Self {
        Self { storage }
    }

    /// Register a developer workspace path with command allowlist
    pub fn register_workspace(
        &self,
        name: &str,
        path: &Path,
        allowed_commands: Option<&str>,
    ) -> Result<DeveloperWorkspaceRecord, WorkspaceError> {
        let path_str = path.to_string_lossy().to_string();
        if path_str.contains(':') && path_str.matches(':').count() > 1 {
            return Err(WorkspaceError::AdsColonBlocked(path_str));
        }

        let canonical = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => {
                // In testing or fresh directories, create if not existing
                let _ = std::fs::create_dir_all(path);
                std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
            }
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let id = format!("ws-{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let default_cmds = allowed_commands.unwrap_or("git,cargo,pnpm,npm,node,python,uv,pytest,rustc,tsc,ruff,eslint");

        let record = DeveloperWorkspaceRecord {
            id: id.clone(),
            name: name.to_string(),
            path: canonical.to_string_lossy().to_string(),
            allowed_commands: Some(default_cmds.to_string()),
            created_at: now,
            updated_at: now,
        };

        self.storage
            .register_developer_workspace(&record)
            .map_err(|e| WorkspaceError::DbError(e.to_string()))?;

        info!(workspace_id = %id, path = %record.path, "Registered developer workspace");
        Ok(record)
    }

    /// Validate that a requested target file path is safely contained within workspace
    pub fn validate_file_access(&self, workspace_path: &Path, file_path: &Path) -> Result<PathBuf, WorkspaceError> {
        let ws_str = workspace_path.to_string_lossy();
        let target_str = file_path.to_string_lossy();

        if target_str.contains(':') && target_str.matches(':').count() > 1 {
            return Err(WorkspaceError::AdsColonBlocked(target_str.to_string()));
        }

        // 1. Explicit lexical traversal check
        for comp in file_path.components() {
            if comp == std::path::Component::ParentDir {
                return Err(WorkspaceError::PathBoundaryViolation(
                    target_str.to_string(),
                    ws_str.to_string(),
                ));
            }
        }

        let resolved = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            workspace_path.join(file_path)
        };

        let canonical_target = std::fs::canonicalize(&resolved).unwrap_or_else(|_| resolved.clone());
        let canonical_ws = std::fs::canonicalize(workspace_path).unwrap_or_else(|_| workspace_path.to_path_buf());

        let ws_norm = strip_unc_prefix(&canonical_ws.to_string_lossy()).replace('\\', "/").to_lowercase();
        let res_norm = strip_unc_prefix(&canonical_target.to_string_lossy()).replace('\\', "/").to_lowercase();

        if !res_norm.starts_with(&ws_norm) {
            warn!(resolved = %canonical_target.display(), ws = %canonical_ws.display(), "Workspace boundary violation");
            return Err(WorkspaceError::PathBoundaryViolation(
                target_str.to_string(),
                ws_str.to_string(),
            ));
        }

        Ok(resolved)
    }

    pub fn list_workspaces(&self) -> Result<Vec<DeveloperWorkspaceRecord>, WorkspaceError> {
        self.storage
            .list_developer_workspaces()
            .map_err(|e| WorkspaceError::DbError(e.to_string()))
    }
}
