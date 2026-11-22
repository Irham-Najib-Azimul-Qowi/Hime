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

    #[error("Item not found: {0}")]
    NotFound(String),
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

    /// Returns (cache_size, mmap_size) pragmas for resource optimization verification.
    pub fn get_resource_tuning_pragmas(&self) -> Result<(i64, i64), StorageError> {
        let conn = self.conn.lock().unwrap();
        let cache_size: i64 = conn.query_row("PRAGMA cache_size;", [], |r| r.get(0))?;
        let mmap_size: i64 = conn.query_row("PRAGMA mmap_size;", [], |r| r.get(0)).unwrap_or(0);
        Ok((cache_size, mmap_size))
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
             PRAGMA busy_timeout = 5000;
             PRAGMA cache_size = -2000;
             PRAGMA mmap_size = 0;",
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
            CREATE INDEX IF NOT EXISTS idx_transfers_device ON file_transfers(device_id);

            -- Phase 12: Productivity, Accountability & Documents
            CREATE TABLE IF NOT EXISTS kanban_boards (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS kanban_columns (
                id TEXT PRIMARY KEY,
                board_id TEXT NOT NULL,
                title TEXT NOT NULL,
                position INTEGER NOT NULL,
                wip_limit INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_columns_board ON kanban_columns(board_id);

            CREATE TABLE IF NOT EXISTS productivity_tasks (
                id TEXT PRIMARY KEY,
                board_id TEXT NOT NULL,
                column_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                priority TEXT NOT NULL,
                status TEXT NOT NULL,
                due_date INTEGER,
                estimated_minutes INTEGER,
                actual_minutes INTEGER,
                labels TEXT,
                parent_task_id TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                completed_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_prod_tasks_board ON productivity_tasks(board_id);
            CREATE INDEX IF NOT EXISTS idx_prod_tasks_column ON productivity_tasks(column_id);

            CREATE TABLE IF NOT EXISTS study_schedules (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                subject TEXT NOT NULL,
                start_time INTEGER NOT NULL,
                end_time INTEGER NOT NULL,
                priority TEXT NOT NULL,
                status TEXT NOT NULL,
                rescheduled_reason TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_schedules_start ON study_schedules(start_time);

            CREATE TABLE IF NOT EXISTS accountability_settings (
                id TEXT PRIMARY KEY,
                mode TEXT NOT NULL,
                screen_time_opt_in INTEGER NOT NULL,
                last_checkin_at INTEGER,
                daily_reflection_notes TEXT
            );

            CREATE TABLE IF NOT EXISTS screen_time_sessions (
                id TEXT PRIMARY KEY,
                app_name TEXT NOT NULL,
                window_title TEXT,
                category TEXT NOT NULL,
                duration_secs INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                date_str TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_screen_time_timestamp ON screen_time_sessions(timestamp);
            CREATE INDEX IF NOT EXISTS idx_screen_time_app ON screen_time_sessions(app_name);

            -- Stage U3: Routines, Commitments, Enforcement & Pauses
            CREATE TABLE IF NOT EXISTS routines (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                category TEXT NOT NULL,
                schedule_json TEXT NOT NULL,
                enforcement TEXT NOT NULL,
                allowed_apps_json TEXT,
                distraction_rules_json TEXT,
                start_grace_min INTEGER NOT NULL DEFAULT 5,
                escalation_json TEXT,
                current_streak INTEGER NOT NULL DEFAULT 0,
                best_streak INTEGER NOT NULL DEFAULT 0,
                last_completed_date TEXT,
                frozen_on_date TEXT,
                active INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_routines_active ON routines(active);

            CREATE TABLE IF NOT EXISTS commitments (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                tier TEXT NOT NULL,
                target_apps_json TEXT NOT NULL,
                daily_quota_min INTEGER,
                active INTEGER NOT NULL DEFAULT 1,
                pending_weakening_until INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_commitments_active ON commitments(active);

            CREATE TABLE IF NOT EXISTS enforcement_history (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                target_app TEXT NOT NULL,
                tier TEXT NOT NULL,
                action_taken TEXT NOT NULL,
                details TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_enforcement_hist_ts ON enforcement_history(timestamp);

            CREATE TABLE IF NOT EXISTS emergency_pauses (
                id TEXT PRIMARY KEY,
                started_at INTEGER NOT NULL,
                duration_min INTEGER NOT NULL,
                reason TEXT NOT NULL,
                expired_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_emergency_pauses_exp ON emergency_pauses(expired_at);

            CREATE TABLE IF NOT EXISTS document_jobs (
                id TEXT PRIMARY KEY,
                operation TEXT NOT NULL,
                source_path TEXT,
                target_path TEXT,
                status TEXT NOT NULL,
                format TEXT NOT NULL,
                metadata TEXT,
                created_at INTEGER NOT NULL,
                completed_at INTEGER
            );

            -- Phase 13: Browser, Integrations & Developer Workspace
            CREATE TABLE IF NOT EXISTS browser_sessions (
                id TEXT PRIMARY KEY,
                profile_path TEXT NOT NULL,
                current_url TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                closed_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS browser_history (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                url TEXT NOT NULL,
                title TEXT,
                visited_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_browser_hist_session ON browser_history(session_id);

            CREATE TABLE IF NOT EXISTS browser_downloads (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                source_url TEXT NOT NULL,
                quarantine_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                sha256 TEXT NOT NULL,
                status TEXT NOT NULL,
                downloaded_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_browser_dl_session ON browser_downloads(session_id);

            CREATE TABLE IF NOT EXISTS developer_workspaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                allowed_commands TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS developer_command_runs (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                command TEXT NOT NULL,
                args TEXT NOT NULL,
                exit_code INTEGER,
                duration_ms INTEGER NOT NULL,
                is_allowed INTEGER NOT NULL,
                output_snippet TEXT,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_dev_runs_ws ON developer_command_runs(workspace_id);

            CREATE TABLE IF NOT EXISTS integration_accounts (
                id TEXT PRIMARY KEY,
                service_name TEXT NOT NULL,
                account_label TEXT NOT NULL,
                auth_type TEXT NOT NULL,
                token_vault_id TEXT NOT NULL,
                scopes TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_integ_svc ON integration_accounts(service_name);

            CREATE TABLE IF NOT EXISTS specialist_registry (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                role_title TEXT NOT NULL,
                domain TEXT NOT NULL,
                responsibilities TEXT NOT NULL,
                system_prompt TEXT NOT NULL,
                allowed_tools TEXT NOT NULL,
                max_permission_level TEXT NOT NULL,
                resource_budget TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                evaluation_score REAL NOT NULL DEFAULT 0.0,
                version TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_spec_domain ON specialist_registry(domain);

            CREATE TABLE IF NOT EXISTS specialist_delegations (
                id TEXT PRIMARY KEY,
                specialist_id TEXT NOT NULL,
                task_id TEXT NOT NULL,
                input_prompt TEXT NOT NULL,
                status TEXT NOT NULL,
                output_summary TEXT NOT NULL,
                evidence TEXT NOT NULL,
                tokens_used INTEGER NOT NULL DEFAULT 0,
                duration_ms INTEGER NOT NULL DEFAULT 0,
                delegated_at INTEGER NOT NULL,
                completed_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_spec_del_spec ON specialist_delegations(specialist_id);
            CREATE INDEX IF NOT EXISTS idx_spec_del_task ON specialist_delegations(task_id);

            CREATE TABLE IF NOT EXISTS proactive_suggestions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                category TEXT NOT NULL,
                status TEXT NOT NULL,
                score REAL NOT NULL DEFAULT 1.0,
                created_at INTEGER NOT NULL,
                responded_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_proact_status ON proactive_suggestions(status);

            CREATE TABLE IF NOT EXISTS proactive_settings (
                id TEXT PRIMARY KEY,
                is_enabled INTEGER NOT NULL DEFAULT 1,
                cooldown_minutes INTEGER NOT NULL DEFAULT 30,
                muted_until INTEGER,
                updated_at INTEGER NOT NULL
            );

            -- Stage 10R: Proactive Interaction Preference Rules
            CREATE TABLE IF NOT EXISTS interaction_prefs (
                kind TEXT PRIMARY KEY,
                decision TEXT NOT NULL DEFAULT 'default',
                backoff_count INTEGER NOT NULL DEFAULT 0,
                backoff_until INTEGER,
                last_shown_at INTEGER,
                updated_at INTEGER NOT NULL
            );

            -- Stage 11R: Pinned Shelf Items for Mobile & Device Sync
            CREATE TABLE IF NOT EXISTS shelf_items (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                item_type TEXT NOT NULL,
                is_pinned INTEGER NOT NULL DEFAULT 1,
                source_device TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_shelf_pinned ON shelf_items(is_pinned);

            -- Stage U2: Real-Time Lyrics Cache
            CREATE TABLE IF NOT EXISTS lyrics_cache (
                cache_key TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT,
                duration_sec INTEGER NOT NULL,
                source TEXT NOT NULL,
                synced_lyrics TEXT,
                parsed_json TEXT,
                plain_lyrics TEXT,
                is_instrumental INTEGER NOT NULL DEFAULT 0,
                user_offset_ms INTEGER NOT NULL DEFAULT 0,
                cached_at INTEGER NOT NULL,
                expires_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_lyrics_expires ON lyrics_cache(expires_at);",
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

        // Ensure backward compatibility if screen_time_sessions already existed without date_str
        let mut pragma_st_stmt = conn.prepare("PRAGMA table_info(screen_time_sessions)")?;
        let st_rows = pragma_st_stmt.query_map([], |row| row.get::<_, String>(1))?;
        let existing_st_cols: Vec<String> = st_rows.filter_map(Result::ok).collect();
        if !existing_st_cols.iter().any(|c| c == "date_str") {
            let _ = conn.execute("ALTER TABLE screen_time_sessions ADD COLUMN date_str TEXT NOT NULL DEFAULT ''", []);
        }
        let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_screen_time_date ON screen_time_sessions(date_str)", []);

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

    // --- Phase 12: Productivity, Accountability & Documents ---

    pub fn create_kanban_board(&self, board: &KanbanBoardRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO kanban_boards (id, title, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET title = ?2, description = ?3, updated_at = ?5",
            rusqlite::params![board.id, board.title, board.description, board.created_at, board.updated_at],
        )?;
        Ok(())
    }

    pub fn list_kanban_boards(&self) -> Result<Vec<KanbanBoardRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, title, description, created_at, updated_at FROM kanban_boards ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], map_kanban_board)?;
        let mut boards = Vec::new();
        for b in rows {
            boards.push(b?);
        }
        Ok(boards)
    }

    pub fn get_kanban_board(&self, board_id: &str) -> Result<Option<KanbanBoardRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, title, description, created_at, updated_at FROM kanban_boards WHERE id = ?1")?;
        let mut rows = stmt.query(rusqlite::params![board_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_kanban_board(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn create_kanban_column(&self, col: &KanbanColumnRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO kanban_columns (id, board_id, title, position, wip_limit)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET title = ?3, position = ?4, wip_limit = ?5",
            rusqlite::params![col.id, col.board_id, col.title, col.position, col.wip_limit],
        )?;
        Ok(())
    }

    pub fn list_kanban_columns(&self, board_id: &str) -> Result<Vec<KanbanColumnRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, board_id, title, position, wip_limit FROM kanban_columns WHERE board_id = ?1 ORDER BY position ASC")?;
        let rows = stmt.query_map(rusqlite::params![board_id], map_kanban_column)?;
        let mut cols = Vec::new();
        for c in rows {
            cols.push(c?);
        }
        Ok(cols)
    }

    pub fn create_productivity_task(&self, task: &ProductivityTaskRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO productivity_tasks (id, board_id, column_id, title, description, priority, status, due_date, estimated_minutes, actual_minutes, labels, parent_task_id, created_at, updated_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(id) DO UPDATE SET
                column_id = ?3, title = ?4, description = ?5, priority = ?6, status = ?7, due_date = ?8,
                estimated_minutes = ?9, actual_minutes = ?10, labels = ?11, parent_task_id = ?12, updated_at = ?14, completed_at = ?15",
            rusqlite::params![
                task.id, task.board_id, task.column_id, task.title, task.description, task.priority,
                task.status, task.due_date, task.estimated_minutes, task.actual_minutes, task.labels,
                task.parent_task_id, task.created_at, task.updated_at, task.completed_at
            ],
        )?;
        Ok(())
    }

    pub fn list_productivity_tasks(&self, board_id: &str) -> Result<Vec<ProductivityTaskRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, board_id, column_id, title, description, priority, status, due_date, estimated_minutes, actual_minutes, labels, parent_task_id, created_at, updated_at, completed_at
             FROM productivity_tasks WHERE board_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(rusqlite::params![board_id], map_productivity_task)?;
        let mut tasks = Vec::new();
        for t in rows {
            tasks.push(t?);
        }
        Ok(tasks)
    }

    pub fn update_productivity_task_column(&self, task_id: &str, column_id: &str, status: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let completed_at = if status == "done" { Some(now) } else { None };
        let rows = conn.execute(
            "UPDATE productivity_tasks SET column_id = ?2, status = ?3, updated_at = ?4, completed_at = COALESCE(?5, completed_at) WHERE id = ?1",
            rusqlite::params![task_id, column_id, status, now, completed_at],
        )?;
        Ok(rows > 0)
    }

    pub fn delete_productivity_task(&self, task_id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM productivity_tasks WHERE id = ?1",
            rusqlite::params![task_id],
        )?;
        Ok(rows > 0)
    }

    pub fn create_study_schedule(&self, schedule: &StudyScheduleRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO study_schedules (id, title, subject, start_time, end_time, priority, status, rescheduled_reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET title = ?2, subject = ?3, start_time = ?4, end_time = ?5, priority = ?6, status = ?7, rescheduled_reason = ?8",
            rusqlite::params![
                schedule.id, schedule.title, schedule.subject, schedule.start_time, schedule.end_time,
                schedule.priority, schedule.status, schedule.rescheduled_reason
            ],
        )?;
        Ok(())
    }

    pub fn list_study_schedules(&self) -> Result<Vec<StudyScheduleRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, subject, start_time, end_time, priority, status, rescheduled_reason
             FROM study_schedules ORDER BY start_time ASC",
        )?;
        let rows = stmt.query_map([], map_study_schedule)?;
        let mut schedules = Vec::new();
        for s in rows {
            schedules.push(s?);
        }
        Ok(schedules)
    }

    pub fn get_accountability_settings(&self) -> Result<AccountabilitySettingsRecord, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, mode, screen_time_opt_in, last_checkin_at, daily_reflection_notes FROM accountability_settings WHERE id = 'default'")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(map_accountability_settings(row)?)
        } else {
            Ok(AccountabilitySettingsRecord {
                id: "default".to_string(),
                mode: "balanced".to_string(),
                screen_time_opt_in: false,
                last_checkin_at: None,
                daily_reflection_notes: None,
            })
        }
    }

    pub fn set_accountability_settings(&self, settings: &AccountabilitySettingsRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let opt_in = if settings.screen_time_opt_in { 1 } else { 0 };
        conn.execute(
            "INSERT INTO accountability_settings (id, mode, screen_time_opt_in, last_checkin_at, daily_reflection_notes)
             VALUES ('default', ?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET mode = ?1, screen_time_opt_in = ?2, last_checkin_at = ?3, daily_reflection_notes = ?4",
            rusqlite::params![settings.mode, opt_in, settings.last_checkin_at, settings.daily_reflection_notes],
        )?;
        Ok(())
    }

    pub fn record_screen_time_session(&self, session: &ScreenTimeSessionRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        // Sanitize window title to maintain SEC-09 privacy: never store sensitive titles or URLs
        let sanitized_title: Option<String> = None;
        let date_str = session.date_str.clone().unwrap_or_else(|| epoch_to_date_str(session.timestamp));
        conn.execute(
            "INSERT INTO screen_time_sessions (id, app_name, window_title, category, duration_secs, timestamp, date_str)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![session.id, session.app_name, sanitized_title, session.category, session.duration_secs, session.timestamp, date_str],
        )?;
        Ok(())
    }

    pub fn list_screen_time_sessions(&self, limit: usize) -> Result<Vec<ScreenTimeSessionRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, app_name, window_title, category, duration_secs, timestamp, date_str FROM screen_time_sessions ORDER BY timestamp DESC LIMIT ?1")?;
        let rows = stmt.query_map(rusqlite::params![limit], map_screen_time_session)?;
        let mut sessions = Vec::new();
        for s in rows {
            sessions.push(s?);
        }
        Ok(sessions)
    }

    pub fn clear_screen_time_sessions(&self) -> Result<usize, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute("DELETE FROM screen_time_sessions", [])?;
        let _ = conn.execute_batch("VACUUM;");
        Ok(count)
    }

    // ---------------------------------------------------------
    // Stage U3: Routines, Commitments, Streaks & Enforcement
    // ---------------------------------------------------------

    pub fn create_routine(&self, routine: &RoutineRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO routines (
                id, title, category, schedule_json, enforcement, allowed_apps_json,
                distraction_rules_json, start_grace_min, escalation_json, current_streak,
                best_streak, last_completed_date, frozen_on_date, active, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            rusqlite::params![
                routine.id, routine.title, routine.category, routine.schedule_json, routine.enforcement,
                routine.allowed_apps_json, routine.distraction_rules_json, routine.start_grace_min,
                routine.escalation_json, routine.current_streak, routine.best_streak,
                routine.last_completed_date, routine.frozen_on_date, if routine.active { 1 } else { 0 },
                routine.created_at, routine.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn get_routine(&self, id: &str) -> Result<Option<RoutineRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, category, schedule_json, enforcement, allowed_apps_json,
                    distraction_rules_json, start_grace_min, escalation_json, current_streak,
                    best_streak, last_completed_date, frozen_on_date, active, created_at, updated_at
             FROM routines WHERE id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_routine(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_routines(&self, active_only: bool) -> Result<Vec<RoutineRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let sql = if active_only {
            "SELECT id, title, category, schedule_json, enforcement, allowed_apps_json,
                    distraction_rules_json, start_grace_min, escalation_json, current_streak,
                    best_streak, last_completed_date, frozen_on_date, active, created_at, updated_at
             FROM routines WHERE active = 1 ORDER BY created_at ASC"
        } else {
            "SELECT id, title, category, schedule_json, enforcement, allowed_apps_json,
                    distraction_rules_json, start_grace_min, escalation_json, current_streak,
                    best_streak, last_completed_date, frozen_on_date, active, created_at, updated_at
             FROM routines ORDER BY created_at ASC"
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], map_routine)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn update_routine(&self, routine: &RoutineRecord) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "UPDATE routines SET
                title = ?2, category = ?3, schedule_json = ?4, enforcement = ?5,
                allowed_apps_json = ?6, distraction_rules_json = ?7, start_grace_min = ?8,
                escalation_json = ?9, current_streak = ?10, best_streak = ?11,
                last_completed_date = ?12, frozen_on_date = ?13, active = ?14, updated_at = ?15
             WHERE id = ?1",
            rusqlite::params![
                routine.id, routine.title, routine.category, routine.schedule_json, routine.enforcement,
                routine.allowed_apps_json, routine.distraction_rules_json, routine.start_grace_min,
                routine.escalation_json, routine.current_streak, routine.best_streak,
                routine.last_completed_date, routine.frozen_on_date, if routine.active { 1 } else { 0 },
                routine.updated_at
            ],
        )?;
        Ok(count > 0)
    }

    pub fn delete_routine(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute("DELETE FROM routines WHERE id = ?1", rusqlite::params![id])?;
        Ok(count > 0)
    }

    pub fn update_routine_streak(
        &self,
        id: &str,
        current_streak: i64,
        best_streak: i64,
        last_completed_date: Option<&str>,
        frozen_on_date: Option<&str>,
    ) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let count = conn.execute(
            "UPDATE routines SET current_streak = ?2, best_streak = ?3, last_completed_date = ?4, frozen_on_date = ?5, updated_at = ?6 WHERE id = ?1",
            rusqlite::params![id, current_streak, best_streak, last_completed_date, frozen_on_date, now],
        )?;
        Ok(count > 0)
    }

    pub fn create_commitment(&self, commitment: &CommitmentRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO commitments (id, name, tier, target_apps_json, daily_quota_min, active, pending_weakening_until, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                commitment.id, commitment.name, commitment.tier, commitment.target_apps_json,
                commitment.daily_quota_min, if commitment.active { 1 } else { 0 },
                commitment.pending_weakening_until, commitment.created_at, commitment.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn get_commitment(&self, id: &str) -> Result<Option<CommitmentRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, tier, target_apps_json, daily_quota_min, active, pending_weakening_until, created_at, updated_at
             FROM commitments WHERE id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_commitment(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_commitments(&self, active_only: bool) -> Result<Vec<CommitmentRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let sql = if active_only {
            "SELECT id, name, tier, target_apps_json, daily_quota_min, active, pending_weakening_until, created_at, updated_at
             FROM commitments WHERE active = 1 ORDER BY created_at ASC"
        } else {
            "SELECT id, name, tier, target_apps_json, daily_quota_min, active, pending_weakening_until, created_at, updated_at
             FROM commitments ORDER BY created_at ASC"
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], map_commitment)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn update_commitment(&self, commitment: &CommitmentRecord) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "UPDATE commitments SET
                name = ?2, tier = ?3, target_apps_json = ?4, daily_quota_min = ?5,
                active = ?6, pending_weakening_until = ?7, updated_at = ?8
             WHERE id = ?1",
            rusqlite::params![
                commitment.id, commitment.name, commitment.tier, commitment.target_apps_json,
                commitment.daily_quota_min, if commitment.active { 1 } else { 0 },
                commitment.pending_weakening_until, commitment.updated_at
            ],
        )?;
        Ok(count > 0)
    }

    pub fn delete_commitment(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute("DELETE FROM commitments WHERE id = ?1", rusqlite::params![id])?;
        Ok(count > 0)
    }

    pub fn request_commitment_weakening(&self, id: &str, pending_until: i64) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let count = conn.execute(
            "UPDATE commitments SET pending_weakening_until = ?2, updated_at = ?3 WHERE id = ?1",
            rusqlite::params![id, pending_until, now],
        )?;
        Ok(count > 0)
    }

    pub fn record_enforcement_event(&self, event: &EnforcementHistoryRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO enforcement_history (id, timestamp, target_app, tier, action_taken, details)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![event.id, event.timestamp, event.target_app, event.tier, event.action_taken, event.details],
        )?;
        Ok(())
    }

    pub fn list_enforcement_history(&self, limit: usize) -> Result<Vec<EnforcementHistoryRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, target_app, tier, action_taken, details
             FROM enforcement_history ORDER BY timestamp DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit], map_enforcement_history)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn record_emergency_pause(&self, pause: &EmergencyPauseRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO emergency_pauses (id, started_at, duration_min, reason, expired_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![pause.id, pause.started_at, pause.duration_min, pause.reason, pause.expired_at],
        )?;
        Ok(())
    }

    pub fn get_active_emergency_pause(&self, now_secs: i64) -> Result<Option<EmergencyPauseRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, started_at, duration_min, reason, expired_at
             FROM emergency_pauses WHERE expired_at > ?1 ORDER BY expired_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(rusqlite::params![now_secs])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_emergency_pause(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_emergency_pauses(&self, limit: usize) -> Result<Vec<EmergencyPauseRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, started_at, duration_min, reason, expired_at
             FROM emergency_pauses ORDER BY started_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit], map_emergency_pause)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn create_document_job(&self, job: &DocumentJobRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO document_jobs (id, operation, source_path, target_path, status, format, metadata, created_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET status = ?5, target_path = ?4, metadata = ?7, completed_at = ?9",
            rusqlite::params![job.id, job.operation, job.source_path, job.target_path, job.status, job.format, job.metadata, job.created_at, job.completed_at],
        )?;
        Ok(())
    }

    pub fn list_document_jobs(&self) -> Result<Vec<DocumentJobRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, operation, source_path, target_path, status, format, metadata, created_at, completed_at FROM document_jobs ORDER BY created_at DESC LIMIT 50")?;
        let rows = stmt.query_map([], map_document_job)?;
        let mut jobs = Vec::new();
        for j in rows {
            jobs.push(j?);
        }
        Ok(jobs)
    }

    // =========================================================================
    // Phase 13: Browser, Integrations & Developer Workspace Methods
    // =========================================================================

    pub fn create_browser_session(&self, record: &BrowserSessionRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO browser_sessions (id, profile_path, current_url, is_active, created_at, closed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                record.id,
                record.profile_path,
                record.current_url,
                if record.is_active { 1 } else { 0 },
                record.created_at,
                record.closed_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_active_browser_session(&self) -> Result<Option<BrowserSessionRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, profile_path, current_url, is_active, created_at, closed_at
             FROM browser_sessions WHERE is_active = 1 ORDER BY created_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map([], map_browser_session)?;
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn update_browser_session_url(&self, session_id: &str, url: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE browser_sessions SET current_url = ?1 WHERE id = ?2",
            rusqlite::params![url, session_id],
        )?;
        Ok(())
    }

    pub fn close_browser_session(&self, session_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "UPDATE browser_sessions SET is_active = 0, closed_at = ?1 WHERE id = ?2",
            rusqlite::params![now, session_id],
        )?;
        Ok(())
    }

    pub fn record_browser_history(&self, record: &BrowserHistoryRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO browser_history (id, session_id, url, title, visited_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                record.id,
                record.session_id,
                record.url,
                record.title,
                record.visited_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_browser_history(&self, session_id: Option<&str>, limit: usize) -> Result<Vec<BrowserHistoryRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut history = Vec::new();
        if let Some(sid) = session_id {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, url, title, visited_at
                 FROM browser_history WHERE session_id = ?1 ORDER BY visited_at DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(rusqlite::params![sid, limit as i64], map_browser_history)?;
            for r in rows {
                history.push(r?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, url, title, visited_at
                 FROM browser_history ORDER BY visited_at DESC LIMIT ?1",
            )?;
            let rows = stmt.query_map(rusqlite::params![limit as i64], map_browser_history)?;
            for r in rows {
                history.push(r?);
            }
        }
        Ok(history)
    }

    pub fn record_browser_download(&self, record: &BrowserDownloadRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO browser_downloads (id, session_id, source_url, quarantine_path, file_size, sha256, status, downloaded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                record.id,
                record.session_id,
                record.source_url,
                record.quarantine_path,
                record.file_size,
                record.sha256,
                record.status,
                record.downloaded_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_browser_downloads(&self, limit: usize) -> Result<Vec<BrowserDownloadRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, source_url, quarantine_path, file_size, sha256, status, downloaded_at
             FROM browser_downloads ORDER BY downloaded_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit as i64], map_browser_download)?;
        let mut downloads = Vec::new();
        for r in rows {
            downloads.push(r?);
        }
        Ok(downloads)
    }

    pub fn register_developer_workspace(&self, record: &DeveloperWorkspaceRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO developer_workspaces (id, name, path, allowed_commands, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(path) DO UPDATE SET name = ?2, allowed_commands = ?4, updated_at = ?6",
            rusqlite::params![
                record.id,
                record.name,
                record.path,
                record.allowed_commands,
                record.created_at,
                record.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_developer_workspace(&self, id: &str) -> Result<Option<DeveloperWorkspaceRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, path, allowed_commands, created_at, updated_at
             FROM developer_workspaces WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![id], map_developer_workspace)?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub fn get_developer_workspace_by_path(&self, path: &str) -> Result<Option<DeveloperWorkspaceRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, path, allowed_commands, created_at, updated_at
             FROM developer_workspaces WHERE path = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![path], map_developer_workspace)?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub fn list_developer_workspaces(&self) -> Result<Vec<DeveloperWorkspaceRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, path, allowed_commands, created_at, updated_at
             FROM developer_workspaces ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], map_developer_workspace)?;
        let mut workspaces = Vec::new();
        for r in rows {
            workspaces.push(r?);
        }
        Ok(workspaces)
    }

    pub fn delete_developer_workspace(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "DELETE FROM developer_workspaces WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(count > 0)
    }

    pub fn record_developer_command_run(&self, record: &DeveloperCommandRunRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO developer_command_runs (id, workspace_id, command, args, exit_code, duration_ms, is_allowed, output_snippet, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                record.id,
                record.workspace_id,
                record.command,
                record.args,
                record.exit_code,
                record.duration_ms,
                if record.is_allowed { 1 } else { 0 },
                record.output_snippet,
                record.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_developer_command_runs(&self, workspace_id: &str, limit: usize) -> Result<Vec<DeveloperCommandRunRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, command, args, exit_code, duration_ms, is_allowed, output_snippet, created_at
             FROM developer_command_runs WHERE workspace_id = ?1 ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![workspace_id, limit as i64], map_developer_command_run)?;
        let mut runs = Vec::new();
        for r in rows {
            runs.push(r?);
        }
        Ok(runs)
    }

    pub fn save_integration_account(&self, record: &IntegrationAccountRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO integration_accounts (id, service_name, account_label, auth_type, token_vault_id, scopes, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET account_label = ?3, auth_type = ?4, token_vault_id = ?5, scopes = ?6, status = ?7, updated_at = ?9",
            rusqlite::params![
                record.id,
                record.service_name,
                record.account_label,
                record.auth_type,
                record.token_vault_id,
                record.scopes,
                record.status,
                record.created_at,
                record.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_integration_account(&self, service_name: &str) -> Result<Option<IntegrationAccountRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, service_name, account_label, auth_type, token_vault_id, scopes, status, created_at, updated_at
             FROM integration_accounts WHERE service_name = ?1 ORDER BY updated_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![service_name], map_integration_account)?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub fn list_integration_accounts(&self) -> Result<Vec<IntegrationAccountRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, service_name, account_label, auth_type, token_vault_id, scopes, status, created_at, updated_at
             FROM integration_accounts ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], map_integration_account)?;
        let mut accounts = Vec::new();
        for r in rows {
            accounts.push(r?);
        }
        Ok(accounts)
    }

    pub fn delete_integration_account(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "DELETE FROM integration_accounts WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(count > 0)
    }

    // ---------------------------------------------------------------------------
    // Phase 14: Specialist Agents, Skills & Proactive Storage Operations
    // ---------------------------------------------------------------------------

    pub fn upsert_specialist(&self, spec: &SpecialistRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO specialist_registry (
                id, name, role_title, domain, responsibilities, system_prompt, allowed_tools,
                max_permission_level, resource_budget, is_active, evaluation_score, version,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                role_title = excluded.role_title,
                domain = excluded.domain,
                responsibilities = excluded.responsibilities,
                system_prompt = excluded.system_prompt,
                allowed_tools = excluded.allowed_tools,
                max_permission_level = excluded.max_permission_level,
                resource_budget = excluded.resource_budget,
                is_active = excluded.is_active,
                evaluation_score = excluded.evaluation_score,
                version = excluded.version,
                updated_at = excluded.updated_at",
            rusqlite::params![
                spec.id,
                spec.name,
                spec.role_title,
                spec.domain,
                spec.responsibilities,
                spec.system_prompt,
                spec.allowed_tools,
                spec.max_permission_level,
                spec.resource_budget,
                if spec.is_active { 1 } else { 0 },
                spec.evaluation_score,
                spec.version,
                spec.created_at,
                spec.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_specialist(&self, id: &str) -> Result<Option<SpecialistRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, role_title, domain, responsibilities, system_prompt, allowed_tools,
                    max_permission_level, resource_budget, is_active, evaluation_score, version,
                    created_at, updated_at
             FROM specialist_registry WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![id], map_specialist)?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub fn list_specialists(&self, active_only: bool) -> Result<Vec<SpecialistRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let query = if active_only {
            "SELECT id, name, role_title, domain, responsibilities, system_prompt, allowed_tools,
                    max_permission_level, resource_budget, is_active, evaluation_score, version,
                    created_at, updated_at
             FROM specialist_registry WHERE is_active = 1 ORDER BY evaluation_score DESC, name ASC"
        } else {
            "SELECT id, name, role_title, domain, responsibilities, system_prompt, allowed_tools,
                    max_permission_level, resource_budget, is_active, evaluation_score, version,
                    created_at, updated_at
             FROM specialist_registry ORDER BY is_active DESC, evaluation_score DESC, name ASC"
        };
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map([], map_specialist)?;
        let mut specialists = Vec::new();
        for r in rows {
            specialists.push(r?);
        }
        Ok(specialists)
    }

    pub fn update_specialist_evaluation(&self, id: &str, score: f64) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let count = conn.execute(
            "UPDATE specialist_registry SET evaluation_score = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![score, now, id],
        )?;
        Ok(count > 0)
    }

    pub fn record_specialist_delegation(&self, del: &SpecialistDelegationRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO specialist_delegations (
                id, specialist_id, task_id, input_prompt, status, output_summary, evidence,
                tokens_used, duration_ms, delegated_at, completed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                output_summary = excluded.output_summary,
                evidence = excluded.evidence,
                tokens_used = excluded.tokens_used,
                duration_ms = excluded.duration_ms,
                completed_at = excluded.completed_at",
            rusqlite::params![
                del.id,
                del.specialist_id,
                del.task_id,
                del.input_prompt,
                del.status,
                del.output_summary,
                del.evidence,
                del.tokens_used,
                del.duration_ms,
                del.delegated_at,
                del.completed_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_specialist_delegation(&self, id: &str) -> Result<Option<SpecialistDelegationRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, specialist_id, task_id, input_prompt, status, output_summary, evidence,
                    tokens_used, duration_ms, delegated_at, completed_at
             FROM specialist_delegations WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![id], map_specialist_delegation)?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub fn list_specialist_delegations(
        &self,
        specialist_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SpecialistDelegationRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut delegations = Vec::new();
        if let Some(sid) = specialist_id {
            let mut stmt = conn.prepare(
                "SELECT id, specialist_id, task_id, input_prompt, status, output_summary, evidence,
                        tokens_used, duration_ms, delegated_at, completed_at
                 FROM specialist_delegations WHERE specialist_id = ?1 ORDER BY delegated_at DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(rusqlite::params![sid, limit as i64], map_specialist_delegation)?;
            for r in rows {
                delegations.push(r?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, specialist_id, task_id, input_prompt, status, output_summary, evidence,
                        tokens_used, duration_ms, delegated_at, completed_at
                 FROM specialist_delegations ORDER BY delegated_at DESC LIMIT ?1",
            )?;
            let rows = stmt.query_map(rusqlite::params![limit as i64], map_specialist_delegation)?;
            for r in rows {
                delegations.push(r?);
            }
        }
        Ok(delegations)
    }

    pub fn complete_specialist_delegation(
        &self,
        id: &str,
        status: &str,
        output_summary: &str,
        evidence: &str,
        tokens_used: i64,
        duration_ms: i64,
    ) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let count = conn.execute(
            "UPDATE specialist_delegations SET
                status = ?1,
                output_summary = ?2,
                evidence = ?3,
                tokens_used = ?4,
                duration_ms = ?5,
                completed_at = ?6
             WHERE id = ?7",
            rusqlite::params![status, output_summary, evidence, tokens_used, duration_ms, now, id],
        )?;
        Ok(count > 0)
    }

    pub fn insert_proactive_suggestion(&self, sug: &ProactiveSuggestionRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO proactive_suggestions (
                id, title, content, category, status, score, created_at, responded_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                category = excluded.category,
                status = excluded.status,
                score = excluded.score,
                responded_at = excluded.responded_at",
            rusqlite::params![
                sug.id,
                sug.title,
                sug.content,
                sug.category,
                sug.status,
                sug.score,
                sug.created_at,
                sug.responded_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_proactive_suggestions(
        &self,
        status: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ProactiveSuggestionRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut suggestions = Vec::new();
        if let Some(st) = status {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, category, status, score, created_at, responded_at
                 FROM proactive_suggestions WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(rusqlite::params![st, limit as i64], map_proactive_suggestion)?;
            for r in rows {
                suggestions.push(r?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, category, status, score, created_at, responded_at
                 FROM proactive_suggestions ORDER BY created_at DESC LIMIT ?1",
            )?;
            let rows = stmt.query_map(rusqlite::params![limit as i64], map_proactive_suggestion)?;
            for r in rows {
                suggestions.push(r?);
            }
        }
        Ok(suggestions)
    }

    pub fn respond_proactive_suggestion(&self, id: &str, status: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let count = conn.execute(
            "UPDATE proactive_suggestions SET status = ?1, responded_at = ?2 WHERE id = ?3",
            rusqlite::params![status, now, id],
        )?;
        Ok(count > 0)
    }

    pub fn get_proactive_settings(&self) -> Result<ProactiveSettingsRecord, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, is_enabled, cooldown_minutes, muted_until, updated_at
             FROM proactive_settings WHERE id = 'default'",
        )?;
        let mut rows = stmt.query_map([], map_proactive_settings)?;
        if let Some(r) = rows.next() {
            Ok(r?)
        } else {
            // Default initial settings
            Ok(ProactiveSettingsRecord {
                id: "default".to_string(),
                is_enabled: true,
                cooldown_minutes: 30,
                muted_until: None,
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            })
        }
    }

    pub fn set_proactive_settings(&self, settings: &ProactiveSettingsRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO proactive_settings (id, is_enabled, cooldown_minutes, muted_until, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                is_enabled = excluded.is_enabled,
                cooldown_minutes = excluded.cooldown_minutes,
                muted_until = excluded.muted_until,
                updated_at = excluded.updated_at",
            rusqlite::params![
                settings.id,
                if settings.is_enabled { 1 } else { 0 },
                settings.cooldown_minutes,
                settings.muted_until,
                settings.updated_at,
            ],
        )?;
        Ok(())
    }

    // =========================================================================
    // Stage 10R: Interaction Preference Rules & get_proactive_suggestion
    // =========================================================================

    pub fn get_proactive_suggestion(&self, id: &str) -> Result<Option<ProactiveSuggestionRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, content, category, status, score, created_at, responded_at
             FROM proactive_suggestions WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![id], map_proactive_suggestion)?;
        if let Some(r) = rows.next() { Ok(Some(r?)) } else { Ok(None) }
    }

    pub fn upsert_interaction_pref(&self, pref: &InteractionPrefRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO interaction_prefs (kind, decision, backoff_count, backoff_until, last_shown_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(kind) DO UPDATE SET
                decision = excluded.decision,
                backoff_count = excluded.backoff_count,
                backoff_until = excluded.backoff_until,
                last_shown_at = excluded.last_shown_at,
                updated_at = excluded.updated_at",
            rusqlite::params![
                pref.kind,
                pref.decision,
                pref.backoff_count,
                pref.backoff_until,
                pref.last_shown_at,
                pref.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_interaction_pref(&self, kind: &str) -> Result<Option<InteractionPrefRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT kind, decision, backoff_count, backoff_until, last_shown_at, updated_at
             FROM interaction_prefs WHERE kind = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![kind], map_interaction_pref)?;
        if let Some(r) = rows.next() { Ok(Some(r?)) } else { Ok(None) }
    }

    pub fn list_interaction_prefs(&self) -> Result<Vec<InteractionPrefRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT kind, decision, backoff_count, backoff_until, last_shown_at, updated_at
             FROM interaction_prefs ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], map_interaction_pref)?;
        let mut result = Vec::new();
        for r in rows { result.push(r?); }
        Ok(result)
    }

    // =========================================================================
    // Stage 11R: Pinned Shelf Items & Upserts for Mobile Sync
    // =========================================================================

    pub fn upsert_shelf_item(&self, item: &ShelfItemRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO shelf_items (id, title, content, item_type, is_pinned, source_device, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                item_type = excluded.item_type,
                is_pinned = excluded.is_pinned,
                source_device = excluded.source_device,
                updated_at = excluded.updated_at",
            rusqlite::params![
                item.id,
                item.title,
                item.content,
                item.item_type,
                if item.is_pinned { 1 } else { 0 },
                item.source_device,
                item.created_at,
                item.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_shelf_item(&self, id: &str) -> Result<Option<ShelfItemRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, content, item_type, is_pinned, source_device, created_at, updated_at
             FROM shelf_items WHERE id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_shelf_item(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_shelf_items(&self, pinned_only: bool, limit: usize) -> Result<Vec<ShelfItemRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut items = Vec::new();
        if pinned_only {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, item_type, is_pinned, source_device, created_at, updated_at
                 FROM shelf_items WHERE is_pinned = 1 ORDER BY updated_at DESC LIMIT ?1",
            )?;
            let rows = stmt.query_map(rusqlite::params![limit], map_shelf_item)?;
            for r in rows { items.push(r?); }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, item_type, is_pinned, source_device, created_at, updated_at
                 FROM shelf_items ORDER BY updated_at DESC LIMIT ?1",
            )?;
            let rows = stmt.query_map(rusqlite::params![limit], map_shelf_item)?;
            for r in rows { items.push(r?); }
        }
        Ok(items)
    }

    pub fn delete_shelf_item(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM shelf_items WHERE id = ?1", rusqlite::params![id])?;
        Ok(rows > 0)
    }

    pub fn upsert_routine(&self, routine: &RoutineRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO routines (
                id, title, category, schedule_json, enforcement, allowed_apps_json,
                distraction_rules_json, start_grace_min, escalation_json, current_streak,
                best_streak, last_completed_date, frozen_on_date, active, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                category = excluded.category,
                schedule_json = excluded.schedule_json,
                enforcement = excluded.enforcement,
                allowed_apps_json = excluded.allowed_apps_json,
                distraction_rules_json = excluded.distraction_rules_json,
                start_grace_min = excluded.start_grace_min,
                escalation_json = excluded.escalation_json,
                current_streak = excluded.current_streak,
                best_streak = excluded.best_streak,
                last_completed_date = excluded.last_completed_date,
                frozen_on_date = excluded.frozen_on_date,
                active = excluded.active,
                updated_at = excluded.updated_at",
            rusqlite::params![
                routine.id, routine.title, routine.category, routine.schedule_json, routine.enforcement,
                routine.allowed_apps_json, routine.distraction_rules_json, routine.start_grace_min,
                routine.escalation_json, routine.current_streak, routine.best_streak,
                routine.last_completed_date, routine.frozen_on_date, if routine.active { 1 } else { 0 },
                routine.created_at, routine.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn upsert_commitment(&self, commitment: &CommitmentRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO commitments (id, name, tier, target_apps_json, daily_quota_min, active, pending_weakening_until, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                tier = excluded.tier,
                target_apps_json = excluded.target_apps_json,
                daily_quota_min = excluded.daily_quota_min,
                active = excluded.active,
                pending_weakening_until = excluded.pending_weakening_until,
                updated_at = excluded.updated_at",
            rusqlite::params![
                commitment.id, commitment.name, commitment.tier, commitment.target_apps_json,
                commitment.daily_quota_min, if commitment.active { 1 } else { 0 },
                commitment.pending_weakening_until, commitment.created_at, commitment.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn upsert_productivity_task(&self, task: &ProductivityTaskRecord) -> Result<(), StorageError> {
        self.create_productivity_task(task)
    }

    // =========================================================================
    // Stage U2: Lyrics Cache
    // =========================================================================

    pub fn get_cached_lyrics(&self, cache_key: &str) -> Result<Option<CachedLyricsRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT cache_key, title, artist, album, duration_sec, source,
                    synced_lyrics, parsed_json, plain_lyrics, is_instrumental,
                    user_offset_ms, cached_at, expires_at
             FROM lyrics_cache WHERE cache_key = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![cache_key], map_cached_lyrics)?;
        if let Some(r) = rows.next() {
            let record = r?;
            if let Some(exp) = record.expires_at {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                if exp > 0 && now > exp {
                    return Ok(None);
                }
            }
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    pub fn store_cached_lyrics(&self, record: &CachedLyricsRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO lyrics_cache (
                cache_key, title, artist, album, duration_sec, source,
                synced_lyrics, parsed_json, plain_lyrics, is_instrumental,
                user_offset_ms, cached_at, expires_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(cache_key) DO UPDATE SET
                title = excluded.title,
                artist = excluded.artist,
                album = excluded.album,
                duration_sec = excluded.duration_sec,
                source = excluded.source,
                synced_lyrics = excluded.synced_lyrics,
                parsed_json = excluded.parsed_json,
                plain_lyrics = excluded.plain_lyrics,
                is_instrumental = excluded.is_instrumental,
                user_offset_ms = excluded.user_offset_ms,
                cached_at = excluded.cached_at,
                expires_at = excluded.expires_at",
            rusqlite::params![
                record.cache_key,
                record.title,
                record.artist,
                record.album,
                record.duration_sec as i64,
                record.source,
                record.synced_lyrics,
                record.parsed_json,
                record.plain_lyrics,
                if record.is_instrumental { 1 } else { 0 },
                record.user_offset_ms,
                record.cached_at,
                record.expires_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_lyrics_offset(&self, cache_key: &str, user_offset_ms: i32) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "UPDATE lyrics_cache SET user_offset_ms = ?1 WHERE cache_key = ?2",
            rusqlite::params![user_offset_ms, cache_key],
        )?;
        Ok(count > 0)
    }

    pub fn clear_lyrics_cache(&self) -> Result<usize, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute("DELETE FROM lyrics_cache", [])?;
        Ok(count)
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

// --- Phase 12: Productivity, Accountability & Documents Records ---

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct KanbanBoardRecord {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

fn map_kanban_board(row: &rusqlite::Row<'_>) -> rusqlite::Result<KanbanBoardRecord> {
    Ok(KanbanBoardRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct KanbanColumnRecord {
    pub id: String,
    pub board_id: String,
    pub title: String,
    pub position: i64,
    pub wip_limit: Option<i64>,
}

fn map_kanban_column(row: &rusqlite::Row<'_>) -> rusqlite::Result<KanbanColumnRecord> {
    Ok(KanbanColumnRecord {
        id: row.get(0)?,
        board_id: row.get(1)?,
        title: row.get(2)?,
        position: row.get(3)?,
        wip_limit: row.get(4)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ProductivityTaskRecord {
    pub id: String,
    pub board_id: String,
    pub column_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: String, // "low" | "medium" | "high" | "critical"
    pub status: String,   // "todo" | "in_progress" | "review" | "done"
    pub due_date: Option<i64>,
    pub estimated_minutes: Option<i64>,
    pub actual_minutes: Option<i64>,
    pub labels: Option<String>,
    pub parent_task_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
}

fn map_productivity_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProductivityTaskRecord> {
    Ok(ProductivityTaskRecord {
        id: row.get(0)?,
        board_id: row.get(1)?,
        column_id: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        priority: row.get(5)?,
        status: row.get(6)?,
        due_date: row.get(7)?,
        estimated_minutes: row.get(8)?,
        actual_minutes: row.get(9)?,
        labels: row.get(10)?,
        parent_task_id: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
        completed_at: row.get(14)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct StudyScheduleRecord {
    pub id: String,
    pub title: String,
    pub subject: String,
    pub start_time: i64,
    pub end_time: i64,
    pub priority: String,
    pub status: String,
    pub rescheduled_reason: Option<String>,
}

fn map_study_schedule(row: &rusqlite::Row<'_>) -> rusqlite::Result<StudyScheduleRecord> {
    Ok(StudyScheduleRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        subject: row.get(2)?,
        start_time: row.get(3)?,
        end_time: row.get(4)?,
        priority: row.get(5)?,
        status: row.get(6)?,
        rescheduled_reason: row.get(7)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AccountabilitySettingsRecord {
    pub id: String,
    pub mode: String, // "relaxed" | "balanced" | "strict"
    pub screen_time_opt_in: bool,
    pub last_checkin_at: Option<i64>,
    pub daily_reflection_notes: Option<String>,
}

fn map_accountability_settings(row: &rusqlite::Row<'_>) -> rusqlite::Result<AccountabilitySettingsRecord> {
    let opt_in_int: i64 = row.get(2)?;
    Ok(AccountabilitySettingsRecord {
        id: row.get(0)?,
        mode: row.get(1)?,
        screen_time_opt_in: opt_in_int != 0,
        last_checkin_at: row.get(3)?,
        daily_reflection_notes: row.get(4)?,
    })
}

pub fn epoch_to_date_str(timestamp: i64) -> String {
    let days = timestamp / 86400;
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ScreenTimeSessionRecord {
    pub id: String,
    pub app_name: String,
    #[serde(default)]
    pub window_title: Option<String>,
    pub category: String, // "productive" | "study" | "distracting" / "belajar" | "netral" | "pengalih"
    pub duration_secs: i64,
    pub timestamp: i64,
    #[serde(default)]
    pub date_str: Option<String>,
}

fn map_screen_time_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<ScreenTimeSessionRecord> {
    Ok(ScreenTimeSessionRecord {
        id: row.get(0)?,
        app_name: row.get(1)?,
        window_title: None, // sanitized to maintain SEC-09 privacy
        category: row.get(3)?,
        duration_secs: row.get(4)?,
        timestamp: row.get(5)?,
        date_str: row.get(6).ok(),
    })
}

// =========================================================================
// Stage U3: Routines, Commitments, Enforcement & Emergency Pauses Records
// =========================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RoutineRecord {
    pub id: String,
    pub title: String,
    pub category: String,
    pub schedule_json: String,
    pub enforcement: String,
    pub allowed_apps_json: Option<String>,
    pub distraction_rules_json: Option<String>,
    pub start_grace_min: i64,
    pub escalation_json: Option<String>,
    pub current_streak: i64,
    pub best_streak: i64,
    pub last_completed_date: Option<String>,
    pub frozen_on_date: Option<String>,
    pub active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

fn map_routine(row: &rusqlite::Row<'_>) -> rusqlite::Result<RoutineRecord> {
    let active_int: i64 = row.get(13)?;
    Ok(RoutineRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        category: row.get(2)?,
        schedule_json: row.get(3)?,
        enforcement: row.get(4)?,
        allowed_apps_json: row.get(5)?,
        distraction_rules_json: row.get(6)?,
        start_grace_min: row.get(7)?,
        escalation_json: row.get(8)?,
        current_streak: row.get(9)?,
        best_streak: row.get(10)?,
        last_completed_date: row.get(11)?,
        frozen_on_date: row.get(12)?,
        active: active_int != 0,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CommitmentRecord {
    pub id: String,
    pub name: String,
    pub tier: String, // "ingatkan", "tegas", "paksa"
    pub target_apps_json: String,
    pub daily_quota_min: Option<i64>,
    pub active: bool,
    pub pending_weakening_until: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

fn map_commitment(row: &rusqlite::Row<'_>) -> rusqlite::Result<CommitmentRecord> {
    let active_int: i64 = row.get(5)?;
    Ok(CommitmentRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        tier: row.get(2)?,
        target_apps_json: row.get(3)?,
        daily_quota_min: row.get(4)?,
        active: active_int != 0,
        pending_weakening_until: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct EnforcementHistoryRecord {
    pub id: String,
    pub timestamp: i64,
    pub target_app: String,
    pub tier: String,
    pub action_taken: String,
    pub details: Option<String>,
}

fn map_enforcement_history(row: &rusqlite::Row<'_>) -> rusqlite::Result<EnforcementHistoryRecord> {
    Ok(EnforcementHistoryRecord {
        id: row.get(0)?,
        timestamp: row.get(1)?,
        target_app: row.get(2)?,
        tier: row.get(3)?,
        action_taken: row.get(4)?,
        details: row.get(5)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct EmergencyPauseRecord {
    pub id: String,
    pub started_at: i64,
    pub duration_min: i64,
    pub reason: String,
    pub expired_at: i64,
}

fn map_emergency_pause(row: &rusqlite::Row<'_>) -> rusqlite::Result<EmergencyPauseRecord> {
    Ok(EmergencyPauseRecord {
        id: row.get(0)?,
        started_at: row.get(1)?,
        duration_min: row.get(2)?,
        reason: row.get(3)?,
        expired_at: row.get(4)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DocumentJobRecord {
    pub id: String,
    pub operation: String,
    pub source_path: Option<String>,
    pub target_path: Option<String>,
    pub status: String,
    pub format: String,
    pub metadata: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

fn map_document_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<DocumentJobRecord> {
    Ok(DocumentJobRecord {
        id: row.get(0)?,
        operation: row.get(1)?,
        source_path: row.get(2)?,
        target_path: row.get(3)?,
        status: row.get(4)?,
        format: row.get(5)?,
        metadata: row.get(6)?,
        created_at: row.get(7)?,
        completed_at: row.get(8)?,
    })
}

// --- Phase 13: Browser, Integrations & Developer Workspace Records ---

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BrowserSessionRecord {
    pub id: String,
    pub profile_path: String,
    pub current_url: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub closed_at: Option<i64>,
}

fn map_browser_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<BrowserSessionRecord> {
    let active_int: i32 = row.get(3)?;
    Ok(BrowserSessionRecord {
        id: row.get(0)?,
        profile_path: row.get(1)?,
        current_url: row.get(2)?,
        is_active: active_int != 0,
        created_at: row.get(4)?,
        closed_at: row.get(5)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BrowserHistoryRecord {
    pub id: String,
    pub session_id: String,
    pub url: String,
    pub title: Option<String>,
    pub visited_at: i64,
}

fn map_browser_history(row: &rusqlite::Row<'_>) -> rusqlite::Result<BrowserHistoryRecord> {
    Ok(BrowserHistoryRecord {
        id: row.get(0)?,
        session_id: row.get(1)?,
        url: row.get(2)?,
        title: row.get(3)?,
        visited_at: row.get(4)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BrowserDownloadRecord {
    pub id: String,
    pub session_id: String,
    pub source_url: String,
    pub quarantine_path: String,
    pub file_size: i64,
    pub sha256: String,
    pub status: String,
    pub downloaded_at: i64,
}

fn map_browser_download(row: &rusqlite::Row<'_>) -> rusqlite::Result<BrowserDownloadRecord> {
    Ok(BrowserDownloadRecord {
        id: row.get(0)?,
        session_id: row.get(1)?,
        source_url: row.get(2)?,
        quarantine_path: row.get(3)?,
        file_size: row.get(4)?,
        sha256: row.get(5)?,
        status: row.get(6)?,
        downloaded_at: row.get(7)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DeveloperWorkspaceRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub allowed_commands: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

fn map_developer_workspace(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeveloperWorkspaceRecord> {
    Ok(DeveloperWorkspaceRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        allowed_commands: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DeveloperCommandRunRecord {
    pub id: String,
    pub workspace_id: String,
    pub command: String,
    pub args: String,
    pub exit_code: Option<i32>,
    pub duration_ms: i64,
    pub is_allowed: bool,
    pub output_snippet: Option<String>,
    pub created_at: i64,
}

fn map_developer_command_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeveloperCommandRunRecord> {
    let allowed_int: i32 = row.get(6)?;
    Ok(DeveloperCommandRunRecord {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        command: row.get(2)?,
        args: row.get(3)?,
        exit_code: row.get(4)?,
        duration_ms: row.get(5)?,
        is_allowed: allowed_int != 0,
        output_snippet: row.get(7)?,
        created_at: row.get(8)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct IntegrationAccountRecord {
    pub id: String,
    pub service_name: String,
    pub account_label: String,
    pub auth_type: String,
    pub token_vault_id: String,
    pub scopes: String,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

fn map_integration_account(row: &rusqlite::Row<'_>) -> rusqlite::Result<IntegrationAccountRecord> {
    Ok(IntegrationAccountRecord {
        id: row.get(0)?,
        service_name: row.get(1)?,
        account_label: row.get(2)?,
        auth_type: row.get(3)?,
        token_vault_id: row.get(4)?,
        scopes: row.get(5)?,
        status: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

// ---------------------------------------------------------------------------
// Phase 14 Structs & Mappers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SpecialistRecord {
    pub id: String,
    pub name: String,
    pub role_title: String,
    pub domain: String,
    pub responsibilities: String, // JSON array
    pub system_prompt: String,
    pub allowed_tools: String, // JSON array
    pub max_permission_level: String,
    pub resource_budget: String, // JSON object
    pub is_active: bool,
    pub evaluation_score: f64,
    pub version: String,
    pub created_at: i64,
    pub updated_at: i64,
}

fn map_specialist(row: &rusqlite::Row<'_>) -> rusqlite::Result<SpecialistRecord> {
    let active_int: i32 = row.get(9)?;
    Ok(SpecialistRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        role_title: row.get(2)?,
        domain: row.get(3)?,
        responsibilities: row.get(4)?,
        system_prompt: row.get(5)?,
        allowed_tools: row.get(6)?,
        max_permission_level: row.get(7)?,
        resource_budget: row.get(8)?,
        is_active: active_int != 0,
        evaluation_score: row.get(10)?,
        version: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SpecialistDelegationRecord {
    pub id: String,
    pub specialist_id: String,
    pub task_id: String,
    pub input_prompt: String,
    pub status: String,
    pub output_summary: String,
    pub evidence: String,
    pub tokens_used: i64,
    pub duration_ms: i64,
    pub delegated_at: i64,
    pub completed_at: Option<i64>,
}

fn map_specialist_delegation(row: &rusqlite::Row<'_>) -> rusqlite::Result<SpecialistDelegationRecord> {
    Ok(SpecialistDelegationRecord {
        id: row.get(0)?,
        specialist_id: row.get(1)?,
        task_id: row.get(2)?,
        input_prompt: row.get(3)?,
        status: row.get(4)?,
        output_summary: row.get(5)?,
        evidence: row.get(6)?,
        tokens_used: row.get(7)?,
        duration_ms: row.get(8)?,
        delegated_at: row.get(9)?,
        completed_at: row.get(10)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ProactiveSuggestionRecord {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub status: String,
    pub score: f64,
    pub created_at: i64,
    pub responded_at: Option<i64>,
}

fn map_proactive_suggestion(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProactiveSuggestionRecord> {
    Ok(ProactiveSuggestionRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        category: row.get(3)?,
        status: row.get(4)?,
        score: row.get(5)?,
        created_at: row.get(6)?,
        responded_at: row.get(7)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ProactiveSettingsRecord {
    pub id: String,
    pub is_enabled: bool,
    pub cooldown_minutes: i64,
    pub muted_until: Option<i64>,
    pub updated_at: i64,
}

fn map_proactive_settings(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProactiveSettingsRecord> {
    let enabled_int: i32 = row.get(1)?;
    Ok(ProactiveSettingsRecord {
        id: row.get(0)?,
        is_enabled: enabled_int != 0,
        cooldown_minutes: row.get(2)?,
        muted_until: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct InteractionPrefRecord {
    pub kind: String,
    pub decision: String,
    pub backoff_count: i64,
    pub backoff_until: Option<i64>,
    pub last_shown_at: Option<i64>,
    pub updated_at: i64,
}

fn map_interaction_pref(row: &rusqlite::Row<'_>) -> rusqlite::Result<InteractionPrefRecord> {
    Ok(InteractionPrefRecord {
        kind: row.get(0)?,
        decision: row.get(1)?,
        backoff_count: row.get(2)?,
        backoff_until: row.get(3)?,
        last_shown_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ShelfItemRecord {
    pub id: String,
    pub title: String,
    pub content: String,
    pub item_type: String, // "text" | "file" | "link"
    pub is_pinned: bool,
    pub source_device: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

fn map_shelf_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<ShelfItemRecord> {
    let pinned_int: i32 = row.get(4)?;
    Ok(ShelfItemRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        item_type: row.get(3)?,
        is_pinned: pinned_int != 0,
        source_device: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CachedLyricsRecord {
    pub cache_key: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_sec: u32,
    pub source: String,
    pub synced_lyrics: Option<String>,
    pub parsed_json: Option<String>,
    pub plain_lyrics: Option<String>,
    pub is_instrumental: bool,
    pub user_offset_ms: i32,
    pub cached_at: i64,
    pub expires_at: Option<i64>,
}

fn map_cached_lyrics(row: &rusqlite::Row<'_>) -> rusqlite::Result<CachedLyricsRecord> {
    let dur_i64: i64 = row.get(4)?;
    let inst_int: i32 = row.get(9)?;
    Ok(CachedLyricsRecord {
        cache_key: row.get(0)?,
        title: row.get(1)?,
        artist: row.get(2)?,
        album: row.get(3)?,
        duration_sec: dur_i64 as u32,
        source: row.get(5)?,
        synced_lyrics: row.get(6)?,
        parsed_json: row.get(7)?,
        plain_lyrics: row.get(8)?,
        is_instrumental: inst_int != 0,
        user_offset_ms: row.get(10)?,
        cached_at: row.get(11)?,
        expires_at: row.get(12)?,
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

    #[test]
    fn test_phase12_productivity_and_documents_storage() {
        let db = Database::open_in_memory().unwrap();

        // 1. Kanban Board & Columns
        let board = KanbanBoardRecord {
            id: "board-semester-1".to_string(),
            title: "Academic & Personal Projects".to_string(),
            description: Some("Tracking coursework and development".to_string()),
            created_at: 1000,
            updated_at: 1000,
        };
        db.create_kanban_board(&board).unwrap();

        let boards = db.list_kanban_boards().unwrap();
        assert_eq!(boards.len(), 1);
        assert_eq!(boards[0].id, "board-semester-1");

        let col_todo = KanbanColumnRecord {
            id: "col-todo".to_string(),
            board_id: "board-semester-1".to_string(),
            title: "To Do".to_string(),
            position: 1,
            wip_limit: None,
        };
        let col_done = KanbanColumnRecord {
            id: "col-done".to_string(),
            board_id: "board-semester-1".to_string(),
            title: "Done".to_string(),
            position: 2,
            wip_limit: None,
        };
        db.create_kanban_column(&col_todo).unwrap();
        db.create_kanban_column(&col_done).unwrap();

        let cols = db.list_kanban_columns("board-semester-1").unwrap();
        assert_eq!(cols.len(), 2);

        // 2. Productivity Task
        let task = ProductivityTaskRecord {
            id: "task-p12-01".to_string(),
            board_id: "board-semester-1".to_string(),
            column_id: "col-todo".to_string(),
            title: "Tugas Metodologi Penelitian Bab 1".to_string(),
            description: Some("Latar belakang dan perumusan masalah".to_string()),
            priority: "high".to_string(),
            status: "todo".to_string(),
            due_date: Some(1732000000),
            estimated_minutes: Some(120),
            actual_minutes: None,
            labels: Some("academic,urgent".to_string()),
            parent_task_id: None,
            created_at: 1000,
            updated_at: 1000,
            completed_at: None,
        };
        db.create_productivity_task(&task).unwrap();

        let tasks = db.list_productivity_tasks("board-semester-1").unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Tugas Metodologi Penelitian Bab 1");

        // Move task to Done
        let moved = db.update_productivity_task_column("task-p12-01", "col-done", "done").unwrap();
        assert!(moved);
        let updated_tasks = db.list_productivity_tasks("board-semester-1").unwrap();
        assert_eq!(updated_tasks[0].status, "done");
        assert_eq!(updated_tasks[0].column_id, "col-done");
        assert!(updated_tasks[0].completed_at.is_some());

        // 3. Study Schedules
        let schedule = StudyScheduleRecord {
            id: "sched-01".to_string(),
            title: "Sistem Terdistribusi — Persiapan UTS".to_string(),
            subject: "Distributed Systems".to_string(),
            start_time: 1731600000,
            end_time: 1731607200,
            priority: "critical".to_string(),
            status: "scheduled".to_string(),
            rescheduled_reason: None,
        };
        db.create_study_schedule(&schedule).unwrap();

        let schedules = db.list_study_schedules().unwrap();
        assert_eq!(schedules.len(), 1);
        assert_eq!(schedules[0].subject, "Distributed Systems");

        // 4. Accountability Settings (default vs custom)
        let default_settings = db.get_accountability_settings().unwrap();
        assert_eq!(default_settings.mode, "balanced");
        assert!(!default_settings.screen_time_opt_in);

        let custom_settings = AccountabilitySettingsRecord {
            id: "default".to_string(),
            mode: "strict".to_string(),
            screen_time_opt_in: true,
            last_checkin_at: Some(1500),
            daily_reflection_notes: Some("Productive focus day".to_string()),
        };
        db.set_accountability_settings(&custom_settings).unwrap();

        let fetched_settings = db.get_accountability_settings().unwrap();
        assert_eq!(fetched_settings.mode, "strict");
        assert!(fetched_settings.screen_time_opt_in);

        // 5. Screen Time Tracking (opt-in)
        let session = ScreenTimeSessionRecord {
            id: "st-01".to_string(),
            app_name: "Code.exe".to_string(),
            window_title: Some("Hime Workspace — Rust".to_string()),
            category: "productive".to_string(),
            duration_secs: 1800,
            timestamp: 1600,
            date_str: None,
        };
        db.record_screen_time_session(&session).unwrap();

        let sessions = db.list_screen_time_sessions(10).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].app_name, "Code.exe");

        // Clear sessions
        let cleared = db.clear_screen_time_sessions().unwrap();
        assert_eq!(cleared, 1);
        assert_eq!(db.list_screen_time_sessions(10).unwrap().len(), 0);

        // 6. Document Jobs
        let doc_job = DocumentJobRecord {
            id: "doc-job-01".to_string(),
            operation: "convert".to_string(),
            source_path: Some("report.md".to_string()),
            target_path: Some("report.docx".to_string()),
            status: "completed".to_string(),
            format: "docx".to_string(),
            metadata: Some("{\"template\":\"Academic-ID\"}".to_string()),
            created_at: 2000,
            completed_at: Some(2005),
        };
        db.create_document_job(&doc_job).unwrap();

        let jobs = db.list_document_jobs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].operation, "convert");
        assert_eq!(jobs[0].format, "docx");
    }

    #[test]
    fn test_phase13_browser_and_developer_workspace_storage() {
        let db = Database::open_in_memory().unwrap();

        // 1. Browser session lifecycle
        let session = BrowserSessionRecord {
            id: "sess-01".to_string(),
            profile_path: "C:\\Hime\\profiles\\default".to_string(),
            current_url: Some("https://example.com".to_string()),
            is_active: true,
            created_at: 1000,
            closed_at: None,
        };
        db.create_browser_session(&session).unwrap();

        let active = db.get_active_browser_session().unwrap().unwrap();
        assert_eq!(active.id, "sess-01");
        assert_eq!(active.current_url.as_deref(), Some("https://example.com"));

        db.update_browser_session_url("sess-01", "https://example.com/docs").unwrap();
        let updated = db.get_active_browser_session().unwrap().unwrap();
        assert_eq!(updated.current_url.as_deref(), Some("https://example.com/docs"));

        // History
        let hist = BrowserHistoryRecord {
            id: "hist-01".to_string(),
            session_id: "sess-01".to_string(),
            url: "https://example.com/docs".to_string(),
            title: Some("Example Documentation".to_string()),
            visited_at: 1010,
        };
        db.record_browser_history(&hist).unwrap();
        let history = db.list_browser_history(Some("sess-01"), 10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].title.as_deref(), Some("Example Documentation"));

        // Download
        let dl = BrowserDownloadRecord {
            id: "dl-01".to_string(),
            session_id: "sess-01".to_string(),
            source_url: "https://example.com/report.pdf".to_string(),
            quarantine_path: ".hime/quarantine/dl-01.pdf".to_string(),
            file_size: 20480,
            sha256: "sha256-mock-hash".to_string(),
            status: "quarantined".to_string(),
            downloaded_at: 1020,
        };
        db.record_browser_download(&dl).unwrap();
        let downloads = db.list_browser_downloads(10).unwrap();
        assert_eq!(downloads.len(), 1);
        assert_eq!(downloads[0].status, "quarantined");

        db.close_browser_session("sess-01").unwrap();
        assert!(db.get_active_browser_session().unwrap().is_none());

        // 2. Developer Workspace
        let ws = DeveloperWorkspaceRecord {
            id: "ws-01".to_string(),
            name: "Hime Workspace".to_string(),
            path: "X:\\folder_ai\\Hime".to_string(),
            allowed_commands: Some("git,cargo,pnpm,npm,python".to_string()),
            created_at: 2000,
            updated_at: 2000,
        };
        db.register_developer_workspace(&ws).unwrap();

        let fetched_ws = db.get_developer_workspace_by_path("X:\\folder_ai\\Hime").unwrap().unwrap();
        assert_eq!(fetched_ws.id, "ws-01");
        assert_eq!(fetched_ws.name, "Hime Workspace");

        let run = DeveloperCommandRunRecord {
            id: "run-01".to_string(),
            workspace_id: "ws-01".to_string(),
            command: "cargo".to_string(),
            args: "test".to_string(),
            exit_code: Some(0),
            duration_ms: 1500,
            is_allowed: true,
            output_snippet: Some("test result: ok".to_string()),
            created_at: 2010,
        };
        db.record_developer_command_run(&run).unwrap();

        let runs = db.list_developer_command_runs("ws-01", 10).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].command, "cargo");
        assert!(runs[0].is_allowed);

        // 3. Integration Accounts
        let account = IntegrationAccountRecord {
            id: "acc-gh-01".to_string(),
            service_name: "github".to_string(),
            account_label: "personal-dev".to_string(),
            auth_type: "oauth_device".to_string(),
            token_vault_id: "vault-gh-tok-01".to_string(),
            scopes: "repo,read:user".to_string(),
            status: "active".to_string(),
            created_at: 3000,
            updated_at: 3000,
        };
        db.save_integration_account(&account).unwrap();

        let fetched_acc = db.get_integration_account("github").unwrap().unwrap();
        assert_eq!(fetched_acc.account_label, "personal-dev");
        assert_eq!(fetched_acc.service_name, "github");

        let accounts = db.list_integration_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
    }

    #[test]
    fn test_phase14_specialist_and_proactive_storage() {
        let db = Database::open_in_memory().unwrap();

        // 1. Specialist Registry
        let spec = SpecialistRecord {
            id: "specialist.software_engineer".to_string(),
            name: "Hime Software Engineer".to_string(),
            role_title: "Senior Systems Engineer".to_string(),
            domain: "developer".to_string(),
            responsibilities: "[\"code_review\", \"refactoring\", \"testing\"]".to_string(),
            system_prompt: "You are Hime Software Engineer specialist.".to_string(),
            allowed_tools: "[\"dev.run_command\", \"dev.git_status\"]".to_string(),
            max_permission_level: "L2".to_string(),
            resource_budget: "{\"max_tokens\": 4096, \"timeout_secs\": 60, \"max_concurrency\": 1}".to_string(),
            is_active: true,
            evaluation_score: 94.5,
            version: "1.0.0".to_string(),
            created_at: 1000,
            updated_at: 1000,
        };
        db.upsert_specialist(&spec).unwrap();

        let fetched = db.get_specialist("specialist.software_engineer").unwrap().unwrap();
        assert_eq!(fetched.name, "Hime Software Engineer");
        assert_eq!(fetched.domain, "developer");
        assert!(fetched.is_active);
        assert!((fetched.evaluation_score - 94.5).abs() < f64::EPSILON);

        assert!(db.update_specialist_evaluation("specialist.software_engineer", 98.0).unwrap());
        let updated = db.get_specialist("specialist.software_engineer").unwrap().unwrap();
        assert!((updated.evaluation_score - 98.0).abs() < f64::EPSILON);

        let active_specs = db.list_specialists(true).unwrap();
        assert_eq!(active_specs.len(), 1);

        // 2. Delegations
        let del = SpecialistDelegationRecord {
            id: "del-001".to_string(),
            specialist_id: "specialist.software_engineer".to_string(),
            task_id: "task-999".to_string(),
            input_prompt: "Implement safe boundary check".to_string(),
            status: "running".to_string(),
            output_summary: "Analyzing boundary check".to_string(),
            evidence: "[]".to_string(),
            tokens_used: 120,
            duration_ms: 350,
            delegated_at: 1100,
            completed_at: None,
        };
        db.record_specialist_delegation(&del).unwrap();

        let fetched_del = db.get_specialist_delegation("del-001").unwrap().unwrap();
        assert_eq!(fetched_del.status, "running");

        assert!(db.complete_specialist_delegation(
            "del-001",
            "completed",
            "Boundary check implemented and passing all tests",
            "[\"crates/hime-core/src/developer/workspace.rs:45\"]",
            520,
            1200
        ).unwrap());

        let completed_del = db.get_specialist_delegation("del-001").unwrap().unwrap();
        assert_eq!(completed_del.status, "completed");
        assert_eq!(completed_del.tokens_used, 520);
        assert!(completed_del.completed_at.is_some());

        let list_del = db.list_specialist_delegations(Some("specialist.software_engineer"), 10).unwrap();
        assert_eq!(list_del.len(), 1);

        // 3. Proactive Suggestions & Settings
        let sug = ProactiveSuggestionRecord {
            id: "sug-001".to_string(),
            title: "Tinjau Jadwal Belajar".to_string(),
            content: "Ada blok belajar Skripsi Bab 4 dalam 15 menit.".to_string(),
            category: "schedule".to_string(),
            status: "pending".to_string(),
            score: 0.95,
            created_at: 1200,
            responded_at: None,
        };
        db.insert_proactive_suggestion(&sug).unwrap();

        let pending_sugs = db.list_proactive_suggestions(Some("pending"), 10).unwrap();
        assert_eq!(pending_sugs.len(), 1);
        assert_eq!(pending_sugs[0].title, "Tinjau Jadwal Belajar");

        assert!(db.respond_proactive_suggestion("sug-001", "accepted").unwrap());
        let updated_sugs = db.list_proactive_suggestions(Some("accepted"), 10).unwrap();
        assert_eq!(updated_sugs.len(), 1);

        let default_proact = db.get_proactive_settings().unwrap();
        assert!(default_proact.is_enabled);
        assert_eq!(default_proact.cooldown_minutes, 30);

        let custom_proact = ProactiveSettingsRecord {
            id: "default".to_string(),
            is_enabled: true,
            cooldown_minutes: 45,
            muted_until: Some(1500),
            updated_at: 1300,
        };
        db.set_proactive_settings(&custom_proact).unwrap();

        let fetched_proact = db.get_proactive_settings().unwrap();
        assert_eq!(fetched_proact.cooldown_minutes, 45);
        assert_eq!(fetched_proact.muted_until, Some(1500));
    }

    #[test]
    fn test_stage_u2_lyrics_cache() {
        let db = Database::open_in_memory().unwrap();
        let record = CachedLyricsRecord {
            cache_key: "abc123hash".to_string(),
            title: "Hikaru Nara".to_string(),
            artist: "Goose house".to_string(),
            album: Some("Shigatsu wa Kimi no Uso".to_string()),
            duration_sec: 254,
            source: "lrclib".to_string(),
            synced_lyrics: Some("[00:01.00]Niji-iro no fantaji".to_string()),
            parsed_json: Some("[{\"ms\":1000,\"text\":\"Niji-iro no fantaji\"}]".to_string()),
            plain_lyrics: Some("Niji-iro no fantaji".to_string()),
            is_instrumental: false,
            user_offset_ms: 0,
            cached_at: 1000,
            expires_at: Some(2000000000),
        };

        // 1. Store
        db.store_cached_lyrics(&record).unwrap();

        // 2. Retrieve
        let retrieved = db.get_cached_lyrics("abc123hash").unwrap().expect("found cached lyrics");
        assert_eq!(retrieved.title, "Hikaru Nara");
        assert_eq!(retrieved.artist, "Goose house");
        assert_eq!(retrieved.album.as_deref(), Some("Shigatsu wa Kimi no Uso"));
        assert_eq!(retrieved.duration_sec, 254);
        assert_eq!(retrieved.user_offset_ms, 0);

        // 3. Update offset
        let updated = db.update_lyrics_offset("abc123hash", 500).unwrap();
        assert!(updated);
        let retrieved2 = db.get_cached_lyrics("abc123hash").unwrap().unwrap();
        assert_eq!(retrieved2.user_offset_ms, 500);

        // 4. Expiry logic
        let expired_record = CachedLyricsRecord {
            cache_key: "expired_key".to_string(),
            title: "Old Song".to_string(),
            artist: "Old Artist".to_string(),
            album: None,
            duration_sec: 180,
            source: "not_found".to_string(),
            synced_lyrics: None,
            parsed_json: None,
            plain_lyrics: None,
            is_instrumental: false,
            user_offset_ms: 0,
            cached_at: 10,
            expires_at: Some(50), // expired in the past relative to unix epoch now
        };
        db.store_cached_lyrics(&expired_record).unwrap();
        assert!(db.get_cached_lyrics("expired_key").unwrap().is_none());

        // 5. Clear cache
        let count = db.clear_lyrics_cache().unwrap();
        assert_eq!(count, 2);
        assert!(db.get_cached_lyrics("abc123hash").unwrap().is_none());
    }

    #[test]
    fn test_stage_u3_routines_commitments_and_enforcement_storage() {
        let db = Database::open_in_memory().unwrap();

        // 1. Routines CRUD & streak
        let routine = RoutineRecord {
            id: "rtn-01".to_string(),
            title: "Belajar Skripsi".to_string(),
            category: "belajar".to_string(),
            schedule_json: "{\"days\":[\"Monday\",\"Tuesday\"],\"time\":\"19:00\",\"duration_min\":45}".to_string(),
            enforcement: "tegas".to_string(),
            allowed_apps_json: Some("[\"Code.exe\",\"word.exe\"]".to_string()),
            distraction_rules_json: Some("{\"block_during_routine\":true,\"distraction_apps\":[\"chrome.exe\"]}".to_string()),
            start_grace_min: 5,
            escalation_json: Some("{\"enabled\":true,\"missed_threshold\":3}".to_string()),
            current_streak: 2,
            best_streak: 5,
            last_completed_date: Some("2026-11-20".to_string()),
            frozen_on_date: None,
            active: true,
            created_at: 1000,
            updated_at: 1000,
        };
        db.create_routine(&routine).unwrap();

        let fetched = db.get_routine("rtn-01").unwrap().expect("routine found");
        assert_eq!(fetched.title, "Belajar Skripsi");
        assert_eq!(fetched.enforcement, "tegas");
        assert_eq!(fetched.current_streak, 2);

        // Update streak
        assert!(db.update_routine_streak("rtn-01", 3, 5, Some("2026-11-21"), None).unwrap());
        let updated = db.get_routine("rtn-01").unwrap().unwrap();
        assert_eq!(updated.current_streak, 3);
        assert_eq!(updated.last_completed_date.as_deref(), Some("2026-11-21"));

        let list = db.list_routines(true).unwrap();
        assert_eq!(list.len(), 1);

        // 2. Commitments CRUD & weakening
        let commitment = CommitmentRecord {
            id: "com-01".to_string(),
            name: "Blokir Game".to_string(),
            tier: "paksa".to_string(),
            target_apps_json: "[\"steam.exe\",\"epicgames.exe\"]".to_string(),
            daily_quota_min: Some(60),
            active: true,
            pending_weakening_until: None,
            created_at: 1000,
            updated_at: 1000,
        };
        db.create_commitment(&commitment).unwrap();

        let fetched_com = db.get_commitment("com-01").unwrap().expect("commitment found");
        assert_eq!(fetched_com.name, "Blokir Game");
        assert_eq!(fetched_com.tier, "paksa");
        assert!(fetched_com.pending_weakening_until.is_none());

        assert!(db.request_commitment_weakening("com-01", 1600).unwrap());
        let weakened = db.get_commitment("com-01").unwrap().unwrap();
        assert_eq!(weakened.pending_weakening_until, Some(1600));

        let com_list = db.list_commitments(false).unwrap();
        assert_eq!(com_list.len(), 1);

        // 3. Enforcement history
        let event = EnforcementHistoryRecord {
            id: "enf-01".to_string(),
            timestamp: 1200,
            target_app: "steam.exe".to_string(),
            tier: "paksa".to_string(),
            action_taken: "warned".to_string(),
            details: Some("Warning countdown started".to_string()),
        };
        db.record_enforcement_event(&event).unwrap();
        let history = db.list_enforcement_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].target_app, "steam.exe");

        // 4. Emergency Pauses
        let pause = EmergencyPauseRecord {
            id: "pause-01".to_string(),
            started_at: 1000,
            duration_min: 30,
            reason: "Sakit".to_string(),
            expired_at: 2800,
        };
        db.record_emergency_pause(&pause).unwrap();

        let active_pause = db.get_active_emergency_pause(1500).unwrap().expect("active pause found");
        assert_eq!(active_pause.reason, "Sakit");
        assert_eq!(active_pause.duration_min, 30);

        // Check expired pause
        assert!(db.get_active_emergency_pause(3000).unwrap().is_none());

        // 5. Screen time privacy sanitization verification
        let st_session = ScreenTimeSessionRecord {
            id: "st-priv-01".to_string(),
            app_name: "Chrome.exe".to_string(),
            window_title: Some("https://mybank.com/sensitive-account-details".to_string()),
            category: "pengalih".to_string(),
            duration_secs: 300,
            timestamp: 1700000000,
            date_str: None,
        };
        db.record_screen_time_session(&st_session).unwrap();

        let st_list = db.list_screen_time_sessions(5).unwrap();
        assert_eq!(st_list.len(), 1);
        assert_eq!(st_list[0].app_name, "Chrome.exe");
        // Crucial SEC-09 assertion: sensitive window title MUST be stripped/sanitized
        assert!(st_list[0].window_title.is_none());
        assert_eq!(st_list[0].date_str.as_deref(), Some("2023-11-14"));
    }

    #[test]
    fn test_epoch_to_date_str_roundtrip_30000_days() {
        use super::epoch_to_date_str;

        // Verify across 0..30000 days (1970 to 2052) without any panic or overflow
        for day in 0..30000 {
            let ts = day * 86400;
            let s = epoch_to_date_str(ts);
            assert_eq!(s.len(), 10, "Formatted date string must be YYYY-MM-DD for day {}", day);
        }

        // Explicit critical boundary assertions
        assert_eq!(epoch_to_date_str(20512 * 86400), "2026-02-28");
        assert_eq!(epoch_to_date_str(20513 * 86400), "2026-03-01");
        assert_eq!(epoch_to_date_str(19782 * 86400), "2024-02-29");
        assert_eq!(epoch_to_date_str(11016 * 86400), "2000-02-29");
    }

    #[test]
    fn test_sqlite_cache_tuning_pragmas() {
        // 1. In-memory database
        let db = Database::open_in_memory().unwrap();
        let (cache_size, mmap_size) = db.get_resource_tuning_pragmas().unwrap();
        assert_eq!(cache_size, -2000, "cache_size must be configured to -2000 (~2MB)");
        assert_eq!(mmap_size, 0, "mmap_size must be configured to 0 for resource optimization");

        // 2. File-backed database
        let temp_dir = std::env::temp_dir().join(format!("hime_db_tune_{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_file = temp_dir.join("tuned.db");
        let file_db = Database::open(&db_file).unwrap();
        let (f_cache, f_mmap) = file_db.get_resource_tuning_pragmas().unwrap();
        assert_eq!(f_cache, -2000);
        assert_eq!(f_mmap, 0);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
