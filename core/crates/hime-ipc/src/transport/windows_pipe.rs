use std::io;
use tokio::net::windows::named_pipe::{
    ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
};
use tracing::{debug, info};

pub const DEFAULT_HIME_PIPE: &str = r"\\.\pipe\hime-ipc";
pub const HERMES_HIME_PIPE: &str = r"\\.\pipe\hime-hermes";

/// Creates a new Windows Named Pipe server instance.
/// The first instance should set `first_instance(true)`.
pub fn create_pipe_server(pipe_name: &str, first_instance: bool) -> io::Result<NamedPipeServer> {
    info!(pipe = %pipe_name, first = first_instance, "Creating Windows Named Pipe server");
    let mut options = ServerOptions::new();
    options.first_pipe_instance(first_instance);
    options.reject_remote_clients(true); // Enforce localhost-only security
    options.create(pipe_name)
}

/// Connects to an existing Windows Named Pipe server instance.
pub async fn connect_pipe_client(pipe_name: &str) -> io::Result<NamedPipeClient> {
    debug!(pipe = %pipe_name, "Connecting to Windows Named Pipe client");
    ClientOptions::new().open(pipe_name)
}
