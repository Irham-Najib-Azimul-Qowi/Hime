use std::path::PathBuf;

pub struct PlatformPaths;

impl PlatformPaths {
    /// Returns the Hime application data directory (%APPDATA%\Hime on Windows, ~/.config/hime on Linux).
    pub fn app_data_dir() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join("Hime")
        } else if let Ok(home) = std::env::var("USERPROFILE") {
            PathBuf::from(home).join(".hime")
        } else {
            PathBuf::from(".hime")
        }
    }

    /// Returns the Hime local cache and runtime directory (%LOCALAPPDATA%\Hime on Windows).
    pub fn cache_dir() -> PathBuf {
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(local_appdata).join("Hime")
        } else {
            Self::app_data_dir().join("cache")
        }
    }

    /// Returns the temporary working directory for Hime.
    pub fn temp_dir() -> PathBuf {
        std::env::temp_dir().join("Hime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_paths() {
        let app_data = PlatformPaths::app_data_dir();
        assert!(app_data.to_str().unwrap().contains("Hime"));
        let temp = PlatformPaths::temp_dir();
        assert!(temp.to_str().unwrap().contains("Hime"));
    }
}
