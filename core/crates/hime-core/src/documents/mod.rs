pub mod artcraft;
pub mod operations;
pub mod service;
pub mod templates;

pub use artcraft::{ArtcraftAdapter, ArtcraftDiscoveryResult};
pub use operations::{DocumentOpResult, DocumentOperations, DocumentVerificationReport};
pub use service::{DocumentError, DocumentService};
pub use templates::{DocumentTemplateSpec, TemplateRegistry};

use hime_storage::Database;
use std::path::Path;
use std::sync::Arc;

pub struct DocumentEngine {
    pub db: Arc<Database>,
    pub service: Arc<DocumentService>,
}

impl DocumentEngine {
    pub fn new(db: Arc<Database>, base_dir: &Path) -> Result<Self, DocumentError> {
        let service = Arc::new(DocumentService::new(db.clone(), base_dir)?);
        Ok(Self { db, service })
    }
}

