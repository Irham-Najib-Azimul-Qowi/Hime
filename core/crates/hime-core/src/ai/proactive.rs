use std::sync::Arc;
use hime_storage::{Database, ProactiveSettingsRecord, ProactiveSuggestionRecord};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProactiveSuggestionItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String, // "schedule" | "tasks" | "storage" | "data"
    pub status: String,   // "pending" | "accepted" | "dismissed" | "muted"
    pub score: f64,
    pub created_at: i64,
    pub responded_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProactiveSettings {
    pub is_enabled: bool,
    pub cooldown_minutes: i64,
    pub muted_until: Option<i64>,
    pub updated_at: i64,
}

pub struct ProactiveEngine {
    db: Arc<Database>,
}

impl ProactiveEngine {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn get_settings(&self) -> ProactiveSettings {
        let rec = self.db.get_proactive_settings().unwrap_or_else(|_| ProactiveSettingsRecord {
            id: "default".to_string(),
            is_enabled: true,
            cooldown_minutes: 30,
            muted_until: None,
            updated_at: 0,
        });
        ProactiveSettings {
            is_enabled: rec.is_enabled,
            cooldown_minutes: rec.cooldown_minutes,
            muted_until: rec.muted_until,
            updated_at: rec.updated_at,
        }
    }

    pub fn update_settings(
        &self,
        is_enabled: bool,
        cooldown_minutes: i64,
        mute_hours: Option<i64>,
    ) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let muted_until = mute_hours.map(|hrs| now + (hrs * 3600));

        let rec = ProactiveSettingsRecord {
            id: "default".to_string(),
            is_enabled,
            cooldown_minutes: cooldown_minutes.max(5), // Minimum 5 minutes cooldown
            muted_until,
            updated_at: now,
        };

        self.db.set_proactive_settings(&rec)
            .map_err(|e| format!("Gagal menyimpan pengaturan proaktif: {}", e))
    }

    /// Evaluates context and generates proactive suggestions if opt-in and rate-limits permit.
    pub fn evaluate_and_generate_suggestions(&self) -> Result<Vec<ProactiveSuggestionItem>, String> {
        let settings = self.get_settings();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 1. Opt-in verification
        if !settings.is_enabled {
            return Ok(Vec::new());
        }

        // 2. Mute verification
        if let Some(muted_until) = settings.muted_until {
            if now < muted_until {
                return Ok(Vec::new());
            }
        }

        // 3. Rate-limiting verification (check latest suggestion timestamp)
        let existing = self.db.list_proactive_suggestions(None, 1).unwrap_or_default();
        if let Some(latest) = existing.first() {
            let elapsed_secs = now - latest.created_at;
            let min_gap_secs = settings.cooldown_minutes * 60;
            if elapsed_secs < min_gap_secs {
                // In cooldown
                return Ok(existing
                    .into_iter()
                    .filter(|s| s.status == "pending")
                    .map(Self::map_record)
                    .collect());
            }
        }

        // Generate contextual candidate suggestion
        let candidate = ProactiveSuggestionRecord {
            id: format!("sug-{:x}", now),
            title: "Rekomendasi Fokus & Belajar".to_string(),
            content: "Ada blok belajar yang dijadwalkan dalam 15 menit. Siapkan catatan atau aktifkan Mode Strict?".to_string(),
            category: "schedule".to_string(),
            status: "pending".to_string(),
            score: 0.92,
            created_at: now,
            responded_at: None,
        };

        let _ = self.db.insert_proactive_suggestion(&candidate);
        info!("Generated new proactive recommendation: {}", candidate.title);

        Ok(vec![Self::map_record(candidate)])
    }

    pub fn list_suggestions(&self, status: Option<&str>, limit: usize) -> Vec<ProactiveSuggestionItem> {
        self.db.list_proactive_suggestions(status, limit)
            .unwrap_or_default()
            .into_iter()
            .map(Self::map_record)
            .collect()
    }

    pub fn respond_suggestion(&self, suggestion_id: &str, action: &str) -> Result<bool, String> {
        let valid_status = match action.to_lowercase().as_str() {
            "accept" | "accepted" => "accepted",
            "dismiss" | "dismissed" => "dismissed",
            "mute" | "muted" => {
                // Also apply 4 hours mute
                let _ = self.update_settings(true, 30, Some(4));
                "muted"
            }
            _ => return Err("Aksi tidak valid (hanya accept, dismiss, atau mute).".to_string()),
        };

        self.db.respond_proactive_suggestion(suggestion_id, valid_status)
            .map_err(|e| format!("Gagal memproses respon saran: {}", e))
    }

    fn map_record(r: ProactiveSuggestionRecord) -> ProactiveSuggestionItem {
        ProactiveSuggestionItem {
            id: r.id,
            title: r.title,
            content: r.content,
            category: r.category,
            status: r.status,
            score: r.score,
            created_at: r.created_at,
            responded_at: r.responded_at,
        }
    }
}
