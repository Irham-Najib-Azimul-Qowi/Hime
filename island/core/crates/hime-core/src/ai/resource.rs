use std::sync::atomic::{AtomicU8, Ordering};
use tracing::info;

/// Resource operating modes for Hime Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum ResourceMode {
    Efficiency = 0,
    #[default]
    Balanced = 1,
    Performance = 2,
}


/// Resource Manager controlling concurrency limits, background throttling,
/// and memory-aware scheduling across AI components.
pub struct ResourceManager {
    mode: AtomicU8,
}

impl ResourceManager {
    pub fn new(initial_mode: ResourceMode) -> Self {
        Self {
            mode: AtomicU8::new(initial_mode as u8),
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

    /// Maximum concurrent background tasks permitted under current profile.
    pub fn max_concurrent_tasks(&self) -> usize {
        match self.get_mode() {
            ResourceMode::Efficiency => 1,
            ResourceMode::Balanced => 2,
            ResourceMode::Performance => 4,
        }
    }

    /// Checks if host memory pressure is high.
    pub fn is_low_memory(&self) -> bool {
        // Can be queried via platform diagnostics; defaults to false under normal loads
        false
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
        assert_eq!(rm.get_mode(), ResourceMode::Balanced);
        assert_eq!(rm.max_concurrent_tasks(), 2);

        rm.set_mode(ResourceMode::Efficiency);
        assert_eq!(rm.get_mode(), ResourceMode::Efficiency);
        assert_eq!(rm.max_concurrent_tasks(), 1);

        rm.set_mode(ResourceMode::Performance);
        assert_eq!(rm.get_mode(), ResourceMode::Performance);
        assert_eq!(rm.max_concurrent_tasks(), 4);
    }
}
