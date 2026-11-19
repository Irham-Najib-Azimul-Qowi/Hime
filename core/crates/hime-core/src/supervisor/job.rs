//! Process Supervisor delegation to Hime Platform Abstraction Layer (PAL).
//!
//! Encapsulates OS-level Job Object supervision within `hime-pal` to uphold
//! the architecture rule: "Core must not depend directly on Windows APIs outside PAL".

pub use hime_pal::supervisor::{
    JobSupervisor, ProcessState, SubprocessHandle, SupervisorError,
};

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_job_supervisor_creation() {
        let supervisor = JobSupervisor::new(None);
        assert!(
            supervisor.is_ok(),
            "JobSupervisor creation must succeed on Windows"
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_job_supervisor_spawn_and_terminate() {
        let supervisor = JobSupervisor::new(None).unwrap();
        let pid = supervisor.spawn_child("dummy-worker", "cmd.exe /c ping -n 10 127.0.0.1");
        assert!(pid.is_ok(), "Spawning child under job object must succeed");

        let terminated = supervisor.terminate_child("dummy-worker");
        assert!(terminated, "Explicit termination of child must succeed");
    }
}
