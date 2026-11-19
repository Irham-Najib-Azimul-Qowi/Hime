use thiserror::Error;

#[cfg(windows)]
use windows::Win32::Security::Cryptography::{BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG};

#[derive(Error, Debug)]
pub enum CsprngError {
    #[error("Failed to generate cryptographically secure random bytes: {0}")]
    OsError(String),
}

/// Fills the destination buffer with cryptographically secure random bytes.
pub fn fill_random_bytes(buf: &mut [u8]) -> Result<(), CsprngError> {
    #[cfg(windows)]
    {
        let status = unsafe { BCryptGenRandom(None, buf, BCRYPT_USE_SYSTEM_PREFERRED_RNG) };
        if status.is_err() {
            return Err(CsprngError::OsError(format!(
                "BCryptGenRandom failed with NTSTATUS: 0x{:08x}",
                status.0
            )));
        }
        Ok(())
    }

    #[cfg(not(windows))]
    {
        use std::fs::File;
        use std::io::Read;
        let mut file =
            File::open("/dev/urandom").map_err(|e| CsprngError::OsError(e.to_string()))?;
        file.read_exact(buf)
            .map_err(|e| CsprngError::OsError(e.to_string()))?;
        Ok(())
    }
}

/// Generates a random hexadecimal token of the specified byte length (output is 2x length in hex chars).
pub fn generate_hex_token(byte_len: usize) -> Result<String, CsprngError> {
    let mut bytes = vec![0u8; byte_len];
    fill_random_bytes(&mut bytes)?;
    Ok(bytes.iter().map(|b| format!("{:02x}", b)).collect())
}

/// Generates a 256-bit (32 bytes = 64 hex chars) cryptographically secure random token.
pub fn generate_token_256() -> Result<String, CsprngError> {
    generate_hex_token(32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_length_and_uniqueness() {
        let t1 = generate_token_256().unwrap();
        let t2 = generate_token_256().unwrap();
        assert_eq!(t1.len(), 64);
        assert_eq!(t2.len(), 64);
        assert_ne!(t1, t2);
    }
}
