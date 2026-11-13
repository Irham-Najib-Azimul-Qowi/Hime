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
    pub fn parse_task_natural_language(&self, input: &str) -> (String, String, Option<i64>) {
        let lower = input.to_lowercase();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

        // Determine priority
        let priority = if lower.contains("urgent") || lower.contains("mendesak") || lower.contains("kritis") {
            "critical".to_string()
        } else if lower.contains("penting") || lower.contains("prioritas tinggi") || lower.contains("high") {
            "high".to_string()
        } else if lower.contains("santai") || lower.contains("low") {
            "low".to_string()
        } else {
            "medium".to_string()
        };

        // Determine due date offset
        let due_date = if lower.contains("besok") || lower.contains("tomorrow") {
            Some(now + 86400)
        } else if lower.contains("jumat") || lower.contains("friday") {
            Some(now + (4 * 86400))
        } else if lower.contains("minggu depan") || lower.contains("next week") {
            Some(now + (7 * 86400))
        } else if lower.contains("lusa") {
            Some(now + (2 * 86400))
        } else {
            None
        };

        // Extract clean task title by stripping introductory phrases
        let mut clean_title = input.trim().to_string();
        let prefixes = [
            "saya punya tugas", "tolong buat tugas", "catat tugas", "tambahkan tugas",
            "i have a task to", "remind me to", "add task", "todo:"
        ];
        for prefix in prefixes {
            if clean_title.to_lowercase().starts_with(prefix) {
                clean_title = clean_title[prefix.len()..].trim().to_string();
                break;
            }
        }

        if clean_title.is_empty() {
            clean_title = "New Task".to_string();
        }

        (clean_title, priority, due_date)
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

