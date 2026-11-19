use crate::storage::operations::{FileOpError, SafeFileOperationsEngine};
use hime_storage::{Database, FileRuleRecord, IndexedFileRecord};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrganizationProposal {
    pub proposal_id: String,
    pub source_path: String,
    pub suggested_destination: String,
    pub rule_id: String,
    pub rule_name: String,
    pub reason: String,
}

pub struct SmartOrganizer {
    db: Arc<Database>,
    ops_engine: Arc<SafeFileOperationsEngine>,
}

impl SmartOrganizer {
    pub fn new(db: Arc<Database>, ops_engine: Arc<SafeFileOperationsEngine>) -> Self {
        Self { db, ops_engine }
    }

    /// Evaluates indexed files in a directory against all active rules to generate proposals.
    pub fn evaluate_proposals(&self, parent_dir: &str) -> Result<Vec<OrganizationProposal>, FileOpError> {
        let rules = self.db.list_file_rules()?;
        let active_rules: Vec<FileRuleRecord> = rules.into_iter().filter(|r| r.is_active).collect();

        if active_rules.is_empty() {
            return Ok(Vec::new());
        }

        let files = self.db.list_indexed_files_in_dir(parent_dir)?;
        let mut proposals = Vec::new();

        for file in files {
            if file.is_dir {
                continue;
            }

            for rule in &active_rules {
                if Self::matches_pattern(&file, &rule.pattern) {
                    let target_dir = Path::new(&rule.target_dir);
                    let target_name = if let Some(template) = &rule.naming_template {
                        Self::format_template(template, &file)
                    } else {
                        file.file_name.clone()
                    };

                    let destination = target_dir.join(&target_name);

                    // Skip if file is already in the target directory with that name
                    if Path::new(&file.path) == destination {
                        continue;
                    }

                    let proposal_id = format!("prop-{}", uuid_simple());
                    proposals.push(OrganizationProposal {
                        proposal_id,
                        source_path: file.path.clone(),
                        suggested_destination: destination.to_string_lossy().to_string(),
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        reason: format!("Matches rule '{}' ({})", rule.name, rule.pattern),
                    });
                    break; // Matched first rule
                }
            }
        }

        Ok(proposals)
    }

    /// Executes an approved proposal using the SafeFileOperationsEngine.
    pub fn execute_proposal(
        &self,
        proposal: &OrganizationProposal,
        actor: &str,
        approval_id: Option<&str>,
    ) -> Result<crate::storage::operations::OperationResult, FileOpError> {
        let op_id = format!("op-{}", uuid_simple());
        info!(
            "Executing organization proposal {} for {:?}",
            proposal.proposal_id, proposal.source_path
        );

        self.ops_engine.safe_move(
            &op_id,
            Path::new(&proposal.source_path),
            Path::new(&proposal.suggested_destination),
            actor,
            approval_id,
        )
    }

    fn matches_pattern(file: &IndexedFileRecord, pattern: &str) -> bool {
        let pat = pattern.trim().to_lowercase();
        let fname = file.file_name.to_lowercase();
        let ext = file.extension.as_deref().unwrap_or("").to_lowercase();

        if let Some(target_ext) = pat.strip_prefix("*.") {
            ext == target_ext
        } else if pat.starts_with('*') && pat.ends_with('*') && pat.len() > 2 {
            let inner = &pat[1..pat.len() - 1];
            fname.contains(inner)
        } else {
            fname == pat
        }
    }

    fn format_template(template: &str, file: &IndexedFileRecord) -> String {
        let ext = file.extension.as_deref().unwrap_or("");
        let stem = Path::new(&file.file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&file.file_name);

        let date_str = chrono_simple_date();

        template
            .replace("{name}", stem)
            .replace("{ext}", ext)
            .replace("{date}", &date_str)
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{:x}", now)
}

fn chrono_simple_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    // Rough YYYY-MM-DD approximation from epoch
    let days = secs / 86400;
    format!("day-{}", days)
}
