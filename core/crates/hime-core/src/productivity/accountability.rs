use hime_storage::{AccountabilitySettingsRecord, Database, ScreenTimeSessionRecord, StorageError};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckinGreeting {
    pub mode: String,
    pub title: String,
    pub message: String,
    pub priority_tasks_count: usize,
    pub suggested_focus_minutes: u32,
    pub can_snooze: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NudgeRecommendation {
    pub triggered: bool,
    pub distracting_minutes: i64,
    pub message: String,
    pub actionable_task_id: Option<String>,
}

pub struct AccountabilityManager {
    db: Arc<Database>,
}

impl AccountabilityManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn get_settings(&self) -> Result<AccountabilitySettingsRecord, StorageError> {
        self.db.get_accountability_settings()
    }

    pub fn set_mode(&self, new_mode: &str) -> Result<AccountabilitySettingsRecord, StorageError> {
        let valid_mode = match new_mode.to_lowercase().as_str() {
            "strict" => "strict",
            "relaxed" | "santai" => "relaxed",
            _ => "balanced",
        };

        let mut settings = self.db.get_accountability_settings()?;
        settings.mode = valid_mode.to_string();
        self.db.set_accountability_settings(&settings)?;
        Ok(settings)
    }

    pub fn set_screen_time_opt_in(&self, opt_in: bool) -> Result<AccountabilitySettingsRecord, StorageError> {
        let mut settings = self.db.get_accountability_settings()?;
        settings.screen_time_opt_in = opt_in;
        self.db.set_accountability_settings(&settings)?;
        Ok(settings)
    }

    /// Evaluates startup greeting based on current accountability mode.
    pub fn generate_startup_checkin(&self, pending_critical_count: usize) -> Result<CheckinGreeting, StorageError> {
        let settings = self.db.get_accountability_settings()?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

        // Record check-in timestamp
        let mut updated_settings = settings.clone();
        updated_settings.last_checkin_at = Some(now);
        let _ = self.db.set_accountability_settings(&updated_settings);

        match settings.mode.as_str() {
            "strict" => Ok(CheckinGreeting {
                mode: "strict".to_string(),
                title: "Selamat Datang — Mode Ketegasan Aktif".to_string(),
                message: format!(
                    "Laptop dibuka. Anda memiliki {} tugas berprioritas tinggi. Mari mulai sesi fokus 25 menit sekarang untuk menyelesaikan target hari ini.",
                    pending_critical_count
                ),
                priority_tasks_count: pending_critical_count,
                suggested_focus_minutes: 25,
                can_snooze: true, // Non-punitive: Snooze is always available with reflection
            }),
            "relaxed" => Ok(CheckinGreeting {
                mode: "relaxed".to_string(),
                title: "Selamat Datang di Hime".to_string(),
                message: "Selamat beraktivitas! Semua target berjalan santai sesuai ritme Anda.".to_string(),
                priority_tasks_count: pending_critical_count,
                suggested_focus_minutes: 0,
                can_snooze: true,
            }),
            _ => Ok(CheckinGreeting {
                mode: "balanced".to_string(),
                title: "Sesi Kerja Hime Dimulai".to_string(),
                message: format!(
                    "Ada {} tugas menanti perhatian Anda. Siap untuk produktif hari ini?",
                    pending_critical_count
                ),
                priority_tasks_count: pending_critical_count,
                suggested_focus_minutes: 20,
                can_snooze: true,
            }),
        }
    }

    /// Inspects recent screen time and triggers actionable nudges when appropriate (UR-11).
    pub fn evaluate_distraction_nudge(
        &self,
        recent_sessions: &[ScreenTimeSessionRecord],
        urgent_task_id: Option<String>,
    ) -> Result<NudgeRecommendation, StorageError> {
        let settings = self.db.get_accountability_settings()?;
        if !settings.screen_time_opt_in {
            return Ok(NudgeRecommendation {
                triggered: false,
                distracting_minutes: 0,
                message: "Screen time tracking is disabled (opt-out).".to_string(),
                actionable_task_id: None,
            });
        }

        let distracting_secs: i64 = recent_sessions
            .iter()
            .filter(|s| s.category == "distracting")
            .map(|s| s.duration_secs)
            .sum();

        let threshold_secs = match settings.mode.as_str() {
            "strict" => 600,    // 10 minutes threshold in strict mode
            "relaxed" => 2400,  // 40 minutes threshold in relaxed mode
            _ => 1200,          // 20 minutes in balanced mode
        };

        if distracting_secs >= threshold_secs {
            let mins = distracting_secs / 60;
            Ok(NudgeRecommendation {
                triggered: true,
                distracting_minutes: mins,
                message: format!(
                    "Anda telah menghabiskan {} menit pada aplikasi pengalih. Rekomendasi: Ambil napas sejenak, lalu mulai 25 menit fokus untuk melanjutkan progres.",
                    mins
                ),
                actionable_task_id: urgent_task_id,
            })
        } else {
            Ok(NudgeRecommendation {
                triggered: false,
                distracting_minutes: distracting_secs / 60,
                message: "Penggunaan layar masih dalam batas wajar.".to_string(),
                actionable_task_id: None,
            })
        }
    }
}

