use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;
use hime_storage::{Database, DeveloperCommandRunRecord};
use thiserror::Error;
use tracing::{info, warn};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RunnerError {
    #[error("Command '{0}' is not in the developer execution allowlist.")]
    DisallowedCommand(String),

    #[error("Command arguments contain forbidden shell chaining or redirection metacharacters.")]
    ShellInjectionBlocked,

    #[error("Destructive command requires explicit user approval: {0}")]
    ApprovalRequired(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Workspace error: {0}")]
    WorkspaceError(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandOutputResult {
    pub command: String,
    pub args: Vec<String>,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub is_success: bool,
}

pub struct DeveloperCommandRunner {
    storage: Arc<Database>,
    allowed_binaries: Vec<String>,
}

impl DeveloperCommandRunner {
    pub fn new(storage: Arc<Database>) -> Self {
        Self {
            storage,
            allowed_binaries: vec![
                "git".to_string(),
                "cargo".to_string(),
                "pnpm".to_string(),
                "npm".to_string(),
                "node".to_string(),
                "python".to_string(),
                "python3".to_string(),
                "uv".to_string(),
                "pytest".to_string(),
                "rustc".to_string(),
                "tsc".to_string(),
                "ruff".to_string(),
                "eslint".to_string(),
            ],
        }
    }

    /// Validate command safety, argument structure, and approval gates
    pub fn validate_command(
        &self,
        command: &str,
        args: &[String],
        is_approved: bool,
    ) -> Result<(), RunnerError> {
        let cmd_clean = Path::new(command)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(command)
            .to_lowercase();

        // 1. Allowlist check
        if !self.allowed_binaries.iter().any(|b| *b == cmd_clean) {
            warn!(command = %command, "Disallowed command execution attempted");
            return Err(RunnerError::DisallowedCommand(command.to_string()));
        }

        // 2. Shell injection metacharacters check across all arguments
        let dangerous_chars = [';', '&', '|', '`', '$', '>', '<', '\n', '\r'];
        for arg in args {
            if arg.chars().any(|c| dangerous_chars.contains(&c)) {
                warn!(arg = %arg, "Shell injection character blocked in argument");
                return Err(RunnerError::ShellInjectionBlocked);
            }
        }

        // 3. Destructive operations approval check
        if cmd_clean == "git" {
            let combined = args.join(" ").to_lowercase();
            if combined.contains("reset --hard")
                || combined.contains("push --force")
                || combined.contains("push -f")
                || combined.contains("clean -f")
            {
                if !is_approved {
                    return Err(RunnerError::ApprovalRequired(format!(
                        "git {}",
                        combined
                    )));
                }
            }
        }

        Ok(())
    }

    /// Execute command under workspace with output sanitization
    pub fn run_command(
        &self,
        workspace_id: &str,
        workspace_path: &Path,
        command: &str,
        args: &[String],
        is_approved: bool,
    ) -> Result<CommandOutputResult, RunnerError> {
        self.validate_command(command, args, is_approved)?;

        let start = Instant::now();
        info!(
            workspace = %workspace_id,
            command = %command,
            args = ?args,
            "Executing structured developer command"
        );

        let mut child = Command::new(command);
        child.args(args);
        child.current_dir(workspace_path);

        let output = match child.output() {
            Ok(out) => out,
            Err(e) => {
                return Err(RunnerError::ExecutionFailed(format!(
                    "Failed to launch command '{}': {}",
                    command, e
                )));
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let exit_code = output.status.code().unwrap_or(-1);

        let stdout_raw = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr_raw = String::from_utf8_lossy(&output.stderr).to_string();

        // Redact secrets using security helper
        let stdout_clean = hime_security::redaction::redact_sensitive_text(&stdout_raw);
        let stderr_clean = hime_security::redaction::redact_sensitive_text(&stderr_raw);

        // Record audit
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let run_record = DeveloperCommandRunRecord {
            id: format!("run-{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()),
            workspace_id: workspace_id.to_string(),
            command: command.to_string(),
            args: args.join(" "),
            exit_code: Some(exit_code),
            duration_ms: duration_ms as i64,
            is_allowed: true,
            output_snippet: Some(stdout_clean.chars().take(200).collect()),
            created_at: now,
        };
        let _ = self.storage.record_developer_command_run(&run_record);

        Ok(CommandOutputResult {
            command: command.to_string(),
            args: args.to_vec(),
            exit_code,
            stdout: stdout_clean,
            stderr: stderr_clean,
            duration_ms,
            is_success: output.status.success(),
        })
    }
}
