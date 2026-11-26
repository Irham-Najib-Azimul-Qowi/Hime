#![windows_subsystem = "windows"]

use std::sync::Arc;
use tokio::signal;
use tracing::{error, info};

use hime_core::{init_logging, CoreServer, JobSupervisor, SystemEvent};
use hime_pal::PlatformPaths;
use hime_security::csprng::generate_token_256;
use hime_storage::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging("info");
    info!("Starting Hime Core Kernel v0.1.0 (Modular Monolith)");

    // Single-instance enforcement on Windows
    #[cfg(windows)]
    let _single_instance_mutex = {
        use windows::core::w;
        use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
        use windows::Win32::System::Threading::CreateMutexW;

        let handle = unsafe { CreateMutexW(None, true, w!("Global\\HimeCoreSingleInstanceMutex")) };
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            tracing::warn!("Another instance of Hime Core is already running. Exiting cleanly.");
            return Ok(());
        }
        handle
    };

    // Initialize platform directories
    let data_dir = PlatformPaths::app_data_dir();
    std::fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("hime_core.db");

    // Initialize local storage database
    info!(db = %db_path.display(), "Opening SQLite storage engine");
    let storage = Arc::new(Database::open(&db_path)?);

    // Initialize Win32 Job Object Supervisor
    let supervisor = Arc::new(JobSupervisor::new(Some("HimeSupervisorJob"))?);
    info!("Process Supervisor successfully attached to Windows Job Object");

    // Read runtime session authentication token from environment or generate
    let auth_token = std::env::var("HIME_AUTH_TOKEN")
        .unwrap_or_else(|_| generate_token_256().unwrap_or_else(|_| "hime-default-dev-token".to_string()));
    let token_path = data_dir.join("session.token");
    let _ = std::fs::write(&token_path, &auth_token);
    info!(token_file = %token_path.display(), "IPC authentication token active");

    // Initialize Core Server
    let server = Arc::new(CoreServer::new(auth_token.clone(), storage.clone()));
    server.proactive_engine.clone().start_reminder_scheduler();
    let mut event_rx = server.event_bus.subscribe();

    // Start Windows Named Pipe server loop
    #[cfg(windows)]
    {
        use hime_ipc::transport::windows_pipe::{create_pipe_server, DEFAULT_HIME_PIPE};

        let server_clone = server.clone();
        tokio::spawn(async move {
            info!(pipe = %DEFAULT_HIME_PIPE, "Listening on Windows Named Pipe");
            let mut first = true;
            loop {
                match create_pipe_server(DEFAULT_HIME_PIPE, first) {
                    Ok(pipe_server) => {
                        first = false;
                        if let Err(e) = pipe_server.connect().await {
                            error!(err = %e, "Pipe server connection handshake failed");
                            continue;
                        }
                        let s = server_clone.clone();
                        tokio::spawn(async move {
                            s.handle_connection(pipe_server).await;
                        });
                    }
                    Err(e) => {
                        error!(err = %e, "Failed to create pipe server instance; backing off");
                        // If first instance attempt failed because pipe already exists or is recovering,
                        // gracefully switch first to false for subsequent attempts
                        first = false;
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
            }
        });
    }

    info!("Hime Core is ready and running. Waiting for shutdown signal...");

    #[cfg(windows)]
    let (hime_quit_tx, hime_quit_rx) = tokio::sync::oneshot::channel::<()>();
    #[cfg(windows)]
    {
        std::thread::spawn(move || {
            use windows::core::w;
            use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
            use windows::Win32::System::Threading::{CreateEventW, ResetEvent, WaitForSingleObject, INFINITE};

            let handle = unsafe { CreateEventW(None, true, false, w!("HimeQuit")) };
            match handle {
                Ok(h_event) => {
                    // Always reset event state on fresh startup so stale quit signals from prior runs don't cause instant termination
                    let _ = unsafe { ResetEvent(h_event) };
                    let res = unsafe { WaitForSingleObject(h_event, INFINITE) };
                    if res == WAIT_OBJECT_0 {
                        let _ = hime_quit_tx.send(());
                    }
                    unsafe {
                        let _ = CloseHandle(h_event);
                    }
                }
                Err(e) => {
                    tracing::warn!(err = %e, "Failed to create/open HimeQuit named event in Core");
                }
            }
        });
    }

    // Wait for Ctrl+C, IPC ShutdownInitiated event, or HimeQuit named event
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Ctrl+C signal received. Commencing graceful shutdown.");
        }
        _ = async {
            #[cfg(windows)]
            {
                let _ = hime_quit_rx.await;
                info!("HimeQuit named event signaled. Commencing graceful shutdown.");
            }
            #[cfg(not(windows))]
            {
                std::future::pending::<()>().await;
            }
        } => {}
        _ = async {
            while let Ok(event) = event_rx.recv().await {
                if let SystemEvent::ShutdownInitiated = event {
                    info!("IPC shutdown signal received. Commencing graceful shutdown.");
                    break;
                }
            }
        } => {}
    }

    info!("Gracefully terminating supervised processes...");
    drop(supervisor);

    info!("Flushing and closing SQLite storage engine...");
    drop(storage);

    info!("Hime Core successfully stopped.");
    Ok(())
}
