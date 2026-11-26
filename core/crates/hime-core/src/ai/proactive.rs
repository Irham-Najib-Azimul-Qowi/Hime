//! Hime Proactive Interaction Engine
//!
//! Implements §2 attention budget (Pendiam/Seimbang/Cerewet), quiet hours
//! (23:00–06:00 local time), exponential backoff after 3 dismissals per kind,
//! the interaction_prefs preference table, and catalog situations 1–22.
//!
//! Architecture: single-exit-point design — all proactive interactions are
//! decided and emitted here; no other module talks directly to the user.

use std::collections::HashMap;
use std::sync::Arc;
use hime_storage::{Database, InteractionPrefRecord, ProactiveSettingsRecord, ProactiveSuggestionRecord};
use serde::{Deserialize, Serialize};
use tracing::info;

// ---------------------------------------------------------------------------
// Win32 Inter-Process Wake Signaling
// ---------------------------------------------------------------------------

#[cfg(windows)]
pub fn signal_named_event(name: &str) -> bool {
    unsafe {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenEventW, SetEvent, SYNCHRONIZATION_ACCESS_RIGHTS};
        let event_name: Vec<u16> = name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        // EVENT_MODIFY_STATE | SYNCHRONIZE = 0x00100002
        if let Ok(h_event) = OpenEventW(
            SYNCHRONIZATION_ACCESS_RIGHTS(0x00100002),
            false,
            windows::core::PCWSTR(event_name.as_ptr()),
        ) {
            let _ = SetEvent(h_event);
            let _ = CloseHandle(h_event);
            true
        } else {
            false
        }
    }
}

#[cfg(not(windows))]
pub fn signal_named_event(_name: &str) -> bool {
    false
}

/// Signals the Win32 Named Event `HimeShowIsland` to wake/unminimize the Island.
pub fn signal_hime_show_island() -> bool {
    signal_named_event("HimeShowIsland")
}

/// Signals `HimeToggleAssistant` to immediately open Hime Chat panel.
pub fn signal_hime_toggle_assistant() -> bool {
    signal_named_event("HimeToggleAssistant")
}

// ---------------------------------------------------------------------------
// Attention budget
// ---------------------------------------------------------------------------

/// Disturbance mode — matches Settings → Hime toggle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AttentionMode {
    /// Colek 1/hr, Tanya 1/hr
    Pendiam,
    /// Colek 3/hr, Tanya 3/hr (default)
    Seimbang,
    /// Colek 6/hr, Tanya 5/hr
    Cerewet,
}

impl AttentionMode {
    /// Maximum Colek (non-action bubble) per hour.
    pub fn colek_per_hour(self) -> u32 {
        match self {
            AttentionMode::Pendiam => 1,
            AttentionMode::Seimbang => 3,
            AttentionMode::Cerewet => 6,
        }
    }

    /// Maximum Tanya (action bubble) per hour.
    pub fn tanya_per_hour(self) -> u32 {
        match self {
            AttentionMode::Pendiam => 1,
            AttentionMode::Seimbang => 3,
            AttentionMode::Cerewet => 5,
        }
    }
}

impl Default for AttentionMode {
    fn default() -> Self {
        AttentionMode::Seimbang
    }
}

impl std::fmt::Display for AttentionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttentionMode::Pendiam => write!(f, "pendiam"),
            AttentionMode::Seimbang => write!(f, "seimbang"),
            AttentionMode::Cerewet => write!(f, "cerewet"),
        }
    }
}

impl std::str::FromStr for AttentionMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pendiam" => Ok(AttentionMode::Pendiam),
            "seimbang" => Ok(AttentionMode::Seimbang),
            "cerewet" => Ok(AttentionMode::Cerewet),
            other => Err(format!("Unknown attention mode: {}", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// Disturbance level (matches §2 table)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DisturbanceLevel {
    /// Subtle change (eye expression, pill dot) — not budget-counted
    Bisik,
    /// Auto-dismiss bubble, no mandatory action — counts as Colek
    Colek,
    /// Bubble with ≤3 actions — counts as Tanya
    Tanya,
    /// Expansion + short chime — always allowed (Penting)
    Penting,
}

impl std::fmt::Display for DisturbanceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisturbanceLevel::Bisik => write!(f, "bisik"),
            DisturbanceLevel::Colek => write!(f, "colek"),
            DisturbanceLevel::Tanya => write!(f, "tanya"),
            DisturbanceLevel::Penting => write!(f, "penting"),
        }
    }
}

// ---------------------------------------------------------------------------
// Catalog situation kinds (1–22)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SituationKind {
    FileDuplicate,
    NewDownload,
    StorageLow,
    DeadlineNear,
    RoutineStart,
    RoutineMissed,
    DistractionOverLimit,
    MorningBriefing,
    EveningRecap,
    TaskCompleted,
    IdleDuringStudy,
    ForeignText,
    BatteryLow,
    DeviceConnected,
    AgentTaskDone,
    RiskyActionApproval,
    RuntimeOffline,
    LateNight,
    WelcomeBack,
    NewAppUsed,
    ClipboardSave,
    UpdateOrError,
}

impl SituationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SituationKind::FileDuplicate => "file_duplicate",
            SituationKind::NewDownload => "new_download",
            SituationKind::StorageLow => "storage_low",
            SituationKind::DeadlineNear => "deadline_near",
            SituationKind::RoutineStart => "routine_start",
            SituationKind::RoutineMissed => "routine_missed",
            SituationKind::DistractionOverLimit => "distraction_over_limit",
            SituationKind::MorningBriefing => "morning_briefing",
            SituationKind::EveningRecap => "evening_recap",
            SituationKind::TaskCompleted => "task_completed",
            SituationKind::IdleDuringStudy => "idle_during_study",
            SituationKind::ForeignText => "foreign_text",
            SituationKind::BatteryLow => "battery_low",
            SituationKind::DeviceConnected => "device_connected",
            SituationKind::AgentTaskDone => "agent_task_done",
            SituationKind::RiskyActionApproval => "risky_action_approval",
            SituationKind::RuntimeOffline => "runtime_offline",
            SituationKind::LateNight => "late_night",
            SituationKind::WelcomeBack => "welcome_back",
            SituationKind::NewAppUsed => "new_app_used",
            SituationKind::ClipboardSave => "clipboard_save",
            SituationKind::UpdateOrError => "update_or_error",
        }
    }

    /// Disturbance level for this situation (§2 table / catalog table).
    pub fn disturbance_level(&self) -> DisturbanceLevel {
        match self {
            SituationKind::FileDuplicate => DisturbanceLevel::Tanya,
            SituationKind::NewDownload => DisturbanceLevel::Tanya,
            SituationKind::StorageLow => DisturbanceLevel::Colek,
            SituationKind::DeadlineNear => DisturbanceLevel::Penting,
            SituationKind::RoutineStart => DisturbanceLevel::Penting,
            SituationKind::RoutineMissed => DisturbanceLevel::Colek,
            SituationKind::DistractionOverLimit => DisturbanceLevel::Penting,
            SituationKind::MorningBriefing => DisturbanceLevel::Tanya,
            SituationKind::EveningRecap => DisturbanceLevel::Colek,
            SituationKind::TaskCompleted => DisturbanceLevel::Bisik,
            SituationKind::IdleDuringStudy => DisturbanceLevel::Colek,
            SituationKind::ForeignText => DisturbanceLevel::Bisik,
            SituationKind::BatteryLow => DisturbanceLevel::Colek,
            SituationKind::DeviceConnected => DisturbanceLevel::Bisik,
            SituationKind::AgentTaskDone => DisturbanceLevel::Colek,
            SituationKind::RiskyActionApproval => DisturbanceLevel::Penting,
            SituationKind::RuntimeOffline => DisturbanceLevel::Colek,
            SituationKind::LateNight => DisturbanceLevel::Bisik,
            SituationKind::WelcomeBack => DisturbanceLevel::Bisik,
            SituationKind::NewAppUsed => DisturbanceLevel::Colek,
            SituationKind::ClipboardSave => DisturbanceLevel::Bisik,
            SituationKind::UpdateOrError => DisturbanceLevel::Colek,
        }
    }
}

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProactiveSuggestionItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub status: String, // "pending" | "accepted" | "dismissed" | "muted"
    pub score: f64,
    pub created_at: i64,
    pub responded_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProactiveSettings {
    pub is_enabled: bool,
    pub cooldown_minutes: i64,
    pub attention_mode: String, // "pendiam" | "seimbang" | "cerewet"
    pub quiet_hour_start: u8,   // default 23
    pub quiet_hour_end: u8,     // default 6
    pub muted_until: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InteractionPref {
    pub kind: String,
    pub decision: String,  // "always" | "never" | "default"
    pub backoff_count: i64,
    pub backoff_until: Option<i64>,
    pub last_shown_at: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeriodicReminderConfig {
    pub id: String,
    pub title: String,
    pub message: String,
    pub interval_secs: u64, // e.g. 1800 (30m), 3600 (1h)
    pub last_triggered_at: u64,
    pub is_active: bool,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

pub struct ProactiveEngine {
    db: Arc<Database>,
    reminders: Arc<std::sync::RwLock<HashMap<String, PeriodicReminderConfig>>>,
    event_bus: Arc<std::sync::RwLock<Option<crate::bus::EventBus>>>,
}

impl ProactiveEngine {
    pub fn new(db: Arc<Database>) -> Self {
        let mut default_reminders = HashMap::new();
        let now = now_secs() as u64;

        default_reminders.insert(
            "rem-hourly-focus".to_string(),
            PeriodicReminderConfig {
                id: "rem-hourly-focus".to_string(),
                title: "Pengingat Fokus Hime".to_string(),
                message: "Sudah 1 jam beraktivitas. Luangkan sejenak untuk istirahat mata dan minum air.".to_string(),
                interval_secs: 3600,
                last_triggered_at: now,
                is_active: true,
            },
        );

        default_reminders.insert(
            "rem-30min-posture".to_string(),
            PeriodicReminderConfig {
                id: "rem-30min-posture".to_string(),
                title: "Peregangan 30 Menit".to_string(),
                message: "Cek postur tubuh dan tarik napas dalam sebelum melanjutkan tugas.".to_string(),
                interval_secs: 1800,
                last_triggered_at: now,
                is_active: false,
            },
        );

        Self {
            db,
            reminders: Arc::new(std::sync::RwLock::new(default_reminders)),
            event_bus: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    pub fn set_event_bus(&self, bus: crate::bus::EventBus) {
        if let Ok(mut lock) = self.event_bus.write() {
            *lock = Some(bus);
        }
    }

    pub fn register_periodic_reminder(
        &self,
        id: &str,
        title: &str,
        message: &str,
        interval_secs: u64,
        is_active: bool,
    ) -> PeriodicReminderConfig {
        let now = now_secs() as u64;
        let config = PeriodicReminderConfig {
            id: id.to_string(),
            title: title.to_string(),
            message: message.to_string(),
            interval_secs: interval_secs.max(60),
            last_triggered_at: now,
            is_active,
        };
        if let Ok(mut lock) = self.reminders.write() {
            lock.insert(id.to_string(), config.clone());
        }
        config
    }

    pub fn list_periodic_reminders(&self) -> Vec<PeriodicReminderConfig> {
        self.reminders.read().map(|m| m.values().cloned().collect()).unwrap_or_default()
    }

    pub fn toggle_periodic_reminder(&self, id: &str, active: bool) -> bool {
        if let Ok(mut lock) = self.reminders.write() {
            if let Some(r) = lock.get_mut(id) {
                r.is_active = active;
                return true;
            }
        }
        false
    }

    pub fn remove_periodic_reminder(&self, id: &str) -> bool {
        if let Ok(mut lock) = self.reminders.write() {
            lock.remove(id).is_some()
        } else {
            false
        }
    }

    /// Checks active periodic reminders and triggers any that have exceeded their interval.
    /// When triggered:
    /// 1. Signals Win32 `HimeShowIsland` and `HimeToggleAssistant` events
    /// 2. Dispatches `SystemEvent::ProactiveReminderTriggered` to the system event bus
    /// 3. Records a proactive suggestion in the database
    pub fn check_and_trigger_periodic_reminders(&self) -> Vec<ProactiveSuggestionItem> {
        let now = now_secs() as u64;
        let mut triggered = Vec::new();

        let mut to_fire = Vec::new();
        if let Ok(mut lock) = self.reminders.write() {
            for (_, r) in lock.iter_mut() {
                if r.is_active && (now >= r.last_triggered_at.saturating_add(r.interval_secs)) {
                    r.last_triggered_at = now;
                    to_fire.push(r.clone());
                }
            }
        }

        for rem in to_fire {
            signal_hime_show_island();
            signal_hime_toggle_assistant();

            if let Ok(guard) = self.event_bus.read() {
                if let Some(ref bus) = *guard {
                    bus.publish(crate::bus::SystemEvent::ProactiveReminderTriggered {
                        reminder_id: rem.id.clone(),
                        title: rem.title.clone(),
                        message: rem.message.clone(),
                        deadline_near: false,
                    });
                }
            }

            let kind = SituationKind::RoutineStart;
            let title = truncate(&rem.title, 40);
            let sug_id = self.insert_suggestion(&kind, title, rem.message.clone(), 0.9);
            if let Ok(Some(sug)) = self.db.get_proactive_suggestion(&sug_id) {
                triggered.push(Self::map_record(sug));
            }
        }

        triggered
    }

    /// Inspects tasks in database for upcoming deadlines (e.g. within threshold_minutes).
    /// When deadline is near:
    /// 1. Signals `HimeShowIsland` and opens chat
    /// 2. Dispatches `ProactiveReminderTriggered`
    /// 3. Records `on_deadline_near`
    pub fn check_and_trigger_deadline_reminders(&self, threshold_minutes: i64) -> Vec<ProactiveSuggestionItem> {
        let now = now_secs();
        let mut triggered = Vec::new();

        if let Ok(tasks) = self.db.list_productivity_tasks("board-default") {
            for task in tasks {
                if task.status == "done" || task.status == "completed" {
                    continue;
                }
                if let Some(due_date) = task.due_date {
                    let diff_secs = due_date - now;
                    let diff_mins = diff_secs / 60;
                    if diff_secs > 0 && diff_mins <= threshold_minutes {
                        let sentinel = format!("__deadline_reminded_{}__", task.id);
                        if let Ok(Some(_)) = self.db.get_interaction_pref(&sentinel) {
                            continue;
                        }

                        let desc = format!("Tenggat waktu dalam {} menit!", diff_mins);
                        if let Some(sug_id) = self.on_deadline_near(&task.title, &desc) {
                            signal_hime_show_island();
                            signal_hime_toggle_assistant();

                            if let Ok(guard) = self.event_bus.read() {
                                if let Some(ref bus) = *guard {
                                    bus.publish(crate::bus::SystemEvent::ProactiveReminderTriggered {
                                        reminder_id: format!("deadline-{}", task.id),
                                        title: format!("Deadline: {}", task.title),
                                        message: desc.clone(),
                                        deadline_near: true,
                                    });
                                }
                            }

                            let _ = self.db.upsert_interaction_pref(&InteractionPrefRecord {
                                kind: sentinel,
                                decision: "reminded".to_string(),
                                backoff_count: 0,
                                backoff_until: None,
                                last_shown_at: Some(now),
                                updated_at: now,
                            });

                            if let Ok(Some(sug)) = self.db.get_proactive_suggestion(&sug_id) {
                                triggered.push(Self::map_record(sug));
                            }
                        }
                    }
                }
            }
        }

        triggered
    }

    pub fn start_reminder_scheduler(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                self.check_and_trigger_periodic_reminders();
                self.check_and_trigger_deadline_reminders(60);
            }
        });
    }

    // -----------------------------------------------------------------------
    // Settings
    // -----------------------------------------------------------------------

    pub fn get_settings(&self) -> ProactiveSettings {
        let rec = self.db.get_proactive_settings().unwrap_or_else(|_| ProactiveSettingsRecord {
            id: "default".to_string(),
            is_enabled: true,
            cooldown_minutes: 30,
            muted_until: None,
            updated_at: 0,
        });
        let mode = self.db.get_interaction_pref("__attention_mode__")
            .ok()
            .flatten()
            .map(|r| r.decision)
            .unwrap_or_else(|| "seimbang".to_string());
        let qstart = self.db.get_interaction_pref("__quiet_hour_start__")
            .ok()
            .flatten()
            .and_then(|r| r.decision.parse::<u8>().ok())
            .unwrap_or(23);
        let qend = self.db.get_interaction_pref("__quiet_hour_end__")
            .ok()
            .flatten()
            .and_then(|r| r.decision.parse::<u8>().ok())
            .unwrap_or(6);

        ProactiveSettings {
            is_enabled: rec.is_enabled,
            cooldown_minutes: rec.cooldown_minutes,
            attention_mode: mode,
            quiet_hour_start: qstart,
            quiet_hour_end: qend,
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
        let now = now_secs();
        let muted_until = mute_hours.map(|hrs| now + (hrs * 3600));

        let rec = ProactiveSettingsRecord {
            id: "default".to_string(),
            is_enabled,
            cooldown_minutes: cooldown_minutes.max(5),
            muted_until,
            updated_at: now,
        };
        self.db.set_proactive_settings(&rec)
            .map_err(|e| format!("Gagal menyimpan pengaturan proaktif: {}", e))
    }

    pub fn update_extended_settings(
        &self,
        is_enabled: bool,
        attention_mode: &str,
        quiet_hour_start: u8,
        quiet_hour_end: u8,
        cooldown_minutes: i64,
        mute_hours: Option<i64>,
    ) -> Result<(), String> {
        let now = now_secs();
        let _mode: AttentionMode = attention_mode.parse()?;
        let muted_until = mute_hours.map(|hrs| now + (hrs * 3600));

        let rec = ProactiveSettingsRecord {
            id: "default".to_string(),
            is_enabled,
            cooldown_minutes: cooldown_minutes.max(5),
            muted_until,
            updated_at: now,
        };
        self.db.set_proactive_settings(&rec)
            .map_err(|e| format!("Gagal menyimpan pengaturan proaktif: {}", e))?;

        // Store extended settings in interaction_prefs sentinel keys
        self.db.upsert_interaction_pref(&InteractionPrefRecord {
            kind: "__attention_mode__".to_string(),
            decision: attention_mode.to_string(),
            backoff_count: 0,
            backoff_until: None,
            last_shown_at: None,
            updated_at: now,
        }).map_err(|e| format!("Gagal simpan mode: {}", e))?;

        self.db.upsert_interaction_pref(&InteractionPrefRecord {
            kind: "__quiet_hour_start__".to_string(),
            decision: quiet_hour_start.to_string(),
            backoff_count: 0,
            backoff_until: None,
            last_shown_at: None,
            updated_at: now,
        }).map_err(|e| format!("Gagal simpan quiet_start: {}", e))?;

        self.db.upsert_interaction_pref(&InteractionPrefRecord {
            kind: "__quiet_hour_end__".to_string(),
            decision: quiet_hour_end.to_string(),
            backoff_count: 0,
            backoff_until: None,
            last_shown_at: None,
            updated_at: now,
        }).map_err(|e| format!("Gagal simpan quiet_end: {}", e))
    }

    // -----------------------------------------------------------------------
    // Interaction Preference Rules
    // -----------------------------------------------------------------------

    pub fn get_preference(&self, kind: &str) -> Option<InteractionPref> {
        self.db.get_interaction_pref(kind)
            .ok()
            .flatten()
            .map(Self::map_pref)
    }

    pub fn list_preferences(&self) -> Vec<InteractionPref> {
        self.db.list_interaction_prefs()
            .unwrap_or_default()
            .into_iter()
            .filter(|r| !r.kind.starts_with("__"))
            .map(Self::map_pref)
            .collect()
    }

    /// Set "always", "never", or "default" rule for a situation kind.
    pub fn set_preference(&self, kind: &str, decision: &str) -> Result<(), String> {
        match decision {
            "always" | "never" | "default" => {}
            other => return Err(format!("Keputusan tidak valid: {}. Gunakan always/never/default", other)),
        }
        let now = now_secs();
        self.db.upsert_interaction_pref(&InteractionPrefRecord {
            kind: kind.to_string(),
            decision: decision.to_string(),
            backoff_count: 0,
            backoff_until: None,
            last_shown_at: None,
            updated_at: now,
        }).map_err(|e| format!("Gagal simpan preferensi: {}", e))
    }

    /// Reset backoff for a kind (visible in Settings, can be reset by user).
    pub fn reset_backoff(&self, kind: &str) -> Result<(), String> {
        let now = now_secs();
        let existing = self.db.get_interaction_pref(kind)
            .ok()
            .flatten()
            .unwrap_or_else(|| InteractionPrefRecord {
                kind: kind.to_string(),
                decision: "default".to_string(),
                backoff_count: 0,
                backoff_until: None,
                last_shown_at: None,
                updated_at: now,
            });
        self.db.upsert_interaction_pref(&InteractionPrefRecord {
            backoff_count: 0,
            backoff_until: None,
            updated_at: now,
            ..existing
        }).map_err(|e| format!("Gagal reset backoff: {}", e))
    }

    // -----------------------------------------------------------------------
    // Core gate: should a situation be shown right now?
    // -----------------------------------------------------------------------

    pub fn gate_situation(&self, kind: &SituationKind) -> bool {
        let settings = self.get_settings();

        if !settings.is_enabled {
            return false;
        }

        let now = now_secs();
        if let Some(mu) = settings.muted_until {
            if now < mu {
                return false;
            }
        }

        // Quiet hours: only Colek/Tanya are blocked (Bisik/Penting pass through)
        let level = kind.disturbance_level();
        if level == DisturbanceLevel::Colek || level == DisturbanceLevel::Tanya {
            if is_quiet_hour(settings.quiet_hour_start, settings.quiet_hour_end) {
                return false;
            }
        }

        // Per-kind preference rule
        let pref = self.db.get_interaction_pref(kind.as_str()).ok().flatten();
        if let Some(ref p) = pref {
            if p.decision == "never" {
                return false;
            }
            if p.backoff_count >= 3 {
                if let Some(bu) = p.backoff_until {
                    if now < bu {
                        return false;
                    }
                }
            }
        }

        true
    }

    // -----------------------------------------------------------------------
    // Budget accounting (hourly rate limiting)
    // -----------------------------------------------------------------------

    pub fn consume_budget(&self, level: DisturbanceLevel) -> bool {
        let (budget_key, limit) = match level {
            DisturbanceLevel::Bisik | DisturbanceLevel::Penting => return true,
            DisturbanceLevel::Colek => {
                let mode = self.get_attention_mode();
                ("__budget_colek__", mode.colek_per_hour())
            }
            DisturbanceLevel::Tanya => {
                let mode = self.get_attention_mode();
                ("__budget_tanya__", mode.tanya_per_hour())
            }
        };

        let now = now_secs();
        let hour_start = now - (now % 3600);

        let pref = self.db.get_interaction_pref(budget_key).ok().flatten();
        let (count, window_start) = match &pref {
            Some(p) => {
                let ws = p.last_shown_at.unwrap_or(hour_start);
                let cnt = if ws >= hour_start { p.backoff_count } else { 0 };
                (cnt, hour_start)
            }
            None => (0, hour_start),
        };

        if count >= limit as i64 {
            return false;
        }

        let _ = self.db.upsert_interaction_pref(&InteractionPrefRecord {
            kind: budget_key.to_string(),
            decision: "budget".to_string(),
            backoff_count: count + 1,
            backoff_until: None,
            last_shown_at: Some(window_start),
            updated_at: now,
        });
        true
    }

    fn get_attention_mode(&self) -> AttentionMode {
        self.db.get_interaction_pref("__attention_mode__")
            .ok()
            .flatten()
            .and_then(|r| r.decision.parse().ok())
            .unwrap_or_default()
    }

    // -----------------------------------------------------------------------
    // Catalog situation event handlers (1–22)
    // -----------------------------------------------------------------------

    /// Situation 1: Duplicate file detected.
    pub fn on_file_duplicate(&self, file_name: &str, existing_path: &str) -> Option<String> {
        let kind = SituationKind::FileDuplicate;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate("File sudah ada. Ganti?", 40);
        let content = format!("{} → {}", file_name, existing_path);
        Some(self.insert_suggestion(&kind, title, content, 0.9))
    }

    /// Situation 2: New file completed download.
    pub fn on_new_download(&self, file_name: &str) -> Option<String> {
        let kind = SituationKind::NewDownload;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate("Simpan ke mana?", 40);
        Some(self.insert_suggestion(&kind, title, file_name.to_string(), 0.75))
    }

    /// Situation 3: Storage almost full.
    pub fn on_storage_low(&self, drive: &str, percent_free: u8) -> Option<String> {
        let kind = SituationKind::StorageLow;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate(&format!("{} tersisa {}%", drive, percent_free), 40);
        Some(self.insert_suggestion(&kind, title, String::new(), 0.85))
    }

    /// Situation 4: Deadline near (H-1 or day-of morning).
    pub fn on_deadline_near(&self, task_title: &str, deadline_desc: &str) -> Option<String> {
        let kind = SituationKind::DeadlineNear;
        if !self.gate_situation(&kind) { return None; }
        // Penting — no budget limit
        let title = truncate(&format!("{} {}", task_title, deadline_desc), 40);
        Some(self.insert_suggestion(&kind, title, String::new(), 0.95))
    }

    /// Situation 5: Routine starting now.
    pub fn on_routine_start(&self, routine_name: &str, duration_minutes: u32) -> Option<String> {
        let kind = SituationKind::RoutineStart;
        if !self.gate_situation(&kind) { return None; }
        // Penting — no budget limit
        let title = truncate(&format!("Waktunya {} {} mnt", routine_name, duration_minutes), 40);
        Some(self.insert_suggestion(&kind, title, String::new(), 0.95))
    }

    /// Situation 6: Routine missed.
    pub fn on_routine_missed(&self, routine_name: &str) -> Option<String> {
        let kind = SituationKind::RoutineMissed;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate(&format!("{} terlewat. Jadwal ulang?", routine_name), 40);
        Some(self.insert_suggestion(&kind, title, String::new(), 0.7))
    }

    /// Situation 7: Distraction app exceeded time limit.
    pub fn on_distraction_over_limit(&self, app_name: &str, minutes: u32) -> Option<String> {
        let kind = SituationKind::DistractionOverLimit;
        if !self.gate_situation(&kind) { return None; }
        // Penting — no budget limit
        let title = truncate(&format!("Sudah {} mnt di luar belajar", minutes), 40);
        Some(self.insert_suggestion(&kind, title, app_name.to_string(), 0.9))
    }

    /// Situation 8: Morning briefing (first use of the day).
    pub fn on_morning_briefing(&self, task_count: u32, study_time_desc: &str) -> Option<String> {
        let kind = SituationKind::MorningBriefing;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate(&format!("{} tugas hari ini. {}", task_count, study_time_desc), 40);
        Some(self.insert_suggestion(&kind, title, String::new(), 0.8))
    }

    /// Situation 9: Evening recap (optional, 21:30).
    pub fn on_evening_recap(&self, done: u32, deferred: u32) -> Option<String> {
        let kind = SituationKind::EveningRecap;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate(&format!("{} selesai, {} geser besok", done, deferred), 40);
        Some(self.insert_suggestion(&kind, title, String::new(), 0.6))
    }

    /// Situation 10: Task completed (Bisik/brief).
    pub fn on_task_completed(&self, task_name: &str) -> Option<String> {
        let kind = SituationKind::TaskCompleted;
        if !self.gate_situation(&kind) { return None; }
        // Bisik — no budget
        let title = truncate("Selesai. Mantap.", 40);
        Some(self.insert_suggestion(&kind, title, task_name.to_string(), 0.5))
    }

    /// Situation 11: User idle during scheduled study block.
    pub fn on_idle_during_study(&self) -> Option<String> {
        let kind = SituationKind::IdleDuringStudy;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate("Masih belajar?", 40);
        Some(self.insert_suggestion(&kind, title, String::new(), 0.65))
    }

    /// Situation 12: Foreign text detected (auto-translate mode).
    pub fn on_foreign_text(&self, text_preview: &str) -> Option<String> {
        let kind = SituationKind::ForeignText;
        if !self.gate_situation(&kind) { return None; }
        // Bisik — no budget
        let title = truncate("Terjemahkan?", 40);
        Some(self.insert_suggestion(&kind, title, truncate(text_preview, 60), 0.5))
    }

    /// Situation 13: Battery low.
    pub fn on_battery_low(&self, percent: u8) -> Option<String> {
        let kind = SituationKind::BatteryLow;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate(&format!("Baterai {}%", percent), 40);
        Some(self.insert_suggestion(&kind, title, String::new(), 0.8))
    }

    /// Situation 14: Device connected or file transfer done.
    pub fn on_device_connected(&self, device_name: &str, file_count: Option<u32>) -> Option<String> {
        let kind = SituationKind::DeviceConnected;
        if !self.gate_situation(&kind) { return None; }
        // Bisik — no budget
        let title = if let Some(n) = file_count {
            truncate(&format!("{} file diterima", n), 40)
        } else {
            truncate(&format!("HP terhubung: {}", device_name), 40)
        };
        Some(self.insert_suggestion(&kind, title, String::new(), 0.4))
    }

    /// Situation 15: Agent finished a long task.
    pub fn on_agent_task_done(&self, summary_title: &str) -> Option<String> {
        let kind = SituationKind::AgentTaskDone;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate("Ringkasan siap", 40);
        Some(self.insert_suggestion(&kind, title, summary_title.to_string(), 0.75))
    }

    /// Situation 16: Risky action requires approval (P0 — bypasses mute; not quiet hour gated).
    pub fn on_risky_action_approval(&self, action_desc: &str, file_count: u32) -> Option<String> {
        let kind = SituationKind::RiskyActionApproval;
        // P0: only blocked if globally disabled
        if !self.db.get_proactive_settings()
            .map(|r| r.is_enabled)
            .unwrap_or(true)
        {
            return None;
        }
        let title = truncate(&format!("Pindahkan {} file?", file_count), 40);
        Some(self.insert_suggestion(&kind, title, action_desc.to_string(), 1.0))
    }

    /// Situation 17: AI runtime offline (shown once per episode).
    pub fn on_runtime_offline(&self) -> Option<String> {
        let kind = SituationKind::RuntimeOffline;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate("Hime offline", 40);
        Some(self.insert_suggestion(&kind, title, String::new(), 0.95))
    }

    /// Situation 18: Late night — user still active past quiet hour.
    pub fn on_late_night(&self) -> Option<String> {
        let kind = SituationKind::LateNight;
        if !self.gate_situation(&kind) { return None; }
        // Bisik — no budget
        let title = truncate("Sudah larut. Tidur?", 40);
        Some(self.insert_suggestion(&kind, title, String::new(), 0.3))
    }

    /// Situation 19: Welcome back after long absence.
    pub fn on_welcome_back(&self) -> Option<String> {
        let kind = SituationKind::WelcomeBack;
        if !self.gate_situation(&kind) { return None; }
        // Bisik — no budget
        let title = truncate("Selamat datang kembali.", 40);
        Some(self.insert_suggestion(&kind, title, String::new(), 0.4))
    }

    /// Situation 20: Frequently used uncategorized app.
    pub fn on_new_app_used(&self, app_name: &str) -> Option<String> {
        let kind = SituationKind::NewAppUsed;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate(&format!("Kategori untuk {}?", app_name), 40);
        Some(self.insert_suggestion(&kind, title, app_name.to_string(), 0.5))
    }

    /// Situation 21: Clipboard contains a link or long text.
    pub fn on_clipboard_save(&self, preview: &str) -> Option<String> {
        let kind = SituationKind::ClipboardSave;
        if !self.gate_situation(&kind) { return None; }
        // Bisik — no budget
        let title = truncate("Simpan sebagai tugas?", 40);
        Some(self.insert_suggestion(&kind, title, truncate(preview, 60), 0.5))
    }

    /// Situation 22: Important update or error.
    pub fn on_update_or_error(&self, description: &str) -> Option<String> {
        let kind = SituationKind::UpdateOrError;
        if !self.gate_situation(&kind) { return None; }
        if !self.consume_budget(kind.disturbance_level()) { return None; }
        let title = truncate("Ada masalah. Lihat?", 40);
        Some(self.insert_suggestion(&kind, title, truncate(description, 100), 0.85))
    }

    // -----------------------------------------------------------------------
    // Suggestions CRUD
    // -----------------------------------------------------------------------

    pub fn list_suggestions(&self, status: Option<&str>, limit: usize) -> Vec<ProactiveSuggestionItem> {
        self.db.list_proactive_suggestions(status, limit)
            .unwrap_or_default()
            .into_iter()
            .map(Self::map_record)
            .collect()
    }

    /// Respond: accept, dismiss, or mute. Dismissals trigger exponential backoff.
    pub fn respond_suggestion(&self, suggestion_id: &str, action: &str) -> Result<bool, String> {
        let valid_status = match action.to_lowercase().as_str() {
            "accept" | "accepted" => "accepted",
            "dismiss" | "dismissed" => {
                if let Ok(Some(sug)) = self.db.get_proactive_suggestion(suggestion_id) {
                    self.apply_dismissal_backoff(&sug.category);
                }
                "dismissed"
            }
            "mute" | "muted" => {
                // Global mute for 4 hours
                let now = now_secs();
                let _ = self.db.set_proactive_settings(&ProactiveSettingsRecord {
                    id: "default".to_string(),
                    is_enabled: true,
                    cooldown_minutes: 20,
                    muted_until: Some(now + 4 * 3600),
                    updated_at: now,
                });
                "muted"
            }
            _ => return Err("Aksi tidak valid (hanya accept, dismiss, atau mute).".to_string()),
        };

        self.db.respond_proactive_suggestion(suggestion_id, valid_status)
            .map_err(|e| format!("Gagal memproses respon saran: {}", e))
    }

    // -----------------------------------------------------------------------
    // Legacy compatibility (server.rs "proactive.suggestions.evaluate" endpoint)
    // -----------------------------------------------------------------------

    pub fn evaluate_and_generate_suggestions(&self) -> Result<Vec<ProactiveSuggestionItem>, String> {
        let settings = self.get_settings();
        let now = now_secs();

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

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn insert_suggestion(
        &self,
        kind: &SituationKind,
        title: String,
        content: String,
        score: f64,
    ) -> String {
        let now = now_secs();
        let id = format!("sug-{}-{:x}", kind.as_str(), now);
        let rec = ProactiveSuggestionRecord {
            id: id.clone(),
            title,
            content,
            category: kind.as_str().to_string(),
            status: "pending".to_string(),
            score,
            created_at: now,
            responded_at: None,
        };
        let _ = self.db.insert_proactive_suggestion(&rec);

        // Update last_shown_at
        if let Ok(existing_opt) = self.db.get_interaction_pref(kind.as_str()) {
            let pref = existing_opt.unwrap_or_else(|| InteractionPrefRecord {
                kind: kind.as_str().to_string(),
                decision: "default".to_string(),
                backoff_count: 0,
                backoff_until: None,
                last_shown_at: None,
                updated_at: now,
            });
            let _ = self.db.upsert_interaction_pref(&InteractionPrefRecord {
                last_shown_at: Some(now),
                updated_at: now,
                ..pref
            });
        }

        id
    }

    fn apply_dismissal_backoff(&self, kind: &str) {
        let now = now_secs();
        let existing = self.db.get_interaction_pref(kind)
            .ok()
            .flatten()
            .unwrap_or_else(|| InteractionPrefRecord {
                kind: kind.to_string(),
                decision: "default".to_string(),
                backoff_count: 0,
                backoff_until: None,
                last_shown_at: None,
                updated_at: now,
            });

        let new_count = existing.backoff_count + 1;
        // Exponential backoff after 3 dismissals: 2^(count-3) hours, capped at 2^10
        let backoff_until = if new_count >= 3 {
            let exp = (new_count - 3).min(10);
            let hours = 1i64 << exp;
            Some(now + hours * 3600)
        } else {
            None
        };

        let _ = self.db.upsert_interaction_pref(&InteractionPrefRecord {
            kind: kind.to_string(),
            decision: existing.decision,
            backoff_count: new_count,
            backoff_until,
            last_shown_at: existing.last_shown_at,
            updated_at: now,
        });
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

    fn map_pref(r: InteractionPrefRecord) -> InteractionPref {
        InteractionPref {
            kind: r.kind,
            decision: r.decision,
            backoff_count: r.backoff_count,
            backoff_until: r.backoff_until,
            last_shown_at: r.last_shown_at,
            updated_at: r.updated_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Returns `true` during quiet hours using UTC hour as approximation.
fn is_quiet_hour(start: u8, end: u8) -> bool {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let now_hour = ((secs / 3600) % 24) as u8;
    // start=23, end=6: wraps midnight, so hour >=23 OR hour < 6
    if start > end {
        now_hour >= start || now_hour < end
    } else if start == end {
        // No quiet period
        false
    } else {
        now_hour >= start && now_hour < end
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        chars[..max_chars].iter().collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use hime_storage::Database;

    fn engine() -> ProactiveEngine {
        let db = Arc::new(Database::open_in_memory().unwrap());
        ProactiveEngine::new(db)
    }

    // Disable quiet hours by setting start==end (no quiet period)
    fn engine_no_quiet() -> ProactiveEngine {
        let e = engine();
        e.update_extended_settings(true, "seimbang", 0, 0, 30, None).unwrap();
        e
    }

    #[test]
    fn test_attention_mode_budget_limits() {
        assert_eq!(AttentionMode::Pendiam.colek_per_hour(), 1);
        assert_eq!(AttentionMode::Pendiam.tanya_per_hour(), 1);
        assert_eq!(AttentionMode::Seimbang.colek_per_hour(), 3);
        assert_eq!(AttentionMode::Seimbang.tanya_per_hour(), 3);
        assert_eq!(AttentionMode::Cerewet.colek_per_hour(), 6);
        assert_eq!(AttentionMode::Cerewet.tanya_per_hour(), 5);
    }

    #[test]
    fn test_attention_mode_parse() {
        assert_eq!("pendiam".parse::<AttentionMode>().unwrap(), AttentionMode::Pendiam);
        assert_eq!("seimbang".parse::<AttentionMode>().unwrap(), AttentionMode::Seimbang);
        assert_eq!("cerewet".parse::<AttentionMode>().unwrap(), AttentionMode::Cerewet);
        assert!("unknown".parse::<AttentionMode>().is_err());
    }

    #[test]
    fn test_default_settings() {
        let e = engine();
        let s = e.get_settings();
        assert!(s.is_enabled);
        assert_eq!(s.attention_mode, "seimbang");
        assert_eq!(s.quiet_hour_start, 23);
        assert_eq!(s.quiet_hour_end, 6);
        assert!(s.muted_until.is_none());
    }

    #[test]
    fn test_update_settings_roundtrip() {
        let e = engine();
        e.update_extended_settings(true, "pendiam", 22, 7, 30, None).unwrap();
        let s = e.get_settings();
        assert_eq!(s.attention_mode, "pendiam");
        assert_eq!(s.quiet_hour_start, 22);
        assert_eq!(s.quiet_hour_end, 7);
    }

    #[test]
    fn test_update_settings_invalid_mode() {
        let e = engine();
        assert!(e.update_extended_settings(true, "invalid", 23, 6, 30, None).is_err());
    }

    #[test]
    fn test_budget_colek_limit_in_pendiam_mode() {
        let e = engine_no_quiet();
        e.update_extended_settings(true, "pendiam", 0, 0, 30, None).unwrap();
        assert!(e.consume_budget(DisturbanceLevel::Colek));
        assert!(!e.consume_budget(DisturbanceLevel::Colek));
    }

    #[test]
    fn test_budget_tanya_seimbang_allows_3() {
        let e = engine_no_quiet();
        assert!(e.consume_budget(DisturbanceLevel::Tanya));
        assert!(e.consume_budget(DisturbanceLevel::Tanya));
        assert!(e.consume_budget(DisturbanceLevel::Tanya));
        assert!(!e.consume_budget(DisturbanceLevel::Tanya));
    }

    #[test]
    fn test_budget_penting_always_passes() {
        let e = engine_no_quiet();
        e.update_extended_settings(true, "pendiam", 0, 0, 30, None).unwrap();
        for _ in 0..10 {
            assert!(e.consume_budget(DisturbanceLevel::Penting));
        }
    }

    #[test]
    fn test_mute_blocks_colek_tanya_suggestions() {
        let e = engine();
        let future = now_secs() + 3600;
        e.db.set_proactive_settings(&hime_storage::ProactiveSettingsRecord {
            id: "default".to_string(),
            is_enabled: true,
            cooldown_minutes: 20,
            muted_until: Some(future),
            updated_at: now_secs(),
        }).unwrap();
        assert!(!e.gate_situation(&SituationKind::NewDownload));
        assert!(!e.gate_situation(&SituationKind::StorageLow));
    }

    #[test]
    fn test_never_rule_blocks_kind() {
        let e = engine_no_quiet();
        e.set_preference("storage_low", "never").unwrap();
        assert!(!e.gate_situation(&SituationKind::StorageLow));
    }

    #[test]
    fn test_preference_roundtrip() {
        let e = engine();
        e.set_preference("file_duplicate", "always").unwrap();
        let p = e.get_preference("file_duplicate").unwrap();
        assert_eq!(p.decision, "always");
        assert_eq!(p.backoff_count, 0);
    }

    #[test]
    fn test_preference_invalid_decision() {
        let e = engine();
        assert!(e.set_preference("file_duplicate", "maybe").is_err());
    }

    #[test]
    fn test_exponential_backoff_after_3_dismissals() {
        let e = engine_no_quiet();
        e.apply_dismissal_backoff("storage_low");
        e.apply_dismissal_backoff("storage_low");
        e.apply_dismissal_backoff("storage_low"); // 3rd → triggers backoff

        let p = e.get_preference("storage_low").unwrap();
        assert_eq!(p.backoff_count, 3);
        assert!(p.backoff_until.is_some());
        let bu = p.backoff_until.unwrap();
        let now = now_secs();
        assert!(bu > now);
        assert!(bu <= now + 3700); // ~1 hour
    }

    #[test]
    fn test_backoff_increases_exponentially() {
        let e = engine_no_quiet();
        // 6 dismissals → backoff_count=6, exp=3, 2^3=8 hrs
        for _ in 0..6 {
            e.apply_dismissal_backoff("storage_low");
        }
        let p = e.get_preference("storage_low").unwrap();
        let now = now_secs();
        let bu = p.backoff_until.unwrap();
        // Should be ~8 hours in the future
        assert!(bu > now + 7 * 3600);
        assert!(bu <= now + 9 * 3600);
    }

    #[test]
    fn test_reset_backoff() {
        let e = engine_no_quiet();
        for _ in 0..3 {
            e.apply_dismissal_backoff("storage_low");
        }
        e.reset_backoff("storage_low").unwrap();
        let p = e.get_preference("storage_low").unwrap();
        assert_eq!(p.backoff_count, 0);
        assert!(p.backoff_until.is_none());
    }

    #[test]
    fn test_catalog_1_file_duplicate() {
        let e = engine_no_quiet();
        let id = e.on_file_duplicate("report.pdf", "X:/Documents/report.pdf");
        assert!(id.is_some());
        let sugs = e.list_suggestions(Some("pending"), 10);
        assert!(sugs.iter().any(|s| s.category == "file_duplicate"));
    }

    #[test]
    fn test_catalog_3_storage_low() {
        let e = engine_no_quiet();
        let id = e.on_storage_low("C:", 8);
        assert!(id.is_some());
        let sugs = e.list_suggestions(None, 10);
        assert!(sugs.iter().any(|s| s.category == "storage_low"));
        // Check text constraint: title ≤ 40 chars
        let sug = sugs.iter().find(|s| s.category == "storage_low").unwrap();
        assert!(sug.title.chars().count() <= 40);
    }

    #[test]
    fn test_catalog_5_routine_start_penting_no_budget() {
        let e = engine_no_quiet();
        e.update_extended_settings(true, "pendiam", 0, 0, 30, None).unwrap();
        // Penting — should bypass budget even in Pendiam mode
        let id = e.on_routine_start("belajar", 30);
        assert!(id.is_some());
    }

    #[test]
    fn test_catalog_7_distraction_penting() {
        let e = engine_no_quiet();
        let id = e.on_distraction_over_limit("YouTube", 45);
        assert!(id.is_some());
        let sugs = e.list_suggestions(Some("pending"), 10);
        assert!(sugs.iter().any(|s| s.category == "distraction_over_limit"));
    }

    #[test]
    fn test_catalog_8_morning_briefing() {
        let e = engine_no_quiet();
        let id = e.on_morning_briefing(3, "Belajar 19:00");
        assert!(id.is_some());
        let sugs = e.list_suggestions(Some("pending"), 10);
        assert!(sugs.iter().any(|s| s.category == "morning_briefing"));
    }

    #[test]
    fn test_catalog_16_risky_always_passes_mute() {
        let e = engine();
        // Set global mute
        let future = now_secs() + 3600;
        e.db.set_proactive_settings(&hime_storage::ProactiveSettingsRecord {
            id: "default".to_string(),
            is_enabled: true,
            cooldown_minutes: 20,
            muted_until: Some(future),
            updated_at: now_secs(),
        }).unwrap();
        // P0 risky action approval bypasses mute
        let id = e.on_risky_action_approval("move project files", 12);
        assert!(id.is_some());
    }

    #[test]
    fn test_respond_dismiss_increments_backoff() {
        let e = engine_no_quiet();
        let sug_id = e.on_storage_low("C:", 5).unwrap();
        e.respond_suggestion(&sug_id, "dismiss").unwrap();
        let pref = e.get_preference("storage_low").unwrap();
        assert_eq!(pref.backoff_count, 1);
    }

    #[test]
    fn test_global_disable_blocks_all() {
        let e = engine();
        e.update_extended_settings(false, "cerewet", 0, 0, 30, None).unwrap();
        assert!(e.on_storage_low("C:", 5).is_none());
        assert!(e.on_new_download("file.zip").is_none());
        assert!(e.on_routine_start("study", 30).is_none());
    }

    #[test]
    fn test_title_truncation_max_40() {
        let long = "A".repeat(100);
        let result = truncate(&long, 40);
        assert_eq!(result.chars().count(), 40);
    }

    #[test]
    fn test_all_22_situations_fire() {
        // Test that each of the 22 catalog situations successfully produces a suggestion
        // when within budget and not gated
        let check = |f: Box<dyn FnOnce(&ProactiveEngine) -> bool>| {
            let e = engine_no_quiet();
            f(&e)
        };

        let results = vec![
            check(Box::new(|e| e.on_file_duplicate("a.pdf", "b/a.pdf").is_some())),
            check(Box::new(|e| e.on_new_download("file.zip").is_some())),
            check(Box::new(|e| e.on_storage_low("C:", 5).is_some())),
            check(Box::new(|e| e.on_deadline_near("SKPL", "besok 19:00").is_some())),
            check(Box::new(|e| e.on_routine_start("belajar", 30).is_some())),
            check(Box::new(|e| e.on_routine_missed("belajar").is_some())),
            check(Box::new(|e| e.on_distraction_over_limit("YouTube", 45).is_some())),
            check(Box::new(|e| e.on_morning_briefing(3, "Belajar 19:00").is_some())),
            check(Box::new(|e| e.on_evening_recap(2, 1).is_some())),
            check(Box::new(|e| e.on_task_completed("Skripsi Bab 4").is_some())),
            check(Box::new(|e| e.on_idle_during_study().is_some())),
            check(Box::new(|e| e.on_foreign_text("Hello world").is_some())),
            check(Box::new(|e| e.on_battery_low(15).is_some())),
            check(Box::new(|e| e.on_device_connected("Samsung S24", None).is_some())),
            check(Box::new(|e| e.on_agent_task_done("Code Review").is_some())),
            check(Box::new(|e| e.on_risky_action_approval("move files", 5).is_some())),
            check(Box::new(|e| e.on_runtime_offline().is_some())),
            check(Box::new(|e| e.on_late_night().is_some())),
            check(Box::new(|e| e.on_welcome_back().is_some())),
            check(Box::new(|e| e.on_new_app_used("Discord").is_some())),
            check(Box::new(|e| e.on_clipboard_save("https://example.com").is_some())),
            check(Box::new(|e| e.on_update_or_error("Error 500 on sync").is_some())),
        ];
        let fired = results.iter().filter(|&&v| v).count();
        assert_eq!(fired, 22, "Expected all 22 situations to fire, got {}", fired);
    }

    #[test]
    fn test_list_preferences_excludes_sentinel_keys() {
        let e = engine_no_quiet();
        e.set_preference("file_duplicate", "always").unwrap();
        let prefs = e.list_preferences();
        // Sentinel keys (__attention_mode__ etc.) should not appear
        for p in &prefs {
            assert!(!p.kind.starts_with("__"), "Sentinel key leaked: {}", p.kind);
        }
        assert!(prefs.iter().any(|p| p.kind == "file_duplicate"));
    }

    #[test]
    fn test_periodic_reminder_registration_and_trigger() {
        let e = engine_no_quiet();
        let event_bus = crate::bus::EventBus::new(16);
        let mut rx = event_bus.subscribe();
        e.set_event_bus(event_bus);

        // Register custom 60s reminder
        let config = e.register_periodic_reminder(
            "rem-test-1",
            "Istirahat Belajar",
            "Waktunya istirahat sejenak",
            60,
            true,
        );
        assert_eq!(config.id, "rem-test-1");
        assert!(config.is_active);

        // Force elapsed time by setting last_triggered_at in past
        if let Ok(mut lock) = e.reminders.write() {
            if let Some(r) = lock.get_mut("rem-test-1") {
                r.last_triggered_at = (now_secs() as u64).saturating_sub(120);
            }
        }

        let triggered = e.check_and_trigger_periodic_reminders();
        assert!(!triggered.is_empty());
        assert!(triggered.iter().any(|t| t.title.contains("Istirahat")));

        // Verify EventBus message
        let event = rx.try_recv().expect("Event should be published");
        match event {
            crate::bus::SystemEvent::ProactiveReminderTriggered { reminder_id, title, .. } => {
                assert_eq!(reminder_id, "rem-test-1");
                assert_eq!(title, "Istirahat Belajar");
            }
            other => panic!("Unexpected event: {:?}", other),
        }
    }

    #[test]
    fn test_deadline_reminder_trigger() {
        let e = engine_no_quiet();
        let event_bus = crate::bus::EventBus::new(16);
        let mut rx = event_bus.subscribe();
        e.set_event_bus(event_bus);

        // Create a task due in 20 minutes
        let now = now_secs();
        let due = now + 20 * 60;
        let task = hime_storage::ProductivityTaskRecord {
            id: "task-deadline-test".to_string(),
            board_id: "board-default".to_string(),
            column_id: "col-todo".to_string(),
            title: "Pengumpulan Laporan Praktikum".to_string(),
            description: Some("Kirim ke portal".to_string()),
            priority: "high".to_string(),
            status: "todo".to_string(),
            due_date: Some(due),
            estimated_minutes: Some(30),
            actual_minutes: None,
            labels: None,
            parent_task_id: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };
        e.db.create_productivity_task(&task).unwrap();

        let triggered = e.check_and_trigger_deadline_reminders(30);
        assert!(!triggered.is_empty());
        assert!(triggered.iter().any(|t| t.title.contains("Pengumpulan Laporan")));

        // Verify EventBus message
        let event = rx.try_recv().expect("Deadline event should be published");
        match event {
            crate::bus::SystemEvent::ProactiveReminderTriggered { deadline_near, .. } => {
                assert!(deadline_near);
            }
            other => panic!("Unexpected event: {:?}", other),
        }
    }
}
