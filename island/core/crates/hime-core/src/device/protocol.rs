use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPosture {
    pub is_device_secure: bool,
    pub security_patch_level: String,
    pub storage_encrypted: bool,
    pub unknown_sources_enabled: bool,
    pub os_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileDeviceStatus {
    pub device_id: String,
    pub battery_percent: u8,
    pub is_charging: bool,
    pub storage_free_bytes: i64,
    pub storage_total_bytes: i64,
    pub security_posture: SecurityPosture,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteApprovalSubmission {
    pub ticket_id: String,
    pub decision: String, // "approve" | "deny"
    pub device_id: String,
    pub signed_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteChatSubmission {
    pub conversation_id: String,
    pub prompt: String,
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferChunkPayload {
    pub transfer_id: String,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub data_base64: String,
    pub chunk_sha256: String,
}

