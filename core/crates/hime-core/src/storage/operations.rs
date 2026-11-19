use hime_security::policy::PolicyEngine;
use hime_storage::{Database, FileOperationRecord};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{info, warn};

#[derive(Error, Debug)]
pub enum FileOpError {
    #[error("Security policy violation: {0}")]
    PolicyViolation(String),

    #[error("Path traversal or invalid path: {0}")]
    InvalidPath(String),

    #[error("Source file does not exist: {0}")]
    SourceNotFound(String),

    #[error("Destination already exists: {0}")]
    DestinationExists(String),

    #[error("Hash mismatch: expected {expected}, calculated {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Storage error: {0}")]
    Storage(#[from] hime_storage::StorageError),

    #[error("Operation failed: {0}")]
    General(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperationResult {
    pub operation_id: String,
    pub status: String,
    pub source_path: String,
    pub destination_path: Option<String>,
    pub backup_path: Option<String>,
    pub source_hash: Option<String>,
    pub dest_hash: Option<String>,
}

pub struct SafeFileOperationsEngine {
    db: Arc<Database>,
    policy_engine: PolicyEngine,
    recycle_root: PathBuf,
    staging_root: PathBuf,
}

impl SafeFileOperationsEngine {
    pub fn new(db: Arc<Database>, base_hime_dir: &Path) -> Result<Self, FileOpError> {
        let recycle_root = base_hime_dir.join("recycle");
        let staging_root = base_hime_dir.join("staging");

        fs::create_dir_all(&recycle_root)?;
        fs::create_dir_all(&staging_root)?;

        let engine = Self {
            db,
            policy_engine: PolicyEngine::new(),
            recycle_root,
            staging_root,
        };

        // Run crash recovery on startup to clean or roll back half-finished operations
        engine.recover_uncommitted_operations()?;

        Ok(engine)
    }

    pub fn recycle_dir(&self) -> &Path {
        &self.recycle_root
    }

    pub fn staging_dir(&self) -> &Path {
        &self.staging_root
    }

    /// Validates path against security policies (traversal, ADS, reserved names, sensitive paths).
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf, FileOpError> {
        let path_str = path.to_string_lossy();
        self.policy_engine
            .validate_path_safety(&path_str)
            .map_err(FileOpError::PolicyViolation)?;

        if path_str.contains("..") {
            return Err(FileOpError::InvalidPath(format!("Path contains traversal '..': {}", path_str)));
        }

        // NTFS alternate data stream check (colon in file name part)
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            if file_name.contains(':') {
                return Err(FileOpError::InvalidPath(format!("NTFS Alternate Data Stream detected: {}", path_str)));
            }
        }

        // Windows reserved device names
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_uppercase();
        let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
        if reserved.contains(&stem.as_str()) {
            return Err(FileOpError::InvalidPath(format!("Windows reserved device name: {}", stem)));
        }

        let canonical = if path.exists() {
            fs::canonicalize(path)?
        } else {
            // For not-yet-created destinations, canonicalize parent
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    let parent_canon = fs::canonicalize(parent)?;
                    parent_canon.join(path.file_name().unwrap_or_default())
                } else {
                    path.to_path_buf()
                }
            } else {
                path.to_path_buf()
            }
        };

        Ok(canonical)
    }

    /// Computes SHA-256 hash of a file.
    pub fn compute_sha256(path: &Path) -> Result<String, FileOpError> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536]; // 64KB chunks
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Atomic safe move:
    /// Staging -> verify staged hash -> atomic rename -> source moved to Hime Recycle -> journal committed.
    pub fn safe_move(
        &self,
        op_id: &str,
        source: &Path,
        destination: &Path,
        actor: &str,
        approval_id: Option<&str>,
    ) -> Result<OperationResult, FileOpError> {
        let src_canon = self.validate_path(source)?;
        if !src_canon.is_file() {
            return Err(FileOpError::SourceNotFound(format!("File does not exist: {:?}", src_canon)));
        }

        let dst_canon = self.validate_path(destination)?;
        if dst_canon.exists() {
            return Err(FileOpError::DestinationExists(format!("Destination already exists: {:?}", dst_canon)));
        }

        if let Some(parent) = dst_canon.parent() {
            fs::create_dir_all(parent)?;
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let src_hash = Self::compute_sha256(&src_canon)?;

        // Prepare unique staging path
        let file_name = src_canon.file_name().unwrap_or_default().to_string_lossy();
        let staging_file = self.staging_root.join(format!("{}_{}.staging", op_id, file_name));

        // Prepare backup / recycle path
        let recycle_file = self.recycle_root.join(format!("{}_{}", now, file_name));

        // 1. Record journal entry in staged status
        let op_record = FileOperationRecord {
            id: op_id.to_string(),
            op_type: "move".to_string(),
            source_path: src_canon.to_string_lossy().to_string(),
            destination_path: Some(dst_canon.to_string_lossy().to_string()),
            backup_path: Some(recycle_file.to_string_lossy().to_string()),
            source_hash: Some(src_hash.clone()),
            dest_hash: None,
            actor: actor.to_string(),
            approval_id: approval_id.map(ToString::to_string),
            status: "staged".to_string(),
            started_at: now,
            completed_at: None,
            error_message: None,
        };
        self.db.record_file_operation(&op_record)?;

        // 2. Copy source to staging
        fs::copy(&src_canon, &staging_file)?;
        {
            let f = OpenOptions::new().write(true).open(&staging_file)?;
            f.sync_all()?;
        }

        // 3. Verify staging hash
        let staged_hash = Self::compute_sha256(&staging_file)?;
        if staged_hash != src_hash {
            let _ = fs::remove_file(&staging_file);
            self.db.update_file_operation_status(
                op_id,
                "failed",
                None,
                true,
                Some("Staged file hash mismatch"),
            )?;
            return Err(FileOpError::HashMismatch {
                expected: src_hash,
                actual: staged_hash,
            });
        }

        // 4. Atomic move staging to destination
        fs::rename(&staging_file, &dst_canon)?;

        // 5. Verify destination hash
        let dest_hash = Self::compute_sha256(&dst_canon)?;
        if dest_hash != src_hash {
            let _ = fs::remove_file(&dst_canon);
            self.db.update_file_operation_status(
                op_id,
                "failed",
                None,
                true,
                Some("Destination file hash mismatch"),
            )?;
            return Err(FileOpError::HashMismatch {
                expected: src_hash,
                actual: dest_hash,
            });
        }

        // 6. Move source to Hime Recycle
        fs::rename(&src_canon, &recycle_file)?;

        // 7. Commit operation journal
        self.db.update_file_operation_status(
            op_id,
            "committed",
            Some(&dest_hash),
            true,
            None,
        )?;

        info!(
            "Safe move completed: {:?} -> {:?} (recycled to {:?})",
            src_canon, dst_canon, recycle_file
        );

        Ok(OperationResult {
            operation_id: op_id.to_string(),
            status: "committed".to_string(),
            source_path: src_canon.to_string_lossy().to_string(),
            destination_path: Some(dst_canon.to_string_lossy().to_string()),
            backup_path: Some(recycle_file.to_string_lossy().to_string()),
            source_hash: Some(src_hash),
            dest_hash: Some(dest_hash),
        })
    }

    /// Safe delete with approval:
    /// Source file is moved to Hime Recycle (.hime/recycle) instead of being permanently destroyed.
    pub fn safe_delete(
        &self,
        op_id: &str,
        source: &Path,
        actor: &str,
        approval_id: Option<&str>,
    ) -> Result<OperationResult, FileOpError> {
        let src_canon = self.validate_path(source)?;
        if !src_canon.is_file() {
            return Err(FileOpError::SourceNotFound(format!("File does not exist: {:?}", src_canon)));
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let src_hash = Self::compute_sha256(&src_canon)?;
        let file_name = src_canon.file_name().unwrap_or_default().to_string_lossy();
        let recycle_file = self.recycle_root.join(format!("{}_{}", now, file_name));

        let op_record = FileOperationRecord {
            id: op_id.to_string(),
            op_type: "delete".to_string(),
            source_path: src_canon.to_string_lossy().to_string(),
            destination_path: None,
            backup_path: Some(recycle_file.to_string_lossy().to_string()),
            source_hash: Some(src_hash.clone()),
            dest_hash: None,
            actor: actor.to_string(),
            approval_id: approval_id.map(ToString::to_string),
            status: "staged".to_string(),
            started_at: now,
            completed_at: None,
            error_message: None,
        };
        self.db.record_file_operation(&op_record)?;

        // Move to Recycle
        fs::rename(&src_canon, &recycle_file)?;

        self.db.update_file_operation_status(
            op_id,
            "committed",
            None,
            true,
            None,
        )?;

        info!("Safe delete completed: {:?} moved to Hime Recycle {:?}", src_canon, recycle_file);

        Ok(OperationResult {
            operation_id: op_id.to_string(),
            status: "committed".to_string(),
            source_path: src_canon.to_string_lossy().to_string(),
            destination_path: None,
            backup_path: Some(recycle_file.to_string_lossy().to_string()),
            source_hash: Some(src_hash),
            dest_hash: None,
        })
    }

    /// One-click restore from Hime Recycle back to original path.
    pub fn restore_file(
        &self,
        op_id: &str,
        original_op_id: &str,
        actor: &str,
    ) -> Result<OperationResult, FileOpError> {
        let original_op = self
            .db
            .get_file_operation(original_op_id)?
            .ok_or_else(|| FileOpError::General(format!("Original operation {} not found", original_op_id)))?;

        let recycle_path_str = original_op
            .backup_path
            .ok_or_else(|| FileOpError::General("No recycle/backup path found in operation record".to_string()))?;
        let recycle_path = PathBuf::from(&recycle_path_str);

        if !recycle_path.is_file() {
            return Err(FileOpError::SourceNotFound(format!(
                "Recycle file does not exist: {:?}",
                recycle_path
            )));
        }

        let target_path = PathBuf::from(&original_op.source_path);
        if target_path.exists() {
            return Err(FileOpError::DestinationExists(format!(
                "Target file already exists at original path: {:?}",
                target_path
            )));
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let recycle_hash = Self::compute_sha256(&recycle_path)?;

        let restore_record = FileOperationRecord {
            id: op_id.to_string(),
            op_type: "restore".to_string(),
            source_path: recycle_path.to_string_lossy().to_string(),
            destination_path: Some(target_path.to_string_lossy().to_string()),
            backup_path: None,
            source_hash: Some(recycle_hash.clone()),
            dest_hash: None,
            actor: actor.to_string(),
            approval_id: None,
            status: "staged".to_string(),
            started_at: now,
            completed_at: None,
            error_message: None,
        };
        self.db.record_file_operation(&restore_record)?;

        // Move back from recycle
        fs::rename(&recycle_path, &target_path)?;

        let dest_hash = Self::compute_sha256(&target_path)?;
        self.db.update_file_operation_status(
            op_id,
            "committed",
            Some(&dest_hash),
            true,
            None,
        )?;

        info!("Restore completed: {:?} -> {:?}", recycle_path, target_path);

        Ok(OperationResult {
            operation_id: op_id.to_string(),
            status: "committed".to_string(),
            source_path: recycle_path.to_string_lossy().to_string(),
            destination_path: Some(target_path.to_string_lossy().to_string()),
            backup_path: None,
            source_hash: Some(recycle_hash),
            dest_hash: Some(dest_hash),
        })
    }

    /// Startup crash recovery: rolls back or cleans any uncommitted operations left in 'staged' or 'in_progress'.
    pub fn recover_uncommitted_operations(&self) -> Result<usize, FileOpError> {
        let uncommitted = self.db.get_uncommitted_file_operations()?;
        let mut recovered_count = 0;

        for op in uncommitted {
            warn!("Recovering uncommitted file operation: id={}, type={}, status={}", op.id, op.op_type, op.status);

            // Clean any matching staging files
            if let Ok(entries) = fs::read_dir(&self.staging_root) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                        if file_name.starts_with(&op.id) {
                            let _ = fs::remove_file(&path);
                            info!("Cleaned dangling staging file: {:?}", path);
                        }
                    }
                }
            }

            self.db.update_file_operation_status(
                &op.id,
                "rolled_back",
                None,
                true,
                Some("Recovered on startup crash recovery"),
            )?;
            recovered_count += 1;
        }

        if recovered_count > 0 {
            info!("Crash recovery completed: {} uncommitted operations resolved", recovered_count);
        }

        Ok(recovered_count)
    }
}
