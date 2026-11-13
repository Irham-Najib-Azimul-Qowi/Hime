use crate::storage::operations::{FileOpError, SafeFileOperationsEngine};
use hime_storage::Database;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DownloadStatus {
    InProgress,
    ExactDuplicate { existing_path: String },
    NewVersionCandidate { existing_path: String, suggested_action: String },
    NewUniqueFile,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadEventReport {
    pub download_path: String,
    pub status: DownloadStatus,
    pub notification_prompt: Option<String>,
}

pub struct DownloadIntelligence {
    db: Arc<Database>,
    ops_engine: Arc<SafeFileOperationsEngine>,
}

impl DownloadIntelligence {
    pub fn new(db: Arc<Database>, ops_engine: Arc<SafeFileOperationsEngine>) -> Self {
        Self { db, ops_engine }
    }

    /// Evaluates a file in Downloads directory.
    pub fn inspect_download(&self, path: &Path) -> Result<DownloadEventReport, FileOpError> {
        let path_str = path.to_string_lossy();

        // 1. Check if it is an incomplete download
        if path_str.ends_with(".crdownload") || path_str.ends_with(".part") || path_str.ends_with(".tmp") {
            return Ok(DownloadEventReport {
                download_path: path_str.to_string(),
                status: DownloadStatus::InProgress,
                notification_prompt: None,
            });
        }

        if !path.exists() || !path.is_file() {
            return Err(FileOpError::SourceNotFound(format!("File does not exist: {:?}", path)));
        }

        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let file_hash = SafeFileOperationsEngine::compute_sha256(path)?;

        // 2. Check for exact duplicate in database
        let exact_matches = self.db.find_files_by_hash(&file_hash)?;
        for f in exact_matches {
            if Path::new(&f.path) != path {
                return Ok(DownloadEventReport {
                    download_path: path_str.to_string(),
                    status: DownloadStatus::ExactDuplicate {
                        existing_path: f.path.clone(),
                    },
                    notification_prompt: Some(format!(
                        "Exact duplicate detected: '{}' is identical to existing file '{}'. Would you like to discard or keep both?",
                        file_name, f.path
                    )),
                });
            }
        }

        // 3. Check for name match or version candidate
        let similar_files = self.db.search_indexed_files(&file_name, 10)?;
        for f in similar_files {
            if Path::new(&f.path) != path && f.file_name == file_name {
                return Ok(DownloadEventReport {
                    download_path: path_str.to_string(),
                    status: DownloadStatus::NewVersionCandidate {
                        existing_path: f.path.clone(),
                        suggested_action: "Archive existing file to Hime Recycle as older version and place new download into folder".to_string(),
                    },
                    notification_prompt: Some(format!(
                        "File '{}' already exists at '{}'. Is this a newer version? Options: replace & archive old, keep both, or discard.",
                        file_name, f.path
                    )),
                });
            }
        }

        Ok(DownloadEventReport {
            download_path: path_str.to_string(),
            status: DownloadStatus::NewUniqueFile,
            notification_prompt: None,
        })
    }

    /// Resolves download conflict by archiving existing file and moving new download in place.
    pub fn replace_and_archive_old(
        &self,
        new_download: &Path,
        existing_file: &Path,
        actor: &str,
    ) -> Result<crate::storage::operations::OperationResult, FileOpError> {
        let op_id = format!("dl-replace-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());
        info!("Replacing old file {:?} with new download {:?}", existing_file, new_download);

        // First archive/move old file to recycle
        self.ops_engine.safe_delete(&format!("{}-del", op_id), existing_file, actor, None)?;

        // Now move new download to existing location
        self.ops_engine.safe_move(&op_id, new_download, existing_file, actor, None)
    }
}

use std::time::{SystemTime, UNIX_EPOCH};
