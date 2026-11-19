use async_trait::async_trait;
use hime_security::{Action, DpapiError, PolicyDecision};
use hime_storage::StorageError;

#[async_trait]
pub trait StoragePort: Send + Sync {
    async fn set_metadata(&self, key: &str, value: &str) -> Result<(), StorageError>;
    async fn get_metadata(&self, key: &str) -> Result<Option<String>, StorageError>;
}

pub trait VaultPort: Send + Sync {
    fn protect(&self, plaintext: &[u8]) -> Result<Vec<u8>, DpapiError>;
    fn unprotect(&self, ciphertext: &[u8]) -> Result<Vec<u8>, DpapiError>;
}

pub trait PolicyPort: Send + Sync {
    fn evaluate_action(&self, client_id: &str, action: &Action) -> PolicyDecision;
}
