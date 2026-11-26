use std::path::Path;
use thiserror::Error;

/// Granular risk classification for system operations (L0 to L4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum RiskLevel {
    /// L0: Read-only informational operations (metadata lookup, ping, read safe file).
    L0ReadOnly = 0,
    /// L1: Non-destructive create operations in temporary/workspace areas.
    L1CreateSafe = 1,
    /// L2: Modify or move operations on safe workspace files, tasks update.
    L2Modify = 2,
    /// L3: Destructive operations (deletion, file overwrite, session termination).
    L3Destructive = 3,
    /// L4: Critical system operations (shell command execution, OS configuration changes).
    L4System = 4,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Action {
    SystemPing,
    SystemInfo,
    FileRead { path: String },
    FileWrite { path: String },
    FileDelete { path: String },
    ExecuteProcess { binary: String, args: Vec<String> },
    NetworkRequest { host: String, port: u16 },
    ToolCall {
        tool_id: String,
        domain: String,
        level: RiskLevel,
        payload: String,
    },
    TranslateText { text_length: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PolicyDecision {
    Allow,
    Deny { reason: String },
    RequireUserApproval { description: String },
}

pub trait PolicyEnforcer: Send + Sync {
    fn evaluate(&self, client_id: &str, action: &Action) -> PolicyDecision;
    fn is_superadmin(&self) -> bool {
        false
    }
    fn set_superadmin(&self, _enabled: bool) {}
}

/// Comprehensive Declarative Security Policy Engine for Hime Core.
///
/// Evaluates operations against allowlists, denylists, path traversal attacks,
/// NTFS alternate data streams, and forbidden shell commands deterministically.
pub struct PolicyEngine {
    restricted_prefixes: Vec<String>,
    sensitive_path_keywords: Vec<String>,
    forbidden_command_patterns: Vec<String>,
    reserved_device_names: Vec<String>,
    superadmin_mode: std::sync::atomic::AtomicBool,
}

impl PolicyEngine {
    pub fn new() -> Self {
        let superadmin = std::env::var("HIME_SUPERADMIN")
            .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
            .unwrap_or(false);
        Self::with_superadmin(superadmin)
    }

    pub fn with_superadmin(superadmin: bool) -> Self {
        Self {
            restricted_prefixes: vec![
                r"c:\windows".to_string(),
                r"c:\program files".to_string(),
                r"c:\program files (x86)".to_string(),
                r"c:\programdata\microsoft".to_string(),
                r"c:\recovery".to_string(),
                r"/etc".to_string(),
                r"/usr".to_string(),
                r"/bin".to_string(),
                r"/sbin".to_string(),
                r"/sys".to_string(),
                r"/proc".to_string(),
            ],
            sensitive_path_keywords: vec![
                r"\.ssh".to_string(),
                r"\.aws".to_string(),
                r"\.gnupg".to_string(),
                r"\id_rsa".to_string(),
                r"\id_ed25519".to_string(),
                r"\cookies".to_string(),
                "/.ssh".to_string(),
                "/.aws".to_string(),
                "/.gnupg".to_string(),
            ],
            forbidden_command_patterns: vec![
                "rmdir /s".to_string(),
                "del /f /s /q".to_string(),
                "format ".to_string(),
                "powershell stop-computer".to_string(),
                "invoke-expression".to_string(),
                "iex ".to_string(),
                "rm -rf /".to_string(),
                ":(){ :|:& };:".to_string(),
            ],
            reserved_device_names: vec![
                "con".to_string(),
                "prn".to_string(),
                "aux".to_string(),
                "nul".to_string(),
                "com1".to_string(),
                "com2".to_string(),
                "com3".to_string(),
                "com4".to_string(),
                "com5".to_string(),
                "com6".to_string(),
                "com7".to_string(),
                "com8".to_string(),
                "com9".to_string(),
                "lpt1".to_string(),
                "lpt2".to_string(),
                "lpt3".to_string(),
                "lpt4".to_string(),
                "lpt5".to_string(),
                "lpt6".to_string(),
                "lpt7".to_string(),
                "lpt8".to_string(),
                "lpt9".to_string(),
            ],
            superadmin_mode: std::sync::atomic::AtomicBool::new(superadmin),
        }
    }

    pub fn is_superadmin(&self) -> bool {
        self.superadmin_mode.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_superadmin(&self, enabled: bool) {
        self.superadmin_mode.store(enabled, std::sync::atomic::Ordering::Relaxed);
    }

    /// Verifies whether a path string contains path traversal tokens or dangerous constructs.
    pub fn validate_path_safety(&self, path_str: &str) -> Result<(), String> {
        let normalized = path_str.to_lowercase().replace('/', "\\");

        // 1. Detect Path Traversal `..` escape attempts
        if path_str.contains("..") {
            return Err("Path traversal attempt detected ('..')".to_string());
        }

        // 2. Detect NTFS Alternate Data Streams (colon outside drive letter)
        let colon_indices: Vec<_> = path_str.match_indices(':').map(|(i, _)| i).collect();
        for &idx in &colon_indices {
            // Drive letter colon is valid only at index 1 (e.g. C:\)
            if idx != 1 {
                return Err("NTFS Alternate Data Stream (ADS) syntax forbidden".to_string());
            }
        }

        // 3. Detect Reserved Windows Device Names (CON, PRN, AUX, NUL, COM1..9, LPT1..9)
        let file_stem = Path::new(path_str)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if self.reserved_device_names.contains(&file_stem) {
            return Err(format!(
                "Path refers to reserved Windows device name '{}'",
                file_stem
            ));
        }

        // If in Superadmin mode, allow accessing system directories and custom paths without restriction
        if self.is_superadmin() {
            return Ok(());
        }

        // 4. Detect System Directory Access
        for prefix in &self.restricted_prefixes {
            if normalized.starts_with(prefix) {
                return Err(format!("Access to system directory '{}' restricted", prefix));
            }
        }

        // 5. Detect Sensitive Credential Directories (.ssh, .aws, etc.)
        for keyword in &self.sensitive_path_keywords {
            if normalized.contains(keyword) {
                return Err(format!(
                    "Access to sensitive credential path '{}' denied",
                    keyword
                ));
            }
        }

        Ok(())
    }

    /// Checks if a command line string contains forbidden destructive patterns.
    pub fn is_forbidden_command(&self, binary: &str, args: &[String]) -> bool {
        let full_cmd = format!("{} {}", binary, args.join(" ")).to_lowercase();
        self.forbidden_command_patterns
            .iter()
            .any(|pattern| full_cmd.contains(pattern))
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyEnforcer for PolicyEngine {
    fn is_superadmin(&self) -> bool {
        self.superadmin_mode.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn set_superadmin(&self, enabled: bool) {
        self.superadmin_mode.store(enabled, std::sync::atomic::Ordering::Relaxed);
    }

    fn evaluate(&self, _client_id: &str, action: &Action) -> PolicyDecision {
        // In Superadmin mode, grant unrestricted execution for system operations and tools
        if self.is_superadmin() {
            match action {
                Action::ExecuteProcess { binary, args } => {
                    // Prevent catastrophic self-destruction patterns like rm -rf / or format
                    if self.is_forbidden_command(binary, args) {
                        return PolicyDecision::Deny {
                            reason: "Command contains forbidden destructive patterns".to_string(),
                        };
                    }
                    return PolicyDecision::Allow;
                }
                Action::FileRead { path } | Action::FileWrite { path } | Action::FileDelete { path } => {
                    if let Err(reason) = self.validate_path_safety(path) {
                        return PolicyDecision::Deny { reason };
                    }
                    return PolicyDecision::Allow;
                }
                _ => return PolicyDecision::Allow,
            }
        }

        match action {
            Action::SystemPing | Action::SystemInfo => PolicyDecision::Allow,

            Action::FileRead { path } => {
                if let Err(reason) = self.validate_path_safety(path) {
                    PolicyDecision::Deny { reason }
                } else {
                    PolicyDecision::Allow
                }
            }

            Action::FileWrite { path } => {
                if let Err(reason) = self.validate_path_safety(path) {
                    PolicyDecision::Deny { reason }
                } else {
                    PolicyDecision::RequireUserApproval {
                        description: format!("File write requested for target: {}", path),
                    }
                }
            }

            Action::FileDelete { path } => {
                if let Err(reason) = self.validate_path_safety(path) {
                    PolicyDecision::Deny { reason }
                } else {
                    PolicyDecision::RequireUserApproval {
                        description: format!("Destructive file deletion requested: {}", path),
                    }
                }
            }

            Action::ExecuteProcess { binary, args } => {
                if self.is_forbidden_command(binary, args) {
                    PolicyDecision::Deny {
                        reason: "Command contains forbidden destructive patterns".to_string(),
                    }
                } else {
                    PolicyDecision::RequireUserApproval {
                        description: format!(
                            "Execution of external process requested: {} with {} args",
                            binary,
                            args.len()
                        ),
                    }
                }
            }

            Action::NetworkRequest { host, port } => {
                if host == "127.0.0.1" || host == "localhost" {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::RequireUserApproval {
                        description: format!(
                            "Outbound network connection requested to {}:{}",
                            host, port
                        ),
                    }
                }
            }

            Action::ToolCall {
                tool_id,
                domain: _,
                level,
                payload: _,
            } => match level {
                RiskLevel::L0ReadOnly | RiskLevel::L1CreateSafe => PolicyDecision::Allow,
                RiskLevel::L2Modify => PolicyDecision::Allow,
                RiskLevel::L3Destructive => PolicyDecision::RequireUserApproval {
                    description: format!(
                        "Tool '{}' requires user approval (Destructive Action)",
                        tool_id
                    ),
                },
                RiskLevel::L4System => PolicyDecision::RequireUserApproval {
                    description: format!(
                        "Tool '{}' requires explicit confirmation (System Level Action)",
                        tool_id
                    ),
                },
            },

            Action::TranslateText { text_length } => {
                if *text_length > 10_000 {
                    PolicyDecision::Deny {
                        reason: "Text selection exceeds maximum allowed translation buffer (10k chars)".to_string(),
                    }
                } else {
                    PolicyDecision::Allow
                }
            }
        }
    }
}

/// Backward-compatible alias for existing code referencing `DefaultSecurityPolicy`.
pub type DefaultSecurityPolicy = PolicyEngine;

#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("Action denied by policy: {0}")]
    Denied(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_decisions() {
        let policy = DefaultSecurityPolicy::new();

        assert_eq!(
            policy.evaluate("client", &Action::SystemPing),
            PolicyDecision::Allow
        );

        assert_eq!(
            policy.evaluate("client", &Action::SystemInfo),
            PolicyDecision::Allow
        );

        // System directory write denied
        assert!(matches!(
            policy.evaluate(
                "client",
                &Action::FileWrite {
                    path: r"C:\Windows\System32\malicious.dll".to_string()
                }
            ),
            PolicyDecision::Deny { .. }
        ));

        // Normal file write requires approval
        assert!(matches!(
            policy.evaluate(
                "client",
                &Action::FileWrite {
                    path: r"C:\Users\test\workspace\document.txt".to_string()
                }
            ),
            PolicyDecision::RequireUserApproval { .. }
        ));

        // External binary requires approval
        assert!(matches!(
            policy.evaluate(
                "client",
                &Action::ExecuteProcess {
                    binary: "cmd.exe".to_string(),
                    args: vec!["/c".to_string(), "dir".to_string()]
                }
            ),
            PolicyDecision::RequireUserApproval { .. }
        ));

        // Forbidden destructive command denied
        assert!(matches!(
            policy.evaluate(
                "client",
                &Action::ExecuteProcess {
                    binary: "cmd.exe".to_string(),
                    args: vec!["/c".to_string(), "rmdir /s c:\\test".to_string()]
                }
            ),
            PolicyDecision::Deny { .. }
        ));
    }

    #[test]
    fn test_path_traversal_and_ads_prevention() {
        let policy = PolicyEngine::new();

        // Path traversal with ..
        assert!(policy
            .validate_path_safety("docs/../../Windows/System32")
            .is_err());

        // NTFS Alternate Data Streams
        assert!(policy
            .validate_path_safety("test.txt:hidden_stream")
            .is_err());

        // Reserved Windows device name
        assert!(policy.validate_path_safety("con.txt").is_err());
        assert!(policy.validate_path_safety("nul").is_err());
        assert!(policy.validate_path_safety("com1.log").is_err());

        // Sensitive .ssh folder
        assert!(policy
            .validate_path_safety(r"C:\Users\Satoshi\.ssh\id_rsa")
            .is_err());

        // Valid safe relative and workspace paths
        assert!(policy.validate_path_safety("docs/notes.md").is_ok());
        assert!(policy
            .validate_path_safety(r"C:\Users\Satoshi\Projects\Hime\README.md")
            .is_ok());
    }

    #[test]
    fn test_tool_risk_levels() {
        let policy = PolicyEngine::new();

        // L0 is allowed
        assert_eq!(
            policy.evaluate(
                "client",
                &Action::ToolCall {
                    tool_id: "system.info".to_string(),
                    domain: "system".to_string(),
                    level: RiskLevel::L0ReadOnly,
                    payload: "{}".to_string()
                }
            ),
            PolicyDecision::Allow
        );

        // L3 requires approval
        assert!(matches!(
            policy.evaluate(
                "client",
                &Action::ToolCall {
                    tool_id: "files.delete".to_string(),
                    domain: "files".to_string(),
                    level: RiskLevel::L3Destructive,
                    payload: "{}".to_string()
                }
            ),
            PolicyDecision::RequireUserApproval { .. }
        ));
    }

    #[test]
    fn test_superadmin_mode() {
        let policy = PolicyEngine::with_superadmin(true);
        assert!(policy.is_superadmin());

        // FileWrite allowed without approval
        assert_eq!(
            policy.evaluate(
                "client",
                &Action::FileWrite {
                    path: r"C:\Windows\System32\drivers\etc\hosts".to_string()
                }
            ),
            PolicyDecision::Allow
        );

        // System directories accessible in superadmin mode
        assert!(policy.validate_path_safety(r"C:\Windows\System32\notepad.exe").is_ok());
        assert!(policy.validate_path_safety(r"C:\Program Files\App\config.json").is_ok());

        // ToolCall L4 allowed directly without blocking approval hurdles
        assert_eq!(
            policy.evaluate(
                "client",
                &Action::ToolCall {
                    tool_id: "system.execute_task".to_string(),
                    domain: "system".to_string(),
                    level: RiskLevel::L4System,
                    payload: "{}".to_string()
                }
            ),
            PolicyDecision::Allow
        );

        // Destructive catastrophic patterns still rejected for safety
        assert!(matches!(
            policy.evaluate(
                "client",
                &Action::ExecuteProcess {
                    binary: "cmd.exe".to_string(),
                    args: vec!["/c".to_string(), "format c:".to_string()]
                }
            ),
            PolicyDecision::Deny { .. }
        ));
    }
}
