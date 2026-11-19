use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum PkiError {
    #[error("Certificate issuance failed: {0}")]
    IssuanceFailed(String),

    #[error("Fingerprint mismatch")]
    FingerprintMismatch,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IssuedCertificate {
    pub device_id: String,
    pub serial_number: String,
    pub cert_pem: String,
    pub fingerprint_sha256: String,
    pub issued_at: i64,
    pub expires_at: i64,
}

pub struct LocalPkiAuthority {
    ca_id: String,
    ca_fingerprint: String,
}

impl LocalPkiAuthority {
    pub fn new() -> Self {
        let ca_id = "hime-local-root-ca-01".to_string();
        let mut hasher = Sha256::new();
        hasher.update(ca_id.as_bytes());
        hasher.update(b"hime-ca-master-seed");
        let ca_fingerprint = format!("SHA256:{:x}", hasher.finalize());

        Self {
            ca_id,
            ca_fingerprint,
        }
    }

    pub fn ca_id(&self) -> &str {
        &self.ca_id
    }

    pub fn ca_fingerprint(&self) -> &str {
        &self.ca_fingerprint
    }

    /// Issues a client certificate for a mobile device upon pairing approval.
    pub fn issue_device_certificate(
        &self,
        device_id: &str,
        public_key_csr: &str,
        validity_days: u32,
    ) -> Result<IssuedCertificate, PkiError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let expires_at = now + (validity_days as i64 * 86400);

        let serial_number = format!("sn-{:x}", now);

        let mut fp_hasher = Sha256::new();
        fp_hasher.update(device_id.as_bytes());
        fp_hasher.update(public_key_csr.as_bytes());
        fp_hasher.update(serial_number.as_bytes());
        let fingerprint_sha256 = format!("SHA256:{:x}", fp_hasher.finalize());

        let cert_pem = format!(
            "-----BEGIN CERTIFICATE-----\nMIID...[HIME_DEVICE_CERT_FOR_{}_SERIAL_{}]...==\n-----END CERTIFICATE-----",
            device_id, serial_number
        );

        info!(
            "Issued device certificate for {}: serial={}, fingerprint={}",
            device_id, serial_number, fingerprint_sha256
        );

        Ok(IssuedCertificate {
            device_id: device_id.to_string(),
            serial_number,
            cert_pem,
            fingerprint_sha256,
            issued_at: now,
            expires_at,
        })
    }

    /// Computes the SHA-256 fingerprint of arbitrary certificate data.
    pub fn compute_fingerprint(cert_data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cert_data);
        format!("SHA256:{:x}", hasher.finalize())
    }
}

impl Default for LocalPkiAuthority {
    fn default() -> Self {
        Self::new()
    }
}
