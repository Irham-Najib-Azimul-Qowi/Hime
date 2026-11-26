use crate::server::ApprovalRecord;
use hime_storage::{CommitmentRecord, ProductivityTaskRecord, RoutineRecord, ShelfItemRecord};
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

// ---------------------------------------------------------------------------
// Stage 11R: Mobile Sync, Screen-Time & Enforcement Protocol Payloads
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShelfItemSync {
    pub id: String,
    pub title: String,
    pub content: String,
    pub item_type: String, // "text" | "file" | "link"
    pub is_pinned: bool,
    pub source_device: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<ShelfItemRecord> for ShelfItemSync {
    fn from(r: ShelfItemRecord) -> Self {
        Self {
            id: r.id,
            title: r.title,
            content: r.content,
            item_type: r.item_type,
            is_pinned: r.is_pinned,
            source_device: r.source_device,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

impl From<ShelfItemSync> for ShelfItemRecord {
    fn from(s: ShelfItemSync) -> Self {
        Self {
            id: s.id,
            title: s.title,
            content: s.content,
            item_type: s.item_type,
            is_pinned: s.is_pinned,
            source_device: s.source_device,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncPayload {
    pub device_id: String,
    pub timestamp: i64,
    #[serde(default)]
    pub tasks: Vec<ProductivityTaskRecord>,
    #[serde(default)]
    pub routines: Vec<RoutineRecord>,
    #[serde(default)]
    pub commitments: Vec<CommitmentRecord>,
    #[serde(default)]
    pub pinned_shelf: Vec<ShelfItemSync>,
    #[serde(default)]
    pub approvals: Vec<ApprovalRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncResponse {
    pub success: bool,
    pub timestamp: i64,
    pub server_tasks: Vec<ProductivityTaskRecord>,
    pub server_routines: Vec<RoutineRecord>,
    pub server_commitments: Vec<CommitmentRecord>,
    pub server_pinned_shelf: Vec<ShelfItemSync>,
    pub server_approvals: Vec<ApprovalRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileScreenTimeEntry {
    pub package_name: String,
    pub app_name: String,
    pub category: String, // "belajar" | "netral" | "pengalih" | "productive" | "study" | "distracting"
    pub duration_secs: i64,
    pub timestamp: i64,
    #[serde(default)]
    pub date_str: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenTimeSyncPayload {
    pub device_id: String,
    pub timestamp: i64,
    pub entries: Vec<MobileScreenTimeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenTimeSyncResponse {
    pub success: bool,
    pub entries_recorded: usize,
    pub total_today_study_secs: i64,
    pub total_today_distracting_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEnforcementRequest {
    pub device_id: String,
    #[serde(default)]
    pub active_routine_id: Option<String>,
    #[serde(default)]
    pub reported_action: Option<String>,
    #[serde(default)]
    pub app_package: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEnforcementResponse {
    pub tier: String, // "ingatkan" | "tegas" | "paksa"
    pub is_study_block_active: bool,
    pub emergency_pause_active: bool,
    pub emergency_pause_remaining_secs: u64,
    pub protected_packages: Vec<String>,
    pub distracting_packages: Vec<String>,
    pub daily_quota_remaining_mins: Option<i64>,
}

