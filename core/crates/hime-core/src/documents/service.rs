use crate::documents::operations::{DocumentOpResult, DocumentOperations};
use crate::documents::templates::TemplateRegistry;
use hime_storage::{Database, DocumentJobRecord, StorageError};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum DocumentError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Worker busy: another document processing job is currently running")]
    WorkerBusy,
}

pub struct DocumentService {
    db: Arc<Database>,
    sandbox_dir: PathBuf,
    worker_lock: Mutex<()>,
}

impl DocumentService {
    pub fn new(db: Arc<Database>, base_dir: &Path) -> Result<Self, DocumentError> {
        let sandbox_dir = base_dir.join(".hime").join("documents_sandbox");
        fs::create_dir_all(&sandbox_dir)?;

        Ok(Self {
            db,
            sandbox_dir,
            worker_lock: Mutex::new(()),
        })
    }

    pub fn sandbox_dir(&self) -> &Path {
        &self.sandbox_dir
    }

    /// Creates an academic or business report document inside the sandbox and commits to target path.
    pub async fn create_report_job(
        &self,
        job_id: &str,
        title: &str,
        markdown_body: &str,
        template_name: &str,
        destination_path: &Path,
    ) -> Result<DocumentOpResult, DocumentError> {
        let _guard = self.worker_lock.lock().await;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let job = DocumentJobRecord {
            id: job_id.to_string(),
            operation: "create".to_string(),
            source_path: None,
            target_path: Some(destination_path.to_string_lossy().to_string()),
            status: "processing".to_string(),
            format: "docx".to_string(),
            metadata: Some(format!("{{\"template\":\"{}\",\"title\":\"{}\"}}", template_name, title)),
            created_at: now,
            completed_at: None,
        };
        self.db.create_document_job(&job)?;

        let template = if template_name.to_lowercase().contains("modern") {
            TemplateRegistry::modern_report()
        } else {
            TemplateRegistry::academic_id()
        };

        // Write directly to sandbox staging first
        let staging_file = self.sandbox_dir.join(format!("{}.html", job_id));
        let res = DocumentOperations::create_report(&staging_file, title, markdown_body, Some(&template))?;

        // Commit to target destination
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&staging_file, destination_path)?;
        let _ = fs::remove_file(&staging_file);

        let completed_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut completed_job = job;
        completed_job.status = "completed".to_string();
        completed_job.completed_at = Some(completed_now);
        self.db.create_document_job(&completed_job)?;

        Ok(res)
    }

    /// Merges multiple PDFs into a single output document.
    pub async fn merge_pdfs_job(
        &self,
        job_id: &str,
        source_paths: &[PathBuf],
        destination_path: &Path,
    ) -> Result<DocumentOpResult, DocumentError> {
        let _guard = self.worker_lock.lock().await;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let job = DocumentJobRecord {
            id: job_id.to_string(),
            operation: "merge".to_string(),
            source_path: Some(format!("{:?}", source_paths)),
            target_path: Some(destination_path.to_string_lossy().to_string()),
            status: "processing".to_string(),
            format: "pdf".to_string(),
            metadata: None,
            created_at: now,
            completed_at: None,
        };
        self.db.create_document_job(&job)?;

        let staging_file = self.sandbox_dir.join(format!("{}.pdf", job_id));
        let res = DocumentOperations::merge_documents(source_paths, &staging_file)?;

        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&staging_file, destination_path)?;
        let _ = fs::remove_file(&staging_file);

        let completed_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut completed_job = job;
        completed_job.status = "completed".to_string();
        completed_job.completed_at = Some(completed_now);
        self.db.create_document_job(&completed_job)?;

        Ok(res)
    }

    /// Compresses a document file.
    pub async fn compress_job(
        &self,
        job_id: &str,
        source_path: &Path,
        destination_path: &Path,
    ) -> Result<DocumentOpResult, DocumentError> {
        let _guard = self.worker_lock.lock().await;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let job = DocumentJobRecord {
            id: job_id.to_string(),
            operation: "compress".to_string(),
            source_path: Some(source_path.to_string_lossy().to_string()),
            target_path: Some(destination_path.to_string_lossy().to_string()),
            status: "processing".to_string(),
            format: source_path.extension().and_then(|e| e.to_str()).unwrap_or("bin").to_string(),
            metadata: None,
            created_at: now,
            completed_at: None,
        };
        self.db.create_document_job(&job)?;

        let res = DocumentOperations::compress_document(source_path, destination_path)?;

        let completed_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut completed_job = job;
        completed_job.status = "completed".to_string();
        completed_job.completed_at = Some(completed_now);
        self.db.create_document_job(&completed_job)?;

        Ok(res)
    }
}

