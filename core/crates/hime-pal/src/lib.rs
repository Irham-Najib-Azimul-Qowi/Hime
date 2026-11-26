pub mod paths;
pub mod selection;
pub mod supervisor;
pub mod system;
pub mod window;

pub use paths::PlatformPaths;
pub use selection::{SelectionError, SelectionReader};
pub use supervisor::{JobSupervisor, ProcessState, SubprocessHandle, SupervisorError};
pub use system::{
    cleanup_active_marker_on_quit, cleanup_active_marker_with_policy, get_system_metrics,
    is_os_shutting_down, SystemMetrics,
};
pub use window::{PalWindowHelper, WindowGeometry};
