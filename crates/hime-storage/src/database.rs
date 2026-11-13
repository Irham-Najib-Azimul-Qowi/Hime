use rusqlite::{Connection, OpenFlags};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Mutex;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Storage migration failed: {0}")]
    MigrationFailed(String),
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Opens or creates an SQLite database with WAL mode and foreign key constraints enabled.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        Self::configure_pragmas(&conn)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    /// Opens an in-memory database for testing purposes.
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        Self::configure_pragmas(&conn)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn configure_pragmas(conn: &Connection) -> Result<(), StorageError> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )?;
        Ok(())
    }

    pub fn run_migrations(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        info!("Running core storage migrations");

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS system_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS audit_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                source TEXT NOT NULL,
                payload TEXT NOT NULL,
                prev_hash TEXT,
                hash TEXT
            );

            CREATE TABLE IF NOT EXISTS credentials (
                provider_id TEXT PRIMARY KEY,
                encrypted_secret BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                model TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                token_count INTEGER,
                FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                priority TEXT NOT NULL,
                status TEXT NOT NULL,
                progress REAL DEFAULT 0.0,
                retry_count INTEGER DEFAULT 0,
                max_retries INTEGER DEFAULT 3,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                completed_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS tool_executions (
                idempotency_key TEXT PRIMARY KEY,
                tool_id TEXT NOT NULL,
                args_hash TEXT NOT NULL,
                result TEXT NOT NULL,
                status TEXT NOT NULL,
                executed_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS memory_items (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                category TEXT NOT NULL,
                content TEXT NOT NULL,
                relevance_score REAL DEFAULT 1.0,
                retention_policy TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );",
        )?;

        // Ensure backward compatibility if audit_events already existed without hash columns
        let mut pragma_stmt = conn.prepare("PRAGMA table_info(audit_events)")?;
        let col_rows = pragma_stmt.query_map([], |row| row.get::<_, String>(1))?;
        let existing_cols: Vec<String> = col_rows.filter_map(Result::ok).collect();
        if !existing_cols.iter().any(|c| c == "prev_hash") {
            let _ = conn.execute("ALTER TABLE audit_events ADD COLUMN prev_hash TEXT", []);
        }
        if !existing_cols.iter().any(|c| c == "hash") {
            let _ = conn.execute("ALTER TABLE audit_events ADD COLUMN hash TEXT", []);
        }

        Ok(())
    }

    pub fn set_metadata(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO system_metadata (key, value, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3",
            rusqlite::params![key, value, now],
        )?;
        Ok(())
    }

    pub fn get_metadata(&self, key: &str) -> Result<Option<String>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM system_metadata WHERE key = ?1")?;
        let mut rows = stmt.query(rusqlite::params![key])?;

        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub fn record_audit_event(
        &self,
        event_type: &str,
        source: &str,
        payload: &str,
    ) -> Result<i64, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Fetch the hash of the latest event
        let prev_hash: String = {
            let mut stmt = conn.prepare("SELECT hash FROM audit_events ORDER BY id DESC LIMIT 1")?;
            let mut rows = stmt.query([])?;
            if let Some(row) = rows.next()? {
                let h: Option<String> = row.get(0)?;
                h.unwrap_or_else(|| "0000000000000000000000000000000000000000000000000000000000000000".to_string())
            } else {
                "0000000000000000000000000000000000000000000000000000000000000000".to_string()
            }
        };

        let mut hasher = Sha256::new();
        hasher.update(prev_hash.as_bytes());
        hasher.update(now.to_le_bytes());
        hasher.update(event_type.as_bytes());
        hasher.update(source.as_bytes());
        hasher.update(payload.as_bytes());
        let current_hash = format!("{:x}", hasher.finalize());

        conn.execute(
            "INSERT INTO audit_events (timestamp, event_type, source, payload, prev_hash, hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![now, event_type, source, payload, prev_hash, current_hash],
        )?;

        let id = conn.last_insert_rowid();
        Ok(id)
    }

    pub fn get_audit_events(&self, limit: usize) -> Result<Vec<AuditEventRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, source, payload, prev_hash, hash
             FROM audit_events
             ORDER BY id DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
            Ok(AuditEventRecord {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                event_type: row.get(2)?,
                source: row.get(3)?,
                payload: row.get(4)?,
                prev_hash: row.get(5)?,
                hash: row.get(6)?,
            })
        })?;

        let mut events = Vec::new();
        for r in rows {
            events.push(r?);
        }
        Ok(events)
    }

    pub fn verify_audit_trail(&self) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, source, payload, prev_hash, hash
             FROM audit_events
             ORDER BY id ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(AuditEventRecord {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                event_type: row.get(2)?,
                source: row.get(3)?,
                payload: row.get(4)?,
                prev_hash: row.get(5)?,
                hash: row.get(6)?,
            })
        })?;

        let mut expected_prev_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        for item in rows {
            let record = item?;
            let prev_hash = record.prev_hash.unwrap_or_default();
            let hash = record.hash.unwrap_or_default();

            if prev_hash != expected_prev_hash {
                return Ok(false);
            }

            let mut hasher = Sha256::new();
            hasher.update(prev_hash.as_bytes());
            hasher.update(record.timestamp.to_le_bytes());
            hasher.update(record.event_type.as_bytes());
            hasher.update(record.source.as_bytes());
            hasher.update(record.payload.as_bytes());
            let computed_hash = format!("{:x}", hasher.finalize());

            if computed_hash != hash {
                return Ok(false);
            }

            expected_prev_hash = hash;
        }

        Ok(true)
    }

    pub fn store_credential(
        &self,
        provider_id: &str,
        encrypted_secret: &[u8],
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO credentials (provider_id, encrypted_secret, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?3)
             ON CONFLICT(provider_id) DO UPDATE SET encrypted_secret = ?2, updated_at = ?3",
            rusqlite::params![provider_id, encrypted_secret, now],
        )?;
        Ok(())
    }

    pub fn get_credential(&self, provider_id: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT encrypted_secret FROM credentials WHERE provider_id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![provider_id])?;

        if let Some(row) = rows.next()? {
            let secret: Vec<u8> = row.get(0)?;
            Ok(Some(secret))
        } else {
            Ok(None)
        }
    }

    pub fn list_configured_providers(&self) -> Result<Vec<String>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT provider_id FROM credentials ORDER BY provider_id ASC")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut providers = Vec::new();
        for p in rows {
            providers.push(p?);
        }
        Ok(providers)
    }

    pub fn create_conversation(
        &self,
        id: &str,
        title: &str,
        model: &str,
    ) -> Result<ConversationRecord, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO conversations (id, title, model, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            rusqlite::params![id, title, model, now],
        )?;

        Ok(ConversationRecord {
            id: id.to_string(),
            title: title.to_string(),
            model: model.to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn list_conversations(&self) -> Result<Vec<ConversationRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, model, created_at, updated_at
             FROM conversations
             ORDER BY updated_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ConversationRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                model: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        let mut convs = Vec::new();
        for c in rows {
            convs.push(c?);
        }
        Ok(convs)
    }

    pub fn add_message(
        &self,
        id: &str,
        conversation_id: &str,
        role: &str,
        content: &str,
        token_count: Option<i64>,
    ) -> Result<MessageRecord, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at, token_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, conversation_id, role, content, now, token_count],
        )?;

        // Update conversation updated_at
        conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, conversation_id],
        )?;

        Ok(MessageRecord {
            id: id.to_string(),
            conversation_id: conversation_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            created_at: now,
            token_count,
        })
    }

    pub fn get_messages(&self, conversation_id: &str) -> Result<Vec<MessageRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, role, content, created_at, token_count
             FROM messages
             WHERE conversation_id = ?1
             ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map(rusqlite::params![conversation_id], |row| {
            Ok(MessageRecord {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                token_count: row.get(5)?,
            })
        })?;

        let mut msgs = Vec::new();
        for m in rows {
            msgs.push(m?);
        }
        Ok(msgs)
    }

    // ---------------------------------------------------------
    // Tasks Operations (Task Engine)
    // ---------------------------------------------------------

    pub fn create_task(
        &self,
        id: &str,
        title: &str,
        description: Option<&str>,
        priority: &str,
        status: &str,
    ) -> Result<TaskRecord, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO tasks (id, title, description, priority, status, progress, retry_count, max_retries, created_at, updated_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0.0, 0, 3, ?6, ?6, NULL)",
            rusqlite::params![id, title, description, priority, status, now],
        )?;

        Ok(TaskRecord {
            id: id.to_string(),
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            priority: priority.to_string(),
            status: status.to_string(),
            progress: 0.0,
            retry_count: 0,
            max_retries: 3,
            created_at: now,
            updated_at: now,
            completed_at: None,
        })
    }

    pub fn get_task(&self, id: &str) -> Result<Option<TaskRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, priority, status, progress, retry_count, max_retries, created_at, updated_at, completed_at
             FROM tasks WHERE id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(TaskRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                status: row.get(4)?,
                progress: row.get(5)?,
                retry_count: row.get(6)?,
                max_retries: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                completed_at: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_tasks(&self, status_filter: Option<&str>) -> Result<Vec<TaskRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let query = if status_filter.is_some() {
            "SELECT id, title, description, priority, status, progress, retry_count, max_retries, created_at, updated_at, completed_at
             FROM tasks WHERE status = ?1 ORDER BY created_at DESC"
        } else {
            "SELECT id, title, description, priority, status, progress, retry_count, max_retries, created_at, updated_at, completed_at
             FROM tasks ORDER BY created_at DESC"
        };

        let mut stmt = conn.prepare(query)?;
        let mut tasks = Vec::new();
        if let Some(sf) = status_filter {
            let rows = stmt.query_map(rusqlite::params![sf], map_task_record)?;
            for t in rows {
                tasks.push(t?);
            }
        } else {
            let rows = stmt.query_map([], map_task_record)?;
            for t in rows {
                tasks.push(t?);
            }
        }
        Ok(tasks)
    }

    pub fn update_task_status(
        &self,
        id: &str,
        status: &str,
        progress: f64,
        is_completed: bool,
    ) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let completed_at = if is_completed { Some(now) } else { None };

        let rows_affected = conn.execute(
            "UPDATE tasks SET status = ?1, progress = ?2, updated_at = ?3, completed_at = ?4 WHERE id = ?5",
            rusqlite::params![status, progress, now, completed_at, id],
        )?;

        Ok(rows_affected > 0)
    }

    pub fn increment_task_retry(&self, id: &str) -> Result<i64, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "UPDATE tasks SET retry_count = retry_count + 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id],
        )?;

        let mut stmt = conn.prepare("SELECT retry_count FROM tasks WHERE id = ?1")?;
        let count: i64 = stmt.query_row(rusqlite::params![id], |row| row.get(0))?;
        Ok(count)
    }

    pub fn delete_task(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM tasks WHERE id = ?1", rusqlite::params![id])?;
        Ok(rows > 0)
    }

    // ---------------------------------------------------------
    // Tool Execution Operations (Idempotency)
    // ---------------------------------------------------------

    pub fn record_tool_execution(
        &self,
        idempotency_key: &str,
        tool_id: &str,
        args_hash: &str,
        result: &str,
        status: &str,
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO tool_executions (idempotency_key, tool_id, args_hash, result, status, executed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(idempotency_key) DO UPDATE SET result = ?4, status = ?5, executed_at = ?6",
            rusqlite::params![idempotency_key, tool_id, args_hash, result, status, now],
        )?;
        Ok(())
    }

    pub fn get_tool_execution(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<ToolExecutionRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT idempotency_key, tool_id, args_hash, result, status, executed_at
             FROM tool_executions WHERE idempotency_key = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![idempotency_key])?;

        if let Some(row) = rows.next()? {
            Ok(Some(ToolExecutionRecord {
                idempotency_key: row.get(0)?,
                tool_id: row.get(1)?,
                args_hash: row.get(2)?,
                result: row.get(3)?,
                status: row.get(4)?,
                executed_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    // ---------------------------------------------------------
    // Memory Operations (Context & Memory)
    // ---------------------------------------------------------

    pub fn insert_memory_item(
        &self,
        id: &str,
        source: &str,
        category: &str,
        content: &str,
        relevance_score: f64,
        retention_policy: &str,
    ) -> Result<MemoryRecord, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO memory_items (id, source, category, content, relevance_score, retention_policy, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
             ON CONFLICT(id) DO UPDATE SET content = ?4, relevance_score = ?5, retention_policy = ?6, updated_at = ?7",
            rusqlite::params![id, source, category, content, relevance_score, retention_policy, now],
        )?;

        Ok(MemoryRecord {
            id: id.to_string(),
            source: source.to_string(),
            category: category.to_string(),
            content: content.to_string(),
            relevance_score,
            retention_policy: retention_policy.to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_memory_item(&self, id: &str) -> Result<Option<MemoryRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source, category, content, relevance_score, retention_policy, created_at, updated_at
             FROM memory_items WHERE id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(MemoryRecord {
                id: row.get(0)?,
                source: row.get(1)?,
                category: row.get(2)?,
                content: row.get(3)?,
                relevance_score: row.get(4)?,
                retention_policy: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_memory_items(
        &self,
        category_filter: Option<&str>,
    ) -> Result<Vec<MemoryRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let query = if category_filter.is_some() {
            "SELECT id, source, category, content, relevance_score, retention_policy, created_at, updated_at
             FROM memory_items WHERE category = ?1 ORDER BY created_at DESC"
        } else {
            "SELECT id, source, category, content, relevance_score, retention_policy, created_at, updated_at
             FROM memory_items ORDER BY created_at DESC"
        };

        let mut stmt = conn.prepare(query)?;
        let mut items = Vec::new();
        if let Some(cat) = category_filter {
            let rows = stmt.query_map(rusqlite::params![cat], map_memory_record)?;
            for item in rows {
                items.push(item?);
            }
        } else {
            let rows = stmt.query_map([], map_memory_record)?;
            for item in rows {
                items.push(item?);
            }
        }
        Ok(items)
    }

    pub fn search_memory_items(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, source, category, content, relevance_score, retention_policy, created_at, updated_at
             FROM memory_items
             WHERE content LIKE ?1 OR category LIKE ?1
             ORDER BY relevance_score DESC, updated_at DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(rusqlite::params![pattern, limit as i64], |row| {
            Ok(MemoryRecord {
                id: row.get(0)?,
                source: row.get(1)?,
                category: row.get(2)?,
                content: row.get(3)?,
                relevance_score: row.get(4)?,
                retention_policy: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn delete_memory_item(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM memory_items WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(rows > 0)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ConversationRecord {
    pub id: String,
    pub title: String,
    pub model: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct MessageRecord {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
    pub token_count: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AuditEventRecord {
    pub id: i64,
    pub timestamp: i64,
    pub event_type: String,
    pub source: String,
    pub payload: String,
    pub prev_hash: Option<String>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TaskRecord {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub status: String,
    pub progress: f64,
    pub retry_count: i64,
    pub max_retries: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ToolExecutionRecord {
    pub idempotency_key: String,
    pub tool_id: String,
    pub args_hash: String,
    pub result: String,
    pub status: String,
    pub executed_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MemoryRecord {
    pub id: String,
    pub source: String,
    pub category: String,
    pub content: String,
    pub relevance_score: f64,
    pub retention_policy: String,
    pub created_at: i64,
    pub updated_at: i64,
}

fn map_task_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskRecord> {
    Ok(TaskRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        priority: row.get(3)?,
        status: row.get(4)?,
        progress: row.get(5)?,
        retry_count: row.get(6)?,
        max_retries: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        completed_at: row.get(10)?,
    })
}

fn map_memory_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryRecord> {
    Ok(MemoryRecord {
        id: row.get(0)?,
        source: row.get(1)?,
        category: row.get(2)?,
        content: row.get(3)?,
        relevance_score: row.get(4)?,
        retention_policy: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_metadata_roundtrip() {
        let db = Database::open_in_memory().unwrap();
        db.set_metadata("hime.version", "0.1.0").unwrap();

        let val = db.get_metadata("hime.version").unwrap();
        assert_eq!(val.as_deref(), Some("0.1.0"));

        let nonexistent = db.get_metadata("missing.key").unwrap();
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_storage_audit_events() {
        let db = Database::open_in_memory().unwrap();
        let id1 = db
            .record_audit_event("system.startup", "core", "{\"status\":\"ok\"}")
            .unwrap();
        let id2 = db
            .record_audit_event("tool.execute", "desktop", "{\"tool\":\"read_file\"}")
            .unwrap();

        assert!(id2 > id1);

        let events = db.get_audit_events(10).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, "tool.execute");
        assert_eq!(events[1].event_type, "system.startup");
    }

    #[test]
    fn test_storage_credentials() {
        let db = Database::open_in_memory().unwrap();
        let encrypted_secret = vec![0xDE, 0xAD, 0xBE, 0xEF];
        db.store_credential("openai", &encrypted_secret).unwrap();

        let retrieved = db.get_credential("openai").unwrap();
        assert_eq!(retrieved, Some(encrypted_secret));

        let providers = db.list_configured_providers().unwrap();
        assert_eq!(providers, vec!["openai"]);

        let missing = db.get_credential("anthropic").unwrap();
        assert_eq!(missing, None);
    }

    #[test]
    fn test_storage_conversations_and_messages() {
        let db = Database::open_in_memory().unwrap();
        let conv = db.create_conversation("conv-1", "Test Chat", "hermes-3").unwrap();
        assert_eq!(conv.id, "conv-1");
        assert_eq!(conv.title, "Test Chat");

        let _m1 = db.add_message("msg-1", "conv-1", "user", "Hello Hermes", Some(4)).unwrap();
        let _m2 = db.add_message("msg-2", "conv-1", "assistant", "Hello! How can I assist?", Some(8)).unwrap();

        let messages = db.get_messages("conv-1").unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, "msg-1");
        assert_eq!(messages[1].id, "msg-2");

        let convs = db.list_conversations().unwrap();
        assert_eq!(convs.len(), 1);
        assert_eq!(convs[0].id, "conv-1");
    }

    #[test]
    fn test_storage_audit_hash_chain_verification_and_tamper_detection() {
        let db = Database::open_in_memory().unwrap();

        // 1. Record 3 chained audit events
        db.record_audit_event("system.startup", "core", "{}").unwrap();
        db.record_audit_event("user.login", "desktop", "{\"user\":\"satoshi\"}").unwrap();
        db.record_audit_event("tool.execute", "hermes", "{\"tool\":\"files.stat\"}").unwrap();

        // 2. Verification must succeed
        assert!(db.verify_audit_trail().unwrap());

        // 3. Tamper with a record directly in SQLite
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "UPDATE audit_events SET payload = '{\"hacked\":true}' WHERE id = 2",
                [],
            ).unwrap();
        }

        // 4. Verification must detect the tampering and fail
        assert!(!db.verify_audit_trail().unwrap());
    }

    #[test]
    fn test_storage_task_crud_and_status() {
        let db = Database::open_in_memory().unwrap();

        // Create
        let task = db
            .create_task("task-101", "Compile Rust Crates", Some("Run cargo build"), "high", "queued")
            .unwrap();
        assert_eq!(task.id, "task-101");
        assert_eq!(task.status, "queued");

        // Update status to running with 50% progress
        let ok = db.update_task_status("task-101", "running", 0.5, false).unwrap();
        assert!(ok);

        let fetched = db.get_task("task-101").unwrap().expect("task found");
        assert_eq!(fetched.status, "running");
        assert_eq!(fetched.progress, 0.5);
        assert!(fetched.completed_at.is_none());

        // Increment retry
        let retries = db.increment_task_retry("task-101").unwrap();
        assert_eq!(retries, 1);

        // Update to succeeded
        let ok = db.update_task_status("task-101", "succeeded", 1.0, true).unwrap();
        assert!(ok);

        let completed = db.get_task("task-101").unwrap().unwrap();
        assert_eq!(completed.status, "succeeded");
        assert!(completed.completed_at.is_some());

        // List tasks
        let all_tasks = db.list_tasks(None).unwrap();
        assert_eq!(all_tasks.len(), 1);

        let succeeded_tasks = db.list_tasks(Some("succeeded")).unwrap();
        assert_eq!(succeeded_tasks.len(), 1);

        let queued_tasks = db.list_tasks(Some("queued")).unwrap();
        assert_eq!(queued_tasks.len(), 0);

        // Delete
        assert!(db.delete_task("task-101").unwrap());
        assert!(db.get_task("task-101").unwrap().is_none());
    }

    #[test]
    fn test_storage_tool_execution_idempotency() {
        let db = Database::open_in_memory().unwrap();

        let idempotency_key = "req-idemp-001";
        let tool_id = "files.stat";
        let args_hash = "sha256-hash-of-path";

        // Check non-existent
        assert!(db.get_tool_execution(idempotency_key).unwrap().is_none());

        // Record execution
        db.record_tool_execution(
            idempotency_key,
            tool_id,
            args_hash,
            "{\"size\": 1024}",
            "completed",
        ).unwrap();

        // Retrieve recorded execution
        let exec = db.get_tool_execution(idempotency_key).unwrap().expect("found execution");
        assert_eq!(exec.idempotency_key, idempotency_key);
        assert_eq!(exec.tool_id, tool_id);
        assert_eq!(exec.result, "{\"size\": 1024}");
        assert_eq!(exec.status, "completed");

        // Repeated insert updates/preserves idempotency entry cleanly
        db.record_tool_execution(
            idempotency_key,
            tool_id,
            args_hash,
            "{\"size\": 1024, \"cached\": true}",
            "completed",
        ).unwrap();

        let updated = db.get_tool_execution(idempotency_key).unwrap().unwrap();
        assert_eq!(updated.result, "{\"size\": 1024, \"cached\": true}");
    }

    #[test]
    fn test_storage_memory_items_search_and_delete() {
        let db = Database::open_in_memory().unwrap();

        db.insert_memory_item(
            "mem-1",
            "conversation",
            "user_preference",
            "User prefers concise answers in Indonesian language",
            0.95,
            "permanent",
        ).unwrap();

        db.insert_memory_item(
            "mem-2",
            "task_engine",
            "project_context",
            "Project Hime uses Tauri 2.0 with Rust and React",
            0.90,
            "permanent",
        ).unwrap();

        // Search by keyword
        let results = db.search_memory_items("Indonesian", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "mem-1");

        let project_results = db.search_memory_items("Tauri", 10).unwrap();
        assert_eq!(project_results.len(), 1);
        assert_eq!(project_results[0].id, "mem-2");

        // List by category
        let prefs = db.list_memory_items(Some("user_preference")).unwrap();
        assert_eq!(prefs.len(), 1);
        assert_eq!(prefs[0].id, "mem-1");

        // Delete
        assert!(db.delete_memory_item("mem-1").unwrap());
        assert!(db.get_memory_item("mem-1").unwrap().is_none());
    }
}
