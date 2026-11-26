use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::{info, warn};

/// Default IPC handshake timeout in seconds before triggering automatic rollback.
pub const HANDSHAKE_TIMEOUT_SECS: u64 = 10;

/// Package manifest for an optional Hime module (e.g., `hime-ai`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModulePackageManifest {
    pub version: String,
    pub download_url: String,
    pub sha256: String,
    pub min_shell_version: String,
}

/// Persisted active/previous state for a module under `modules/<id>/state.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleInstallState {
    pub module_id: String,
    pub active_version: Option<String>,
    pub previous_version: Option<String>,
    pub state: String,
}

/// Outcome of an atomic module installation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleInstallOutcome {
    pub module_id: String,
    pub installed_version: String,
    pub active_path: PathBuf,
    pub rolled_back: bool,
    pub status: String,
}

/// Outcome of a module rollback.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleRollbackOutcome {
    pub module_id: String,
    pub restored_version: Option<String>,
    pub status: String,
}

#[derive(Debug, Error)]
pub enum ModuleInstallerError {
    #[error("Invalid module identifier or version: {0}")]
    InvalidIdentifier(String),

    #[error("Shell version {shell_version} is older than required min_shell_version {min_shell_version}")]
    IncompatibleShellVersion {
        shell_version: String,
        min_shell_version: String,
    },

    #[error("SHA-256 verification failed: expected {expected}, got {actual}")]
    Sha256Mismatch { expected: String, actual: String },

    #[error("Download failed for '{url}': {reason}")]
    DownloadFailed { url: String, reason: String },

    #[error("No previous module version available for rollback of '{0}'")]
    NoPreviousVersion(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Atomic module installer & rollback manager.
///
/// Layout under `root_dir`:
/// - `modules/<id>/<version>/` — staged & activated module files
/// - `modules/<id>/previous/`  — backup of the previously active version
/// - `modules/<id>/state.json` — current installation metadata
#[derive(Debug, Clone)]
pub struct ModuleInstaller {
    modules_root: PathBuf,
    shell_version: String,
    handshake_timeout: Duration,
}

impl ModuleInstaller {
    pub fn new(base_dir: impl AsRef<Path>, shell_version: impl Into<String>) -> Self {
        Self {
            modules_root: base_dir.as_ref().join("modules"),
            shell_version: shell_version.into(),
            handshake_timeout: Duration::from_secs(HANDSHAKE_TIMEOUT_SECS),
        }
    }

    pub fn with_handshake_timeout(mut self, timeout: Duration) -> Self {
        self.handshake_timeout = timeout;
        self
    }

    pub fn modules_root(&self) -> &Path {
        &self.modules_root
    }

    pub fn module_dir(&self, module_id: &str) -> Result<PathBuf, ModuleInstallerError> {
        validate_segment(module_id)?;
        Ok(self.modules_root.join(module_id))
    }

    pub fn version_dir(
        &self,
        module_id: &str,
        version: &str,
    ) -> Result<PathBuf, ModuleInstallerError> {
        validate_segment(module_id)?;
        validate_segment(version)?;
        Ok(self.modules_root.join(module_id).join(version))
    }

    pub fn previous_dir(&self, module_id: &str) -> Result<PathBuf, ModuleInstallerError> {
        validate_segment(module_id)?;
        Ok(self.modules_root.join(module_id).join("previous"))
    }

    /// Returns the current install state for `module_id`.
    pub fn status(&self, module_id: &str) -> Result<ModuleInstallState, ModuleInstallerError> {
        let mod_dir = self.module_dir(module_id)?;
        let state_path = mod_dir.join("state.json");
        if !state_path.exists() {
            return Ok(ModuleInstallState {
                module_id: module_id.to_string(),
                active_version: None,
                previous_version: None,
                state: "not_installed".to_string(),
            });
        }
        let raw = std::fs::read_to_string(&state_path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    /// Computes lowercase hex SHA-256 digest of `bytes`.
    pub fn compute_sha256_hex(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let digest = hasher.finalize();
        let mut hex = String::with_capacity(digest.len() * 2);
        for b in digest {
            hex.push_str(&format!("{:02x}", b));
        }
        hex
    }

    /// Verifies `bytes` against `expected_hex` (case-insensitive).
    pub fn verify_sha256(bytes: &[u8], expected_hex: &str) -> Result<(), ModuleInstallerError> {
        let expected = expected_hex.trim().to_ascii_lowercase();
        let actual = Self::compute_sha256_hex(bytes);
        if actual != expected {
            return Err(ModuleInstallerError::Sha256Mismatch { expected, actual });
        }
        Ok(())
    }

    /// Checks whether `shell_version >= min_shell_version` using numeric semver components.
    pub fn check_shell_compatibility(
        shell_version: &str,
        min_shell_version: &str,
    ) -> Result<(), ModuleInstallerError> {
        let shell_parts = parse_semver(shell_version);
        let min_parts = parse_semver(min_shell_version);
        if shell_parts < min_parts {
            return Err(ModuleInstallerError::IncompatibleShellVersion {
                shell_version: shell_version.to_string(),
                min_shell_version: min_shell_version.to_string(),
            });
        }
        Ok(())
    }

    /// Downloads package bytes from `download_url` (`http://`, `https://`, `file://`, or local path).
    pub async fn fetch_package_bytes(
        &self,
        download_url: &str,
    ) -> Result<Vec<u8>, ModuleInstallerError> {
        if let Some(local_path) = download_url.strip_prefix("file://") {
            let trimmed = local_path.trim_start_matches('/');
            let candidate = if cfg!(windows) && trimmed.contains(':') {
                PathBuf::from(trimmed)
            } else {
                PathBuf::from(local_path)
            };
            return std::fs::read(&candidate).map_err(|e| ModuleInstallerError::DownloadFailed {
                url: download_url.to_string(),
                reason: e.to_string(),
            });
        }

        if !download_url.starts_with("http://") && !download_url.starts_with("https://") {
            return std::fs::read(download_url).map_err(|e| ModuleInstallerError::DownloadFailed {
                url: download_url.to_string(),
                reason: e.to_string(),
            });
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| ModuleInstallerError::DownloadFailed {
                url: download_url.to_string(),
                reason: e.to_string(),
            })?;

        let resp = client
            .get(download_url)
            .send()
            .await
            .map_err(|e| ModuleInstallerError::DownloadFailed {
                url: download_url.to_string(),
                reason: e.to_string(),
            })?;

        if !resp.status().is_success() {
            return Err(ModuleInstallerError::DownloadFailed {
                url: download_url.to_string(),
                reason: format!("HTTP {}", resp.status()),
            });
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ModuleInstallerError::DownloadFailed {
                url: download_url.to_string(),
                reason: e.to_string(),
            })?;

        Ok(bytes.to_vec())
    }

    /// Atomically stages verified package bytes into `modules/<id>/<version>/`,
    /// preserving the previous active version in `modules/<id>/previous/`.
    pub fn stage_verified_bytes(
        &self,
        module_id: &str,
        manifest: &ModulePackageManifest,
        package_bytes: &[u8],
        shell_version_override: Option<&str>,
    ) -> Result<PathBuf, ModuleInstallerError> {
        validate_segment(module_id)?;
        validate_segment(&manifest.version)?;
        let effective_shell = shell_version_override.unwrap_or(&self.shell_version);
        Self::check_shell_compatibility(effective_shell, &manifest.min_shell_version)?;

        // Mandatory SHA-256 verification before any extraction or activation
        Self::verify_sha256(package_bytes, &manifest.sha256)?;

        let mod_dir = self.module_dir(module_id)?;
        std::fs::create_dir_all(&mod_dir)?;

        let current_state = self.status(module_id)?;
        let prev_dir = self.previous_dir(module_id)?;

        // Snapshot currently active version into `previous/` before activating the new one
        if let Some(ref active_ver) = current_state.active_version {
            if active_ver != &manifest.version {
                let active_dir = self.version_dir(module_id, active_ver)?;
                if active_dir.exists() {
                    if prev_dir.exists() {
                        std::fs::remove_dir_all(&prev_dir)?;
                    }
                    copy_dir_recursive(&active_dir, &prev_dir)?;
                }
            }
        }

        // Atomic staging: write into `.staging-<version>` first, then rename to `<version>/`
        let staging_dir = mod_dir.join(format!(".staging-{}", manifest.version));
        if staging_dir.exists() {
            std::fs::remove_dir_all(&staging_dir)?;
        }
        std::fs::create_dir_all(&staging_dir)?;

        let artifact_name = package_filename(module_id, &manifest.download_url);
        std::fs::write(staging_dir.join(&artifact_name), package_bytes)?;
        std::fs::write(
            staging_dir.join("manifest.json"),
            serde_json::to_vec_pretty(manifest)?,
        )?;

        let target_dir = self.version_dir(module_id, &manifest.version)?;
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)?;
        }
        std::fs::rename(&staging_dir, &target_dir)?;

        let next_prev = match current_state.active_version {
            Some(ref prev) if prev != &manifest.version => Some(prev.clone()),
            _ => current_state.previous_version,
        };

        let new_state = ModuleInstallState {
            module_id: module_id.to_string(),
            active_version: Some(manifest.version.clone()),
            previous_version: next_prev,
            state: "active".to_string(),
        };
        self.save_state(module_id, &new_state)?;

        info!(
            module_id = module_id,
            version = %manifest.version,
            path = %target_dir.display(),
            "Module staged and activated"
        );

        Ok(target_dir)
    }

    /// Downloads, verifies SHA-256, atomically stages, and validates IPC handshake within
    /// `self.handshake_timeout` (default 10s). If handshake fails or times out, automatically
    /// rolls back to `previous/`.
    pub async fn install_with_handshake<F, Fut>(
        &self,
        module_id: &str,
        manifest: &ModulePackageManifest,
        shell_version_override: Option<&str>,
        handshake_check: F,
    ) -> Result<ModuleInstallOutcome, ModuleInstallerError>
    where
        F: FnOnce(PathBuf) -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        let bytes = self.fetch_package_bytes(&manifest.download_url).await?;
        let active_path =
            self.stage_verified_bytes(module_id, manifest, &bytes, shell_version_override)?;

        let handshake_result =
            tokio::time::timeout(self.handshake_timeout, handshake_check(active_path.clone())).await;

        let ok = match handshake_result {
            Ok(Ok(())) => true,
            Ok(Err(reason)) => {
                warn!(module_id = module_id, reason = %reason, "Module IPC handshake failed; rolling back");
                false
            }
            Err(_) => {
                warn!(
                    module_id = module_id,
                    timeout_secs = self.handshake_timeout.as_secs(),
                    "Module IPC handshake timed out; rolling back"
                );
                false
            }
        };

        if ok {
            return Ok(ModuleInstallOutcome {
                module_id: module_id.to_string(),
                installed_version: manifest.version.clone(),
                active_path,
                rolled_back: false,
                status: "active".to_string(),
            });
        }

        // Auto-rollback on handshake failure/timeout
        match self.rollback(module_id, Some("ipc_handshake_failed")) {
            Ok(rb) => Ok(ModuleInstallOutcome {
                module_id: module_id.to_string(),
                installed_version: rb
                    .restored_version
                    .unwrap_or_else(|| manifest.version.clone()),
                active_path: self.module_dir(module_id)?,
                rolled_back: true,
                status: rb.status,
            }),
            Err(_) => {
                // No previous version existed; mark failed
                let failed_state = ModuleInstallState {
                    module_id: module_id.to_string(),
                    active_version: None,
                    previous_version: None,
                    state: "handshake_failed".to_string(),
                };
                self.save_state(module_id, &failed_state)?;
                Ok(ModuleInstallOutcome {
                    module_id: module_id.to_string(),
                    installed_version: manifest.version.clone(),
                    active_path,
                    rolled_back: true,
                    status: "handshake_failed_no_previous".to_string(),
                })
            }
        }
    }

    /// Rolls back `module_id` to `modules/<id>/previous/`.
    pub fn rollback(
        &self,
        module_id: &str,
        reason: Option<&str>,
    ) -> Result<ModuleRollbackOutcome, ModuleInstallerError> {
        validate_segment(module_id)?;
        let state = self.status(module_id)?;
        let prev_dir = self.previous_dir(module_id)?;
        let prev_version = state
            .previous_version
            .clone()
            .filter(|_| prev_dir.exists())
            .ok_or_else(|| ModuleInstallerError::NoPreviousVersion(module_id.to_string()))?;

        let restored_dir = self.version_dir(module_id, &prev_version)?;
        if restored_dir.exists() {
            std::fs::remove_dir_all(&restored_dir)?;
        }
        copy_dir_recursive(&prev_dir, &restored_dir)?;

        let next_state = ModuleInstallState {
            module_id: module_id.to_string(),
            active_version: Some(prev_version.clone()),
            previous_version: None,
            state: "rolled_back".to_string(),
        };
        self.save_state(module_id, &next_state)?;

        info!(
            module_id = module_id,
            restored_version = %prev_version,
            reason = reason.unwrap_or("manual"),
            "Module rolled back to previous version"
        );

        Ok(ModuleRollbackOutcome {
            module_id: module_id.to_string(),
            restored_version: Some(prev_version),
            status: "rolled_back".to_string(),
        })
    }

    fn save_state(
        &self,
        module_id: &str,
        state: &ModuleInstallState,
    ) -> Result<(), ModuleInstallerError> {
        let mod_dir = self.module_dir(module_id)?;
        std::fs::create_dir_all(&mod_dir)?;
        let state_path = mod_dir.join("state.json");
        let tmp_path = mod_dir.join("state.json.tmp");
        std::fs::write(&tmp_path, serde_json::to_vec_pretty(state)?)?;
        if state_path.exists() {
            std::fs::remove_file(&state_path)?;
        }
        std::fs::rename(&tmp_path, &state_path)?;
        Ok(())
    }
}

fn validate_segment(seg: &str) -> Result<(), ModuleInstallerError> {
    if seg.is_empty()
        || seg.contains("..")
        || seg.contains('/')
        || seg.contains('\\')
        || seg.contains(':')
    {
        return Err(ModuleInstallerError::InvalidIdentifier(seg.to_string()));
    }
    Ok(())
}

fn parse_semver(v: &str) -> (u64, u64, u64) {
    let clean = v.trim().trim_start_matches('v');
    let mut parts = clean.split('.').map(|p| {
        p.split('-')
            .next()
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or(0)
    });
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    (major, minor, patch)
}

fn package_filename(module_id: &str, url: &str) -> String {
    let candidate = url
        .rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty() && !s.contains(':'))
        .unwrap_or("package.bin");
    if candidate.is_empty() {
        format!("{}.bin", module_id)
    } else {
        candidate.to_string()
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("hime_modinst_{}_{}", name, nanos));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_sha256_verification_passes_and_rejects_mismatch() {
        let payload = b"hime-ai-binary-v0.2.0";
        let digest = ModuleInstaller::compute_sha256_hex(payload);

        assert!(ModuleInstaller::verify_sha256(payload, &digest).is_ok());
        assert!(ModuleInstaller::verify_sha256(payload, &digest.to_uppercase()).is_ok());

        let err = ModuleInstaller::verify_sha256(
            payload,
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap_err();
        assert!(matches!(err, ModuleInstallerError::Sha256Mismatch { .. }));
    }

    #[test]
    fn test_atomic_staging_and_manual_rollback() {
        let root = temp_root("staging_rollback");
        let installer = ModuleInstaller::new(&root, "0.2.0");

        let v1_bytes = b"hime-ai-v0.1.0";
        let v1_manifest = ModulePackageManifest {
            version: "0.1.0".to_string(),
            download_url: "hime-ai.exe".to_string(),
            sha256: ModuleInstaller::compute_sha256_hex(v1_bytes),
            min_shell_version: "0.1.0".to_string(),
        };
        let v1_path = installer
            .stage_verified_bytes("hime-ai", &v1_manifest, v1_bytes, None)
            .unwrap();
        assert!(v1_path.join("hime-ai.exe").exists());

        let v2_bytes = b"hime-ai-v0.2.0";
        let v2_manifest = ModulePackageManifest {
            version: "0.2.0".to_string(),
            download_url: "hime-ai.exe".to_string(),
            sha256: ModuleInstaller::compute_sha256_hex(v2_bytes),
            min_shell_version: "0.1.0".to_string(),
        };
        let v2_path = installer
            .stage_verified_bytes("hime-ai", &v2_manifest, v2_bytes, None)
            .unwrap();
        assert!(v2_path.join("hime-ai.exe").exists());
        assert!(installer.previous_dir("hime-ai").unwrap().exists());

        let st = installer.status("hime-ai").unwrap();
        assert_eq!(st.active_version.as_deref(), Some("0.2.0"));
        assert_eq!(st.previous_version.as_deref(), Some("0.1.0"));

        let rb = installer.rollback("hime-ai", Some("test")).unwrap();
        assert_eq!(rb.restored_version.as_deref(), Some("0.1.0"));
        assert_eq!(rb.status, "rolled_back");

        let st_after = installer.status("hime-ai").unwrap();
        assert_eq!(st_after.active_version.as_deref(), Some("0.1.0"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn test_auto_rollback_on_handshake_timeout() {
        let root = temp_root("auto_rollback");
        let pkg_file = root.join("hime-ai-v2.bin");
        let v1_bytes = b"hime-ai-v1";
        let v2_bytes = b"hime-ai-v2";
        std::fs::write(&pkg_file, v2_bytes).unwrap();

        let installer = ModuleInstaller::new(&root, "0.2.0")
            .with_handshake_timeout(Duration::from_millis(50));

        // Seed v1
        let v1_manifest = ModulePackageManifest {
            version: "0.1.0".to_string(),
            download_url: "hime-ai.bin".to_string(),
            sha256: ModuleInstaller::compute_sha256_hex(v1_bytes),
            min_shell_version: "0.1.0".to_string(),
        };
        installer
            .stage_verified_bytes("hime-ai", &v1_manifest, v1_bytes, None)
            .unwrap();

        // Install v2 with a hanging handshake that exceeds timeout
        let v2_manifest = ModulePackageManifest {
            version: "0.2.0".to_string(),
            download_url: pkg_file.to_string_lossy().to_string(),
            sha256: ModuleInstaller::compute_sha256_hex(v2_bytes),
            min_shell_version: "0.1.0".to_string(),
        };
        let outcome = installer
            .install_with_handshake("hime-ai", &v2_manifest, None, |_path| async {
                tokio::time::sleep(Duration::from_millis(250)).await;
                Ok(())
            })
            .await
            .unwrap();

        assert!(outcome.rolled_back);
        assert_eq!(outcome.installed_version, "0.1.0");
        assert_eq!(outcome.status, "rolled_back");

        let st = installer.status("hime-ai").unwrap();
        assert_eq!(st.active_version.as_deref(), Some("0.1.0"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_incompatible_shell_version_rejected() {
        let root = temp_root("shell_compat");
        let installer = ModuleInstaller::new(&root, "0.1.0");
        let bytes = b"payload";
        let manifest = ModulePackageManifest {
            version: "0.2.0".to_string(),
            download_url: "hime-ai.bin".to_string(),
            sha256: ModuleInstaller::compute_sha256_hex(bytes),
            min_shell_version: "0.5.0".to_string(),
        };
        let err = installer
            .stage_verified_bytes("hime-ai", &manifest, bytes, None)
            .unwrap_err();
        assert!(matches!(
            err,
            ModuleInstallerError::IncompatibleShellVersion { .. }
        ));
        let _ = std::fs::remove_dir_all(&root);
    }
}
