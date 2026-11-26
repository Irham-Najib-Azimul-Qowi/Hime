use crate::storage::operations::{FileOpError, SafeFileOperationsEngine};
use hime_storage::Database;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
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
    pub file_name: String,
    pub status: DownloadStatus,
    pub notification_prompt: Option<String>,
    pub timestamp_ms: u64,
}

pub struct DownloadIntelligence {
    db: Arc<Database>,
    ops_engine: Arc<SafeFileOperationsEngine>,
    seen_files: Mutex<HashSet<PathBuf>>,
    recent_reports: Mutex<Vec<DownloadEventReport>>,
}

impl DownloadIntelligence {
    pub fn new(db: Arc<Database>, ops_engine: Arc<SafeFileOperationsEngine>) -> Self {
        Self {
            db,
            ops_engine,
            seen_files: Mutex::new(HashSet::new()),
            recent_reports: Mutex::new(Vec::new()),
        }
    }

    /// Evaluates a file in Downloads directory.
    pub fn inspect_download(&self, path: &Path) -> Result<DownloadEventReport, FileOpError> {
        let path_str = path.to_string_lossy();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // 1. Check if it is an incomplete download
        if path_str.ends_with(".crdownload") || path_str.ends_with(".part") || path_str.ends_with(".tmp") {
            return Ok(DownloadEventReport {
                download_path: path_str.to_string(),
                file_name,
                status: DownloadStatus::InProgress,
                notification_prompt: None,
                timestamp_ms: now,
            });
        }

        if !path.exists() || !path.is_file() {
            return Err(FileOpError::SourceNotFound(format!("File does not exist: {:?}", path)));
        }

        let file_hash = SafeFileOperationsEngine::compute_sha256(path)?;

        // 2. Check for exact duplicate in database
        let exact_matches = self.db.find_files_by_hash(&file_hash)?;
        for f in exact_matches {
            if Path::new(&f.path) != path {
                return Ok(DownloadEventReport {
                    download_path: path_str.to_string(),
                    file_name: file_name.clone(),
                    status: DownloadStatus::ExactDuplicate {
                        existing_path: f.path.clone(),
                    },
                    notification_prompt: Some(format!(
                        "Exact duplicate detected: '{}' is identical to existing file '{}'. Would you like to discard or keep both?",
                        file_name, f.path
                    )),
                    timestamp_ms: now,
                });
            }
        }

        // 3. Check for name match or version candidate
        let similar_files = self.db.search_indexed_files(&file_name, 10)?;
        for f in similar_files {
            if Path::new(&f.path) != path && f.file_name == file_name {
                return Ok(DownloadEventReport {
                    download_path: path_str.to_string(),
                    file_name: file_name.clone(),
                    status: DownloadStatus::NewVersionCandidate {
                        existing_path: f.path.clone(),
                        suggested_action: "Archive existing file to Hime Recycle as older version and place new download into folder".to_string(),
                    },
                    notification_prompt: Some(format!(
                        "File '{}' already exists at '{}'. Is this a newer version? Options: replace & archive old, keep both, or discard.",
                        file_name, f.path
                    )),
                    timestamp_ms: now,
                });
            }
        }

        Ok(DownloadEventReport {
            download_path: path_str.to_string(),
            file_name,
            status: DownloadStatus::NewUniqueFile,
            notification_prompt: None,
            timestamp_ms: now,
        })
    }

    /// Scans a directory (such as Downloads) for newly added completed files.
    pub fn scan_and_inspect_directory(&self, dir: &Path) -> Vec<DownloadEventReport> {
        let mut new_reports = Vec::new();
        if !dir.exists() || !dir.is_dir() {
            return new_reports;
        }

        let read_dir = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => return new_reports,
        };

        let mut seen = self.seen_files.lock().unwrap();
        // If seen is empty (initial run), seed it with current files so we don't spam for all existing downloads
        if seen.is_empty() {
            for entry in read_dir.flatten() {
                seen.insert(entry.path());
            }
            return new_reports;
        }

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_file() && !seen.contains(&path) {
                let p_str = path.to_string_lossy();
                if p_str.ends_with(".crdownload") || p_str.ends_with(".part") || p_str.ends_with(".tmp") {
                    continue;
                }
                seen.insert(path.clone());
                if let Ok(report) = self.inspect_download(&path) {
                    new_reports.push(report.clone());
                    self.recent_reports.lock().unwrap().push(report);
                }
            }
        }

        new_reports
    }

    /// Drains and returns all recent download reports since last poll.
    pub fn drain_recent_reports(&self) -> Vec<DownloadEventReport> {
        let mut lock = self.recent_reports.lock().unwrap();
        std::mem::take(&mut *lock)
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
