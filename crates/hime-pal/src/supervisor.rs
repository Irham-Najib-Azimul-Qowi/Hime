use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;
use tracing::{error, info};

#[cfg(windows)]
use windows::core::PWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
#[cfg(windows)]
use windows::Win32::System::Threading::{
    CreateProcessW, ResumeThread, TerminateProcess, CREATE_SUSPENDED, PROCESS_INFORMATION,
    STARTUPINFOW,
};

#[derive(Error, Debug)]
pub enum SupervisorError {
    #[error("Failed to create Win32 Job Object: {0}")]
    CreateJobFailed(String),

    #[error("Failed to set Job Object limits (KILL_ON_JOB_CLOSE): {0}")]
    SetLimitFailed(String),

    #[error("Failed to create suspended process: {0}")]
    SpawnSuspendedFailed(String),

    #[error("Failed to assign process to Job Object (Fail-Closed triggered): {0}")]
    JobAssignmentFailed(String),

    #[error("Failed to resume thread: {0}")]
    ResumeThreadFailed(String),

    #[error("Platform unsupported")]
    UnsupportedPlatform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

#[derive(Debug)]
pub struct SubprocessHandle {
    pub name: String,
    pub pid: u32,
    pub state: ProcessState,
    #[cfg(windows)]
    process_handle: HANDLE,
}

#[cfg(windows)]
unsafe impl Send for SubprocessHandle {}
#[cfg(windows)]
unsafe impl Sync for SubprocessHandle {}

impl Drop for SubprocessHandle {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            if !self.process_handle.is_invalid() {
                let _ = CloseHandle(self.process_handle);
            }
        }
    }
}

/// Win32 Job Object Process Supervisor encapsulated within Platform Abstraction Layer (PAL).
///
/// Ensures child processes are assigned to a kernel Job Object with KILL_ON_JOB_CLOSE
/// before their main thread is resumed.
pub struct JobSupervisor {
    #[cfg(windows)]
    job_handle: HANDLE,
    processes: Mutex<HashMap<String, SubprocessHandle>>,
}

#[cfg(windows)]
unsafe impl Send for JobSupervisor {}
#[cfg(windows)]
unsafe impl Sync for JobSupervisor {}

impl JobSupervisor {
    pub fn new(job_name: Option<&str>) -> Result<Self, SupervisorError> {
        #[cfg(windows)]
        {
            let name_wide: Option<Vec<u16>> =
                job_name.map(|n| n.encode_utf16().chain(Some(0)).collect());
            let pwstr = match &name_wide {
                Some(w) => PWSTR(w.as_ptr() as *mut u16),
                None => PWSTR::null(),
            };

            let job = unsafe { CreateJobObjectW(None, pwstr) }
                .map_err(|e| SupervisorError::CreateJobFailed(e.to_string()))?;

            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let set_res = unsafe {
                SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const std::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            };

            if let Err(e) = set_res {
                unsafe {
                    let _ = CloseHandle(job);
                }
                return Err(SupervisorError::SetLimitFailed(e.to_string()));
            }

            info!("Initialized Windows Job Object with KILL_ON_JOB_CLOSE limit (PAL)");
            Ok(Self {
                job_handle: job,
                processes: Mutex::new(HashMap::new()),
            })
        }

        #[cfg(not(windows))]
        {
            Ok(Self {
                processes: Mutex::new(HashMap::new()),
            })
        }
    }

    /// Spawns an external subprocess with CREATE_SUSPENDED, assigns it to the Job Object,
    /// and only resumes the main thread if assignment succeeds (Fail-Closed).
    pub fn spawn_child(&self, name: &str, command_line: &str) -> Result<u32, SupervisorError> {
        #[cfg(windows)]
        {
            info!(name = %name, cmd = %command_line, "Spawning child under Job Object via PAL (CREATE_SUSPENDED)");

            let mut cmd_wide: Vec<u16> = command_line.encode_utf16().chain(Some(0)).collect();
            let startup_info = STARTUPINFOW::default();
            let mut proc_info = PROCESS_INFORMATION::default();

            let create_res = unsafe {
                CreateProcessW(
                    PWSTR::null(),
                    PWSTR(cmd_wide.as_mut_ptr()),
                    None,
                    None,
                    false,
                    CREATE_SUSPENDED,
                    None,
                    PWSTR::null(),
                    &startup_info,
                    &mut proc_info,
                )
            };

            if let Err(e) = create_res {
                return Err(SupervisorError::SpawnSuspendedFailed(e.to_string()));
            }

            let h_process = proc_info.hProcess;
            let h_thread = proc_info.hThread;
            let pid = proc_info.dwProcessId;

            // Fail-Closed assignment policy: assign to job before resuming
            let assign_res = unsafe { AssignProcessToJobObject(self.job_handle, h_process) };

            if let Err(e) = assign_res {
                error!(name = %name, pid = pid, err = %e, "Assignment to Job Object failed. Triggering FAIL-CLOSED termination.");
                unsafe {
                    let _ = TerminateProcess(h_process, 1);
                    let _ = CloseHandle(h_thread);
                    let _ = CloseHandle(h_process);
                }
                return Err(SupervisorError::JobAssignmentFailed(e.to_string()));
            }

            // Resume thread only after successful assignment
            let resume_res = unsafe { ResumeThread(h_thread) };
            unsafe {
                let _ = CloseHandle(h_thread);
            };

            if resume_res == u32::MAX {
                let os_err = std::io::Error::last_os_error();
                unsafe {
                    let _ = TerminateProcess(h_process, 1);
                    let _ = CloseHandle(h_process);
                }
                return Err(SupervisorError::ResumeThreadFailed(os_err.to_string()));
            }

            info!(name = %name, pid = pid, "Child process successfully assigned to Job Object and resumed (PAL)");

            let handle = SubprocessHandle {
                name: name.to_string(),
                pid,
                state: ProcessState::Running,
                process_handle: h_process,
            };

            let mut map = self.processes.lock().unwrap();
            map.insert(name.to_string(), handle);

            Ok(pid)
        }

        #[cfg(not(windows))]
        {
            Err(SupervisorError::UnsupportedPlatform)
        }
    }

    pub fn terminate_child(&self, name: &str) -> bool {
        #[cfg(windows)]
        {
            let mut map = self.processes.lock().unwrap();
            if let Some(mut proc) = map.remove(name) {
                unsafe {
                    let _ = TerminateProcess(proc.process_handle, 0);
                }
                proc.state = ProcessState::Stopped;
                info!(name = %name, pid = proc.pid, "Child process explicitly terminated (PAL)");
                return true;
            }
            false
        }

        #[cfg(not(windows))]
        {
            false
        }
    }
}

impl Drop for JobSupervisor {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            if !self.job_handle.is_invalid() {
                info!("Closing Job Object handle (PAL) — triggering kernel termination of all member processes");
                let _ = CloseHandle(self.job_handle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_pal_job_supervisor_creation() {
        let supervisor = JobSupervisor::new(None);
        assert!(
            supervisor.is_ok(),
            "PAL JobSupervisor creation must succeed on Windows"
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_pal_job_supervisor_spawn_and_terminate() {
        let supervisor = JobSupervisor::new(None).unwrap();
        let pid = supervisor.spawn_child("dummy-worker", "cmd.exe /c ping -n 10 127.0.0.1");
        assert!(pid.is_ok(), "Spawning child under job object via PAL must succeed");

        let terminated = supervisor.terminate_child("dummy-worker");
        assert!(terminated, "Explicit termination of child via PAL must succeed");
    }
}
