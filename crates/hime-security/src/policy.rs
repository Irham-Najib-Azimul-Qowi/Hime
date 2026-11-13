use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    SystemPing,
    FileRead { path: String },
    FileWrite { path: String },
    ExecuteProcess { binary: String, args: Vec<String> },
    NetworkRequest { host: String, port: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny { reason: String },
    RequireUserApproval { description: String },
}

pub trait PolicyEnforcer: Send + Sync {
    fn evaluate(&self, client_id: &str, action: &Action) -> PolicyDecision;
}

pub struct DefaultSecurityPolicy {
    restricted_prefixes: Vec<String>,
}

impl DefaultSecurityPolicy {
    pub fn new() -> Self {
        Self {
            restricted_prefixes: vec![
                r"C:\Windows".to_lowercase(),
                r"C:\Program Files".to_lowercase(),
                r"C:\Program Files (x86)".to_lowercase(),
            ],
        }
    }

    fn is_restricted_path(&self, path_str: &str) -> bool {
        let p = path_str.to_lowercase();
        self.restricted_prefixes
            .iter()
            .any(|prefix| p.starts_with(prefix))
    }
}

impl Default for DefaultSecurityPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyEnforcer for DefaultSecurityPolicy {
    fn evaluate(&self, _client_id: &str, action: &Action) -> PolicyDecision {
        match action {
            Action::SystemPing => PolicyDecision::Allow,
            Action::FileRead { path } => {
                let p = Path::new(path);
                if p.is_relative() {
                    PolicyDecision::Allow
                } else if self.is_restricted_path(path) {
                    PolicyDecision::Deny {
                        reason: "Access to system directories is restricted by default policy"
                            .to_string(),
                    }
                } else {
                    PolicyDecision::Allow
                }
            }
            Action::FileWrite { path } => {
                if self.is_restricted_path(path) {
                    PolicyDecision::Deny {
                        reason: "Writing to system directories is strictly prohibited".to_string(),
                    }
                } else {
                    PolicyDecision::RequireUserApproval {
                        description: format!("Subprocess requests write access to file: {}", path),
                    }
                }
            }
            Action::ExecuteProcess { binary, .. } => PolicyDecision::RequireUserApproval {
                description: format!(
                    "Subprocess requests execution of external binary: {}",
                    binary
                ),
            },
            Action::NetworkRequest { host, port } => {
                if host == "127.0.0.1" || host == "localhost" {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::RequireUserApproval {
                        description: format!(
                            "Outbound network access requested to {}:{}",
                            host, port
                        ),
                    }
                }
            }
        }
    }
}

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

        assert!(matches!(
            policy.evaluate(
                "client",
                &Action::FileWrite {
                    path: r"C:\Windows\System32\malicious.dll".to_string()
                }
            ),
            PolicyDecision::Deny { .. }
        ));

        assert!(matches!(
            policy.evaluate(
                "client",
                &Action::ExecuteProcess {
                    binary: "cmd.exe".to_string(),
                    args: vec![]
                }
            ),
            PolicyDecision::RequireUserApproval { .. }
        ));
    }
}
