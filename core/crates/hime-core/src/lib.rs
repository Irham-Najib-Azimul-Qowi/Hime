pub mod ai;
pub mod browser;
pub mod bus;
pub mod developer;
pub mod device;
pub mod documents;
pub mod enforce;
pub mod integrations;
pub mod logging;
pub mod lyrics;
pub mod module_installer;
pub mod ports;
pub mod productivity;
pub mod reminders;
pub mod routines;
pub mod server;
pub mod storage;
pub mod supervisor;

pub use ai::{
    AgentActivityEvent, AiOrchestrator, AiRuntimeOverview, ConversationService,
    HermesRuntimeAdapter, HermesRuntimeStatus, ModelInfo, ProviderCredentialAdapter,
    RouterGatewayAdapter, RouterHealthStatus,
};
pub use browser::BrowserService;
pub use bus::{EventBus, SystemEvent};
pub use developer::DeveloperEngine;
pub use device::DeviceManager;
pub use documents::DocumentEngine;
pub use enforce::{Commitment, EmergencyPauseStatus, EnforcementAction, EnforcementManager};
pub use integrations::{GitHubAdapter, GoogleDriveAdapter};
pub use logging::init_logging;
pub use lyrics::{LyricLine, LyricsResult, LyricsService};
pub use module_installer::{
    ModuleInstallOutcome, ModuleInstallState, ModuleInstaller, ModuleInstallerError,
    ModulePackageManifest, ModuleRollbackOutcome,
};
pub use productivity::ProductivityEngine;
pub use reminders::{
    calculate_coalesced_next_wake, NextWakeEvent, NextWakeEventKind, ReminderScheduler,
};
pub use routines::{
    AutoEscalationConfig, DistractionRules, EnforcementTier, Routine, RoutineManager,
    RoutineSchedule, RoutineSession, RoutineSessionState, RoutineStreak, Weekday,
};
pub use server::{ApprovalRecord, CoreServer, TaskRecord};
pub use storage::StorageEngine;
pub use supervisor::{JobSupervisor, ProcessState, SubprocessHandle, SupervisorError};
pub use hime_pal::{
    cleanup_active_marker_on_quit, cleanup_active_marker_with_policy, is_os_shutting_down,
};
