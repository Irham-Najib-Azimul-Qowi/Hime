use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Unauthenticated,
    Authenticated,
    Active,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub auth_token: String,
    pub client_id: String,
    pub client_type: String, // "desktop", "hermes", "tool"
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub session_id: String,
    pub reconnect_ticket: String,
    pub valid_until: u64,
}

#[derive(Debug)]
pub struct ScopedNonceTracker {
    last_request_nonce: AtomicU64,
    stream_sequences: Mutex<HashMap<String, u64>>,
}

impl Default for ScopedNonceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopedNonceTracker {
    pub fn new() -> Self {
        Self {
            last_request_nonce: AtomicU64::new(0),
            stream_sequences: Mutex::new(HashMap::new()),
        }
    }

    /// Validates and records a client request nonce.
    /// Strictly enforces N > N_last to protect against replay attacks.
    pub fn validate_and_advance_request_nonce(&self, nonce: u64) -> Result<(), NonceError> {
        let current = self.last_request_nonce.load(Ordering::SeqCst);
        if nonce <= current {
            return Err(NonceError::ReplayDetected {
                received: nonce,
                last_seen: current,
            });
        }
        self.last_request_nonce.store(nonce, Ordering::SeqCst);
        Ok(())
    }

    /// Validates chunk sequence order for a streaming request (0, 1, 2...).
    pub fn validate_and_advance_stream_chunk(
        &self,
        request_id: &str,
        chunk_seq: u64,
    ) -> Result<(), NonceError> {
        let mut streams = self.stream_sequences.lock().unwrap();
        let expected = streams.get(request_id).copied().unwrap_or(0);
        if chunk_seq != expected {
            return Err(NonceError::StreamChunkOutOfOrder {
                request_id: request_id.to_string(),
                received: chunk_seq,
                expected,
            });
        }
        streams.insert(request_id.to_string(), expected + 1);
        Ok(())
    }

    pub fn complete_stream(&self, request_id: &str) {
        let mut streams = self.stream_sequences.lock().unwrap();
        streams.remove(request_id);
    }
}

#[derive(Error, Debug)]
pub enum NonceError {
    #[error("Replay attack detected: received nonce {received} <= last seen {last_seen}")]
    ReplayDetected { received: u64, last_seen: u64 },

    #[error("Stream chunk out of order for request '{request_id}': received seq {received}, expected {expected}")]
    StreamChunkOutOfOrder {
        request_id: String,
        received: u64,
        expected: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monotonic_nonce_advancement() {
        let tracker = ScopedNonceTracker::new();
        assert!(tracker.validate_and_advance_request_nonce(1).is_ok());
        assert!(tracker.validate_and_advance_request_nonce(5).is_ok());
        assert!(tracker.validate_and_advance_request_nonce(6).is_ok());

        // Replay of earlier nonce
        assert!(tracker.validate_and_advance_request_nonce(5).is_err());
        // Same nonce
        assert!(tracker.validate_and_advance_request_nonce(6).is_err());
    }

    #[test]
    fn test_stream_chunk_ordering() {
        let tracker = ScopedNonceTracker::new();
        assert!(tracker
            .validate_and_advance_stream_chunk("req-1", 0)
            .is_ok());
        assert!(tracker
            .validate_and_advance_stream_chunk("req-1", 1)
            .is_ok());
        assert!(tracker
            .validate_and_advance_stream_chunk("req-1", 2)
            .is_ok());

        // Out of order
        assert!(tracker
            .validate_and_advance_stream_chunk("req-1", 4)
            .is_err());
    }
}
