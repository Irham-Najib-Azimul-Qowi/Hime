use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    CoreStarted {
        version: String,
    },
    ClientConnected {
        client_id: String,
    },
    ClientDisconnected {
        client_id: String,
    },
    SubprocessSpawned {
        name: String,
        pid: u32,
    },
    SubprocessTerminated {
        name: String,
        pid: u32,
        exit_code: Option<u32>,
    },
    AgentStateChanged {
        new_state: String,
    },
    AgentStreamChunk {
        delta: String,
    },
    TaskUpdated {
        task_id: String,
        status: String,
    },
    ApprovalResolved {
        request_id: String,
        decision: String,
    },
    DeviceConnected {
        device_id: String,
        device_name: String,
    },
    DeviceDisconnected {
        device_id: String,
    },
    DeviceRevoked {
        device_id: String,
    },
    FileTransferProgress {
        transfer_id: String,
        bytes_transferred: i64,
        total_bytes: i64,
    },
    ShutdownInitiated,
}

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<SystemEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(128)
    }
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: SystemEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.sender.subscribe()
    }
}
