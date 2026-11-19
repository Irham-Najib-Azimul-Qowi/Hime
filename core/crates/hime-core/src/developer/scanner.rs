use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretFinding {
    pub file_path: String,
    pub line_number: usize,
    pub secret_type: String,
    pub masked_match: String,
}

pub struct SecretScanner;

impl SecretScanner {
    /// Scan file content for known secret patterns
    pub fn scan_text(file_path: &str, content: &str) -> Vec<SecretFinding> {
        let mut findings = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;

            // 1. Private keys
            if line.contains("-----BEGIN") && line.contains("PRIVATE KEY-----") {
                findings.push(SecretFinding {
                    file_path: file_path.to_string(),
                    line_number: line_num,
                    secret_type: "Private Key Header".to_string(),
                    masked_match: "-----BEGIN ... PRIVATE KEY-----".to_string(),
                });
            }

            // 2. AWS Access Key ID
            if let Some(pos) = line.find("AKIA") {
                if line.len() >= pos + 20 {
                    let candidate = &line[pos..pos + 20];
                    if candidate.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
                        findings.push(SecretFinding {
                            file_path: file_path.to_string(),
                            line_number: line_num,
                            secret_type: "AWS Access Key".to_string(),
                            masked_match: format!("AKIA{}...", &candidate[4..8]),
                        });
                    }
                }
            }

            // 3. GitHub Personal Access Token
            if let Some(pos) = line.find("ghp_") {
                if line.len() >= pos + 40 {
                    let _candidate = &line[pos..pos + 40];
                    findings.push(SecretFinding {
                        file_path: file_path.to_string(),
                        line_number: line_num,
                        secret_type: "GitHub Personal Access Token".to_string(),
                        masked_match: "ghp_****".to_string(),
                    });
                }
            }

            // 4. Hardcoded password or secret keywords
            let lower = line.to_lowercase();
            if (lower.contains("secret_key =") || lower.contains("api_key =") || lower.contains("password ="))
                && (line.contains('"') || line.contains('\''))
            {
                findings.push(SecretFinding {
                    file_path: file_path.to_string(),
                    line_number: line_num,
                    secret_type: "Plaintext Secret Assignment".to_string(),
                    masked_match: "[REDACTED ASSIGNMENT]".to_string(),
                });
            }
        }

        findings
    }

    /// Scan a physical file
    pub fn scan_file(path: &Path) -> Result<Vec<SecretFinding>, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::scan_text(&path.to_string_lossy(), &content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_scanner_detects_keys() {
        let mock_env = "DATABASE_URL=postgres://localhost/db\n\
                        AWS_KEY=AKIAIOSFODNN7EXAMPLE\n\
                        GH_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz0123456789\n\
                        NORMAL_TEXT=hello world";

        let findings = SecretScanner::scan_text(".env", mock_env);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].secret_type, "AWS Access Key");
        assert_eq!(findings[1].secret_type, "GitHub Personal Access Token");
    }
}
