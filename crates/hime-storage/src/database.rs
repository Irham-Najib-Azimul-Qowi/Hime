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
            );

            CREATE TABLE IF NOT EXISTS indexed_files (
                path TEXT PRIMARY KEY,
                parent_dir TEXT NOT NULL,
                file_name TEXT NOT NULL,
                extension TEXT,
                size_bytes INTEGER NOT NULL,
                modified_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                is_dir BOOLEAN NOT NULL,
                content_hash TEXT,
                mime_type TEXT,
                indexed_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_files_parent ON indexed_files(parent_dir);
            CREATE INDEX IF NOT EXISTS idx_files_size ON indexed_files(size_bytes);
            CREATE INDEX IF NOT EXISTS idx_files_ext ON indexed_files(extension);
            CREATE INDEX IF NOT EXISTS idx_files_hash ON indexed_files(content_hash);

            CREATE TABLE IF NOT EXISTS file_operations (
                id TEXT PRIMARY KEY,
                op_type TEXT NOT NULL,
                source_path TEXT NOT NULL,
                destination_path TEXT,
                backup_path TEXT,
                source_hash TEXT,
                dest_hash TEXT,
                actor TEXT NOT NULL,
                approval_id TEXT,
                status TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                completed_at INTEGER,
                error_message TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_file_ops_status ON file_operations(status);

            CREATE TABLE IF NOT EXISTS file_rules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                pattern TEXT NOT NULL,
                target_dir TEXT NOT NULL,
                naming_template TEXT,
                action TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS storage_backups (
                id TEXT PRIMARY KEY,
                source_root TEXT NOT NULL,
                backup_destination TEXT NOT NULL,
                file_count INTEGER NOT NULL,
                total_bytes INTEGER NOT NULL,
                manifest_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                verified_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS paired_devices (
                id TEXT PRIMARY KEY,
                device_name TEXT NOT NULL,
                device_type TEXT NOT NULL,
                cert_fingerprint TEXT NOT NULL,
                paired_at INTEGER NOT NULL,
                last_seen_at INTEGER NOT NULL,
                status TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_devices_status ON paired_devices(status);

            CREATE TABLE IF NOT EXISTS device_pairing_tokens (
                token TEXT PRIMARY KEY,
                confirmation_code TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS device_storage_summaries (
                device_id TEXT PRIMARY KEY,
                total_bytes INTEGER NOT NULL,
                free_bytes INTEGER NOT NULL,
                indexed_items_count INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS file_transfers (
                transfer_id TEXT PRIMARY KEY,
                device_id TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                bytes_transferred INTEGER NOT NULL,
                direction TEXT NOT NULL,
                status TEXT NOT NULL,
                sha256 TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                completed_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_transfers_device ON file_transfers(device_id);",
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

    // =========================================================================
    // Phase 10: Filesystem Indexing & Storage Intelligence
    // =========================================================================

    pub fn upsert_indexed_file(&self, file: &IndexedFileRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO indexed_files (
                path, parent_dir, file_name, extension, size_bytes,
                modified_at, created_at, is_dir, content_hash, mime_type, indexed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(path) DO UPDATE SET
                parent_dir = ?2,
                file_name = ?3,
                extension = ?4,
                size_bytes = ?5,
                modified_at = ?6,
                created_at = ?7,
                is_dir = ?8,
                content_hash = ?9,
                mime_type = ?10,
                indexed_at = ?11",
            rusqlite::params![
                file.path,
                file.parent_dir,
                file.file_name,
                file.extension,
                file.size_bytes,
                file.modified_at,
                file.created_at,
                file.is_dir,
                file.content_hash,
                file.mime_type,
                file.indexed_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_indexed_file(&self, path: &str) -> Result<Option<IndexedFileRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT path, parent_dir, file_name, extension, size_bytes, modified_at, created_at, is_dir, content_hash, mime_type, indexed_at
             FROM indexed_files WHERE path = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_indexed_file(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_indexed_file(&self, path: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM indexed_files WHERE path = ?1", rusqlite::params![path])?;
        Ok(rows > 0)
    }

    pub fn delete_indexed_files_under_dir(&self, dir_prefix: &str) -> Result<usize, StorageError> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("{}%", dir_prefix);
        let rows = conn.execute(
            "DELETE FROM indexed_files WHERE path LIKE ?1",
            rusqlite::params![pattern],
        )?;
        Ok(rows)
    }

    pub fn search_indexed_files(&self, query: &str, limit: usize) -> Result<Vec<IndexedFileRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT path, parent_dir, file_name, extension, size_bytes, modified_at, created_at, is_dir, content_hash, mime_type, indexed_at
             FROM indexed_files
             WHERE file_name LIKE ?1 OR path LIKE ?1
             ORDER BY modified_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![pattern, limit as i64], map_indexed_file)?;
        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn list_indexed_files_in_dir(&self, parent_dir: &str) -> Result<Vec<IndexedFileRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT path, parent_dir, file_name, extension, size_bytes, modified_at, created_at, is_dir, content_hash, mime_type, indexed_at
             FROM indexed_files
             WHERE parent_dir = ?1
             ORDER BY is_dir DESC, file_name ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![parent_dir], map_indexed_file)?;
        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn find_files_by_hash(&self, hash: &str) -> Result<Vec<IndexedFileRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT path, parent_dir, file_name, extension, size_bytes, modified_at, created_at, is_dir, content_hash, mime_type, indexed_at
             FROM indexed_files
             WHERE content_hash = ?1 AND is_dir = 0
             ORDER BY modified_at ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![hash], map_indexed_file)?;
        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn count_indexed_files(&self) -> Result<i64, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM indexed_files")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_storage_analysis(&self, top_n: usize) -> Result<StorageAnalysisReport, StorageError> {
        let conn = self.conn.lock().unwrap();
        let (total_files, total_bytes): (i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(size_bytes), 0) FROM indexed_files WHERE is_dir = 0",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let mut ext_stmt = conn.prepare(
            "SELECT COALESCE(extension, 'none'), COUNT(*), SUM(size_bytes)
             FROM indexed_files
             WHERE is_dir = 0
             GROUP BY extension
             ORDER BY SUM(size_bytes) DESC
             LIMIT 15",
        )?;
        let ext_rows = ext_stmt.query_map([], |row| {
            Ok(ExtensionBreakdown {
                extension: row.get(0)?,
                count: row.get(1)?,
                total_bytes: row.get(2)?,
            })
        })?;
        let mut by_extension = Vec::new();
        for r in ext_rows {
            by_extension.push(r?);
        }

        let mut large_stmt = conn.prepare(
            "SELECT path, parent_dir, file_name, extension, size_bytes, modified_at, created_at, is_dir, content_hash, mime_type, indexed_at
             FROM indexed_files
             WHERE is_dir = 0
             ORDER BY size_bytes DESC
             LIMIT ?1",
        )?;
        let large_rows = large_stmt.query_map(rusqlite::params![top_n as i64], map_indexed_file)?;
        let mut largest_files = Vec::new();
        for r in large_rows {
            largest_files.push(r?);
        }

        let thirty_days_ago = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64 - (30 * 86400);

        let mut old_stmt = conn.prepare(
            "SELECT path, parent_dir, file_name, extension, size_bytes, modified_at, created_at, is_dir, content_hash, mime_type, indexed_at
             FROM indexed_files
             WHERE is_dir = 0 AND (parent_dir LIKE '%download%' OR parent_dir LIKE '%Download%') AND modified_at < ?1
             ORDER BY modified_at ASC
             LIMIT ?2",
        )?;
        let old_rows = old_stmt.query_map(rusqlite::params![thirty_days_ago, top_n as i64], map_indexed_file)?;
        let mut old_downloads = Vec::new();
        for r in old_rows {
            old_downloads.push(r?);
        }

        Ok(StorageAnalysisReport {
            total_files,
            total_bytes,
            by_extension,
            largest_files,
            old_downloads,
        })
    }

    pub fn find_duplicate_candidates(&self) -> Result<Vec<DuplicateGroup>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT content_hash, size_bytes, COUNT(*) as cnt
             FROM indexed_files
             WHERE is_dir = 0 AND content_hash IS NOT NULL AND content_hash != ''
             GROUP BY content_hash, size_bytes
             HAVING cnt > 1
             ORDER BY size_bytes * cnt DESC
             LIMIT 50",
        )?;
        let group_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut duplicate_groups = Vec::new();
        for g in group_rows {
            let (hash, size) = g?;
            let mut file_stmt = conn.prepare(
                "SELECT path, parent_dir, file_name, extension, size_bytes, modified_at, created_at, is_dir, content_hash, mime_type, indexed_at
                 FROM indexed_files
                 WHERE content_hash = ?1
                 ORDER BY modified_at ASC",
            )?;
            let file_rows = file_stmt.query_map(rusqlite::params![hash], map_indexed_file)?;
            let mut files = Vec::new();
            for f in file_rows {
                files.push(f?);
            }
            duplicate_groups.push(DuplicateGroup {
                content_hash: hash,
                size_bytes: size,
                files,
            });
        }
        Ok(duplicate_groups)
    }

    // =========================================================================
    // Phase 10: Safe File Operations Journal
    // =========================================================================

    pub fn record_file_operation(&self, op: &FileOperationRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO file_operations (
                id, op_type, source_path, destination_path, backup_path,
                source_hash, dest_hash, actor, approval_id, status,
                started_at, completed_at, error_message
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
                status = ?10,
                dest_hash = ?7,
                completed_at = ?12,
                error_message = ?13",
            rusqlite::params![
                op.id,
                op.op_type,
                op.source_path,
                op.destination_path,
                op.backup_path,
                op.source_hash,
                op.dest_hash,
                op.actor,
                op.approval_id,
                op.status,
                op.started_at,
                op.completed_at,
                op.error_message,
            ],
        )?;
        Ok(())
    }

    pub fn update_file_operation_status(
        &self,
        id: &str,
        status: &str,
        dest_hash: Option<&str>,
        completed: bool,
        error: Option<&str>,
    ) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = if completed {
            Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            )
        } else {
            None
        };

        let rows = conn.execute(
            "UPDATE file_operations
             SET status = ?2, dest_hash = COALESCE(?3, dest_hash), completed_at = COALESCE(?4, completed_at), error_message = ?5
             WHERE id = ?1",
            rusqlite::params![id, status, dest_hash, now, error],
        )?;
        Ok(rows > 0)
    }

    pub fn get_file_operation(&self, id: &str) -> Result<Option<FileOperationRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, op_type, source_path, destination_path, backup_path,
                    source_hash, dest_hash, actor, approval_id, status,
                    started_at, completed_at, error_message
             FROM file_operations WHERE id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_file_operation(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_file_operations(&self, limit: usize) -> Result<Vec<FileOperationRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, op_type, source_path, destination_path, backup_path,
                    source_hash, dest_hash, actor, approval_id, status,
                    started_at, completed_at, error_message
             FROM file_operations
             ORDER BY started_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit as i64], map_file_operation)?;
        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn get_uncommitted_file_operations(&self) -> Result<Vec<FileOperationRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, op_type, source_path, destination_path, backup_path,
                    source_hash, dest_hash, actor, approval_id, status,
                    started_at, completed_at, error_message
             FROM file_operations
             WHERE status IN ('staged', 'in_progress')",
        )?;
        let rows = stmt.query_map([], map_file_operation)?;
        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    // =========================================================================
    // Phase 10: File Rules & Smart Organization
    // =========================================================================

    pub fn create_file_rule(&self, rule: &FileRuleRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO file_rules (
                id, name, pattern, target_dir, naming_template, action, is_active, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                name = ?2, pattern = ?3, target_dir = ?4, naming_template = ?5, action = ?6, is_active = ?7, updated_at = ?9",
            rusqlite::params![
                rule.id,
                rule.name,
                rule.pattern,
                rule.target_dir,
                rule.naming_template,
                rule.action,
                rule.is_active,
                rule.created_at,
                rule.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_file_rules(&self) -> Result<Vec<FileRuleRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, pattern, target_dir, naming_template, action, is_active, created_at, updated_at
             FROM file_rules
             ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([], map_file_rule)?;
        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn get_file_rule(&self, id: &str) -> Result<Option<FileRuleRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, pattern, target_dir, naming_template, action, is_active, created_at, updated_at
             FROM file_rules WHERE id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_file_rule(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn update_file_rule_active(&self, id: &str, is_active: bool) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let rows = conn.execute(
            "UPDATE file_rules SET is_active = ?2, updated_at = ?3 WHERE id = ?1",
            rusqlite::params![id, is_active, now],
        )?;
        Ok(rows > 0)
    }

    pub fn delete_file_rule(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM file_rules WHERE id = ?1", rusqlite::params![id])?;
        Ok(rows > 0)
    }

    // =========================================================================
    // Phase 10: Storage Backups & Integrity
    // =========================================================================

    pub fn create_backup_record(&self, backup: &StorageBackupRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO storage_backups (
                id, source_root, backup_destination, file_count, total_bytes, manifest_hash, created_at, verified_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                backup.id,
                backup.source_root,
                backup.backup_destination,
                backup.file_count,
                backup.total_bytes,
                backup.manifest_hash,
                backup.created_at,
                backup.verified_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_backup_records(&self) -> Result<Vec<StorageBackupRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_root, backup_destination, file_count, total_bytes, manifest_hash, created_at, verified_at
             FROM storage_backups
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], map_storage_backup)?;
        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn verify_backup_record(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let rows = conn.execute(
            "UPDATE storage_backups SET verified_at = ?2 WHERE id = ?1",
            rusqlite::params![id, now],
        )?;
        Ok(rows > 0)
    }

    // =========================================================================
    // Phase 11: Mobile & Device Layer
    // =========================================================================

    pub fn add_paired_device(&self, dev: &PairedDeviceRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO paired_devices (id, device_name, device_type, cert_fingerprint, paired_at, last_seen_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                device_name = ?2,
                device_type = ?3,
                cert_fingerprint = ?4,
                last_seen_at = ?6,
                status = ?7",
            rusqlite::params![
                dev.id,
                dev.device_name,
                dev.device_type,
                dev.cert_fingerprint,
                dev.paired_at,
                dev.last_seen_at,
                dev.status,
            ],
        )?;
        Ok(())
    }

    pub fn list_paired_devices(&self) -> Result<Vec<PairedDeviceRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, device_name, device_type, cert_fingerprint, paired_at, last_seen_at, status
             FROM paired_devices ORDER BY paired_at ASC",
        )?;
        let rows = stmt.query_map([], map_paired_device)?;
        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn get_paired_device(&self, id: &str) -> Result<Option<PairedDeviceRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, device_name, device_type, cert_fingerprint, paired_at, last_seen_at, status
             FROM paired_devices WHERE id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_paired_device(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn revoke_device(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE paired_devices SET status = 'revoked' WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(rows > 0)
    }

    pub fn update_device_last_seen(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let rows = conn.execute(
            "UPDATE paired_devices SET last_seen_at = ?2 WHERE id = ?1",
            rusqlite::params![id, now],
        )?;
        Ok(rows > 0)
    }

    pub fn create_pairing_token(
        &self,
        token: &str,
        confirmation_code: &str,
        ttl_secs: u64,
    ) -> Result<PairingTokenRecord, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expires_at = now + (ttl_secs as i64);

        let rec = PairingTokenRecord {
            token: token.to_string(),
            confirmation_code: confirmation_code.to_string(),
            expires_at,
            status: "pending".to_string(),
            created_at: now,
        };

        conn.execute(
            "INSERT INTO device_pairing_tokens (token, confirmation_code, expires_at, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                rec.token,
                rec.confirmation_code,
                rec.expires_at,
                rec.status,
                rec.created_at,
            ],
        )?;
        Ok(rec)
    }

    pub fn validate_and_consume_pairing_token(
        &self,
        token: &str,
        confirmation_code: &str,
    ) -> Result<TokenValidationResult, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut stmt = conn.prepare(
            "SELECT confirmation_code, expires_at, status
             FROM device_pairing_tokens WHERE token = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![token])?;

        if let Some(row) = rows.next()? {
            let code: String = row.get(0)?;
            let expires_at: i64 = row.get(1)?;
            let status: String = row.get(2)?;

            if status != "pending" || now > expires_at {
                Ok(TokenValidationResult::InvalidOrExpired)
            } else if code != confirmation_code {
                Ok(TokenValidationResult::CodeMismatch)
            } else {
                conn.execute(
                    "UPDATE device_pairing_tokens SET status = 'consumed' WHERE token = ?1",
                    rusqlite::params![token],
                )?;
                Ok(TokenValidationResult::Valid)
            }
        } else {
            Ok(TokenValidationResult::InvalidOrExpired)
        }
    }

    pub fn set_device_storage_summary(
        &self,
        summary: &DeviceStorageSummaryRecord,
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO device_storage_summaries (device_id, total_bytes, free_bytes, indexed_items_count, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(device_id) DO UPDATE SET
                total_bytes = ?2,
                free_bytes = ?3,
                indexed_items_count = ?4,
                updated_at = ?5",
            rusqlite::params![
                summary.device_id,
                summary.total_bytes,
                summary.free_bytes,
                summary.indexed_items_count,
                summary.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_device_storage_summary(
        &self,
        device_id: &str,
    ) -> Result<Option<DeviceStorageSummaryRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT device_id, total_bytes, free_bytes, indexed_items_count, updated_at
             FROM device_storage_summaries WHERE device_id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![device_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_device_storage_summary(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn record_file_transfer(&self, transfer: &FileTransferRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO file_transfers (transfer_id, device_id, file_name, file_size, bytes_transferred, direction, status, sha256, started_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(transfer_id) DO UPDATE SET
                bytes_transferred = ?5,
                status = ?7,
                completed_at = ?10",
            rusqlite::params![
                transfer.transfer_id,
                transfer.device_id,
                transfer.file_name,
                transfer.file_size,
                transfer.bytes_transferred,
                transfer.direction,
                transfer.status,
                transfer.sha256,
                transfer.started_at,
                transfer.completed_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_transfer_progress(
        &self,
        transfer_id: &str,
        bytes: i64,
        status: &str,
    ) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = if status == "completed" || status == "failed" || status == "cancelled" {
            Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            )
        } else {
            None
        };

        let rows = conn.execute(
            "UPDATE file_transfers
             SET bytes_transferred = ?2, status = ?3, completed_at = COALESCE(?4, completed_at)
             WHERE transfer_id = ?1",
            rusqlite::params![transfer_id, bytes, status, now],
        )?;
        Ok(rows > 0)
    }

    pub fn list_file_transfers(
        &self,
        device_id: Option<&str>,
    ) -> Result<Vec<FileTransferRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut items = Vec::new();

        if let Some(dev) = device_id {
            let mut stmt = conn.prepare(
                "SELECT transfer_id, device_id, file_name, file_size, bytes_transferred, direction, status, sha256, started_at, completed_at
                 FROM file_transfers WHERE device_id = ?1 ORDER BY started_at DESC",
            )?;
            let rows = stmt.query_map(rusqlite::params![dev], map_file_transfer)?;
            for item in rows {
                items.push(item?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT transfer_id, device_id, file_name, file_size, bytes_transferred, direction, status, sha256, started_at, completed_at
                 FROM file_transfers ORDER BY started_at DESC LIMIT 50",
            )?;
            let rows = stmt.query_map([], map_file_transfer)?;
            for item in rows {
                items.push(item?);
            }
        }

        Ok(items)
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct IndexedFileRecord {
    pub path: String,
    pub parent_dir: String,
    pub file_name: String,
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub modified_at: i64,
    pub created_at: i64,
    pub is_dir: bool,
    pub content_hash: Option<String>,
    pub mime_type: Option<String>,
    pub indexed_at: i64,
}

fn map_indexed_file(row: &rusqlite::Row<'_>) -> rusqlite::Result<IndexedFileRecord> {
    Ok(IndexedFileRecord {
        path: row.get(0)?,
        parent_dir: row.get(1)?,
        file_name: row.get(2)?,
        extension: row.get(3)?,
        size_bytes: row.get(4)?,
        modified_at: row.get(5)?,
        created_at: row.get(6)?,
        is_dir: row.get(7)?,
        content_hash: row.get(8)?,
        mime_type: row.get(9)?,
        indexed_at: row.get(10)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FileOperationRecord {
    pub id: String,
    pub op_type: String,
    pub source_path: String,
    pub destination_path: Option<String>,
    pub backup_path: Option<String>,
    pub source_hash: Option<String>,
    pub dest_hash: Option<String>,
    pub actor: String,
    pub approval_id: Option<String>,
    pub status: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub error_message: Option<String>,
}

fn map_file_operation(row: &rusqlite::Row<'_>) -> rusqlite::Result<FileOperationRecord> {
    Ok(FileOperationRecord {
        id: row.get(0)?,
        op_type: row.get(1)?,
        source_path: row.get(2)?,
        destination_path: row.get(3)?,
        backup_path: row.get(4)?,
        source_hash: row.get(5)?,
        dest_hash: row.get(6)?,
        actor: row.get(7)?,
        approval_id: row.get(8)?,
        status: row.get(9)?,
        started_at: row.get(10)?,
        completed_at: row.get(11)?,
        error_message: row.get(12)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FileRuleRecord {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub target_dir: String,
    pub naming_template: Option<String>,
    pub action: String,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

fn map_file_rule(row: &rusqlite::Row<'_>) -> rusqlite::Result<FileRuleRecord> {
    Ok(FileRuleRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        pattern: row.get(2)?,
        target_dir: row.get(3)?,
        naming_template: row.get(4)?,
        action: row.get(5)?,
        is_active: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct StorageBackupRecord {
    pub id: String,
    pub source_root: String,
    pub backup_destination: String,
    pub file_count: i64,
    pub total_bytes: i64,
    pub manifest_hash: String,
    pub created_at: i64,
    pub verified_at: Option<i64>,
}

fn map_storage_backup(row: &rusqlite::Row<'_>) -> rusqlite::Result<StorageBackupRecord> {
    Ok(StorageBackupRecord {
        id: row.get(0)?,
        source_root: row.get(1)?,
        backup_destination: row.get(2)?,
        file_count: row.get(3)?,
        total_bytes: row.get(4)?,
        manifest_hash: row.get(5)?,
        created_at: row.get(6)?,
        verified_at: row.get(7)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ExtensionBreakdown {
    pub extension: String,
    pub count: i64,
    pub total_bytes: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct StorageAnalysisReport {
    pub total_files: i64,
    pub total_bytes: i64,
    pub by_extension: Vec<ExtensionBreakdown>,
    pub largest_files: Vec<IndexedFileRecord>,
    pub old_downloads: Vec<IndexedFileRecord>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DuplicateGroup {
    pub content_hash: String,
    pub size_bytes: i64,
    pub files: Vec<IndexedFileRecord>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairedDeviceRecord {
    pub id: String,
    pub device_name: String,
    pub device_type: String,
    pub cert_fingerprint: String,
    pub paired_at: i64,
    pub last_seen_at: i64,
    pub status: String,
}

fn map_paired_device(row: &rusqlite::Row<'_>) -> rusqlite::Result<PairedDeviceRecord> {
    Ok(PairedDeviceRecord {
        id: row.get(0)?,
        device_name: row.get(1)?,
        device_type: row.get(2)?,
        cert_fingerprint: row.get(3)?,
        paired_at: row.get(4)?,
        last_seen_at: row.get(5)?,
        status: row.get(6)?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TokenValidationResult {
    Valid,
    CodeMismatch,
    InvalidOrExpired,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairingTokenRecord {
    pub token: String,
    pub confirmation_code: String,
    pub expires_at: i64,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DeviceStorageSummaryRecord {
    pub device_id: String,
    pub total_bytes: i64,
    pub free_bytes: i64,
    pub indexed_items_count: i64,
    pub updated_at: i64,
}

fn map_device_storage_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeviceStorageSummaryRecord> {
    Ok(DeviceStorageSummaryRecord {
        device_id: row.get(0)?,
        total_bytes: row.get(1)?,
        free_bytes: row.get(2)?,
        indexed_items_count: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FileTransferRecord {
    pub transfer_id: String,
    pub device_id: String,
    pub file_name: String,
    pub file_size: i64,
    pub bytes_transferred: i64,
    pub direction: String,
    pub status: String,
    pub sha256: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

fn map_file_transfer(row: &rusqlite::Row<'_>) -> rusqlite::Result<FileTransferRecord> {
    Ok(FileTransferRecord {
        transfer_id: row.get(0)?,
        device_id: row.get(1)?,
        file_name: row.get(2)?,
        file_size: row.get(3)?,
        bytes_transferred: row.get(4)?,
        direction: row.get(5)?,
        status: row.get(6)?,
        sha256: row.get(7)?,
        started_at: row.get(8)?,
        completed_at: row.get(9)?,
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

    #[test]
    fn test_phase10_storage_indexed_files_and_analysis() {
        let db = Database::open_in_memory().unwrap();

        let f1 = IndexedFileRecord {
            path: "C:\\Users\\Satoshi\\Documents\\paper.pdf".to_string(),
            parent_dir: "C:\\Users\\Satoshi\\Documents".to_string(),
            file_name: "paper.pdf".to_string(),
            extension: Some("pdf".to_string()),
            size_bytes: 2048,
            modified_at: 1000,
            created_at: 900,
            is_dir: false,
            content_hash: Some("hash-abc-1".to_string()),
            mime_type: Some("application/pdf".to_string()),
            indexed_at: 1050,
        };

        let f2 = IndexedFileRecord {
            path: "C:\\Users\\Satoshi\\Downloads\\paper_copy.pdf".to_string(),
            parent_dir: "C:\\Users\\Satoshi\\Downloads".to_string(),
            file_name: "paper_copy.pdf".to_string(),
            extension: Some("pdf".to_string()),
            size_bytes: 2048,
            modified_at: 500, // old download
            created_at: 500,
            is_dir: false,
            content_hash: Some("hash-abc-1".to_string()), // duplicate hash!
            mime_type: Some("application/pdf".to_string()),
            indexed_at: 1050,
        };

        let f3 = IndexedFileRecord {
            path: "C:\\Users\\Satoshi\\Projects\\app.rs".to_string(),
            parent_dir: "C:\\Users\\Satoshi\\Projects".to_string(),
            file_name: "app.rs".to_string(),
            extension: Some("rs".to_string()),
            size_bytes: 4096,
            modified_at: 2000,
            created_at: 1800,
            is_dir: false,
            content_hash: Some("hash-rs-unique".to_string()),
            mime_type: Some("text/rust".to_string()),
            indexed_at: 2050,
        };

        db.upsert_indexed_file(&f1).unwrap();
        db.upsert_indexed_file(&f2).unwrap();
        db.upsert_indexed_file(&f3).unwrap();

        assert_eq!(db.count_indexed_files().unwrap(), 3);

        // Search
        let search_res = db.search_indexed_files("paper", 10).unwrap();
        assert_eq!(search_res.len(), 2);

        // Analysis
        let analysis = db.get_storage_analysis(5).unwrap();
        assert_eq!(analysis.total_files, 3);
        assert_eq!(analysis.total_bytes, 2048 + 2048 + 4096);
        assert_eq!(analysis.largest_files[0].path, f3.path);
        assert_eq!(analysis.old_downloads.len(), 1);
        assert_eq!(analysis.old_downloads[0].path, f2.path);

        // Duplicates
        let duplicates = db.find_duplicate_candidates().unwrap();
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].content_hash, "hash-abc-1");
        assert_eq!(duplicates[0].files.len(), 2);

        // Delete one file
        assert!(db.delete_indexed_file(&f1.path).unwrap());
        assert_eq!(db.count_indexed_files().unwrap(), 2);
    }

    #[test]
    fn test_phase10_storage_file_operations_journal() {
        let db = Database::open_in_memory().unwrap();

        let op = FileOperationRecord {
            id: "op-001".to_string(),
            op_type: "move".to_string(),
            source_path: "C:\\tmp\\source.txt".to_string(),
            destination_path: Some("C:\\tmp\\dest.txt".to_string()),
            backup_path: Some("C:\\tmp\\.hime\\recycle\\source.txt".to_string()),
            source_hash: Some("hash123".to_string()),
            dest_hash: None,
            actor: "agent".to_string(),
            approval_id: Some("appr-101".to_string()),
            status: "staged".to_string(),
            started_at: 1000,
            completed_at: None,
            error_message: None,
        };

        db.record_file_operation(&op).unwrap();

        // Check uncommitted operations
        let uncommitted = db.get_uncommitted_file_operations().unwrap();
        assert_eq!(uncommitted.len(), 1);
        assert_eq!(uncommitted[0].id, "op-001");

        // Update status to committed
        assert!(db.update_file_operation_status("op-001", "committed", Some("hash123"), true, None).unwrap());

        let committed = db.get_file_operation("op-001").unwrap().unwrap();
        assert_eq!(committed.status, "committed");
        assert_eq!(committed.dest_hash.as_deref(), Some("hash123"));
        assert!(committed.completed_at.is_some());

        let uncommitted_now = db.get_uncommitted_file_operations().unwrap();
        assert_eq!(uncommitted_now.len(), 0);
    }

    #[test]
    fn test_phase10_storage_file_rules_and_backups() {
        let db = Database::open_in_memory().unwrap();

        let rule = FileRuleRecord {
            id: "rule-pdf".to_string(),
            name: "Archive PDF reports".to_string(),
            pattern: "*.pdf".to_string(),
            target_dir: "C:\\Users\\Satoshi\\Documents\\PDFs".to_string(),
            naming_template: Some("{name}_{date}.{ext}".to_string()),
            action: "propose".to_string(),
            is_active: true,
            created_at: 1000,
            updated_at: 1000,
        };

        db.create_file_rule(&rule).unwrap();
        let rules = db.list_file_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "Archive PDF reports");

        db.update_file_rule_active("rule-pdf", false).unwrap();
        let fetched_rule = db.get_file_rule("rule-pdf").unwrap().unwrap();
        assert!(!fetched_rule.is_active);

        // Backup
        let backup = StorageBackupRecord {
            id: "bak-01".to_string(),
            source_root: "C:\\Users\\Satoshi\\Documents".to_string(),
            backup_destination: "D:\\HimeBackups\\Docs".to_string(),
            file_count: 42,
            total_bytes: 1048576,
            manifest_hash: "sha256-manifest-001".to_string(),
            created_at: 2000,
            verified_at: None,
        };

        db.create_backup_record(&backup).unwrap();
        let backups = db.list_backup_records().unwrap();
        assert_eq!(backups.len(), 1);
        assert!(backups[0].verified_at.is_none());

        assert!(db.verify_backup_record("bak-01").unwrap());
        let verified = db.list_backup_records().unwrap();
        assert!(verified[0].verified_at.is_some());
    }

    #[test]
    fn test_phase11_mobile_device_pairing_storage_and_transfers() {
        let db = Database::open_in_memory().unwrap();

        // 1. Pairing token lifecycle
        let token = db.create_pairing_token("tok-12345", "654321", 300).unwrap();
        assert_eq!(token.confirmation_code, "654321");

        // Invalid code rejection
        assert_eq!(db.validate_and_consume_pairing_token("tok-12345", "000000").unwrap(), TokenValidationResult::CodeMismatch);

        // Valid consumption
        assert_eq!(db.validate_and_consume_pairing_token("tok-12345", "654321").unwrap(), TokenValidationResult::Valid);
        // Second consumption must fail (single-use anti-replay)
        assert_eq!(db.validate_and_consume_pairing_token("tok-12345", "654321").unwrap(), TokenValidationResult::InvalidOrExpired);

        // 2. Paired devices
        let dev = PairedDeviceRecord {
            id: "dev-pixel-8".to_string(),
            device_name: "Pixel 8 Pro (Satoshi)".to_string(),
            device_type: "android".to_string(),
            cert_fingerprint: "SHA256:FINGERPRINT123".to_string(),
            paired_at: 1000,
            last_seen_at: 1000,
            status: "active".to_string(),
        };
        db.add_paired_device(&dev).unwrap();

        let devices = db.list_paired_devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_name, "Pixel 8 Pro (Satoshi)");

        // Update last seen
        assert!(db.update_device_last_seen("dev-pixel-8").unwrap());

        // Revoke
        assert!(db.revoke_device("dev-pixel-8").unwrap());
        let fetched = db.get_paired_device("dev-pixel-8").unwrap().unwrap();
        assert_eq!(fetched.status, "revoked");

        // 3. Device Storage Summary
        let summary = DeviceStorageSummaryRecord {
            device_id: "dev-pixel-8".to_string(),
            total_bytes: 256000000000,
            free_bytes: 128000000000,
            indexed_items_count: 1420,
            updated_at: 2000,
        };
        db.set_device_storage_summary(&summary).unwrap();
        let fetched_summary = db.get_device_storage_summary("dev-pixel-8").unwrap().unwrap();
        assert_eq!(fetched_summary.free_bytes, 128000000000);

        // 4. File Transfers
        let transfer = FileTransferRecord {
            transfer_id: "xfer-001".to_string(),
            device_id: "dev-pixel-8".to_string(),
            file_name: "DCIM_20261113.jpg".to_string(),
            file_size: 4096000,
            bytes_transferred: 2048000,
            direction: "upload".to_string(),
            status: "in_progress".to_string(),
            sha256: "sha256-transfer-mock".to_string(),
            started_at: 3000,
            completed_at: None,
        };
        db.record_file_transfer(&transfer).unwrap();

        let transfers = db.list_file_transfers(Some("dev-pixel-8")).unwrap();
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].status, "in_progress");

        db.update_transfer_progress("xfer-001", 4096000, "completed").unwrap();
        let updated = db.list_file_transfers(Some("dev-pixel-8")).unwrap();
        assert_eq!(updated[0].status, "completed");
        assert!(updated[0].completed_at.is_some());
    }
}
