use std::path::PathBuf;

pub struct PlatformPaths;

impl PlatformPaths {
    /// Returns the Hime application configuration directory (%APPDATA%\Hime on Windows, ~/.config/hime on Linux/macOS).
    pub fn app_data_dir() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join("Hime")
        } else if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_config).join("hime")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("hime")
        } else if let Ok(home) = std::env::var("USERPROFILE") {
            PathBuf::from(home).join(".hime")
        } else {
            PathBuf::from(".hime")
        }
    }

    /// Returns the Hime local cache directory (%LOCALAPPDATA%\Hime on Windows, ~/.cache/hime on Linux).
    pub fn cache_dir() -> PathBuf {
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(local_appdata).join("Hime")
        } else if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
            PathBuf::from(xdg_cache).join("hime")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".cache").join("hime")
        } else {
            Self::app_data_dir().join("cache")
        }
    }

    /// Returns the Hime local state/data directory (%LOCALAPPDATA%\Hime\data on Windows, ~/.local/share/hime on Linux).
    pub fn data_dir() -> PathBuf {
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(local_appdata).join("Hime").join("data")
        } else if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg_data).join("hime")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".local").join("share").join("hime")
        } else {
            Self::app_data_dir().join("data")
        }
    }

    /// Returns the Hime runtime directory for sockets/pipes ($XDG_RUNTIME_DIR/hime on Linux, %LOCALAPPDATA%\Hime\run on Windows).
    pub fn runtime_dir() -> PathBuf {
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(xdg_runtime).join("hime")
        } else {
            Self::cache_dir().join("run")
        }
    }

    /// Returns the temporary working directory for Hime.
    pub fn temp_dir() -> PathBuf {
        std::env::temp_dir().join("Hime")
    }

    /// Returns the default IPC socket address (Named Pipe name on Windows, Unix domain socket path on Linux/Unix).
    pub fn default_ipc_endpoint() -> String {
        #[cfg(windows)]
        {
            r"\\.\pipe\hime-core-ipc".to_string()
        }
        #[cfg(not(windows))]
        {
            Self::runtime_dir().join("hime-core.sock").to_string_lossy().to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_paths() {
        let app_data = PlatformPaths::app_data_dir();
        assert!(app_data.to_str().unwrap().to_lowercase().contains("hime"));
        let cache = PlatformPaths::cache_dir();
        assert!(cache.to_str().unwrap().to_lowercase().contains("hime"));
        let data = PlatformPaths::data_dir();
        assert!(data.to_str().unwrap().to_lowercase().contains("hime"));
        let runtime = PlatformPaths::runtime_dir();
        assert!(runtime.to_str().unwrap().to_lowercase().contains("hime"));
        let temp = PlatformPaths::temp_dir();
        assert!(temp.to_str().unwrap().to_lowercase().contains("hime"));
    }

    #[test]
    fn test_default_ipc_endpoint() {
        let endpoint = PlatformPaths::default_ipc_endpoint();
        #[cfg(windows)]
        {
            assert!(endpoint.starts_with(r"\\.\pipe\"));
        }
        #[cfg(not(windows))]
        {
            assert!(endpoint.ends_with(".sock"));
        }
    }
}
