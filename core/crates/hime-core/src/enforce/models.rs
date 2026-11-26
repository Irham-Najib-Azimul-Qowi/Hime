use crate::routines::models::EnforcementTier;
use hime_storage::CommitmentRecord;
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Commitment {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub tier: EnforcementTier,
    #[serde(default)]
    pub target_apps: Vec<String>,
    #[serde(default)]
    pub daily_quota_min: Option<u32>,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub pending_weakening_until: Option<i64>,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

impl Commitment {
    pub fn to_storage_record(&self) -> CommitmentRecord {
        CommitmentRecord {
            id: self.id.clone(),
            name: self.name.clone(),
            tier: self.tier.as_str().to_string(),
            target_apps_json: serde_json::to_string(&self.target_apps).unwrap_or_default(),
            daily_quota_min: self.daily_quota_min.map(|q| q as i64),
            active: self.active,
            pending_weakening_until: self.pending_weakening_until,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn from_storage_record(record: &CommitmentRecord) -> Self {
        let target_apps: Vec<String> = serde_json::from_str(&record.target_apps_json).unwrap_or_default();
        let tier = EnforcementTier::from_str_lossy(&record.tier);

        Self {
            id: record.id.clone(),
            name: record.name.clone(),
            tier,
            target_apps,
            daily_quota_min: record.daily_quota_min.map(|q| q.max(0) as u32),
            active: record.active,
            pending_weakening_until: record.pending_weakening_until,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum EnforcementAction {
    Allowed {
        reason: String,
    },
    Paused {
        remaining_secs: u64,
        reason: String,
    },
    WarnGentle {
        target_app: String,
        bubble: String,
    },
    WarnFirm {
        target_app: String,
        bubble: String,
        open_panel: bool,
        reason_required: bool,
    },
    EnforceStrict {
        target_app: String,
        countdown_secs: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmergencyPauseStatus {
    pub is_paused: bool,
    pub remaining_secs: i64,
    pub reason: Option<String>,
}
