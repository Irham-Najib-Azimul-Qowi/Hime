use sha2::{Digest, Sha256};
use std::sync::Arc;
use thiserror::Error;
use zeroize::Zeroize;

#[cfg(windows)]
use windows::Win32::Foundation::{LocalFree, HLOCAL};
#[cfg(windows)]
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
};

#[derive(Error, Debug)]
pub enum DpapiError {
    #[error("DPAPI encryption failed: {0}")]
    ProtectFailed(String),

    #[error("DPAPI decryption failed: {0}")]
    UnprotectFailed(String),

    #[error("Platform unsupported for DPAPI operations")]
    UnsupportedPlatform,
}

pub trait SecretVault: Send + Sync {
    fn protect(&self, plaintext: &[u8]) -> Result<Vec<u8>, DpapiError>;
    fn unprotect(&self, ciphertext: &[u8]) -> Result<Vec<u8>, DpapiError>;
}

/// Windows Data Protection API (DPAPI) hardware-backed user credential vault.
pub struct WindowsDpapiVault;

impl WindowsDpapiVault {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsDpapiVault {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretVault for WindowsDpapiVault {
    fn protect(&self, plaintext: &[u8]) -> Result<Vec<u8>, DpapiError> {
        #[cfg(windows)]
        {
            let data_in = CRYPT_INTEGER_BLOB {
                cbData: plaintext.len() as u32,
                pbData: plaintext.as_ptr() as *mut u8,
            };
            let mut data_out = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: std::ptr::null_mut(),
            };

            unsafe {
                CryptProtectData(
                    &data_in,
                    None,
                    None,
                    None,
                    None,
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut data_out,
                )
            }
            .map_err(|e| DpapiError::ProtectFailed(e.to_string()))?;

            let result = unsafe {
                std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec()
            };

            unsafe {
                let _ = LocalFree(HLOCAL(data_out.pbData as _));
            }

            Ok(result)
        }

        #[cfg(not(windows))]
        {
            let _ = plaintext;
            Err(DpapiError::UnsupportedPlatform)
        }
    }

    fn unprotect(&self, ciphertext: &[u8]) -> Result<Vec<u8>, DpapiError> {
        #[cfg(windows)]
        {
            let data_in = CRYPT_INTEGER_BLOB {
                cbData: ciphertext.len() as u32,
                pbData: ciphertext.as_ptr() as *mut u8,
            };
            let mut data_out = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: std::ptr::null_mut(),
            };

            unsafe {
                CryptUnprotectData(
                    &data_in,
                    None,
                    None,
                    None,
                    None,
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut data_out,
                )
            }
            .map_err(|e| DpapiError::UnprotectFailed(e.to_string()))?;

            let result = unsafe {
                std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec()
            };

            unsafe {
                // Zeroize memory before releasing handle
                std::ptr::write_bytes(data_out.pbData, 0, data_out.cbData as usize);
                let _ = LocalFree(HLOCAL(data_out.pbData as _));
            }

            Ok(result)
        }

        #[cfg(not(windows))]
        {
            let _ = ciphertext;
            Err(DpapiError::UnsupportedPlatform)
        }
    }
}

/// Software-encrypted secret vault for Linux / cross-platform fallback.
/// Uses SHA-256 CTR keystream derived from a machine/user secret seed with automatic zeroization.
pub struct SoftwareEncryptedVault {
    key: [u8; 32],
}

impl SoftwareEncryptedVault {
    pub fn new(seed: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"hime-vault-key-derivation-salt-v1");
        hasher.update(seed);
        let key_bytes = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Self { key }
    }

    /// Derives key from platform environment (e.g. machine-id or user id).
    pub fn from_environment() -> Self {
        let machine_id = std::env::var("HIME_VAULT_SEED")
            .or_else(|_| std::env::var("USER"))
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "hime-default-user".to_string());
        Self::new(machine_id.as_bytes())
    }

    fn apply_keystream(&self, input: &[u8], nonce: &[u8; 16]) -> Vec<u8> {
        let mut output = Vec::with_capacity(input.len());
        let mut block_idx: u32 = 0;

        for chunk in input.chunks(32) {
            let mut hasher = Sha256::new();
            hasher.update(self.key);
            hasher.update(nonce);
            hasher.update(block_idx.to_be_bytes());
            let block_keystream = hasher.finalize();

            for (b, k) in chunk.iter().zip(block_keystream.iter()) {
                output.push(b ^ k);
            }
            block_idx += 1;
        }

        output
    }
}

impl SecretVault for SoftwareEncryptedVault {
    fn protect(&self, plaintext: &[u8]) -> Result<Vec<u8>, DpapiError> {
        // Generate pseudo-nonce from length and sha of plaintext
        let mut nonce = [0u8; 16];
        let mut h = Sha256::new();
        h.update(b"hime-nonce");
        h.update(plaintext);
        let digest = h.finalize();
        nonce.copy_from_slice(&digest[0..16]);

        let ciphertext = self.apply_keystream(plaintext, &nonce);

        // Prepend nonce (16 bytes)
        let mut result = Vec::with_capacity(16 + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn unprotect(&self, payload: &[u8]) -> Result<Vec<u8>, DpapiError> {
        if payload.len() < 16 {
            return Err(DpapiError::UnprotectFailed("Payload too short".to_string()));
        }

        let mut nonce = [0u8; 16];
        nonce.copy_from_slice(&payload[0..16]);
        let ciphertext = &payload[16..];

        let plaintext = self.apply_keystream(ciphertext, &nonce);
        Ok(plaintext)
    }
}

impl Drop for SoftwareEncryptedVault {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

/// Returns the recommended vault for the current operating system.
pub fn create_default_vault() -> Arc<dyn SecretVault> {
    #[cfg(windows)]
    {
        Arc::new(WindowsDpapiVault::new())
    }
    #[cfg(not(windows))]
    {
        Arc::new(SoftwareEncryptedVault::from_environment())
    }
}

/// A protected secret stored in memory with automatic zeroization on drop.
pub struct ProtectedSecret {
    data: Vec<u8>,
}

impl ProtectedSecret {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl Drop for ProtectedSecret {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_dpapi_protect_unprotect_roundtrip() {
        let vault = WindowsDpapiVault::new();
        let secret = b"super-secret-api-token-9router-123456";

        let encrypted = vault.protect(secret).expect("DPAPI protect should succeed");
        assert_ne!(encrypted, secret);

        let decrypted = vault
            .unprotect(&encrypted)
            .expect("DPAPI unprotect should succeed");
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_software_encrypted_vault_roundtrip() {
        let vault = SoftwareEncryptedVault::new(b"test-seed-secret");
        let secret = b"secret-token-for-cross-platform-linux-vault";

        let encrypted = vault.protect(secret).expect("Protect should succeed");
        assert_ne!(encrypted, secret);
        assert!(encrypted.len() >= 16);

        let decrypted = vault.unprotect(&encrypted).expect("Unprotect should succeed");
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_create_default_vault() {
        let vault = create_default_vault();
        let secret = b"vault-integration-payload";
        let encrypted = vault.protect(secret).expect("Protect should succeed");
        let decrypted = vault.unprotect(&encrypted).expect("Unprotect should succeed");
        assert_eq!(decrypted, secret);
    }
}
