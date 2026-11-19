# Bloom Architecture Notes & System Inventory

> **Source**: Empirical analysis of Bloom v3.10.2 (`0ab9f2f08b75cfeffd3040330e312c2244d7726a`).

---

## 1. System Overview & Window Topology

Bloom runs a multi-webview Tauri v2 architecture consisting of four independent webview windows, each served by a separate Vite HTML entry point:

| Window Label | Entry Point | Target React Component | Geometry & Role |
|---|---|---|---|
| **`main`** | `index.html` | `src/main.tsx` → `src/App.tsx` | **Dynamic Island (Notch)**. Frameless, transparent, top-docked (`alwaysOnTop: true`, `skipTaskbar: true`, `decorations: false`). Geometry morphs via spring physics. Window hit-test regions are dynamically clipped via `window_region.rs` to allow click-through outside the pill/panel. |
| **`dock`** | `dock.html` | `src/dock.tsx` → `src/Dock.tsx` | **Taskbar Replacement Dock**. Frameless, transparent, bottom-docked. Replaces Windows `Shell_TrayWnd`. Manages running task icons, window thumbnail previews, quick launchers, and extras (Recycle Bin, drives). |
| **`overlay`** | `overlay.html` | `src/overlay.tsx` → `src/Overlay.tsx` | **HUD Overlays**. Full-screen transparent overlay (`width: 1920`, `height: 1080`, `focus: false`, `skipTaskbar: true`) displaying transient volume, brightness, and media HUD pills. |
| **`settings`** | `settings.html` | `src/settings.tsx` → `src/Settings.tsx` | **Configuration Window**. Created on-demand (`create: false` in `tauri.conf.json`), centered (`width: 740`, `height: 560`), styled with Windows 11 Mica vibrancy effect. |

---

## 2. Rust Backend Module Breakdown (`src-tauri/src/`)

| File / Module | Lines | Role & Key Responsibilities |
|---|---|---|
| **`main.rs`** | ~475 | Application initialization, plugin setup (`clipboard`, `autostart`, `opener`, `drag`, `updater`), tray menu construction, and the master `tauri::generate_handler![...]` registry. |
| **`commands.rs`** | ~2,250 | All frontend-facing IPC handlers (`#[tauri::command]`). Handles window positioning, AppBar registration, audio volume/mute, brightness, dock extras, and shell operations. |
| **`services.rs`** | ~4,750 | Long-running Win32 background worker threads and OS hooks: keyboard hooks (`WH_KEYBOARD_LL`), WinEvent hooks (`SetWinEventHook`) for window focus/z-order tracking, WASAPI audio loop with FFT visualizer, and display/DPI monitors. |
| **`utils.rs`** | ~1,650 | Settings persistence (`%APPDATA%/bloom/settings.json`), cached key-value lookups, taskbar visibility APIs (`ShowWindow(tray_hwnd, SW_SHOW/SW_HIDE)`), and path utilities. |
| **`state.rs`** | ~120 | Process-wide thread-safe state: atomic flags (`SHUTTING_DOWN`, `NATIVE_TASKBAR_HIDDEN`, `DOCK_APPBAR_REGISTERED`, `TRAY_INTERACTION_ACTIVE`) and mutex caches. |
| **`types.rs`** | ~350 | Shared data transfer objects (DTOs) for windows, media sessions, volume, displays, and battery status. |
| **`updater.rs`** | ~436 | Auto-update check and install logic (**neutralized for Hime Shell** via TP-002). |
| **`battery.rs`** | ~180 | WMI and Win32 Power API integration for battery health, charge state, and power plan telemetry. |
| **`wifi.rs`** | ~350 | Native WLAN profile management (`WlanGetNetworkBssList`, `WlanConnect`). |
| **`bluetooth.rs`** | ~280 | Windows Runtime (`Windows.Devices.Bluetooth`) radio enumeration and GATT battery service reader. |
| **`dock_extras.rs`** | ~220 | Drive status, Recycle Bin item count, and folder shortcut resolution. |
| **`window_region.rs`** | ~280 | Dynamic Win32 window clipping (`SetWindowRgn`) to keep transparent webview click-through accurate. |
| **`tray.rs`** | ~150 | Windows notification tray icon integration. |

---

## 3. IPC Commands Inventory (`main.rs:160-230`)

The master handler registers the following native commands:
- **Settings**: `get_settings`, `save_setting`, `reset_settings`, `import_settings_from_file`, `export_settings_to_file`.
- **Taskbar & Window Placement**: `sync_appbar`, `update_dock_overlap`, `update_notch_overlap`, `set_window_region`, `open_settings_window`, `close_settings_window`.
- **System Controls**: `set_system_volume`, `toggle_system_mute`, `set_display_brightness`, `get_display_brightness`, `set_power_mode`.
- **Hardware & Peripherals**: `get_wifi_networks`, `connect_wifi`, `disconnect_wifi`, `toggle_wifi`, `get_bluetooth_devices`, `toggle_bluetooth`, `get_battery_details`.
- **Dock & Launcher**: `get_running_windows`, `focus_window`, `close_window`, `get_dock_extras`, `open_dock_extra`.
- **Media**: `media_play_pause`, `media_next`, `media_prev`, `get_media_state`.
- **Updater (Neutralized)**: `check_for_updates`, `install_update`, `get_update_state`.

---

## 4. Backend-to-Frontend Events Inventory

| Event Name | Source | Frequency | Purpose |
|---|---|---|---|
| `settings-changed` | `utils.rs` | On save | Notifies all windows that a setting was modified. |
| `settings-external-changed` | `utils.rs` | On file watch | Reloads settings if changed by an external process. |
| `media-update` | `services.rs` | On playback change | Updates title, artist, playback state, and album art. |
| `volume-change` | `services.rs` | On volume change | Triggers overlay HUD volume indicator. |
| `brightness-change` | `services.rs` | On brightness change | Triggers overlay HUD brightness indicator. |
| `windows-changed` | `services.rs` | On focus / app launch | Updates running app icons and task list in Dock. |
| `dock-overlap` | `services.rs` | When maximized window hits dock | Auto-hides dock in Smart mode. |
| `notch-overlap` | `services.rs` | When maximized window hits notch | Adjusts notch opacity/vis in Smart mode. |

---

## 5. CPU & Memory Idle Guarantees

Bloom maintains low idle resource utilization through strict event-driven design:
1. **Audio Visualizer**: The WASAPI FFT capture loop sleeps immediately when audio is silent or paused; zero CPU cycles spent on audio analysis when idle.
2. **Window Region Watchdog**: Clipping regions are measured and transmitted only when modal geometries change, never on a high-frequency polling timer.
3. **Thumbnail Capture**: Window previews in the dock are captured on hover, never pre-cached in background.
4. **Hime Compliance Requirement**: All Hime Shell extensions must follow this model. **No idle polling loops**; only reactive listening on Named Pipe events from Hime Core.

