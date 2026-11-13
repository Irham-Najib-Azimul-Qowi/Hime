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
            let mut data_in = CRYPT_INTEGER_BLOB {
                cbData: plaintext.len() as u32,
                pbData: plaintext.as_ptr() as *mut u8,
            };
            let mut data_out = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: std::ptr::null_mut(),
            };

            unsafe {
                CryptProtectData(
                    &mut data_in,
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
            Err(DpapiError::UnsupportedPlatform)
        }
    }

    fn unprotect(&self, ciphertext: &[u8]) -> Result<Vec<u8>, DpapiError> {
        #[cfg(windows)]
        {
            let mut data_in = CRYPT_INTEGER_BLOB {
                cbData: ciphertext.len() as u32,
                pbData: ciphertext.as_ptr() as *mut u8,
            };
            let mut data_out = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: std::ptr::null_mut(),
            };

            unsafe {
                CryptUnprotectData(
                    &mut data_in,
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
            Err(DpapiError::UnsupportedPlatform)
        }
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
}
