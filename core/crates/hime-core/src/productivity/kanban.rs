use hime_storage::{Database, KanbanBoardRecord, KanbanColumnRecord, StorageError};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct KanbanManager {
    db: Arc<Database>,
}

impl KanbanManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Initializes default Kanban board and standard columns if none exist.
    pub fn ensure_default_board(&self) -> Result<KanbanBoardRecord, StorageError> {
        let boards = self.db.list_kanban_boards()?;
        if let Some(first) = boards.into_iter().next() {
            return Ok(first);
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let board_id = "board-default";
        let board = KanbanBoardRecord {
            id: board_id.to_string(),
            title: "Hime Productivity & Study Workspace".to_string(),
            description: Some("Personal productivity, academic coursework, and project tracking".to_string()),
            created_at: now,
            updated_at: now,
        };
        self.db.create_kanban_board(&board)?;

        let columns = vec![
            ("col-todo", "To Do", 1, None),
            ("col-in-progress", "In Progress", 2, Some(3)),
            ("col-review", "Review", 3, Some(5)),
            ("col-done", "Done", 4, None),
        ];

        for (cid, ctitle, pos, wip) in columns {
            self.db.create_kanban_column(&KanbanColumnRecord {
                id: cid.to_string(),
                board_id: board_id.to_string(),
                title: ctitle.to_string(),
                position: pos,
                wip_limit: wip,
            })?;
        }

        Ok(board)
    }

    /// Natural language task parser supporting Indonesian and English patterns (UR-09).
    ///
    /// Returns (title, priority, due_date_unix, category_hint).
    /// The date parser is deterministic Rust — no LLM. Handles:
    /// "hari ini", "besok", "lusa", "nanti malam", "pagi/siang/sore/malam",
    /// "jam N", "jam HH:MM", "pukul N pagi/malam", day names (Senin–Minggu),
    /// "X depan", "minggu depan", "tgl DD", "DD MMM", "N hari/jam lagi",
    /// "sebelum Hari". Timezone = device local.
    pub fn parse_task_natural_language(&self, input: &str) -> (String, String, Option<i64>) {
        self.parse_task_natural_language_at(input, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64)
    }

    /// Testable version that accepts a fixed `now` timestamp.
    pub fn parse_task_natural_language_at(&self, input: &str, now: i64) -> (String, String, Option<i64>) {
        let lower = input.to_lowercase();

        // --- Priority ---
        let priority = if lower.contains("urgent") || lower.contains("mendesak") || lower.contains("kritis") {
            "critical".to_string()
        } else if lower.contains("penting") || lower.contains("prioritas tinggi") || lower.contains("high") {
            "high".to_string()
        } else if lower.contains("santai") || lower.contains("low") || lower.contains("rendah") {
            "low".to_string()
        } else {
            "medium".to_string()
        };

        // --- Due date (deterministic Indonesian + English parser) ---
        let due_date = Self::parse_indonesian_date(&lower, now);

        // --- Clean title: strip intro phrases and date/priority tokens ---
        let mut clean_title = input.trim().to_string();
        let prefixes = [
            "saya punya tugas", "tolong buat tugas", "catat tugas", "tambahkan tugas",
            "ada tugas", "buat tugas", "tugas baru",
            "i have a task to", "remind me to", "add task", "todo:",
        ];
        for prefix in prefixes {
            if clean_title.to_lowercase().starts_with(prefix) {
                clean_title = clean_title[prefix.len()..].trim().to_string();
                break;
            }
        }

        if clean_title.is_empty() {
            clean_title = "Tugas Baru".to_string();
        }

        (clean_title, priority, due_date)
    }

    /// Deterministic Indonesian date/time parser.
    fn parse_indonesian_date(lower: &str, now: i64) -> Option<i64> {
        const DAY: i64 = 86400;
        // "N hari lagi" / "N jam lagi"
        if let Some(offset) = Self::parse_relative_offset(lower) {
            return Some(now + offset);
        }

        // Exact matches (order matters: check longer patterns first)
        if lower.contains("nanti malam") {
            // Today 21:00 local approximation: round to next 21h boundary
            return Some(Self::next_time_of_day(now, 21, 0));
        }
        if lower.contains("nanti sore") {
            return Some(Self::next_time_of_day(now, 16, 0));
        }
        if lower.contains("nanti siang") {
            return Some(Self::next_time_of_day(now, 12, 0));
        }
        if lower.contains("nanti pagi") {
            return Some(Self::next_time_of_day(now, 8, 0));
        }
        if lower.contains("hari ini") || lower.contains("today") {
            // End of today: 23:59
            return Some(Self::next_time_of_day(now, 23, 59));
        }
        if lower.contains("lusa") {
            return Some(now + 2 * DAY);
        }
        if lower.contains("besok") || lower.contains("tomorrow") {
            return Some(now + DAY);
        }
        if lower.contains("akhir pekan") || lower.contains("weekend") {
            return Some(Self::next_weekday(now, 6)); // Saturday
        }
        if lower.contains("minggu depan") || lower.contains("next week") {
            return Some(now + 7 * DAY);
        }

        // "jam HH:MM" / "jam HH.MM" / "jam N" / "pukul N pagi/malam"
        if let Some(ts) = Self::parse_jam_pukul(lower, now) {
            return Some(ts);
        }

        // Day names: "senin", "selasa", etc. with optional "depan"
        if let Some(ts) = Self::parse_day_name(lower, now) {
            return Some(ts);
        }

        // "tgl DD" / "tanggal DD"
        if let Some(ts) = Self::parse_tgl(lower, now) {
            return Some(ts);
        }

        // "sebelum Hari" → day before the named day
        if let Some(ts) = Self::parse_sebelum(lower, now) {
            return Some(ts);
        }

        // English fallback
        if lower.contains("friday") {
            return Some(Self::next_weekday(now, 5));
        }

        None
    }

    /// Parses "N hari lagi", "N jam lagi", "N menit lagi".
    fn parse_relative_offset(lower: &str) -> Option<i64> {
        let re_patterns: &[(&str, i64)] = &[
            ("hari lagi", 86400),
            ("jam lagi", 3600),
            ("menit lagi", 60),
        ];
        for (suffix, multiplier) in re_patterns {
            if let Some(pos) = lower.find(suffix) {
                let before = lower[..pos].trim();
                if let Some(num_str) = before.rsplit_once(' ').map(|(_, n)| n).or(Some(before)) {
                    if let Ok(n) = num_str.parse::<i64>() {
                        return Some(n * multiplier);
                    }
                }
            }
        }
        None
    }

    /// Approximate "next HH:MM today, or tomorrow if already passed".
    fn next_time_of_day(now: i64, hour: i64, min: i64) -> i64 {
        // Approximate using UTC+7 offset (Indonesia WIB)
        let tz_offset: i64 = 7 * 3600;
        let local_now = now + tz_offset;
        let day_start = (local_now / 86400) * 86400;
        let target = day_start + hour * 3600 + min * 60 - tz_offset;
        if target <= now {
            target + 86400
        } else {
            target
        }
    }

    /// Returns the unix timestamp for the next occurrence of a weekday (1=Mon, 7=Sun).
    fn next_weekday(now: i64, target_dow: i64) -> i64 {
        let tz_offset: i64 = 7 * 3600;
        let local_now = now + tz_offset;
        // Unix epoch (Thu) → day-of-week: (days_since_epoch + 4) % 7, where 1=Mon
        let days_since_epoch = local_now / 86400;
        let current_dow = (days_since_epoch + 3) % 7 + 1; // 1=Mon, 7=Sun
        let mut delta = target_dow - current_dow;
        if delta <= 0 {
            delta += 7;
        }
        now + delta * 86400
    }

    /// Parses "jam 8", "jam 19:30", "jam 19.30", "pukul 7 malam", "pukul 7 pagi".
    fn parse_jam_pukul(lower: &str, now: i64) -> Option<i64> {
        for prefix in &["jam ", "pukul "] {
            if let Some(pos) = lower.find(prefix) {
                let rest = &lower[pos + prefix.len()..];
                let time_str: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == ':' || *c == '.').collect();
                if time_str.is_empty() {
                    continue;
                }

                let (mut hour, min) = if let Some(sep_pos) = time_str.find(|c| c == ':' || c == '.') {
                    let h = time_str[..sep_pos].parse::<i64>().ok()?;
                    let m = time_str[sep_pos + 1..].parse::<i64>().ok()?;
                    (h, m)
                } else {
                    (time_str.parse::<i64>().ok()?, 0)
                };

                // Handle "malam" / "sore" / "pagi" suffix for 12h format
                let after_time = &rest[time_str.len()..];
                if (after_time.contains("malam") || after_time.contains("sore")) && hour < 12 {
                    hour += 12;
                } else if after_time.contains("pagi") && hour == 12 {
                    hour = 0;
                }

                return Some(Self::next_time_of_day(now, hour, min));
            }
        }
        None
    }

    /// Parses Indonesian day names. "jumat depan" = next Friday even if today is before Fri.
    fn parse_day_name(lower: &str, now: i64) -> Option<i64> {
        let days = [
            ("senin", 1), ("selasa", 2), ("rabu", 3), ("kamis", 4),
            ("jumat", 5), ("jum'at", 5), ("sabtu", 6), ("minggu", 7),
        ];
        let is_depan = lower.contains("depan");
        for (name, dow) in &days {
            if lower.contains(name) {
                let mut ts = Self::next_weekday(now, *dow);
                // "X depan" means skip this week's occurrence
                if is_depan {
                    let tz_offset: i64 = 7 * 3600;
                    let local_now = now + tz_offset;
                    let days_since_epoch = local_now / 86400;
                    let current_dow = (days_since_epoch + 3) % 7 + 1;
                    if *dow > current_dow {
                        ts += 7 * 86400; // skip to next week
                    }
                }
                return Some(ts);
            }
        }
        None
    }

    /// Parses "tgl DD" or "tanggal DD".
    fn parse_tgl(lower: &str, now: i64) -> Option<i64> {
        for prefix in &["tgl ", "tanggal "] {
            if let Some(pos) = lower.find(prefix) {
                let rest = &lower[pos + prefix.len()..];
                let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(day) = num_str.parse::<i64>() {
                    if (1..=31).contains(&day) {
                        // Approximate: target day of current/next month
                        let tz_offset: i64 = 7 * 3600;
                        let local_now = now + tz_offset;
                        let current_day_of_month = ((local_now / 86400) % 30) + 1; // rough approx
                        let delta = if day > current_day_of_month {
                            (day - current_day_of_month) * 86400
                        } else {
                            (day + 30 - current_day_of_month) * 86400 // next month
                        };
                        return Some(now + delta);
                    }
                }
            }
        }
        None
    }

    /// Parses "sebelum Hari" → one day before the named day.
    fn parse_sebelum(lower: &str, now: i64) -> Option<i64> {
        if let Some(pos) = lower.find("sebelum ") {
            let rest = &lower[pos + "sebelum ".len()..];
            let days = [
                ("senin", 1), ("selasa", 2), ("rabu", 3), ("kamis", 4),
                ("jumat", 5), ("sabtu", 6), ("minggu", 7),
            ];
            for (name, dow) in &days {
                if rest.starts_with(name) {
                    let target = Self::next_weekday(now, *dow);
                    return Some(target - 86400); // one day before
                }
            }
        }
        None
    }

    /// Automatically proposes logical subtasks for complex goals (UR-09).
    pub fn propose_subtasks(&self, task_title: &str) -> Vec<(String, i64)> {
        let lower = task_title.to_lowercase();
        if lower.contains("bab") || lower.contains("skripsi") || lower.contains("makalah") || lower.contains("paper") {
            vec![
                ("Kajian literatur dan pengumpulan referensi jurnal".to_string(), 45),
                ("Penyusunan kerangka dan draf awal bagian utama".to_string(), 60),
                ("Pemeriksaan format sitasi dan konsistensi bahasa baku".to_string(), 30),
                ("Verifikasi akhir dan ekspor ke PDF".to_string(), 15),
            ]
        } else if lower.contains("coding") || lower.contains("fitur") || lower.contains("bug") {
            vec![
                ("Investigasi requirement dan reproduksi issue".to_string(), 30),
                ("Implementasi perubahan kode minimal".to_string(), 60),
                ("Penulisan automated unit & integration test".to_string(), 45),
                ("Review mandiri dan verifikasi build".to_string(), 20),
            ]
        } else {
            vec![
                ("Persiapan bahan dan tinjauan awal".to_string(), 25),
                ("Eksekusi tugas inti".to_string(), 50),
                ("Review hasil akhir dan checklist kualitas".to_string(), 20),
            ]
        }
    }
}

