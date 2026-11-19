use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use super::scanner::{SecretFinding, SecretScanner};

#[derive(Error, Debug)]
pub enum GitOpError {
    #[error("Git execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Commit blocked: SecretScanner found potential credentials: {0:?}")]
    SecretDetected(Vec<SecretFinding>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusResult {
    pub current_branch: String,
    pub staged_files: Vec<String>,
    pub modified_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub is_clean: bool,
}

pub struct GitOperations;

impl GitOperations {
    /// Get parsed Git status for workspace
    pub fn status(workspace_path: &Path) -> Result<GitStatusResult, GitOpError> {
        let out = Command::new("git")
            .args(["status", "--porcelain=v1", "-b"])
            .current_dir(workspace_path)
            .output()
            .map_err(|e| GitOpError::ExecutionFailed(e.to_string()))?;

        let text = String::from_utf8_lossy(&out.stdout);
        let mut current_branch = "unknown".to_string();
        let mut staged_files = Vec::new();
        let mut modified_files = Vec::new();
        let mut untracked_files = Vec::new();

        for line in text.lines() {
            if let Some(branch_part) = line.strip_prefix("## ") {
                current_branch = branch_part.split("...").next().unwrap_or(branch_part).trim().to_string();
                continue;
            }

            if line.len() < 3 {
                continue;
            }

            let index_status = &line[0..1];
            let work_status = &line[1..2];
            let filename = line[3..].trim().to_string();

            if index_status == "?" && work_status == "?" {
                untracked_files.push(filename);
            } else {
                if index_status != " " && index_status != "?" {
                    staged_files.push(filename.clone());
                }
                if work_status != " " && work_status != "?" {
                    modified_files.push(filename);
                }
            }
        }

        let is_clean = staged_files.is_empty() && modified_files.is_empty() && untracked_files.is_empty();

        Ok(GitStatusResult {
            current_branch,
            staged_files,
            modified_files,
            untracked_files,
            is_clean,
        })
    }

    /// Get unified diff
    pub fn diff(workspace_path: &Path, cached: bool) -> Result<String, GitOpError> {
        let mut cmd = Command::new("git");
        cmd.arg("diff");
        if cached {
            cmd.arg("--cached");
        }
        cmd.current_dir(workspace_path);

        let out = cmd.output().map_err(|e| GitOpError::ExecutionFailed(e.to_string()))?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    /// Commit staged changes with automatic pre-commit secret scanning
    pub fn commit(
        workspace_path: &Path,
        message: &str,
        scan_secrets: bool,
    ) -> Result<String, GitOpError> {
        if scan_secrets {
            let diff_text = Self::diff(workspace_path, true)?;
            let findings = SecretScanner::scan_text("staged-diff", &diff_text);
            if !findings.is_empty() {
                return Err(GitOpError::SecretDetected(findings));
            }
        }

        let out = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(workspace_path)
            .output()
            .map_err(|e| GitOpError::ExecutionFailed(e.to_string()))?;

        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(GitOpError::ExecutionFailed(err.to_string()));
        }

        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        info!(workspace = %workspace_path.display(), message = %message, "Git commit completed");
        Ok(stdout)
    }
}

