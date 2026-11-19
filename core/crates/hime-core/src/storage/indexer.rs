use crate::storage::operations::FileOpError;
use hime_storage::{Database, IndexedFileRecord};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

pub struct IndexerService {
    db: Arc<Database>,
    max_file_size_for_hashing: u64,
    ignored_directories: Vec<String>,
}

impl IndexerService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            max_file_size_for_hashing: 50 * 1024 * 1024, // 50 MB
            ignored_directories: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                ".cache".to_string(),
                "target".to_string(),
                "dist".to_string(),
                ".vscode".to_string(),
                ".idea".to_string(),
                "$RECYCLE.BIN".to_string(),
                "System Volume Information".to_string(),
            ],
        }
    }

    /// Indexes a directory recursively, updating SQLite indexed_files table.
    pub fn scan_directory(&self, root_path: &Path) -> Result<usize, FileOpError> {
        if !root_path.exists() {
            return Err(FileOpError::SourceNotFound(format!(
                "Root directory not found: {:?}",
                root_path
            )));
        }

        let mut count = 0;
        self.walk_and_index(root_path, &mut count)?;
        info!("Indexed {} files and directories under {:?}", count, root_path);
        Ok(count)
    }

    fn walk_and_index(&self, dir: &Path, count: &mut usize) -> Result<(), FileOpError> {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(err) => {
                warn!("Cannot read directory {:?}: {}", dir, err);
                return Ok(());
            }
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

            // Check if ignored
            if self.ignored_directories.iter().any(|ig| file_name.eq_ignore_ascii_case(ig)) {
                debug!("Skipping ignored folder: {:?}", path);
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(err) => {
                    warn!("Cannot read metadata for {:?}: {}", path, err);
                    continue;
                }
            };

            let is_dir = metadata.is_dir();
            let size_bytes = if is_dir { 0 } else { metadata.len() as i64 };
            let modified_at = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let created_at = metadata
                .created()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(modified_at);

            let extension = path.extension().and_then(|ext| ext.to_str()).map(|e| e.to_lowercase());
            let parent_dir = path
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            // Calculate content hash for files below max size
            let content_hash = if !is_dir && metadata.len() <= self.max_file_size_for_hashing {
                self.compute_sha256(&path).ok()
            } else {
                None
            };

            let mime_type = Self::guess_mime_type(extension.as_deref());
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

            let record = IndexedFileRecord {
                path: path.to_string_lossy().to_string(),
                parent_dir,
                file_name,
                extension,
                size_bytes,
                modified_at,
                created_at,
                is_dir,
                content_hash,
                mime_type,
                indexed_at: now,
            };

            self.db.upsert_indexed_file(&record)?;
            *count += 1;

            if is_dir {
                self.walk_and_index(&path, count)?;
            }
        }

        Ok(())
    }

    fn compute_sha256(&self, path: &Path) -> Result<String, FileOpError> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn guess_mime_type(ext: Option<&str>) -> Option<String> {
        match ext {
            Some("pdf") => Some("application/pdf".to_string()),
            Some("docx") => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()),
            Some("xlsx") => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
            Some("pptx") => Some("application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string()),
            Some("txt") | Some("md") => Some("text/plain".to_string()),
            Some("rs") => Some("text/rust".to_string()),
            Some("py") => Some("text/python".to_string()),
            Some("js") | Some("ts") | Some("tsx") => Some("text/javascript".to_string()),
            Some("png") => Some("image/png".to_string()),
            Some("jpg") | Some("jpeg") => Some("image/jpeg".to_string()),
            Some("zip") => Some("application/zip".to_string()),
            _ => None,
        }
    }
}
