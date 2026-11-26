use serde::{Deserialize, Serialize};
use hime_storage::RoutineRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    pub fn from_index(idx: u32) -> Option<Self> {
        match idx % 7 {
            0 => Some(Weekday::Sunday),
            1 => Some(Weekday::Monday),
            2 => Some(Weekday::Tuesday),
            3 => Some(Weekday::Wednesday),
            4 => Some(Weekday::Thursday),
            5 => Some(Weekday::Friday),
            6 => Some(Weekday::Saturday),
            _ => None,
        }
    }

    pub fn to_index(self) -> u32 {
        match self {
            Weekday::Sunday => 0,
            Weekday::Monday => 1,
            Weekday::Tuesday => 2,
            Weekday::Wednesday => 3,
            Weekday::Thursday => 4,
            Weekday::Friday => 5,
            Weekday::Saturday => 6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutineSchedule {
    pub days: Vec<Weekday>,
    pub time: String,            // "HH:MM" 24h format
    pub duration_min: u32,       // e.g. 30
    #[serde(default)]
    pub window_min: Option<u32>, // optional flex window
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EnforcementTier {
    Ingatkan, // Gentle reminder: bubble P1/P2, single reminder, easily snoozed
    Tegas,    // Firm: repeated reminders, opens panel, capped snooze with reason pills
    Paksa,    // Strict: warning countdown, graceful WM_CLOSE
}

impl EnforcementTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnforcementTier::Ingatkan => "ingatkan",
            EnforcementTier::Tegas => "tegas",
            EnforcementTier::Paksa => "paksa",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "paksa" | "strict" => EnforcementTier::Paksa,
            "tegas" | "firm" => EnforcementTier::Tegas,
            _ => EnforcementTier::Ingatkan,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DistractionRules {
    pub block_during_routine: bool,
    #[serde(default)]
    pub daily_quota_min: Option<u32>,
    #[serde(default)]
    pub sequential_after_focus_min: Option<u32>,
    #[serde(default)]
    pub distraction_apps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoEscalationConfig {
    pub enabled: bool,
    pub missed_threshold: u32, // e.g. escalate to Tegas after 3 missed reminders
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RoutineStreak {
    pub current_count: u32,
    pub best_count: u32,
    #[serde(default)]
    pub last_completed_date: Option<String>, // "YYYY-MM-DD"
    #[serde(default)]
    pub frozen_on_date: Option<String>,      // for excused skip
}

fn default_start_grace_min() -> u32 {
    5
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Routine {
    #[serde(default)]
    pub id: String,
    pub title: String,
    pub category: String,
    pub schedule: RoutineSchedule,
    pub enforcement: EnforcementTier,
    #[serde(default)]
    pub allowed_apps: Vec<String>,
    #[serde(default)]
    pub distraction_rules: DistractionRules,
    #[serde(default = "default_start_grace_min")]
    pub start_grace_min: u32,
    #[serde(default)]
    pub escalation: Option<AutoEscalationConfig>,
    #[serde(default)]
    pub streak: RoutineStreak,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

impl Routine {
    pub fn to_storage_record(&self) -> RoutineRecord {
        RoutineRecord {
            id: self.id.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            schedule_json: serde_json::to_string(&self.schedule).unwrap_or_default(),
            enforcement: self.enforcement.as_str().to_string(),
            allowed_apps_json: if self.allowed_apps.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&self.allowed_apps).unwrap_or_default())
            },
            distraction_rules_json: Some(serde_json::to_string(&self.distraction_rules).unwrap_or_default()),
            start_grace_min: self.start_grace_min as i64,
            escalation_json: self.escalation.as_ref().map(|e| serde_json::to_string(e).unwrap_or_default()),
            current_streak: self.streak.current_count as i64,
            best_streak: self.streak.best_count as i64,
            last_completed_date: self.streak.last_completed_date.clone(),
            frozen_on_date: self.streak.frozen_on_date.clone(),
            active: self.active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn from_storage_record(record: &RoutineRecord) -> Self {
        let schedule: RoutineSchedule = serde_json::from_str(&record.schedule_json).unwrap_or(RoutineSchedule {
            days: vec![],
            time: "19:00".to_string(),
            duration_min: 30,
            window_min: None,
        });

        let enforcement = EnforcementTier::from_str_lossy(&record.enforcement);

        let allowed_apps: Vec<String> = record
            .allowed_apps_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        let distraction_rules: DistractionRules = record
            .distraction_rules_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        let escalation: Option<AutoEscalationConfig> = record
            .escalation_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());

        Routine {
            id: record.id.clone(),
            title: record.title.clone(),
            category: record.category.clone(),
            schedule,
            enforcement,
            allowed_apps,
            distraction_rules,
            start_grace_min: record.start_grace_min.max(1) as u32,
            escalation,
            streak: RoutineStreak {
                current_count: record.current_streak.max(0) as u32,
                best_count: record.best_streak.max(0) as u32,
                last_completed_date: record.last_completed_date.clone(),
                frozen_on_date: record.frozen_on_date.clone(),
            },
            active: record.active,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}
