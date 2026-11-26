//! System information and hardware telemetry abstraction.

#[cfg(windows)]
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

/// Snapshot of physical system and environment metrics.
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub os_name: String,
    pub arch: String,
    pub hostname: String,
    pub username: String,
    pub cpu_cores: usize,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub memory_used_percent: u32,
}

/// Gathers current system performance and environment telemetry.
pub fn get_system_metrics() -> SystemMetrics {
    let os_name = "Windows".to_string();
    let arch = std::env::consts::ARCH.to_string();
    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string());
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "User".to_string());
    let cpu_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let (total_memory_mb, available_memory_mb, memory_used_percent) = {
        #[cfg(windows)]
        {
            let mut mem = MEMORYSTATUSEX::default();
            mem.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            unsafe {
                if GlobalMemoryStatusEx(&mut mem).is_ok() {
                    let total = mem.ullTotalPhys / (1024 * 1024);
                    let avail = mem.ullAvailPhys / (1024 * 1024);
                    let load = mem.dwMemoryLoad;
                    (total, avail, load)
                } else {
                    (0, 0, 0)
                }
            }
        }
        #[cfg(not(windows))]
        {
            (0, 0, 0)
        }
    };

    SystemMetrics {
        os_name,
        arch,
        hostname,
        username,
        cpu_cores,
        total_memory_mb,
        available_memory_mb,
        memory_used_percent,
    }
}

/// Returns true if the host operating system is shutting down or the user is logging off.
/// On Windows, queries `GetSystemMetrics(SM_SHUTTINGDOWN)`.
pub fn is_os_shutting_down() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_SHUTTINGDOWN};
        unsafe { GetSystemMetrics(SM_SHUTTINGDOWN) != 0 }
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Evaluates shutdown persistence policy for the active marker file.
/// If `is_os_shutting_down_fn()` returns true (OS shutdown or restart in progress),
/// the marker file is preserved to support persistence across reboots, returning `Ok(false)`.
/// If `is_os_shutting_down_fn()` returns false (user-initiated quit), the marker file
/// is deleted, returning `Ok(true)` if removed (or `Ok(false)` if absent).
pub fn cleanup_active_marker_with_policy<P: AsRef<std::path::Path>, F: Fn() -> bool>(
    marker_path: P,
    is_os_shutting_down_fn: F,
) -> std::io::Result<bool> {
    if is_os_shutting_down_fn() {
        // OS shutdown/reboot: preserve marker file across reboot
        Ok(false)
    } else {
        // User quit: clean up marker file
        let path = marker_path.as_ref();
        if path.exists() {
            std::fs::remove_file(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Encapsulates marker cleanup using the live host OS shutdown discrimination.
pub fn cleanup_active_marker_on_quit<P: AsRef<std::path::Path>>(
    marker_path: P,
) -> std::io::Result<bool> {
    cleanup_active_marker_with_policy(marker_path, is_os_shutting_down)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_metrics_query() {
        let metrics = get_system_metrics();
        assert!(!metrics.os_name.is_empty());
        assert!(!metrics.arch.is_empty());
        assert!(metrics.cpu_cores > 0);
        #[cfg(windows)]
        {
            assert!(metrics.total_memory_mb > 0);
        }
    }
}
