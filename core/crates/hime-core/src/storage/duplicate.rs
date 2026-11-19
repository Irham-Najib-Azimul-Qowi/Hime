use crate::storage::operations::FileOpError;
use hime_storage::{Database, DuplicateGroup, IndexedFileRecord};
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionCandidate {
    pub base_file: IndexedFileRecord,
    pub candidate_file: IndexedFileRecord,
    pub pattern: String,
    pub recommendation: String,
}

pub struct DuplicateDetector {
    db: Arc<Database>,
}

impl DuplicateDetector {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Finds exact duplicate groups by content hash.
    pub fn find_exact_duplicates(&self) -> Result<Vec<DuplicateGroup>, FileOpError> {
        let duplicates = self.db.find_duplicate_candidates()?;
        Ok(duplicates)
    }

    /// Finds version candidates: files with similar naming conventions but different hashes.
    pub fn find_version_candidates(&self) -> Result<Vec<VersionCandidate>, FileOpError> {
        let mut candidates = Vec::new();

        // Sample recent or indexed files to inspect
        let files = self.db.search_indexed_files("", 500)?;
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                let f1 = &files[i];
                let f2 = &files[j];

                if f1.is_dir || f2.is_dir {
                    continue;
                }

                if f1.extension != f2.extension {
                    continue;
                }

                // If identical hash, it's an exact duplicate, not a version candidate
                if f1.content_hash.is_some() && f1.content_hash == f2.content_hash {
                    continue;
                }

                if let Some(pattern) = Self::detect_version_pattern(&f1.file_name, &f2.file_name) {
                    let (base, newer) = if f1.modified_at >= f2.modified_at {
                        (f2.clone(), f1.clone())
                    } else {
                        (f1.clone(), f2.clone())
                    };

                    candidates.push(VersionCandidate {
                        base_file: base,
                        candidate_file: newer,
                        pattern,
                        recommendation: "Keep newer version, archive older version to Hime Recycle or archive folder".to_string(),
                    });
                }
            }
        }

        Ok(candidates)
    }

    fn detect_version_pattern(name1: &str, name2: &str) -> Option<String> {
        let stem1 = Path::new(name1).file_stem()?.to_str()?.to_lowercase();
        let stem2 = Path::new(name2).file_stem()?.to_str()?.to_lowercase();

        // Pattern 1: name (1) vs name
        if stem1.starts_with(&stem2) && (stem1.contains("(1)") || stem1.contains("(2)") || stem1.contains("_copy") || stem1.contains("-copy")) {
            return Some("copy_suffix".to_string());
        }
        if stem2.starts_with(&stem1) && (stem2.contains("(1)") || stem2.contains("(2)") || stem2.contains("_copy") || stem2.contains("-copy")) {
            return Some("copy_suffix".to_string());
        }

        // Pattern 2: _v1, _v2, _final, _draft
        if stem1.starts_with(&stem2) && (stem1.contains("_v") || stem1.contains("-v") || stem1.contains("_final") || stem1.contains("_rev")) {
            return Some("version_tag".to_string());
        }
        if stem2.starts_with(&stem1) && (stem2.contains("_v") || stem2.contains("-v") || stem2.contains("_final") || stem2.contains("_rev")) {
            return Some("version_tag".to_string());
        }

        None
    }
}

use std::path::Path;

