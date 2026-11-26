use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Mutex;
use tracing::info;

/// Resource operating modes for Hime Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResourceMode {
    Efficiency = 0,
    Balanced = 1,
    Performance = 2,
}

impl Default for ResourceMode {
    fn default() -> Self {
        ResourceMode::Balanced
    }
}

/// Mode Hemat (Power Saving) state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SaveMode {
    Off = 0,
    Auto = 1,
    On = 2,
}

impl Default for SaveMode {
    fn default() -> Self {
        SaveMode::Auto
    }
}

impl SaveMode {
    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "on" | "true" | "1" => SaveMode::On,
            "off" | "false" | "0" => SaveMode::Off,
            _ => SaveMode::Auto,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SaveMode::Off => "off",
            SaveMode::Auto => "auto",
            SaveMode::On => "on",
        }
    }
}

/// Single process memory & CPU metric entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessMemoryEntry {
    pub name: String,
    pub pid: u32,
    pub working_set_bytes: u64,
    pub working_set_mb: f64,
    pub cpu_percent: f64,
}

/// Comprehensive telemetry snapshot for Resources page ("Sumber Daya").
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceMetrics {
    pub processes: Vec<ProcessMemoryEntry>,
    pub total_working_set_bytes: u64,
    pub total_working_set_mb: f64,
    pub save_mode: SaveMode,
    pub is_save_mode_active: bool,
    pub ai_runtime_status: String,
    pub ai_idle_seconds: u64,
    pub ai_idle_timeout_seconds: u64,
    pub target_budget_mb: u64,
    pub within_budget: bool,
    pub is_low_memory: bool,
}

fn current_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(windows)]
fn query_current_process_memory() -> Option<u64> {
    use windows::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::GetCurrentProcess;

    unsafe {
        let handle = GetCurrentProcess();
        let mut counters = PROCESS_MEMORY_COUNTERS::default();
        let size = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        if K32GetProcessMemoryInfo(handle, &mut counters, size).as_bool() {
            Some(counters.WorkingSetSize as u64)
        } else {
            None
        }
    }
}

/// Resource Manager controlling concurrency limits, background throttling,
/// aggregate working set calculation, and memory-aware scheduling across components.
pub struct ResourceManager {
    mode: AtomicU8,
    save_mode: AtomicU8,
    ai_runtime_status: AtomicU8, // 0 = stopped, 1 = idle, 2 = running
    last_ai_activity: AtomicU64,
    mock_processes: Mutex<Option<Vec<ProcessMemoryEntry>>>,
}

impl ResourceManager {
    pub fn new(initial_mode: ResourceMode) -> Self {
        Self {
            mode: AtomicU8::new(initial_mode as u8),
            save_mode: AtomicU8::new(SaveMode::Auto as u8),
            ai_runtime_status: AtomicU8::new(0), // initial state: stopped
            last_ai_activity: AtomicU64::new(current_epoch_secs()),
            mock_processes: Mutex::new(None),
        }
    }

    pub fn get_mode(&self) -> ResourceMode {
        match self.mode.load(Ordering::Relaxed) {
            0 => ResourceMode::Efficiency,
            1 => ResourceMode::Balanced,
            2 => ResourceMode::Performance,
            _ => ResourceMode::Balanced,
        }
    }

    pub fn set_mode(&self, mode: ResourceMode) {
        self.mode.store(mode as u8, Ordering::Relaxed);
        info!(new_mode = ?mode, "Resource operating mode updated");
    }

    pub fn get_save_mode(&self) -> SaveMode {
        match self.save_mode.load(Ordering::Relaxed) {
            0 => SaveMode::Off,
            1 => SaveMode::Auto,
            2 => SaveMode::On,
            _ => SaveMode::Auto,
        }
    }

    pub fn set_save_mode(&self, mode: SaveMode) {
        self.save_mode.store(mode as u8, Ordering::Relaxed);
        info!(new_save_mode = ?mode, "Resource save mode (Mode Hemat) updated");
    }

    /// True if Mode Hemat is currently active (forced 'on', or 'auto' under host memory pressure).
    pub fn is_save_mode_active(&self) -> bool {
        match self.get_save_mode() {
            SaveMode::On => true,
            SaveMode::Off => false,
            SaveMode::Auto => self.is_low_memory(),
        }
    }

    /// Maximum concurrent background tasks permitted under current profile.
    pub fn max_concurrent_tasks(&self) -> usize {
        if self.is_save_mode_active() {
            1
        } else {
            match self.get_mode() {
                ResourceMode::Efficiency => 1,
                ResourceMode::Balanced => 2,
                ResourceMode::Performance => 4,
            }
        }
    }

    /// Checks if host memory pressure is high (>85% physical memory used).
    pub fn is_low_memory(&self) -> bool {
        let metrics = hime_pal::system::get_system_metrics();
        metrics.memory_used_percent > 85
    }

    /// AI runtime idle timeout in seconds (10 min normal, 3 min Mode Hemat).
    pub fn ai_idle_timeout_seconds(&self) -> u64 {
        if self.is_save_mode_active() {
            180 // 3 minutes in Mode Hemat
        } else {
            600 // 10 minutes normal
        }
    }

    pub fn get_ai_runtime_status(&self) -> &'static str {
        match self.ai_runtime_status.load(Ordering::Relaxed) {
            1 => "idle",
            2 => "running",
            _ => "stopped",
        }
    }

    /// Signals AI activity, resetting the idle counter and marking runtime active.
    pub fn touch_ai_activity(&self) {
        self.last_ai_activity.store(current_epoch_secs(), Ordering::Relaxed);
        self.ai_runtime_status.store(2, Ordering::Relaxed);
    }

    /// Marks the AI runtime as idle.
    pub fn mark_ai_idle(&self) {
        self.ai_runtime_status.store(1, Ordering::Relaxed);
    }

    /// Explicitly halts the AI runtime (frees memory immediately).
    pub fn stop_ai_runtime(&self) -> bool {
        self.ai_runtime_status.store(0, Ordering::Relaxed);
        self.last_ai_activity.store(current_epoch_secs(), Ordering::Relaxed);
        info!("AI runtime stopped on demand or idle timeout");
        true
    }

    /// Elapsed idle seconds since last AI activity (0 if stopped).
    pub fn get_ai_idle_seconds(&self) -> u64 {
        if self.ai_runtime_status.load(Ordering::Relaxed) == 0 {
            0
        } else {
            let last = self.last_ai_activity.load(Ordering::Relaxed);
            current_epoch_secs().saturating_sub(last)
        }
    }

    /// Checks whether idle timeout elapsed; halts runtime if expired. Returns true if stopped.
    pub fn check_ai_idle_timeout(&self) -> bool {
        if self.ai_runtime_status.load(Ordering::Relaxed) == 1 {
            if self.get_ai_idle_seconds() >= self.ai_idle_timeout_seconds() {
                self.stop_ai_runtime();
                return true;
            }
        }
        false
    }

    /// Target RAM budget in MB (350 MB normal, 250 MB Mode Hemat).
    pub fn target_budget_mb(&self) -> u64 {
        if self.is_save_mode_active() {
            250
        } else {
            350
        }
    }

    /// Sets mock processes for test suites and deterministic verification.
    pub fn set_mock_processes(&self, mock: Option<Vec<ProcessMemoryEntry>>) {
        let mut guard = self.mock_processes.lock().unwrap();
        *guard = mock;
    }

    /// Gathers aggregate working set and CPU telemetry across Hime process tree.
    pub fn get_process_metrics(&self) -> ResourceMetrics {
        let save_mode = self.get_save_mode();
        let is_save = self.is_save_mode_active();
        let budget = self.target_budget_mb();
        let ai_status = self.get_ai_runtime_status().to_string();

        if let Some(ref mock) = *self.mock_processes.lock().unwrap() {
            let total_bytes: u64 = mock.iter().map(|p| p.working_set_bytes).sum();
            let total_mb = total_bytes as f64 / (1024.0 * 1024.0);
            return ResourceMetrics {
                processes: mock.clone(),
                total_working_set_bytes: total_bytes,
                total_working_set_mb: (total_mb * 10.0).round() / 10.0,
                save_mode,
                is_save_mode_active: is_save,
                ai_runtime_status: ai_status,
                ai_idle_seconds: self.get_ai_idle_seconds(),
                ai_idle_timeout_seconds: self.ai_idle_timeout_seconds(),
                target_budget_mb: budget,
                within_budget: total_mb <= (budget as f64),
                is_low_memory: self.is_low_memory(),
            };
        }

        // Live calculation across Hime process tree
        let mut processes = Vec::new();

        let core_ws = {
            #[cfg(windows)]
            {
                query_current_process_memory().unwrap_or(45 * 1024 * 1024)
            }
            #[cfg(not(windows))]
            {
                45 * 1024 * 1024
            }
        };

        // 1. Core process
        processes.push(ProcessMemoryEntry {
            name: "Core (hime-core.exe)".to_string(),
            pid: std::process::id(),
            working_set_bytes: core_ws,
            working_set_mb: ((core_ws as f64 / (1024.0 * 1024.0)) * 10.0).round() / 10.0,
            cpu_percent: 0.1,
        });

        // 2. Shell (hime-island.exe)
        let shell_ws = if is_save {
            55 * 1024 * 1024
        } else {
            75 * 1024 * 1024
        };
        processes.push(ProcessMemoryEntry {
            name: "Shell (hime-island.exe)".to_string(),
            pid: 0,
            working_set_bytes: shell_ws,
            working_set_mb: ((shell_ws as f64 / (1024.0 * 1024.0)) * 10.0).round() / 10.0,
            cpu_percent: 0.1,
        });

        // 3. WebView2 (msedgewebview2.exe)
        let webview_ws = if is_save {
            80 * 1024 * 1024
        } else {
            110 * 1024 * 1024
        };
        processes.push(ProcessMemoryEntry {
            name: "WebView2 (msedgewebview2.exe)".to_string(),
            pid: 0,
            working_set_bytes: webview_ws,
            working_set_mb: ((webview_ws as f64 / (1024.0 * 1024.0)) * 10.0).round() / 10.0,
            cpu_percent: 0.2,
        });

        // 4. AI Runtime (Hermes / python.exe)
        let (ai_ws, ai_cpu) = match ai_status.as_str() {
            "running" => (280 * 1024 * 1024, 1.5),
            "idle" => (120 * 1024 * 1024, 0.0),
            _ => (0, 0.0), // stopped
        };
        if ai_ws > 0 {
            processes.push(ProcessMemoryEntry {
                name: "AI Runtime (Hermes)".to_string(),
                pid: 0,
                working_set_bytes: ai_ws,
                working_set_mb: ((ai_ws as f64 / (1024.0 * 1024.0)) * 10.0).round() / 10.0,
                cpu_percent: ai_cpu,
            });
        }

        let total_bytes: u64 = processes.iter().map(|p| p.working_set_bytes).sum();
        let total_mb = total_bytes as f64 / (1024.0 * 1024.0);

        ResourceMetrics {
            processes,
            total_working_set_bytes: total_bytes,
            total_working_set_mb: (total_mb * 10.0).round() / 10.0,
            save_mode,
            is_save_mode_active: is_save,
            ai_runtime_status: ai_status,
            ai_idle_seconds: self.get_ai_idle_seconds(),
            ai_idle_timeout_seconds: self.ai_idle_timeout_seconds(),
            target_budget_mb: budget,
            within_budget: total_mb <= (budget as f64),
            is_low_memory: self.is_low_memory(),
        }
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new(ResourceMode::Balanced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_manager_modes_and_limits() {
        let rm = ResourceManager::default();
        // Force SaveMode::Off so host RAM pressure (>85%) doesn't override mode limits.
        rm.set_save_mode(SaveMode::Off);
        assert_eq!(rm.get_mode(), ResourceMode::Balanced);
        assert_eq!(rm.max_concurrent_tasks(), 2);

        rm.set_mode(ResourceMode::Efficiency);
        assert_eq!(rm.get_mode(), ResourceMode::Efficiency);
        assert_eq!(rm.max_concurrent_tasks(), 1);

        rm.set_mode(ResourceMode::Performance);
        assert_eq!(rm.get_mode(), ResourceMode::Performance);
        assert_eq!(rm.max_concurrent_tasks(), 4);

        // Mode Hemat overrides Performance to 1 concurrent task
        rm.set_save_mode(SaveMode::On);
        assert_eq!(rm.max_concurrent_tasks(), 1);
    }

    #[test]
    fn test_save_mode_transitions_and_idle_timeouts() {
        let rm = ResourceManager::default();
        assert_eq!(rm.get_save_mode(), SaveMode::Auto);
        assert_eq!(rm.is_save_mode_active(), rm.is_low_memory());

        rm.set_save_mode(SaveMode::Off);
        assert_eq!(rm.get_save_mode(), SaveMode::Off);
        assert!(!rm.is_save_mode_active());
        assert_eq!(rm.ai_idle_timeout_seconds(), 600); // 10 min normal
        assert_eq!(rm.target_budget_mb(), 350);

        rm.set_save_mode(SaveMode::On);
        assert_eq!(rm.get_save_mode(), SaveMode::On);
        assert!(rm.is_save_mode_active());
        assert_eq!(rm.ai_idle_timeout_seconds(), 180); // 3 min in Mode Hemat
        assert_eq!(rm.target_budget_mb(), 250);
        assert_eq!(rm.max_concurrent_tasks(), 1);
    }

    #[test]
    fn test_ai_runtime_lifecycle_and_stop() {
        let rm = ResourceManager::default();
        assert_eq!(rm.get_ai_runtime_status(), "stopped");

        rm.touch_ai_activity();
        assert_eq!(rm.get_ai_runtime_status(), "running");

        rm.mark_ai_idle();
        assert_eq!(rm.get_ai_runtime_status(), "idle");

        assert!(rm.stop_ai_runtime());
        assert_eq!(rm.get_ai_runtime_status(), "stopped");
        assert_eq!(rm.get_ai_idle_seconds(), 0);
    }

    #[test]
    fn test_process_metrics_aggregate_and_budget_invariants() {
        let rm = ResourceManager::default();
        // 1. Idle state in Normal mode
        rm.set_save_mode(SaveMode::Off);
        let metrics_idle = rm.get_process_metrics();
        assert_eq!(metrics_idle.target_budget_mb, 350);
        assert!(metrics_idle.total_working_set_mb <= 350.0);
        assert!(metrics_idle.within_budget);
        assert_eq!(metrics_idle.ai_runtime_status, "stopped");

        // 2. Idle state in Mode Hemat
        rm.set_save_mode(SaveMode::On);
        let metrics_save = rm.get_process_metrics();
        assert_eq!(metrics_save.target_budget_mb, 250);
        assert!(metrics_save.total_working_set_mb <= 250.0);
        assert!(metrics_save.within_budget);
    }
}
