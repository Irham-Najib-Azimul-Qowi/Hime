use rusqlite::{Connection, OpenFlags};
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
                payload TEXT NOT NULL
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
            );",
        )?;

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

        conn.execute(
            "INSERT INTO audit_events (timestamp, event_type, source, payload)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![now, event_type, source, payload],
        )?;

        let id = conn.last_insert_rowid();
        Ok(id)
    }

    pub fn get_audit_events(&self, limit: usize) -> Result<Vec<AuditEventRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, source, payload
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
            })
        })?;

        let mut events = Vec::new();
        for r in rows {
            events.push(r?);
        }
        Ok(events)
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
}
