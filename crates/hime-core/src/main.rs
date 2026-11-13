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

    // Generate runtime session authentication token
    let auth_token = generate_token_256().unwrap_or_else(|_| "hime-default-dev-token".to_string());
    info!("Generated runtime IPC authentication token");

    // Initialize Core Server
    let server = Arc::new(CoreServer::new(auth_token.clone(), storage.clone()));
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
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
            }
        });
    }

    info!("Hime Core is ready and running. Waiting for shutdown signal...");

    // Wait for Ctrl+C or IPC ShutdownInitiated event
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Ctrl+C signal received. Commencing graceful shutdown.");
        }
        event = event_rx.recv() => {
            if let Ok(SystemEvent::ShutdownInitiated) = event {
                info!("IPC shutdown signal received. Commencing graceful shutdown.");
            }
        }
    }

    info!("Gracefully terminating supervised processes...");
    drop(supervisor);

    info!("Hime Core successfully stopped.");
    Ok(())
}
