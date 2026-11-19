pub mod analyzer;
pub mod backup;
pub mod download;
pub mod duplicate;
pub mod indexer;
pub mod operations;
pub mod organizer;

pub use analyzer::{AnalyzerService, CleanableSuggestion, DetailedStorageReport};
pub use backup::BackupService;
pub use download::{DownloadEventReport, DownloadIntelligence, DownloadStatus};
pub use duplicate::{DuplicateDetector, VersionCandidate};
pub use indexer::IndexerService;
pub use operations::{FileOpError, OperationResult, SafeFileOperationsEngine};
pub use organizer::{OrganizationProposal, SmartOrganizer};

use hime_storage::Database;
use std::path::Path;
use std::sync::Arc;

pub struct StorageEngine {
    pub db: Arc<Database>,
    pub ops_engine: Arc<SafeFileOperationsEngine>,
    pub indexer: Arc<IndexerService>,
    pub analyzer: Arc<AnalyzerService>,
    pub duplicate_detector: Arc<DuplicateDetector>,
    pub organizer: Arc<SmartOrganizer>,
    pub download_intel: Arc<DownloadIntelligence>,
    pub backup: Arc<BackupService>,
}

impl StorageEngine {
    pub fn new(db: Arc<Database>, base_hime_dir: &Path) -> Result<Self, FileOpError> {
        let ops_engine = Arc::new(SafeFileOperationsEngine::new(db.clone(), base_hime_dir)?);
        let indexer = Arc::new(IndexerService::new(db.clone()));
        let analyzer = Arc::new(AnalyzerService::new(db.clone()));
        let duplicate_detector = Arc::new(DuplicateDetector::new(db.clone()));
        let organizer = Arc::new(SmartOrganizer::new(db.clone(), ops_engine.clone()));
        let download_intel = Arc::new(DownloadIntelligence::new(db.clone(), ops_engine.clone()));
        let backup = Arc::new(BackupService::new(db.clone()));

        Ok(Self {
            db,
            ops_engine,
            indexer,
            analyzer,
            duplicate_detector,
            organizer,
            download_intel,
            backup,
        })
    }
}
