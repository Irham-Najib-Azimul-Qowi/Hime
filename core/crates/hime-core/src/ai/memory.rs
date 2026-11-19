use std::sync::Arc;
use tracing::info;

use hime_storage::{Database, MemoryRecord};

/// Context & Memory Engine for Hime Core.
///
/// Implements provenance-tracked memory storage, token budget-bounded prompt context retrieval,
/// and granular user deletion controls for privacy compliance.
pub struct MemoryEngine {
    storage: Arc<Database>,
}

impl MemoryEngine {
    pub fn new(storage: Arc<Database>) -> Self {
        Self { storage }
    }

    pub fn insert_memory(
        &self,
        id: &str,
        source: &str,
        category: &str,
        content: &str,
        relevance_score: f64,
        retention_policy: &str,
    ) -> Result<MemoryRecord, String> {
        info!(id = %id, category = %category, source = %source, "Inserting memory item with provenance");
        self.storage
            .insert_memory_item(id, source, category, content, relevance_score, retention_policy)
            .map_err(|e| format!("Storage error inserting memory: {}", e))
    }

    pub fn get_memory(&self, id: &str) -> Result<Option<MemoryRecord>, String> {
        self.storage
            .get_memory_item(id)
            .map_err(|e| format!("Storage error fetching memory: {}", e))
    }

    pub fn list_memories(&self, category_filter: Option<&str>) -> Result<Vec<MemoryRecord>, String> {
        self.storage
            .list_memory_items(category_filter)
            .map_err(|e| format!("Storage error listing memories: {}", e))
    }

    pub fn delete_memory(&self, id: &str) -> Result<bool, String> {
        info!(id = %id, "User requested memory deletion");
        self.storage
            .delete_memory_item(id)
            .map_err(|e| format!("Storage error deleting memory: {}", e))
    }

    pub fn search_memories(&self, query: &str, limit: usize) -> Result<Vec<MemoryRecord>, String> {
        self.storage
            .search_memory_items(query, limit)
            .map_err(|e| format!("Storage search error: {}", e))
    }

    /// Builds a prompt context from relevant memories within a strict token budget.
    ///
    /// Evaluates approximate token usage (1 token ≈ 4 characters) and excludes items
    /// exceeding the budget to avoid unbounded context expansion.
    pub fn build_prompt_context(&self, query: &str, max_tokens: usize) -> Vec<MemoryRecord> {
        let candidates = self.search_memories(query, 20).unwrap_or_default();
        let mut selected = Vec::new();
        let mut current_tokens = 0;

        for item in candidates {
            // Approximate token count: chars / 4
            let est_tokens = (item.content.len() / 4).max(1);
            if current_tokens + est_tokens <= max_tokens {
                current_tokens += est_tokens;
                selected.push(item);
            } else {
                break;
            }
        }

        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_engine_token_budget_context() {
        let storage = Arc::new(Database::open_in_memory().unwrap());
        let engine = MemoryEngine::new(storage);

        // Insert items
        engine
            .insert_memory(
                "mem-1",
                "chat",
                "preference",
                "User likes dark mode UI and concise answers",
                1.0,
                "permanent",
            )
            .unwrap();

        engine
            .insert_memory(
                "mem-2",
                "tasks",
                "project",
                "User is developing Hime Personal AI OS on Windows",
                0.9,
                "permanent",
            )
            .unwrap();

        // Query with small token budget (e.g. 15 tokens ~ 60 chars)
        let ctx = engine.build_prompt_context("User", 15);
        assert!(!ctx.is_empty());
        assert!(ctx.len() <= 2);

        // Delete memory item
        assert!(engine.delete_memory("mem-1").unwrap());
        assert!(engine.get_memory("mem-1").unwrap().is_none());
    }
}
