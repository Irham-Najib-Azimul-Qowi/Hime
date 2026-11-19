#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::io;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
#[cfg(unix)]
use tracing::{debug, info};

pub const DEFAULT_HIME_UNIX_SOCKET: &str = "/tmp/hime-core.sock";

#[cfg(unix)]
/// Creates a Unix Domain Socket listener with strict 0600 (user-only) permissions.
pub fn create_unix_listener<P: AsRef<Path>>(socket_path: P) -> io::Result<UnixListener> {
    let path = socket_path.as_ref();
    info!(socket = %path.display(), "Creating Unix domain socket server");

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Clean up stale socket file if present
    if path.exists() {
        let _ = fs::remove_file(path);
    }

    let listener = UnixListener::bind(path)?;

    // Set permission to 0600 (read/write by owner only)
    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms)?;

    Ok(listener)
}

#[cfg(unix)]
/// Connects to a Unix Domain Socket server.
pub async fn connect_unix_client<P: AsRef<Path>>(socket_path: P) -> io::Result<UnixStream> {
    let path = socket_path.as_ref();
    debug!(socket = %path.display(), "Connecting to Unix domain socket client");
    UnixStream::connect(path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_socket_path_constant() {
        assert!(DEFAULT_HIME_UNIX_SOCKET.ends_with(".sock"));
    }
}

