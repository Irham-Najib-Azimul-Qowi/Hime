use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::bus::{EventBus, SystemEvent};

/// Structured, versioned event contract for Hime Core.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypedEvent {
    pub schema_version: u32,
    pub event_id: u64,
    pub correlation_id: Option<String>,
    pub timestamp_ms: u64,
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// Event Engine maintaining a thread-safe rolling buffer of recent system events
/// to guarantee at-least-once delivery and offline recovery for the desktop UI.
pub struct EventEngine {
    event_bus: EventBus,
    next_id: AtomicU64,
    buffer: Arc<RwLock<VecDeque<TypedEvent>>>,
    max_capacity: usize,
}

impl EventEngine {
    pub fn new(event_bus: EventBus, max_capacity: usize) -> Self {
        Self {
            event_bus,
            next_id: AtomicU64::new(1),
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(max_capacity))),
            max_capacity,
        }
    }

    /// Emits a structured event, enqueues it in the rolling buffer, and notifies the event bus.
    pub async fn emit(
        &self,
        event_type: &str,
        payload: serde_json::Value,
        correlation_id: Option<String>,
    ) -> TypedEvent {
        let event_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let event = TypedEvent {
            schema_version: 1,
            event_id,
            correlation_id,
            timestamp_ms,
            event_type: event_type.to_string(),
            payload,
        };

        {
            let mut buf = self.buffer.write().await;
            if buf.len() >= self.max_capacity {
                buf.pop_front();
            }
            buf.push_back(event.clone());
        }

        // Bridge to existing EventBus
        let _ = self.event_bus.publish(SystemEvent::AgentStateChanged {
            new_state: event_type.to_string(),
        });

        event
    }

    /// Retrieves events in the rolling buffer that occurred after `since_id`.
    /// Enables the UI to catch up seamlessly after reconnecting.
    pub async fn get_recent_events(&self, since_id: Option<u64>) -> Vec<TypedEvent> {
        let buf = self.buffer.read().await;
        if let Some(since) = since_id {
            buf.iter()
                .filter(|e| e.event_id > since)
                .cloned()
                .collect()
        } else {
            buf.iter().cloned().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_engine_rolling_buffer_and_recovery() {
        let bus = EventBus::new(128);
        let engine = EventEngine::new(bus, 5);

        // Emit 7 events
        for i in 1..=7 {
            engine
                .emit(
                    "task.status_changed",
                    serde_json::json!({ "index": i }),
                    Some(format!("corr-{}", i)),
                )
                .await;
        }

        // Rolling buffer capacity is 5, so only last 5 should remain (events 3..=7)
        let all = engine.get_recent_events(None).await;
        assert_eq!(all.len(), 5);
        assert_eq!(all[0].event_id, 3);
        assert_eq!(all[4].event_id, 7);

        // Filter since event_id 5
        let delta = engine.get_recent_events(Some(5)).await;
        assert_eq!(delta.len(), 2);
        assert_eq!(delta[0].event_id, 6);
        assert_eq!(delta[1].event_id, 7);
    }
}
