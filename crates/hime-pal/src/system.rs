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
