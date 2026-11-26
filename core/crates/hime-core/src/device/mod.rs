pub mod pairing;
pub mod pki;
pub mod protocol;
pub mod server;
pub mod transfer;

pub use pairing::{PairingError, PairingManager, QrPairingPayload};
pub use pki::{IssuedCertificate, LocalPkiAuthority, PkiError};
pub use protocol::{
    DeviceEnforcementRequest, DeviceEnforcementResponse, DeviceSyncPayload, DeviceSyncResponse,
    MobileDeviceStatus, MobileScreenTimeEntry, RemoteApprovalSubmission, RemoteChatSubmission,
    ScreenTimeSyncPayload, ScreenTimeSyncResponse, SecurityPosture, ShelfItemSync,
    TransferChunkPayload,
};
pub use server::DeviceGatewayServer;
pub use transfer::{TransferEngine, TransferError};

use crate::bus::EventBus;
use crate::server::ApprovalRecord;
use crate::storage::operations::SafeFileOperationsEngine;
use hime_storage::Database;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DeviceManager {
    pub db: Arc<Database>,
    pub pki: Arc<LocalPkiAuthority>,
    pub pairing: Arc<PairingManager>,
    pub transfer: Arc<TransferEngine>,
    pub gateway: Arc<DeviceGatewayServer>,
}

impl DeviceManager {
    pub fn new(
        db: Arc<Database>,
        ops_engine: Arc<SafeFileOperationsEngine>,
        event_bus: EventBus,
        approvals: Arc<Mutex<HashMap<String, ApprovalRecord>>>,
        base_hime_dir: &Path,
        server_address: &str,
        server_port: u16,
    ) -> Result<Self, TransferError> {
        let transfer = Arc::new(TransferEngine::new(db.clone(), ops_engine, base_hime_dir)?);
        let gateway = Arc::new(DeviceGatewayServer::new(
            db.clone(),
            event_bus,
            approvals,
            transfer.clone(),
            server_address,
            server_port,
        ));
        let pki = gateway.pki_authority();
        let pairing = gateway.pairing_manager();

        Ok(Self {
            db,
            pki,
            pairing,
            transfer,
            gateway,
        })
    }
}
