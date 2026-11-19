use crate::storage::operations::{FileOpError, SafeFileOperationsEngine};
use hime_storage::{Database, FileTransferRecord};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum TransferError {
    #[error("Transfer not found: {0}")]
    NotFound(String),

    #[error("Chunk index mismatch: expected {expected}, got {actual}")]
    ChunkMismatch { expected: u32, actual: u32 },

    #[error("Chunk SHA-256 mismatch")]
    ChunkChecksumFailed,

    #[error("Overall file SHA-256 mismatch: expected {expected}, got {actual}")]
    FileChecksumFailed { expected: String, actual: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Storage error: {0}")]
    Storage(#[from] hime_storage::StorageError),

    #[error("File operation error: {0}")]
    FileOp(#[from] FileOpError),
}

pub struct TransferEngine {
    db: Arc<Database>,
    ops_engine: Arc<SafeFileOperationsEngine>,
    transfer_staging_dir: PathBuf,
}

impl TransferEngine {
    pub fn new(
        db: Arc<Database>,
        ops_engine: Arc<SafeFileOperationsEngine>,
        base_hime_dir: &Path,
    ) -> Result<Self, TransferError> {
        let transfer_staging_dir = base_hime_dir.join("transfers");
        fs::create_dir_all(&transfer_staging_dir)?;
        Ok(Self {
            db,
            ops_engine,
            transfer_staging_dir,
        })
    }

    /// Initializes a new incoming transfer from mobile device to desktop.
    pub fn init_incoming_transfer(
        &self,
        transfer_id: &str,
        device_id: &str,
        file_name: &str,
        file_size: i64,
        expected_sha256: &str,
    ) -> Result<FileTransferRecord, TransferError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let rec = FileTransferRecord {
            transfer_id: transfer_id.to_string(),
            device_id: device_id.to_string(),
            file_name: file_name.to_string(),
            file_size,
            bytes_transferred: 0,
            direction: "upload".to_string(),
            status: "in_progress".to_string(),
            sha256: expected_sha256.to_string(),
            started_at: now,
            completed_at: None,
        };

        // Create staging file
        let stage_file = self.transfer_staging_dir.join(format!("{}.part", transfer_id));
        let _ = File::create(&stage_file)?;

        self.db.record_file_transfer(&rec)?;
        info!("Incoming file transfer initialized: {} ({})", file_name, transfer_id);
        Ok(rec)
    }

    /// Appends a verified chunk of data to the staging file.
    pub fn write_chunk(
        &self,
        transfer_id: &str,
        chunk_data: &[u8],
        chunk_sha256: &str,
    ) -> Result<i64, TransferError> {
        // 1. Verify chunk hash
        let mut hasher = Sha256::new();
        hasher.update(chunk_data);
        let actual_hash = format!("{:x}", hasher.finalize());
        if actual_hash != chunk_sha256 {
            return Err(TransferError::ChunkChecksumFailed);
        }

        // 2. Append to part file
        let stage_file = self.transfer_staging_dir.join(format!("{}.part", transfer_id));
        let mut f = OpenOptions::new().append(true).open(&stage_file)?;
        f.write_all(chunk_data)?;
        f.sync_all()?;

        let total_bytes = f.metadata()?.len() as i64;
        self.db.update_transfer_progress(transfer_id, total_bytes, "in_progress")?;
        Ok(total_bytes)
    }

    /// Finalizes the transfer, verifies whole-file checksum, and moves file into final destination.
    pub fn finalize_transfer(
        &self,
        transfer_id: &str,
        destination_path: &Path,
    ) -> Result<crate::storage::operations::OperationResult, TransferError> {
        let transfers = self.db.list_file_transfers(None)?;
        let rec = transfers
            .into_iter()
            .find(|t| t.transfer_id == transfer_id)
            .ok_or_else(|| TransferError::NotFound(transfer_id.to_string()))?;

        let stage_file = self.transfer_staging_dir.join(format!("{}.part", transfer_id));
        let actual_sha = SafeFileOperationsEngine::compute_sha256(&stage_file)
            .map_err(|e| TransferError::FileChecksumFailed { expected: rec.sha256.clone(), actual: e.to_string() })?;

        if actual_sha != rec.sha256 {
            self.db.update_transfer_progress(transfer_id, rec.bytes_transferred, "failed")?;
            return Err(TransferError::FileChecksumFailed {
                expected: rec.sha256,
                actual: actual_sha,
            });
        }

        // Move verified file from transfer staging into destination via safe move
        let op_id = format!("xfer-commit-{}", transfer_id);
        let res = self.ops_engine.safe_move(
            &op_id,
            &stage_file,
            destination_path,
            &format!("device:{}", rec.device_id),
            None,
        )?;

        self.db.update_transfer_progress(transfer_id, rec.file_size, "completed")?;
        info!("Transfer {} completed and committed to {:?}", transfer_id, destination_path);

        Ok(res)
    }
}
