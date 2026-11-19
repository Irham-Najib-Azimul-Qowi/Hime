//! Hime Core Background Supervisor and IPC Bridge Service
//!
//! Manages the lifecycle of the Hime Core daemon process, Named Pipe reconnection,
//! health monitoring, approval lifecycle, and event forwarding into the Tauri frontend.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

use super::client::{connect_named_pipe, rpc_call};

static BRIDGE_SERVICE: OnceLock<Arc<HimeBridgeService>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CoreConnectionState {
    Offline,
    Connecting,
    Online,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HimeCoreStatus {
    pub connection_state: CoreConnectionState,
    pub is_online: bool,
    pub core_version: Option<String>,
    pub uptime_secs: u64,
    pub active_approvals_count: usize,
    pub last_ping_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalItem {
    pub request_id: String,
    pub action_type: String,
    pub path: String,
    pub description: String,
    pub status: String, // "pending" | "approved" | "denied" | "expired"
    pub created_at: u64,
    pub expires_at: u64,
}

pub struct HimeBridgeService {
    app: AppHandle,
    state: Mutex<CoreConnectionState>,
    core_version: Mutex<Option<String>>,
    start_time: Instant,
    last_ping_ms: Mutex<Option<u64>>,
    session_token: Mutex<String>,
    nonce_counter: AtomicU64,
    approvals: Mutex<HashMap<String, ApprovalItem>>,
    core_child: Mutex<Option<Child>>,
    shutdown_flag: AtomicBool,
}

impl HimeBridgeService {
    pub fn new(app: AppHandle) -> Arc<Self> {
        let default_token = std::env::var("HIME_AUTH_TOKEN")
            .unwrap_or_else(|_| "hime-default-dev-token".to_string());

        let service = Arc::new(Self {
            app,
            state: Mutex::new(CoreConnectionState::Offline),
            core_version: Mutex::new(None),
            start_time: Instant::now(),
            last_ping_ms: Mutex::new(None),
            session_token: Mutex::new(default_token),
            nonce_counter: AtomicU64::new(1),
            approvals: Mutex::new(HashMap::new()),
            core_child: Mutex::new(None),
            shutdown_flag: AtomicBool::new(false),
        });

        BRIDGE_SERVICE.set(service.clone()).ok();
        service
    }

    pub fn global() -> Option<Arc<HimeBridgeService>> {
        BRIDGE_SERVICE.get().cloned()
    }

    /// Locates the `hime-core.exe` binary in workspace or build output directories.
    fn locate_core_binary(&self) -> Option<PathBuf> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        let mut candidates = Vec::new();

        if let Some(dir) = &exe_dir {
            candidates.push(dir.join("hime-core.exe"));
            candidates.push(dir.join("../../core/target/debug/hime-core.exe"));
            candidates.push(dir.join("../../core/target/release/hime-core.exe"));
        }

        // Relative to current working directory
        candidates.push(PathBuf::from("core/target/debug/hime-core.exe"));
        candidates.push(PathBuf::from("core/target/release/hime-core.exe"));
        candidates.push(PathBuf::from("target/debug/hime-core.exe"));

        candidates.into_iter().find(|path| path.exists())
    }

    /// Ensures 9Router background gateway is running on 127.0.0.1:20128.
    pub fn ensure_9router_running(&self) {
        use std::net::{SocketAddr, TcpStream};
        let addr: SocketAddr = match "127.0.0.1:20128".parse() {
            Ok(a) => a,
            Err(_) => return,
        };
        if TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok() {
            return;
        }

        let npm_router = std::env::var("APPDATA")
            .ok()
            .map(|a| PathBuf::from(a).join("npm").join("9router.cmd"))
            .filter(|p| p.exists());

        let cmd_target = if let Some(path) = npm_router {
            path.to_string_lossy().to_string()
        } else {
            "9router".to_string()
        };

        let mut cmd = Command::new("cmd.exe");
        cmd.args(["/c", &cmd_target, "-H", "127.0.0.1", "-p", "20128", "--tray", "--skip-update"]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let _ = cmd.spawn();
    }

    /// Spawns the Hime Core child process if not already running.
    pub fn ensure_core_running(&self) {
        // If pipe can already be opened, Core is already running
        if connect_named_pipe(None).is_ok() {
            return;
        }

        if let Some(binary) = self.locate_core_binary() {
            let token = self.session_token.lock().unwrap().clone();
            let mut cmd = Command::new(&binary);
            cmd.env("HIME_AUTH_TOKEN", &token);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x0800_0000;
                cmd.creation_flags(CREATE_NO_WINDOW);
            }
            match cmd.spawn() {
                Ok(child) => {
                    let mut lock = self.core_child.lock().unwrap();
                    *lock = Some(child);
                    // Give Core a brief moment to create the Named Pipe
                    std::thread::sleep(Duration::from_millis(300));
                }
                Err(e) => {
                    eprintln!(
                        "[Hime Bridge] Failed to spawn Hime Core from {:?}: {}",
                        binary, e
                    );
                }
            }
        }
    }

    /// Attempts connection and authentication handshake with Hime Core.
    pub fn connect_and_handshake(&self) -> Result<File, String> {
        let mut file =
            connect_named_pipe(None).map_err(|e| format!("Named Pipe connection failed: {}", e))?;

        let token = self.session_token.lock().unwrap().clone();
        let nonce = self.nonce_counter.fetch_add(1, Ordering::SeqCst);

        let handshake_params = json!({
            "client_id": "bloom-shell-bridge",
            "auth_token": token,
            "nonce": nonce,
        });

        let resp = rpc_call(&mut file, "system.handshake", Some(handshake_params))?;
        if let Some(ver) = resp.get("core_version").and_then(|v| v.as_str()) {
            let mut v_lock = self.core_version.lock().unwrap();
            *v_lock = Some(ver.to_string());
        }

        *self.state.lock().unwrap() = CoreConnectionState::Online;
        self.emit_status_change();
        Ok(file)
    }

    /// Performs an active IPC ping to measure latency and confirm health.
    pub fn ping_core(&self) -> Result<u64, String> {
        let start = Instant::now();
        let mut file =
            connect_named_pipe(None).map_err(|e| format!("Named Pipe unavailable: {}", e))?;

        let _ = rpc_call(&mut file, "system.ping", None)?;
        let latency_ms = start.elapsed().as_millis() as u64;

        *self.last_ping_ms.lock().unwrap() = Some(latency_ms);
        *self.state.lock().unwrap() = CoreConnectionState::Online;
        self.emit_status_change();

        Ok(latency_ms)
    }

    /// Emits status update event to the frontend (`hime-core-status`).
    pub fn emit_status_change(&self) {
        let status = self.get_status();
        let _ = self.app.emit("hime-core-status", status);
    }

    pub fn get_status(&self) -> HimeCoreStatus {
        let state = self.state.lock().unwrap().clone();
        let core_version = self.core_version.lock().unwrap().clone();
        let is_online = state == CoreConnectionState::Online;
        let last_ping_ms = *self.last_ping_ms.lock().unwrap();
        let active_approvals_count = self.approvals.lock().unwrap().len();

        HimeCoreStatus {
            connection_state: state,
            is_online,
            core_version,
            uptime_secs: self.start_time.elapsed().as_secs(),
            active_approvals_count,
            last_ping_ms,
        }
    }

    /// Returns list of active pending approvals.
    pub fn list_approvals(&self) -> Vec<ApprovalItem> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut lock = self.approvals.lock().unwrap();
        // Prune expired approvals (fail-closed)
        lock.retain(|_, item| item.expires_at > now && item.status == "pending");
        lock.values().cloned().collect()
    }

    /// Resolves an approval decision ("approve" | "deny") with Hime Core.
    pub fn resolve_approval(&self, request_id: &str, decision: &str) -> Result<String, String> {
        let mut file = connect_named_pipe(None)
            .map_err(|e| format!("Cannot resolve approval: Hime Core is offline ({})", e))?;

        let params = json!({
            "request_id": request_id,
            "decision": decision,
        });

        let resp = rpc_call(&mut file, "approvals.resolve", Some(params))?;
        let status = resp
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or(decision)
            .to_string();

        // Update local cache and notify frontend
        {
            let mut lock = self.approvals.lock().unwrap();
            if let Some(item) = lock.get_mut(request_id) {
                item.status = status.clone();
            }
        }

        let _ = self.app.emit(
            "hime-approval-resolved",
            json!({
                "request_id": request_id,
                "decision": decision,
                "status": status,
            }),
        );

        self.emit_status_change();
        Ok(status)
    }

    /// Debug-only method to trigger a test approval for visual acceptance (VC-01).
    pub fn dev_trigger_test_approval(
        &self,
        action_type: &str,
        path: &str,
        description: &str,
    ) -> Result<ApprovalItem, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let expires_at = now + 60_000; // 60s TTL

        let req_id = format!("appr-dev-{:x}", now);
        let item = ApprovalItem {
            request_id: req_id.clone(),
            action_type: action_type.to_string(),
            path: path.to_string(),
            description: description.to_string(),
            status: "pending".to_string(),
            created_at: now,
            expires_at,
        };

        {
            let mut lock = self.approvals.lock().unwrap();
            lock.insert(req_id.clone(), item.clone());
        }

        let _ = self.app.emit("hime-approval-pending", item.clone());
        self.emit_status_change();
        Ok(item)
    }

    /// Sends a chat message to Hime Core AI runtime.
    pub fn send_chat(&self, prompt: &str, model: Option<&str>) -> Result<serde_json::Value, String> {
        let mut file = self.connect_and_handshake()?;

        let conv_res = rpc_call(
            &mut file,
            "conversation.create",
            Some(json!({
                "title": "Hime Assistant Chat",
                "model": model.unwrap_or("hermes-3-llama-3.1-8b")
            })),
        )?;

        let conv_id = conv_res
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("conv_default");

        let send_res = rpc_call(
            &mut file,
            "conversation.send_message",
            Some(json!({
                "conversation_id": conv_id,
                "prompt": prompt,
                "model": model.unwrap_or("hermes-3-llama-3.1-8b")
            })),
        )?;

        Ok(send_res)
    }

    /// Stores a provider API key in Hime Core's Windows DPAPI encrypted vault.
    pub fn store_credential(&self, provider: &str, secret: &str) -> Result<serde_json::Value, String> {
        let mut file = self.connect_and_handshake()?;
        rpc_call(
            &mut file,
            "credentials.store",
            Some(json!({
                "provider": provider,
                "secret": secret
            })),
        )
    }

    /// Polls for newly detected completed downloads.
    pub fn poll_new_downloads(&self) -> Result<serde_json::Value, String> {
        let mut file = self.connect_and_handshake()?;
        rpc_call(&mut file, "storage.download.poll_new", None)
    }

    /// Evaluates a file in Downloads directory for duplicates / versioning.
    pub fn inspect_download(&self, download_path: &str) -> Result<serde_json::Value, String> {
        let mut file = self.connect_and_handshake()?;
        rpc_call(
            &mut file,
            "storage.download.inspect",
            Some(json!({
                "download_path": download_path,
            })),
        )
    }

    /// Resolves download conflict by archiving existing file and moving new download in place.
    pub fn replace_and_archive_download(
        &self,
        new_download: &str,
        existing_file: &str,
    ) -> Result<serde_json::Value, String> {
        let mut file = self.connect_and_handshake()?;
        rpc_call(
            &mut file,
            "storage.download.replace_and_archive",
            Some(json!({
                "new_download": new_download,
                "existing_file": existing_file,
                "actor": "desktop",
            })),
        )
    }

    /// Creates an academic or business report document inside Hime documents sandbox and commits to target path.
    pub fn create_academic_report(
        &self,
        title: &str,
        markdown_body: &str,
        destination_path: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let mut file = self.connect_and_handshake()?;
        let job_id = format!("report-{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
        let dest = destination_path.map(|s| s.to_string()).unwrap_or_else(|| {
            if let Ok(profile) = std::env::var("USERPROFILE") {
                format!("{}\\Documents\\{}.html", profile, title.replace(" ", "_"))
            } else {
                format!("{}.html", title.replace(" ", "_"))
            }
        });

        rpc_call(
            &mut file,
            "documents.create_report",
            Some(json!({
                "job_id": job_id,
                "title": title,
                "markdown_body": markdown_body,
                "template_name": "Akademik-ID",
                "destination_path": dest,
            })),
        )
    }

    /// Merges multiple PDFs into a single output document.
    pub fn merge_pdfs(
        &self,
        source_paths: &[String],
        destination_path: &str,
    ) -> Result<serde_json::Value, String> {
        let mut file = self.connect_and_handshake()?;
        let job_id = format!("merge-{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
        rpc_call(
            &mut file,
            "documents.merge_pdfs",
            Some(json!({
                "job_id": job_id,
                "source_paths": source_paths,
                "destination_path": destination_path,
            })),
        )
    }

    /// Background monitor loop: performs backoff reconnect and health checks.
    pub fn start_supervisor_loop(self: Arc<Self>) {
        std::thread::spawn(move || {
            let mut backoff = Duration::from_millis(500);
            while !self.shutdown_flag.load(Ordering::Relaxed) {
                // Ensure 9Router gateway is running in background
                self.ensure_9router_running();

                if connect_named_pipe(None).is_ok() {
                    if *self.state.lock().unwrap() != CoreConnectionState::Online {
                        let _ = self.connect_and_handshake();
                    }

                    // Poll for new completed downloads and forward events to UI
                    if let Ok(mut dl_file) = connect_named_pipe(None) {
                        if let Ok(val) = rpc_call(&mut dl_file, "storage.download.poll_new", None) {
                            if let Some(arr) = val.as_array() {
                                for item in arr {
                                    let _ = self.app.emit("hime-download-detected", item);
                                }
                            }
                        }
                    }

                    backoff = Duration::from_secs(4); // healthy interval
                } else {
                    if *self.state.lock().unwrap() == CoreConnectionState::Online {
                        *self.state.lock().unwrap() = CoreConnectionState::Offline;
                        self.emit_status_change();
                    }
                    // Attempt to spawn if missing
                    self.ensure_core_running();
                    backoff = (backoff * 2).min(Duration::from_secs(10));
                }
                std::thread::sleep(backoff);
            }
        });
    }

    /// Clean shutdown hook to terminate child Core process.
    pub fn shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::Relaxed);
        let mut lock = self.core_child.lock().unwrap();
        if let Some(mut child) = lock.take() {
            let _ = child.kill();
        }
    }
}
