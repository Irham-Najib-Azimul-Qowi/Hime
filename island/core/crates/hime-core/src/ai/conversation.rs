use std::sync::Arc;
use hime_storage::{ConversationRecord, Database, MessageRecord, StorageError};

/// The Conversation Service managing persistent multi-turn chat threads in Hime Core SQLite WAL.
pub struct ConversationService {
    storage: Arc<Database>,
}

impl ConversationService {
    pub fn new(storage: Arc<Database>) -> Self {
        Self { storage }
    }

    pub fn create_conversation(
        &self,
        title: &str,
        model: &str,
    ) -> Result<ConversationRecord, StorageError> {
        let id = format!("conv-{}", uuid_simple());
        self.storage.create_conversation(&id, title, model)
    }

    pub fn list_conversations(&self) -> Result<Vec<ConversationRecord>, StorageError> {
        self.storage.list_conversations()
    }

    pub fn get_messages(&self, conversation_id: &str) -> Result<Vec<MessageRecord>, StorageError> {
        self.storage.get_messages(conversation_id)
    }

    pub fn add_user_message(
        &self,
        conversation_id: &str,
        content: &str,
    ) -> Result<MessageRecord, StorageError> {
        let msg_id = format!("msg-{}", uuid_simple());
        self.storage.add_message(&msg_id, conversation_id, "user", content, None)
    }

    pub fn add_assistant_message(
        &self,
        conversation_id: &str,
        content: &str,
        token_count: Option<i64>,
    ) -> Result<MessageRecord, StorageError> {
        let msg_id = format!("msg-{}", uuid_simple());
        self.storage
            .add_message(&msg_id, conversation_id, "assistant", content, token_count)
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", nanos)
}

