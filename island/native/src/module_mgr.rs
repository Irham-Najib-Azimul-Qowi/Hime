//! Hime AI Module Manager:
//! - Checks `%LOCALAPPDATA%\Hime\modules\hime-ai\`
//! - Verifies SHA-256 checksums (`sha2`)
//! - Performs atomic staging + `previous/` rollback within a 10-second window
//! - Spawns/stops `hime-ai.exe`
//! - Probes `http://127.0.0.1:20128/v1/models` for optional `9Router` detection

use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub const HIME_AI_EXE: &str = "hime-ai.exe";
pub const ROLLBACK_WINDOW_SECS: u64 = 10;
pub const NINE_ROUTER_HOST: &str = "127.0.0.1";
pub const NINE_ROUTER_PORT: u16 = 20128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleError {
    Io(String),
    ChecksumMismatch { expected: String, actual: String },
    MissingExecutable(PathBuf),
    RollbackUnavailable(String),
}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "module I/O error: {msg}"),
            Self::ChecksumMismatch { expected, actual } => {
                write!(f, "SHA-256 mismatch: expected {expected}, got {actual}")
            }
            Self::MissingExecutable(p) => write!(f, "missing executable: {}", p.display()),
            Self::RollbackUnavailable(msg) => write!(f, "rollback unavailable: {msg}"),
        }
    }
}

impl std::error::Error for ModuleError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleManifest {
    pub version: String,
    pub sha256: String,
    pub executable: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleStatus {
    NotInstalled,
    Installed {
        version: Option<String>,
        exe_path: PathBuf,
        has_previous: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouterProbeResult {
    pub available: bool,
    pub models: Vec<String>,
    pub latency_ms: u64,
    pub error: Option<String>,
}

/// Manages the optional `hime-ai` binary lifecycle in `%LOCALAPPDATA%\Hime\modules\hime-ai\`.
pub struct HimeAiModuleManager {
    root_dir: PathBuf,
    child: Option<Child>,
    last_swap_at: Option<Instant>,
}

impl HimeAiModuleManager {
    /// Creates a manager rooted at `%LOCALAPPDATA%\Hime\modules\hime-ai`.
    pub fn with_default_dir() -> Self {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        Self::new(base.join("Hime").join("modules").join("hime-ai"))
    }

    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            root_dir,
            child: None,
            last_swap_at: None,
        }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn current_dir(&self) -> PathBuf {
        self.root_dir.join("current")
    }

    pub fn staging_dir(&self) -> PathBuf {
        self.root_dir.join("staging")
    }

    pub fn previous_dir(&self) -> PathBuf {
        self.root_dir.join("previous")
    }

    pub fn current_exe(&self) -> PathBuf {
        self.current_dir().join(HIME_AI_EXE)
    }

    /// Checks whether `current/hime-ai.exe` is installed.
    pub fn status(&self) -> ModuleStatus {
        let exe = self.current_exe();
        if !exe.is_file() {
            return ModuleStatus::NotInstalled;
        }
        let version = fs::read_to_string(self.current_dir().join("manifest.json"))
            .ok()
            .and_then(|raw| serde_json::from_str::<ModuleManifest>(&raw).ok())
            .map(|m| m.version);
        ModuleStatus::Installed {
            version,
            exe_path: exe,
            has_previous: self.previous_dir().join(HIME_AI_EXE).is_file(),
        }
    }

    /// Computes lowercase hex SHA-256 of `bytes` and verifies it against `expected_hex`.
    pub fn verify_sha256(bytes: &[u8], expected_hex: &str) -> Result<String, ModuleError> {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let digest = hasher.finalize();
        let actual: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        let expected_norm = expected_hex.trim().to_ascii_lowercase();
        if actual != expected_norm {
            return Err(ModuleError::ChecksumMismatch {
                expected: expected_norm,
                actual,
            });
        }
        Ok(actual)
    }

    /// Stages verified `hime-ai.exe` bytes into `staging/`, moves `current/` to `previous/`,
    /// and promotes `staging/` to `current/`.
    pub fn stage_and_install(
        &mut self,
        payload_bytes: &[u8],
        manifest: &ModuleManifest,
    ) -> Result<PathBuf, ModuleError> {
        Self::verify_sha256(payload_bytes, &manifest.sha256)?;

        let staging = self.staging_dir();
        let current = self.current_dir();
        let previous = self.previous_dir();

        if staging.exists() {
            fs::remove_dir_all(&staging).map_err(|e| ModuleError::Io(e.to_string()))?;
        }
        fs::create_dir_all(&staging).map_err(|e| ModuleError::Io(e.to_string()))?;

        let exe_name = if manifest.executable.trim().is_empty() {
            HIME_AI_EXE
        } else {
            manifest.executable.trim()
        };
        fs::write(staging.join(exe_name), payload_bytes)
            .map_err(|e| ModuleError::Io(e.to_string()))?;
        let manifest_json =
            serde_json::to_string_pretty(manifest).map_err(|e| ModuleError::Io(e.to_string()))?;
        fs::write(staging.join("manifest.json"), manifest_json)
            .map_err(|e| ModuleError::Io(e.to_string()))?;

        if previous.exists() {
            fs::remove_dir_all(&previous).map_err(|e| ModuleError::Io(e.to_string()))?;
        }
        if current.exists() {
            fs::rename(&current, &previous).map_err(|e| ModuleError::Io(e.to_string()))?;
        }
        fs::rename(&staging, &current).map_err(|e| ModuleError::Io(e.to_string()))?;

        self.last_swap_at = Some(Instant::now());
        Ok(current.join(exe_name))
    }

    /// Returns `true` if a `previous/` backup exists and was swapped within 10 seconds.
    pub fn within_rollback_window(&self, now: Instant) -> bool {
        let Some(swapped_at) = self.last_swap_at else {
            return false;
        };
        self.previous_dir().exists()
            && now.saturating_duration_since(swapped_at) <= Duration::from_secs(ROLLBACK_WINDOW_SECS)
    }

    /// Rolls back `current/` to `previous/` if within the 10-second rollback window.
    pub fn rollback_if_within_window(&mut self, now: Instant) -> Result<bool, ModuleError> {
        if !self.within_rollback_window(now) {
            return Ok(false);
        }
        self.rollback_force()?;
        Ok(true)
    }

    /// Unconditionally restores `previous/` over `current/` if `previous/` exists.
    pub fn rollback_force(&mut self) -> Result<(), ModuleError> {
        let current = self.current_dir();
        let previous = self.previous_dir();
        if !previous.exists() {
            return Err(ModuleError::RollbackUnavailable(
                "previous/ directory does not exist".to_string(),
            ));
        }
        let failed = self.root_dir.join("failed_staging");
        if failed.exists() {
            let _ = fs::remove_dir_all(&failed);
        }
        if current.exists() {
            fs::rename(&current, &failed).map_err(|e| ModuleError::Io(e.to_string()))?;
        }
        fs::rename(&previous, &current).map_err(|e| ModuleError::Io(e.to_string()))?;
        if failed.exists() {
            let _ = fs::remove_dir_all(&failed);
        }
        self.last_swap_at = None;
        Ok(())
    }

    /// Spawns `hime-ai.exe` from `current/` if not already running.
    pub fn spawn(&mut self) -> Result<u32, ModuleError> {
        if let Some(child) = self.child.as_mut() {
            if child.try_wait().ok().flatten().is_none() {
                return Ok(child.id());
            }
            self.child = None;
        }

        let exe = self.current_exe();
        if !exe.is_file() {
            return Err(ModuleError::MissingExecutable(exe));
        }

        let mut cmd = Command::new(&exe);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let child = cmd.spawn().map_err(|e| ModuleError::Io(e.to_string()))?;
        let pid = child.id();
        self.child = Some(child);
        Ok(pid)
    }

    /// Stops the managed `hime-ai.exe` child process if running.
    pub fn stop(&mut self) -> Result<(), ModuleError> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    /// Probes `http://127.0.0.1:20128/v1/models` to detect an optional local 9Router instance.
    pub fn probe_9router(&self, timeout: Duration) -> RouterProbeResult {
        probe_openai_models_endpoint(NINE_ROUTER_HOST, NINE_ROUTER_PORT, timeout)
    }
}

impl Drop for HimeAiModuleManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Probes `http://<host>:<port>/v1/models` over loopback TCP without external HTTP crates.
pub fn probe_openai_models_endpoint(host: &str, port: u16, timeout: Duration) -> RouterProbeResult {
    let start = Instant::now();
    let addr_str = format!("{host}:{port}");
    let Ok(addr) = addr_str.parse::<SocketAddr>() else {
        return RouterProbeResult {
            available: false,
            models: Vec::new(),
            latency_ms: 0,
            error: Some(format!("invalid loopback address: {addr_str}")),
        };
    };

    let mut stream = match TcpStream::connect_timeout(&addr, timeout) {
        Ok(s) => s,
        Err(e) => {
            return RouterProbeResult {
                available: false,
                models: Vec::new(),
                latency_ms: start.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
            };
        }
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));

    let req = format!(
        "GET /v1/models HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n"
    );
    if let Err(e) = stream.write_all(req.as_bytes()) {
        return RouterProbeResult {
            available: false,
            models: Vec::new(),
            latency_ms: start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        };
    }

    let mut raw = String::new();
    let _ = stream.read_to_string(&mut raw);
    let latency_ms = start.elapsed().as_millis() as u64;

    let (headers, body) = raw.split_once("\r\n\r\n").unwrap_or((raw.as_str(), ""));
    let status_ok = headers
        .lines()
        .next()
        .map(|line| line.contains(" 200 "))
        .unwrap_or(false);

    if !status_ok {
        return RouterProbeResult {
            available: false,
            models: Vec::new(),
            latency_ms,
            error: Some("non-200 HTTP response".to_string()),
        };
    }

    let mut models = Vec::new();
    if let Ok(json_body) = serde_json::from_str::<serde_json::Value>(body.trim()) {
        if let Some(arr) = json_body.get("data").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                    models.push(id.to_string());
                }
            }
        }
    }

    RouterProbeResult {
        available: true,
        models,
        latency_ms,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn unique_temp_dir(name: &str) -> PathBuf {
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("hime_modmgr_{name}_{pid}_{nanos}"))
    }

    fn sha256_hex(data: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(data);
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn sha256_verification_accepts_valid_and_rejects_mismatch() {
        let payload = b"hime-ai-v1-binary";
        let digest = sha256_hex(payload);
        assert!(HimeAiModuleManager::verify_sha256(payload, &digest).is_ok());
        assert!(matches!(
            HimeAiModuleManager::verify_sha256(payload, "0000"),
            Err(ModuleError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn atomic_staging_and_10s_rollback_window() {
        let root = unique_temp_dir("staging");
        let mut mgr = HimeAiModuleManager::new(root.clone());
        assert_eq!(mgr.status(), ModuleStatus::NotInstalled);

        let v1 = b"hime-ai-v1";
        let m1 = ModuleManifest {
            version: "0.1.0".to_string(),
            sha256: sha256_hex(v1),
            executable: HIME_AI_EXE.to_string(),
        };
        mgr.stage_and_install(v1, &m1).unwrap();
        assert!(matches!(
            mgr.status(),
            ModuleStatus::Installed {
                version: Some(ref v),
                has_previous: false,
                ..
            } if v == "0.1.0"
        ));

        let v2 = b"hime-ai-v2";
        let m2 = ModuleManifest {
            version: "0.2.0".to_string(),
            sha256: sha256_hex(v2),
            executable: HIME_AI_EXE.to_string(),
        };
        mgr.stage_and_install(v2, &m2).unwrap();

        // Rollback outside 10s window should return false and keep v2.
        let expired = mgr.last_swap_at.unwrap() + Duration::from_secs(11);
        assert!(!mgr.rollback_if_within_window(expired).unwrap());

        // Rollback within 10s window should restore v1.
        let within = mgr.last_swap_at.unwrap() + Duration::from_secs(3);
        assert!(mgr.rollback_if_within_window(within).unwrap());
        assert!(matches!(
            mgr.status(),
            ModuleStatus::Installed {
                version: Some(ref v),
                ..
            } if v == "0.1.0"
        ));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn probe_9router_parses_local_models_response() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 512];
                let _ = stream.read(&mut buf);
                let body = r#"{"object":"list","data":[{"id":"gpt-4o-mini"},{"id":"gemini-2.5-flash"}]}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });

        let res = probe_openai_models_endpoint("127.0.0.1", port, Duration::from_secs(2));
        let _ = handle.join();
        assert!(res.available);
        assert_eq!(res.models, vec!["gpt-4o-mini", "gemini-2.5-flash"]);
    }
}
