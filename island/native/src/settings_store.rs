//! Persistent settings store compatible with `%APPDATA%\ai.hime.desktop\settings.json`.
//!
//! Reads and writes the legacy `bloom-*` and `hime-*` keys used by the Tauri/WebView
//! frontend while preserving any unknown keys present in `settings.json`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

const SETTINGS_DIR: &str = "ai.hime.desktop";
const SETTINGS_FILE: &str = "settings.json";

pub const DEFAULT_ROUTER_URL: &str = "http://127.0.0.1:20128/v1";
pub const DEFAULT_ROUTER_MODEL: &str = "hime-combo";

/// Canonical native shell settings persisted in `%APPDATA%\ai.hime.desktop\settings.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NativeSettings {
    /// `"fixed"` | `"smart"` | `"peek"`
    pub notch_mode: String,
    pub free_drag_enabled: bool,
    /// `"top"` | `"left"` | `"right"`
    pub notch_edge: String,
    pub time_format_24h: bool,
    pub music_mode_enabled: bool,
    pub suppress_native_notifications: bool,

    pub dock_enabled: bool,
    /// `"fixed"` | `"auto-hide"` | `"smart"`
    pub dock_mode: String,
    pub dock_pinned: Vec<String>,
    pub dock_win_number_enabled: bool,

    pub volume_overlay_enabled: bool,
    pub brightness_overlay_enabled: bool,

    pub hime_ai_enabled: bool,
    pub hime_router_url: String,
    pub hime_router_model: String,
}

impl Default for NativeSettings {
    fn default() -> Self {
        Self {
            notch_mode: "fixed".to_string(),
            free_drag_enabled: false,
            notch_edge: "top".to_string(),
            time_format_24h: true,
            music_mode_enabled: true,
            suppress_native_notifications: true,
            dock_enabled: true,
            dock_mode: "fixed".to_string(),
            dock_pinned: Vec::new(),
            dock_win_number_enabled: true,
            volume_overlay_enabled: true,
            brightness_overlay_enabled: true,
            hime_ai_enabled: true,
            hime_router_url: DEFAULT_ROUTER_URL.to_string(),
            hime_router_model: DEFAULT_ROUTER_MODEL.to_string(),
        }
    }
}

impl NativeSettings {
    /// Normalizes enum strings (`notch_mode`, `notch_edge`, `dock_mode`) and URL/model defaults.
    pub fn normalized(&self) -> Self {
        let mut out = self.clone();
        out.notch_mode = normalize_notch_mode(&out.notch_mode).to_string();
        out.notch_edge = normalize_notch_edge(&out.notch_edge).to_string();
        out.dock_mode = normalize_dock_mode(&out.dock_mode).to_string();
        if out.hime_router_url.trim().is_empty() {
            out.hime_router_url = DEFAULT_ROUTER_URL.to_string();
        } else {
            out.hime_router_url = out.hime_router_url.trim().to_string();
        }
        if out.hime_router_model.trim().is_empty() {
            out.hime_router_model = DEFAULT_ROUTER_MODEL.to_string();
        } else {
            out.hime_router_model = out.hime_router_model.trim().to_string();
        }
        out
    }

    /// Cycles `notch_mode`: `"fixed"` -> `"smart"` -> `"peek"` -> `"fixed"`.
    pub fn cycle_notch_mode(&mut self) -> &str {
        self.notch_mode = match normalize_notch_mode(&self.notch_mode) {
            "fixed" => "smart",
            "smart" => "peek",
            _ => "fixed",
        }
        .to_string();
        &self.notch_mode
    }

    /// Cycles `notch_edge`: `"top"` -> `"left"` -> `"right"` -> `"top"`.
    pub fn cycle_notch_edge(&mut self) -> &str {
        self.notch_edge = match normalize_notch_edge(&self.notch_edge) {
            "top" => "left",
            "left" => "right",
            _ => "top",
        }
        .to_string();
        &self.notch_edge
    }

    /// Cycles `dock_mode`: `"fixed"` -> `"auto-hide"` -> `"smart"` -> `"fixed"`.
    pub fn cycle_dock_mode(&mut self) -> &str {
        self.dock_mode = match normalize_dock_mode(&self.dock_mode) {
            "fixed" => "auto-hide",
            "auto-hide" => "smart",
            _ => "fixed",
        }
        .to_string();
        &self.dock_mode
    }
}

/// Persistence helper for `%APPDATA%\ai.hime.desktop\settings.json`.
pub struct SettingsStore;

impl SettingsStore {
    /// Returns `%APPDATA%\ai.hime.desktop\settings.json`.
    pub fn default_path() -> Option<PathBuf> {
        std::env::var_os("APPDATA").map(|dir| {
            PathBuf::from(dir).join(SETTINGS_DIR).join(SETTINGS_FILE)
        })
    }

    /// Loads settings from `%APPDATA%\ai.hime.desktop\settings.json`, falling back to defaults.
    pub fn load() -> NativeSettings {
        Self::default_path()
            .and_then(|p| Self::load_from_path(&p).ok())
            .unwrap_or_default()
    }

    /// Saves `settings` to `%APPDATA%\ai.hime.desktop\settings.json`.
    pub fn save(settings: &NativeSettings) -> Result<(), String> {
        let path = Self::default_path()
            .ok_or_else(|| "APPDATA environment variable is not set".to_string())?;
        Self::save_to_path(&path, settings)
    }

    /// Loads `NativeSettings` from an explicit `settings.json` path.
    pub fn load_from_path(path: &Path) -> Result<NativeSettings, String> {
        match fs::read_to_string(path) {
            Ok(raw) => Self::from_json_str(&raw),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(NativeSettings::default()),
            Err(err) => Err(format!("Failed to read {}: {err}", path.display())),
        }
    }

    /// Saves `settings` into `path` atomically, merging into existing keys to preserve
    /// unrelated Tauri/WebView settings.
    pub fn save_to_path(path: &Path, settings: &NativeSettings) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create {}: {e}", parent.display()))?;
        }

        let mut map = match fs::read_to_string(path) {
            Ok(raw) => {
                let cleaned = raw.trim_start_matches('\u{feff}');
                serde_json::from_str::<Map<String, Value>>(cleaned).unwrap_or_default()
            }
            Err(_) => Map::new(),
        };

        Self::write_into_map(&mut map, settings);
        let bytes = serde_json::to_vec_pretty(&map)
            .map_err(|e| format!("Failed to serialize settings JSON: {e}"))?;
        atomic_write_file(path, &bytes)
    }

    /// Parses either a legacy `bloom-*`/`hime-*` key-value map or direct `NativeSettings` JSON.
    pub fn from_json_str(raw: &str) -> Result<NativeSettings, String> {
        let cleaned = raw.trim_start_matches('\u{feff}').trim();
        if cleaned.is_empty() {
            return Ok(NativeSettings::default());
        }
        let map: Map<String, Value> = serde_json::from_str(cleaned)
            .map_err(|e| format!("Invalid settings JSON: {e}"))?;
        Ok(Self::from_json_map(&map))
    }

    /// Extracts `NativeSettings` from a `serde_json::Map`, supporting both legacy `bloom-*` /
    /// `hime-*` stringified keys and direct snake_case keys.
    pub fn from_json_map(map: &Map<String, Value>) -> NativeSettings {
        let def = NativeSettings::default();

        let notch_mode = read_str(map, &["bloom-notch-mode", "notch_mode"])
            .map(|s| normalize_notch_mode(&s).to_string())
            .unwrap_or(def.notch_mode);

        let free_drag_enabled = read_bool(
            map,
            &["bloom-free-drag-enabled", "free_drag_enabled"],
            def.free_drag_enabled,
        );

        let notch_edge = read_str(map, &["bloom-notch-edge", "notch_edge"])
            .or_else(|| {
                read_str(map, &["bloom-notch-position"]).and_then(|pos_raw| {
                    serde_json::from_str::<Value>(&pos_raw)
                        .ok()
                        .and_then(|v| v.get("edge").and_then(|e| e.as_str()).map(str::to_string))
                })
            })
            .map(|s| normalize_notch_edge(&s).to_string())
            .unwrap_or(def.notch_edge);

        let time_format_24h = read_bool(
            map,
            &["bloom-time-format-24h", "time_format_24h"],
            def.time_format_24h,
        );
        let music_mode_enabled = read_bool(
            map,
            &["bloom-music-mode-enabled", "music_mode_enabled"],
            def.music_mode_enabled,
        );
        let suppress_native_notifications = read_bool(
            map,
            &[
                "bloom-suppress-native-notifications",
                "suppress_native_notifications",
            ],
            def.suppress_native_notifications,
        );

        let dock_enabled = read_bool(map, &["bloom-dock-enabled", "dock_enabled"], def.dock_enabled);
        let dock_mode = read_str(map, &["bloom-dock-mode", "dock_mode"])
            .map(|s| normalize_dock_mode(&s).to_string())
            .unwrap_or(def.dock_mode);
        let dock_pinned = read_string_list(map, &["bloom-dock-pinned", "dock_pinned"]);
        let dock_win_number_enabled = read_bool(
            map,
            &["bloom-dock-win-number-enabled", "dock_win_number_enabled"],
            def.dock_win_number_enabled,
        );

        let volume_overlay_enabled = read_bool(
            map,
            &["bloom-volume-overlay-enabled", "volume_overlay_enabled"],
            def.volume_overlay_enabled,
        );
        let brightness_overlay_enabled = read_bool(
            map,
            &[
                "bloom-brightness-overlay-enabled",
                "brightness_overlay_enabled",
            ],
            def.brightness_overlay_enabled,
        );

        let hime_ai_enabled = read_bool(
            map,
            &["bloom-assistant-enabled", "hime-enabled", "hime_ai_enabled"],
            def.hime_ai_enabled,
        );
        let hime_router_url = read_str(map, &["hime-9router-url", "hime_router_url"])
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(def.hime_router_url);
        let hime_router_model = read_str(map, &["hime-9router-model", "hime_router_model"])
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(def.hime_router_model);

        NativeSettings {
            notch_mode,
            free_drag_enabled,
            notch_edge,
            time_format_24h,
            music_mode_enabled,
            suppress_native_notifications,
            dock_enabled,
            dock_mode,
            dock_pinned,
            dock_win_number_enabled,
            volume_overlay_enabled,
            brightness_overlay_enabled,
            hime_ai_enabled,
            hime_router_url,
            hime_router_model,
        }
        .normalized()
    }

    /// Writes `settings` into `map` using legacy-compatible `bloom-*` and `hime-*` keys.
    pub fn write_into_map(map: &mut Map<String, Value>, settings: &NativeSettings) {
        let s = settings.normalized();
        map.insert("bloom-notch-mode".into(), Value::String(s.notch_mode));
        map.insert(
            "bloom-free-drag-enabled".into(),
            bool_str_value(s.free_drag_enabled),
        );
        map.insert("bloom-notch-edge".into(), Value::String(s.notch_edge));
        map.insert(
            "bloom-time-format-24h".into(),
            bool_str_value(s.time_format_24h),
        );
        map.insert(
            "bloom-music-mode-enabled".into(),
            bool_str_value(s.music_mode_enabled),
        );
        map.insert(
            "bloom-suppress-native-notifications".into(),
            bool_str_value(s.suppress_native_notifications),
        );

        map.insert("bloom-dock-enabled".into(), bool_str_value(s.dock_enabled));
        map.insert("bloom-dock-mode".into(), Value::String(s.dock_mode));
        map.insert(
            "bloom-dock-pinned".into(),
            Value::Array(s.dock_pinned.into_iter().map(Value::String).collect()),
        );
        map.insert(
            "bloom-dock-win-number-enabled".into(),
            bool_str_value(s.dock_win_number_enabled),
        );

        map.insert(
            "bloom-volume-overlay-enabled".into(),
            bool_str_value(s.volume_overlay_enabled),
        );
        map.insert(
            "bloom-brightness-overlay-enabled".into(),
            bool_str_value(s.brightness_overlay_enabled),
        );

        map.insert(
            "bloom-assistant-enabled".into(),
            bool_str_value(s.hime_ai_enabled),
        );
        map.insert("hime-enabled".into(), bool_str_value(s.hime_ai_enabled));
        map.insert("hime-9router-url".into(), Value::String(s.hime_router_url));
        map.insert(
            "hime-9router-model".into(),
            Value::String(s.hime_router_model),
        );
    }
}

fn bool_str_value(v: bool) -> Value {
    Value::String(if v { "true" } else { "false" }.to_string())
}

fn normalize_notch_mode(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "smart" | "auto-hide" => "smart",
        "peek" => "peek",
        _ => "fixed",
    }
}

fn normalize_notch_edge(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "left" => "left",
        "right" => "right",
        _ => "top",
    }
}

fn normalize_dock_mode(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "auto-hide" | "autohide" => "auto-hide",
        "smart" => "smart",
        _ => "fixed",
    }
}

fn read_str(map: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(val) = map.get(*key) {
            if let Some(s) = val.as_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn read_bool(map: &Map<String, Value>, keys: &[&str], default: bool) -> bool {
    for key in keys {
        if let Some(val) = map.get(*key) {
            if let Some(b) = val.as_bool() {
                return b;
            }
            if let Some(s) = val.as_str() {
                match s.trim().to_ascii_lowercase().as_str() {
                    "true" | "1" | "yes" => return true,
                    "false" | "0" | "no" => return false,
                    _ => {}
                }
            }
        }
    }
    default
}

fn read_string_list(map: &Map<String, Value>, keys: &[&str]) -> Vec<String> {
    for key in keys {
        if let Some(val) = map.get(*key) {
            if let Some(arr) = val.as_array() {
                return arr
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .filter(|s| !s.trim().is_empty())
                    .collect();
            }
            if let Some(s) = val.as_str() {
                if let Ok(parsed) = serde_json::from_str::<Vec<String>>(s) {
                    return parsed
                        .into_iter()
                        .filter(|item| !item.trim().is_empty())
                        .collect();
                }
            }
        }
    }
    Vec::new()
}

fn atomic_write_file(path: &Path, content: &[u8]) -> Result<(), String> {
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    let temp = path.with_file_name(format!(
        "{}.tmp-{}-{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("settings.json"),
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed),
    ));
    if let Err(err) = fs::write(&temp, content) {
        let _ = fs::remove_file(&temp);
        return Err(format!("Failed to write {}: {err}", temp.display()));
    }
    let mut replaced = fs::rename(&temp, path);
    for delay_ms in [25u64, 50, 100] {
        if !replaced
            .as_ref()
            .is_err_and(|e| matches!(e.raw_os_error(), Some(5 | 32 | 33)))
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        replaced = fs::rename(&temp, path);
    }
    if let Err(err) = replaced {
        let _ = fs::remove_file(&temp);
        return Err(format!("Failed to replace {}: {err}", path.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_settings_path(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("hime-settings-test-{name}-{nanos}.json"))
    }

    #[test]
    fn native_settings_serde_json_roundtrip() {
        let settings = NativeSettings {
            notch_mode: "smart".to_string(),
            free_drag_enabled: true,
            notch_edge: "left".to_string(),
            time_format_24h: false,
            music_mode_enabled: false,
            suppress_native_notifications: false,
            dock_enabled: true,
            dock_mode: "auto-hide".to_string(),
            dock_pinned: vec!["wt.exe".to_string(), "msedge.exe".to_string()],
            dock_win_number_enabled: false,
            volume_overlay_enabled: true,
            brightness_overlay_enabled: false,
            hime_ai_enabled: true,
            hime_router_url: "http://127.0.0.1:20128/v1".to_string(),
            hime_router_model: "gemini-2.5-flash".to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: NativeSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, parsed);
    }

    #[test]
    fn settings_store_save_and_load_preserves_legacy_keys() {
        let path = temp_settings_path("legacy-preserve");
        fs::write(
            &path,
            r#"{"bloom-weather-city":"Madiun","bloom-scale":"1","bloom-notch-mode":"auto-hide","bloom-dock-pinned":"[\"Code.exe\"]"}"#,
        )
        .unwrap();

        let loaded = SettingsStore::load_from_path(&path).unwrap();
        assert_eq!(loaded.notch_mode, "smart");
        assert_eq!(loaded.dock_pinned, vec!["Code.exe".to_string()]);

        let mut updated = loaded.clone();
        updated.notch_mode = "peek".to_string();
        updated.notch_edge = "right".to_string();
        updated.dock_mode = "smart".to_string();
        updated.free_drag_enabled = true;
        updated.hime_router_model = "gpt-4o-mini".to_string();
        SettingsStore::save_to_path(&path, &updated).unwrap();

        let reloaded = SettingsStore::load_from_path(&path).unwrap();
        assert_eq!(reloaded, updated);

        let raw = fs::read_to_string(&path).unwrap();
        let raw_map: Map<String, Value> = serde_json::from_str(&raw).unwrap();
        assert_eq!(
            raw_map.get("bloom-weather-city").and_then(|v| v.as_str()),
            Some("Madiun")
        );
        assert_eq!(
            raw_map.get("bloom-notch-mode").and_then(|v| v.as_str()),
            Some("peek")
        );
        assert_eq!(
            raw_map.get("hime-9router-model").and_then(|v| v.as_str()),
            Some("gpt-4o-mini")
        );

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_settings_file_returns_defaults() {
        let path = temp_settings_path("nonexistent");
        let loaded = SettingsStore::load_from_path(&path).unwrap();
        assert_eq!(loaded, NativeSettings::default());
    }
}
