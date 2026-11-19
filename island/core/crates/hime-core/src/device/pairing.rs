use hime_security::csprng::generate_hex_token;
use hime_storage::{Database, PairedDeviceRecord};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum PairingError {
    #[error("Pairing token expired or invalid")]
    InvalidToken,

    #[error("Confirmation code mismatch")]
    CodeMismatch,

    #[error("Device already revoked")]
    DeviceRevoked,

    #[error("Storage error: {0}")]
    Storage(#[from] hime_storage::StorageError),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QrPairingPayload {
    pub server_address: String,
    pub server_port: u16,
    pub ca_fingerprint: String,
    pub pairing_token: String,
    pub confirmation_code: String,
    pub expires_at: i64,
}

pub struct PairingManager {
    db: Arc<Database>,
    server_address: String,
    server_port: u16,
    ca_fingerprint: String,
}

impl PairingManager {
    pub fn new(
        db: Arc<Database>,
        server_address: String,
        server_port: u16,
        ca_fingerprint: String,
    ) -> Self {
        Self {
            db,
            server_address,
            server_port,
            ca_fingerprint,
        }
    }

    /// Generates a new short-lived pairing session (5 minutes TTL) and returns QR payload.
    pub fn create_pairing_session(&self, ttl_secs: u64) -> Result<QrPairingPayload, PairingError> {
        let token = generate_hex_token(16).unwrap_or_else(|_| "pairing-tok-fallback".to_string());
        // Generate a 6-digit confirmation code (e.g. "849201")
        let code_num = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() % 900000) + 100000;
        let confirmation_code = format!("{:06}", code_num);

        let rec = self.db.create_pairing_token(&token, &confirmation_code, ttl_secs)?;
        info!("Pairing session initiated: token={}, expires_at={}", rec.token, rec.expires_at);

        Ok(QrPairingPayload {
            server_address: self.server_address.clone(),
            server_port: self.server_port,
            ca_fingerprint: self.ca_fingerprint.clone(),
            pairing_token: rec.token,
            confirmation_code: rec.confirmation_code,
            expires_at: rec.expires_at,
        })
    }

    /// Verifies the pairing token and confirmation code, then registers the device.
    pub fn complete_pairing(
        &self,
        token: &str,
        confirmation_code: &str,
        device_id: &str,
        device_name: &str,
        device_type: &str,
        client_cert_fingerprint: &str,
    ) -> Result<PairedDeviceRecord, PairingError> {
        use hime_storage::TokenValidationResult;
        match self.db.validate_and_consume_pairing_token(token, confirmation_code)? {
            TokenValidationResult::Valid => {}
            TokenValidationResult::CodeMismatch => return Err(PairingError::CodeMismatch),
            TokenValidationResult::InvalidOrExpired => return Err(PairingError::InvalidToken),
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let dev = PairedDeviceRecord {
            id: device_id.to_string(),
            device_name: device_name.to_string(),
            device_type: device_type.to_string(),
            cert_fingerprint: client_cert_fingerprint.to_string(),
            paired_at: now,
            last_seen_at: now,
            status: "active".to_string(),
        };

        self.db.add_paired_device(&dev)?;
        info!("Device successfully paired: {} ({})", dev.device_name, dev.id);

        Ok(dev)
    }

    /// Checks if a device is registered and not revoked.
    pub fn verify_device_active(&self, device_id: &str, cert_fingerprint: &str) -> Result<bool, PairingError> {
        let dev = self.db.get_paired_device(device_id)?;
        match dev {
            Some(d) => {
                if d.status == "revoked" {
                    return Err(PairingError::DeviceRevoked);
                }
                let valid = d.cert_fingerprint == cert_fingerprint;
                if valid {
                    let _ = self.db.update_device_last_seen(device_id);
                }
                Ok(valid)
            }
            None => Ok(false),
        }
    }

    /// Instantly revokes a paired device.
    pub fn revoke_device(&self, device_id: &str) -> Result<bool, PairingError> {
        let ok = self.db.revoke_device(device_id)?;
        if ok {
            info!("Device {} has been revoked", device_id);
        }
        Ok(ok)
    }
}
