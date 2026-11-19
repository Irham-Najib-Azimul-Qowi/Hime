use crate::storage::operations::FileOpError;
use hime_storage::{Database, StorageAnalysisReport};
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CleanableSuggestion {
    pub category: String,
    pub path: String,
    pub size_bytes: i64,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetailedStorageReport {
    pub base_report: StorageAnalysisReport,
    pub cleanable_suggestions: Vec<CleanableSuggestion>,
    pub cleanable_bytes: i64,
}

pub struct AnalyzerService {
    db: Arc<Database>,
}

impl AnalyzerService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn generate_report(&self, top_n: usize) -> Result<DetailedStorageReport, FileOpError> {
        let base_report = self.db.get_storage_analysis(top_n)?;

        let mut cleanable_suggestions = Vec::new();
        let mut cleanable_bytes = 0i64;

        // 1. Analyze old downloads for installers or archives
        for f in &base_report.old_downloads {
            let ext = f.extension.as_deref().unwrap_or("");
            if matches!(ext, "exe" | "msi" | "iso" | "dmg" | "zip" | "rar" | "7z") {
                cleanable_bytes += f.size_bytes;
                cleanable_suggestions.push(CleanableSuggestion {
                    category: "Old Download Installer/Archive".to_string(),
                    path: f.path.clone(),
                    size_bytes: f.size_bytes,
                    reason: format!("Installer/archive (.{}) in Downloads older than 30 days", ext),
                });
            }
        }

        // 2. Search for temporary or cache files
        let temp_files = self.db.search_indexed_files(".tmp", 20)?;
        for f in temp_files {
            if !f.is_dir && f.path.ends_with(".tmp") {
                cleanable_bytes += f.size_bytes;
                cleanable_suggestions.push(CleanableSuggestion {
                    category: "Temporary File".to_string(),
                    path: f.path,
                    size_bytes: f.size_bytes,
                    reason: "Temporary .tmp file".to_string(),
                });
            }
        }

        Ok(DetailedStorageReport {
            base_report,
            cleanable_suggestions,
            cleanable_bytes,
        })
    }
}
