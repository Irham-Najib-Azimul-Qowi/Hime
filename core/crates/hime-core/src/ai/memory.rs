use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::info;

use hime_storage::{Database, MemoryRecord};

/// Cognitive memory layers supported by Hime Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryLayer {
    /// Long-lived facts, user preferences, identity, and project context.
    Semantic,
    /// Time-bound events, tasks, deadlines, and recent turn episodes.
    Episodic,
    /// Actionable rules, habit corrections, workflows, and folder/path conventions.
    Procedural,
}

impl MemoryLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryLayer::Semantic => "semantic",
            MemoryLayer::Episodic => "episodic",
            MemoryLayer::Procedural => "procedural",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "episodic" => MemoryLayer::Episodic,
            "procedural" => MemoryLayer::Procedural,
            _ => MemoryLayer::Semantic,
        }
    }

    /// Half-life in days for time-decay scoring.
    pub fn half_life_days(&self) -> f64 {
        match self {
            MemoryLayer::Semantic => 90.0,
            MemoryLayer::Procedural => 120.0,
            MemoryLayer::Episodic => 14.0,
        }
    }
}

/// Self-Learning Context & Memory Engine for Hime Core.
///
/// Implements 3-layer memory (`semantic`, `episodic`, `procedural`), provenance tracking,
/// access-frequency + time-decay scoring, deduplication, fast FTS5/LIKE search,
/// deterministic turn extraction, and user privacy controls (list/get/delete).
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
        let inferred_layer = infer_layer_from_category(category);
        self.insert_layered_memory(
            id,
            source,
            category,
            inferred_layer,
            content,
            relevance_score,
            retention_policy,
        )
    }

    pub fn insert_layered_memory(
        &self,
        id: &str,
        source: &str,
        category: &str,
        layer: MemoryLayer,
        content: &str,
        relevance_score: f64,
        retention_policy: &str,
    ) -> Result<MemoryRecord, String> {
        info!(
            id = %id,
            category = %category,
            layer = %layer.as_str(),
            source = %source,
            "Inserting memory item with provenance"
        );
        self.storage
            .insert_memory_item_layered(
                id,
                source,
                category,
                layer.as_str(),
                content,
                relevance_score,
                retention_policy,
            )
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

    pub fn list_memories_by_layer(
        &self,
        layer: Option<MemoryLayer>,
        category_filter: Option<&str>,
    ) -> Result<Vec<MemoryRecord>, String> {
        self.storage
            .list_memory_items_by_layer(layer.map(|l| l.as_str()), category_filter)
            .map_err(|e| format!("Storage error listing memories by layer: {}", e))
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

    /// Computes effective memory score combining base relevance, layer-aware time decay,
    /// and logarithmic access-frequency reinforcement.
    pub fn effective_score(record: &MemoryRecord, now_secs: i64) -> f64 {
        let ref_ts = record
            .last_accessed_at
            .max(record.updated_at)
            .max(record.created_at);
        let age_days = ((now_secs - ref_ts).max(0) as f64) / 86400.0;
        let half_life = MemoryLayer::from_str_lossy(&record.memory_layer).half_life_days();
        let decay = 0.5_f64.powf(age_days / half_life);
        let freq_boost = (1.0 + record.access_count.max(0) as f64).ln() * 0.15;
        record.relevance_score * decay + freq_boost
    }

    /// Builds a prompt context from relevant memories within a strict token budget.
    ///
    /// Ranks candidates by `effective_score` (decay + access frequency), selects within
    /// approximate token usage (1 token ≈ 4 characters), and records access for selected items.
    pub fn build_prompt_context(&self, query: &str, max_tokens: usize) -> Vec<MemoryRecord> {
        let mut candidates = self.search_memories(query, 30).unwrap_or_default();
        let now = current_unix_secs();

        candidates.sort_by(|a, b| {
            let sa = Self::effective_score(a, now);
            let sb = Self::effective_score(b, now);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut selected = Vec::new();
        let mut current_tokens = 0;

        for mut item in candidates {
            let est_tokens = (item.content.len() / 4).max(1);
            if current_tokens + est_tokens <= max_tokens {
                current_tokens += est_tokens;
                if self.storage.record_memory_access(&item.id).is_ok() {
                    item.access_count += 1;
                    item.last_accessed_at = now;
                }
                selected.push(item);
            } else if selected.is_empty() {
                // If even the first item exceeds max_tokens, stop without overflowing budget
                break;
            }
        }

        selected
    }

    /// Lightweight deterministic heuristic extractor that learns from a conversation turn.
    ///
    /// Extracts user preferences (`semantic`), tasks/deadlines (`episodic`),
    /// habit corrections (`procedural`), and folder/path rules (`procedural`),
    /// while filtering out greetings, acknowledgements, and small talk.
    pub fn extract_and_learn_from_turn(
        &self,
        user_prompt: &str,
        _assistant_reply: &str,
    ) -> Result<Vec<MemoryRecord>, String> {
        let trimmed = user_prompt.trim();
        if trimmed.len() < 12 || is_small_talk(trimmed) {
            return Ok(Vec::new());
        }

        let mut learned = Vec::new();
        for clause in split_clauses(trimmed) {
            let clause = clause.trim();
            if clause.len() < 12 || is_small_talk(clause) {
                continue;
            }

            if let Some((layer, category, relevance, retention)) = classify_clause(clause) {
                let id = deterministic_memory_id(layer.as_str(), category, clause);
                let rec = self.insert_layered_memory(
                    &id,
                    "auto_turn_extractor",
                    category,
                    layer,
                    clause,
                    relevance,
                    retention,
                )?;
                if !learned.iter().any(|existing: &MemoryRecord| existing.id == rec.id) {
                    learned.push(rec);
                }
            }
        }

        Ok(learned)
    }
}

fn current_unix_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn infer_layer_from_category(category: &str) -> MemoryLayer {
    let lower = category.trim().to_ascii_lowercase();
    if lower.contains("task")
        || lower.contains("deadline")
        || lower.contains("schedule")
        || lower.contains("episod")
        || lower.contains("event")
    {
        MemoryLayer::Episodic
    } else if lower.contains("rule")
        || lower.contains("habit")
        || lower.contains("correction")
        || lower.contains("workflow")
        || lower.contains("procedur")
    {
        MemoryLayer::Procedural
    } else {
        MemoryLayer::Semantic
    }
}

fn split_clauses(text: &str) -> Vec<&str> {
    text.split(|c| c == '\n' || c == '.' || c == ';' || c == '!')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

fn is_small_talk(text: &str) -> bool {
    let lower = text
        .trim()
        .to_ascii_lowercase()
        .trim_matches(|c: char| !c.is_alphanumeric() && !c.is_whitespace())
        .to_string();

    const SMALL_TALK_EXACT: &[&str] = &[
        "hi",
        "hello",
        "hey",
        "halo",
        "hai",
        "pagi",
        "siang",
        "sore",
        "malam",
        "good morning",
        "good afternoon",
        "good evening",
        "good night",
        "selamat pagi",
        "selamat siang",
        "selamat malam",
        "how are you",
        "apa kabar",
        "thanks",
        "thank you",
        "terima kasih",
        "makasih",
        "oke",
        "ok",
        "okay",
        "siap",
        "mantap",
        "nice",
        "cool",
        "great",
        "got it",
        "paham",
        "bye",
        "goodbye",
        "dadah",
    ];

    if SMALL_TALK_EXACT.contains(&lower.as_str()) {
        return true;
    }

    // Pure greeting/thanks without actionable substance
    let words: Vec<&str> = lower.split_whitespace().collect();
    if words.len() <= 3
        && words.iter().all(|w| {
            matches!(
                *w,
                "hi" | "hello"
                    | "halo"
                    | "hey"
                    | "hime"
                    | "thanks"
                    | "thank"
                    | "you"
                    | "makasih"
                    | "terima"
                    | "kasih"
                    | "ok"
                    | "oke"
                    | "siap"
                    | "mantap"
                    | "pagi"
                    | "malam"
            )
        })
    {
        return true;
    }

    false
}

fn classify_clause(clause: &str) -> Option<(MemoryLayer, &'static str, f64, &'static str)> {
    let lower = clause.to_ascii_lowercase();

    // 1. Folder / path rules -> Procedural
    let has_path_shape = lower.contains(":\\")
        || lower.contains(":/")
        || lower.contains("~/")
        || lower.contains("\\downloads")
        || lower.contains("\\documents")
        || lower.contains("/downloads")
        || lower.contains("/documents");
    let has_folder_verb = contains_any(
        &lower,
        &[
            "save to ",
            "save in ",
            "put in ",
            "store in ",
            "simpan di ",
            "simpan ke ",
            "taruh di ",
            "folder ",
            "directory ",
            "direktori ",
        ],
    );
    if has_path_shape || (has_folder_verb && contains_any(&lower, &["file", "pdf", "doc", "project", "output", "download", "backup"])) {
        return Some((MemoryLayer::Procedural, "folder_rule", 0.92, "permanent"));
    }

    // 2. Habit corrections -> Procedural
    if contains_any(
        &lower,
        &[
            "actually,",
            "actually ",
            "correction:",
            "instead of ",
            "next time ",
            "from now on",
            "remember to always",
            "lain kali",
            "mulai sekarang",
            "seharusnya ",
            "bukan begitu",
            "jangan ulangi",
            "stop doing",
        ],
    ) {
        return Some((MemoryLayer::Procedural, "habit_correction", 0.95, "permanent"));
    }

    // 3. Deadlines / tasks -> Episodic
    if contains_any(
        &lower,
        &[
            "deadline",
            "due ",
            "due on",
            "by tomorrow",
            "remind me",
            "ingatkan saya",
            "jangan lupa",
            "todo:",
            "task:",
            "tugas:",
            "besok harus",
            "minggu depan",
            "hari jumat",
            "hari senin",
            "submit by",
            "kumpulkan ",
        ],
    ) {
        return Some((MemoryLayer::Episodic, "task_deadline", 0.90, "auto_learned"));
    }

    // 4. User preferences -> Semantic
    if contains_any(
        &lower,
        &[
            "i prefer ",
            "i like ",
            "i love ",
            "i hate ",
            "i dislike ",
            "always use ",
            "never use ",
            "don't use ",
            "do not use ",
            "prefer concise",
            "saya suka ",
            "saya lebih suka ",
            "aku suka ",
            "aku lebih suka ",
            "jangan pakai ",
            "pakai bahasa ",
            "gunakan bahasa ",
            "my preference",
            "preferensi saya",
        ],
    ) {
        return Some((MemoryLayer::Semantic, "user_preference", 0.90, "permanent"));
    }

    None
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

fn deterministic_memory_id(layer: &str, category: &str, content: &str) -> String {
    let normalized = content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    let mut hasher = Sha256::new();
    hasher.update(layer.as_bytes());
    hasher.update(b":");
    hasher.update(category.as_bytes());
    hasher.update(b":");
    hasher.update(normalized.as_bytes());
    let hex = format!("{:x}", hasher.finalize());
    format!("auto-{}-{}", layer, &hex[..12])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_engine_token_budget_context() {
        let storage = Arc::new(Database::open_in_memory().unwrap());
        let engine = MemoryEngine::new(storage);

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

        // Verify access_count was incremented
        let fetched = engine.get_memory("mem-1").unwrap().unwrap();
        assert!(fetched.access_count >= 1);

        // Delete memory item
        assert!(engine.delete_memory("mem-1").unwrap());
        assert!(engine.get_memory("mem-1").unwrap().is_none());
    }

    #[test]
    fn test_memory_layers_and_effective_score_decay() {
        let storage = Arc::new(Database::open_in_memory().unwrap());
        let engine = MemoryEngine::new(storage);

        let sem = engine
            .insert_layered_memory(
                "sem-1",
                "chat",
                "user_preference",
                MemoryLayer::Semantic,
                "User prefers Rust over Python for backend services",
                0.9,
                "permanent",
            )
            .unwrap();
        assert_eq!(sem.memory_layer, "semantic");

        let epi = engine
            .insert_layered_memory(
                "epi-1",
                "chat",
                "task_deadline",
                MemoryLayer::Episodic,
                "Submit quarterly report by Friday 5pm",
                0.9,
                "auto_learned",
            )
            .unwrap();
        assert_eq!(epi.memory_layer, "episodic");

        let proc_rec = engine
            .insert_layered_memory(
                "proc-1",
                "chat",
                "folder_rule",
                MemoryLayer::Procedural,
                "Always save exported PDFs to D:\\Reports",
                0.9,
                "permanent",
            )
            .unwrap();
        assert_eq!(proc_rec.memory_layer, "procedural");

        let semantic_list = engine
            .list_memories_by_layer(Some(MemoryLayer::Semantic), None)
            .unwrap();
        assert_eq!(semantic_list.len(), 1);
        assert_eq!(semantic_list[0].id, "sem-1");

        // Episodic decays faster than semantic/procedural after 30 days
        let future_now = sem.created_at + 30 * 86400;
        let sem_score = MemoryEngine::effective_score(&sem, future_now);
        let epi_score = MemoryEngine::effective_score(&epi, future_now);
        assert!(sem_score > epi_score);
    }

    #[test]
    fn test_extract_and_learn_from_turn_filters_small_talk() {
        let storage = Arc::new(Database::open_in_memory().unwrap());
        let engine = MemoryEngine::new(storage);

        // Small talk should be ignored
        let ignored = engine
            .extract_and_learn_from_turn("Halo Hime, makasih ya!", "Sama-sama!")
            .unwrap();
        assert!(ignored.is_empty());

        // Meaningful turn with preference, deadline, correction, and folder rule
        let turn_prompt = "I prefer concise answers in Indonesian. \
            Deadline submit laporan proyek hari Jumat jam 5 sore. \
            Next time please run cargo check before editing. \
            Save all generated reports to D:\\Hime\\Reports";
        let learned = engine
            .extract_and_learn_from_turn(turn_prompt, "Noted, I will follow those rules.")
            .unwrap();

        assert_eq!(learned.len(), 4);
        assert!(learned.iter().any(|m| m.memory_layer == "semantic" && m.category == "user_preference"));
        assert!(learned.iter().any(|m| m.memory_layer == "episodic" && m.category == "task_deadline"));
        assert!(learned.iter().any(|m| m.memory_layer == "procedural" && m.category == "habit_correction"));
        assert!(learned.iter().any(|m| m.memory_layer == "procedural" && m.category == "folder_rule"));

        // Re-running the exact same turn should deduplicate rather than duplicate rows
        let learned_again = engine
            .extract_and_learn_from_turn(turn_prompt, "Noted.")
            .unwrap();
        assert_eq!(learned_again.len(), 4);
        let all = engine.list_memories(None).unwrap();
        assert_eq!(all.len(), 4);
    }
}
