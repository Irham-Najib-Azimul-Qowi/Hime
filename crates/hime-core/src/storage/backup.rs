use crate::storage::operations::{FileOpError, SafeFileOperationsEngine};
use hime_storage::{Database, StorageBackupRecord};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

pub struct BackupService {
    db: Arc<Database>,
}

impl BackupService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Creates a local snapshot backup of source_root into backup_dest.
    pub fn create_backup(
        &self,
        backup_id: &str,
        source_root: &Path,
        backup_dest: &Path,
    ) -> Result<StorageBackupRecord, FileOpError> {
        if !source_root.exists() || !source_root.is_dir() {
            return Err(FileOpError::SourceNotFound(format!(
                "Source root is not a valid directory: {:?}",
                source_root
            )));
        }

        fs::create_dir_all(backup_dest)?;

        let mut file_count = 0i64;
        let mut total_bytes = 0i64;
        let mut manifest_hasher = Sha256::new();

        self.copy_and_hash_recursive(
            source_root,
            source_root,
            backup_dest,
            &mut file_count,
            &mut total_bytes,
            &mut manifest_hasher,
        )?;

        let manifest_hash = format!("{:x}", manifest_hasher.finalize());
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

        let record = StorageBackupRecord {
            id: backup_id.to_string(),
            source_root: source_root.to_string_lossy().to_string(),
            backup_destination: backup_dest.to_string_lossy().to_string(),
            file_count,
            total_bytes,
            manifest_hash: manifest_hash.clone(),
            created_at: now,
            verified_at: Some(now),
        };

        self.db.create_backup_record(&record)?;
        info!("Backup {} created successfully: {} files, {} bytes", backup_id, file_count, total_bytes);

        Ok(record)
    }

    /// Verifies that backup files match the manifest.
    pub fn verify_backup(&self, backup_id: &str) -> Result<bool, FileOpError> {
        let backups = self.db.list_backup_records()?;
        let backup = backups
            .into_iter()
            .find(|b| b.id == backup_id)
            .ok_or_else(|| FileOpError::General(format!("Backup record {} not found", backup_id)))?;

        let backup_dest = Path::new(&backup.backup_destination);
        if !backup_dest.exists() {
            return Ok(false);
        }

        let mut file_count = 0i64;
        let mut total_bytes = 0i64;
        let mut manifest_hasher = Sha256::new();

        self.hash_recursive(
            backup_dest,
            backup_dest,
            &mut file_count,
            &mut total_bytes,
            &mut manifest_hasher,
        )?;

        let current_hash = format!("{:x}", manifest_hasher.finalize());
        let matches = current_hash == backup.manifest_hash;

        if matches {
            self.db.verify_backup_record(backup_id)?;
            info!("Backup {} verified successfully", backup_id);
        } else {
            info!("Backup {} verification failed: hash mismatch", backup_id);
        }

        Ok(matches)
    }

    fn copy_and_hash_recursive(
        &self,
        base_src: &Path,
        current_dir: &Path,
        base_dest: &Path,
        file_count: &mut i64,
        total_bytes: &mut i64,
        manifest_hasher: &mut Sha256,
    ) -> Result<(), FileOpError> {
        for entry in fs::read_dir(current_dir)?.filter_map(Result::ok) {
            let path = entry.path();
            let rel_path = path.strip_prefix(base_src).map_err(|e| FileOpError::General(e.to_string()))?;
            let dest_path = base_dest.join(rel_path);

            if path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                self.copy_and_hash_recursive(base_src, &path, base_dest, file_count, total_bytes, manifest_hasher)?;
            } else if path.is_file() {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&path, &dest_path)?;

                let f_hash = SafeFileOperationsEngine::compute_sha256(&dest_path)?;
                manifest_hasher.update(rel_path.to_string_lossy().as_bytes());
                manifest_hasher.update(f_hash.as_bytes());

                let len = entry.metadata()?.len() as i64;
                *file_count += 1;
                *total_bytes += len;
            }
        }
        Ok(())
    }

    fn hash_recursive(
        &self,
        base_dir: &Path,
        current_dir: &Path,
        file_count: &mut i64,
        total_bytes: &mut i64,
        manifest_hasher: &mut Sha256,
    ) -> Result<(), FileOpError> {
        for entry in fs::read_dir(current_dir)?.filter_map(Result::ok) {
            let path = entry.path();
            let rel_path = path.strip_prefix(base_dir).map_err(|e| FileOpError::General(e.to_string()))?;

            if path.is_dir() {
                self.hash_recursive(base_dir, &path, file_count, total_bytes, manifest_hasher)?;
            } else if path.is_file() {
                let f_hash = SafeFileOperationsEngine::compute_sha256(&path)?;
                manifest_hasher.update(rel_path.to_string_lossy().as_bytes());
                manifest_hasher.update(f_hash.as_bytes());

                let len = entry.metadata()?.len() as i64;
                *file_count += 1;
                *total_bytes += len;
            }
        }
        Ok(())
    }
}
