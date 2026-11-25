#![windows_subsystem = "windows"]

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
    TranslateMessage, MSG, WINDOW_EX_STYLE, WM_HOTKEY, WNDCLASSW, WS_OVERLAPPED,
};

// --- Hotkey ID Definitions ---
const HOTKEY_ASSISTANT_CSH: i32 = 201; // Ctrl + Shift + H
const HOTKEY_ASSISTANT_CAH: i32 = 202; // Ctrl + Alt + H (fallback)
const HOTKEY_ASSISTANT_CWH: i32 = 203; // Ctrl + Win + H (fallback)

const HOTKEY_NOTCH_CAS: i32 = 211;     // Ctrl + Alt + Space
const HOTKEY_NOTCH_CSS: i32 = 212;     // Ctrl + Shift + Space

const HOTKEY_MUSIC_CSM: i32 = 221;     // Ctrl + Shift + M
const HOTKEY_MUSIC_CAM: i32 = 222;     // Ctrl + Alt + M (fallback)

const HOTKEY_DOCK_CSD: i32 = 231;      // Ctrl + Shift + D
const HOTKEY_DOCK_CAD: i32 = 232;      // Ctrl + Alt + D (fallback)

const HOTKEY_STOP_CSQ: i32 = 301;      // Ctrl + Shift + Q
const HOTKEY_STOP_CAQ: i32 = 302;      // Ctrl + Alt + Q (fallback)
const HOTKEY_STOP_CWQ: i32 = 303;      // Ctrl + Win + Q (fallback)

// Single-key activation:
const HOTKEY_TOGGLE_F12: i32 = 241;    // F12 (Single key)
const HOTKEY_TOGGLE_PAUSE: i32 = 242;  // Pause (Single key)
const HOTKEY_TOGGLE_CAF12: i32 = 243;  // Ctrl + Alt + F12 (fallback when F12 is reserved)

// Keybind Cheatsheet:
const HOTKEY_SHORTCUTS_CSO: i32 = 251; // Ctrl + Shift + / (VK_OEM_2)
const HOTKEY_SHORTCUTS_F1: i32 = 252;  // F1 (Single key)
const HOTKEY_SHORTCUTS_CAO: i32 = 253; // Ctrl + Alt + / (fallback)

// Virtual Key Codes
const VK_SPACE: u32 = 0x20;
const VK_D: u32 = 0x44;
const VK_H: u32 = 0x48;
const VK_M: u32 = 0x4D;
const VK_Q: u32 = 0x51;
const VK_F12: u32 = 0x7B;
const VK_PAUSE: u32 = 0x13;
const VK_OEM_2: u32 = 0xBF;
const VK_F1: u32 = 0x70;

static LAST_TRIGGER_BY_ID: Mutex<Option<HashMap<i32, u64>>> = Mutex::new(None);

fn should_trigger_hotkey(id: i32, now_ms: u64) -> bool {
    let Ok(mut guard) = LAST_TRIGGER_BY_ID.lock() else {
        return true;
    };
    let map = guard.get_or_insert_with(HashMap::new);
    if let Some(&prev) = map.get(&id) {
        if now_ms.saturating_sub(prev) < 250 {
            return false;
        }
    }
    map.insert(id, now_ms);
    true
}

fn log(msg: &str) {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let log_dir = format!("{}\\ai.hime.desktop", appdata);
        let _ = std::fs::create_dir_all(&log_dir);
        let log_path = format!("{}\\hotkey.log", log_dir);
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
            let now = chrono_local();
            let _ = writeln!(file, "[{}] [hotkey] {}", now, msg);
        }
    }
}

fn chrono_local() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}

fn is_island_running() -> bool {
    unsafe {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{
            OpenEventW, OpenMutexW, SYNCHRONIZATION_ACCESS_RIGHTS,
        };
        let mutex_name: Vec<u16> = "HimeSingleInstance"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        // 0x00100000 is SYNCHRONIZE
        if let Ok(h) = OpenMutexW(
            SYNCHRONIZATION_ACCESS_RIGHTS(0x00100000),
            false,
            windows::core::PCWSTR(mutex_name.as_ptr()),
        ) {
            let _ = CloseHandle(h);
            return true;
        }
        // Fallback: check the main GUI's named event (excludes --taskbar-watchdog).
        let event_name: Vec<u16> = "HimeShowIsland"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        if let Ok(h) = OpenEventW(
            SYNCHRONIZATION_ACCESS_RIGHTS(0x00100000),
            false,
            windows::core::PCWSTR(event_name.as_ptr()),
        ) {
            let _ = CloseHandle(h);
            return true;
        }
    }
    false
}

fn signal_named_event(name: &str) -> bool {
    unsafe {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenEventW, SetEvent, SYNCHRONIZATION_ACCESS_RIGHTS};
        let event_name: Vec<u16> = name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        // EVENT_MODIFY_STATE | SYNCHRONIZE = 0x00100002
        if let Ok(h_event) = OpenEventW(
            SYNCHRONIZATION_ACCESS_RIGHTS(0x00100002),
            false,
            windows::core::PCWSTR(event_name.as_ptr()),
        ) {
            let _ = SetEvent(h_event);
            let _ = CloseHandle(h_event);
            true
        } else {
            false
        }
    }
}

fn spawn_silent_script(script_name: &str) {
    let script_path = format!("X:\\folder_ai\\Hime\\scripts\\{}", script_name);
    log(&format!("Spawning {} with CREATE_NO_WINDOW", script_name));

    // Prefer wscript launcher for absolute zero console flash if available
    let launcher_vbs = "X:\\folder_ai\\Hime\\tools\\hime_launcher.vbs";
    if std::path::Path::new(launcher_vbs).exists() {
        let action = if script_name.contains("start") { "start" } else { "stop" };
        let mut cmd = Command::new("wscript.exe");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let res = cmd.args([launcher_vbs, action]).spawn();
        if res.is_ok() {
            log(&format!("wscript launched {} successfully", action));
            return;
        }
    }

    // Direct powershell fallback with CREATE_NO_WINDOW
    let mut cmd = Command::new("powershell.exe");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW: prevent console window flash
    }
    let res = cmd
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path,
        ])
        .spawn();
    match res {
        Ok(_) => log(&format!("{} spawned successfully", script_name)),
        Err(e) => log(&format!("Failed to spawn {}: {}", script_name, e)),
    }
}

fn handle_hotkey(id: i32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    if !should_trigger_hotkey(id, now) {
        log(&format!("Ignoring duplicate trigger for ID {} (debounce)", id));
        return;
    }

    log(&format!("WM_HOTKEY triggered: ID {}", id));

    match id {
        HOTKEY_ASSISTANT_CSH | HOTKEY_ASSISTANT_CAH | HOTKEY_ASSISTANT_CWH => {
            if is_island_running() {
                log("hime-island.exe is running -> Signaling HimeToggleAssistant / HimeShowIsland");
                if signal_named_event("HimeToggleAssistant") || signal_named_event("HimeShowIsland") {
                    log("Assistant toggle event signaled successfully");
                    return;
                }
                log("Failed to signal named event, falling back to script spawn");
            }
            spawn_silent_script("start-hime.ps1");
        }
        HOTKEY_NOTCH_CAS | HOTKEY_NOTCH_CSS => {
            if is_island_running() {
                log("hime-island.exe is running -> Signaling HimeShowIsland named event");
                if signal_named_event("HimeShowIsland") {
                    log("HimeShowIsland event signaled successfully");
                    return;
                }
                log("Failed to signal HimeShowIsland event, falling back to script spawn");
            }
            spawn_silent_script("start-hime.ps1");
        }
        HOTKEY_MUSIC_CSM | HOTKEY_MUSIC_CAM => {
            if is_island_running() {
                log("hime-island.exe is running -> Signaling HimeToggleMusic event");
                if signal_named_event("HimeToggleMusic") || signal_named_event("HimeShowIsland") {
                    log("Music toggle event signaled successfully");
                    return;
                }
                log("Failed to signal named event, falling back to script spawn");
            }
            spawn_silent_script("start-hime.ps1");
        }
        HOTKEY_DOCK_CSD | HOTKEY_DOCK_CAD => {
            if is_island_running() {
                log("hime-island.exe is running -> Signaling HimeToggleDock event");
                if signal_named_event("HimeToggleDock") {
                    log("Dock toggle event signaled successfully");
                    return;
                }
                log("Failed to signal HimeToggleDock event, falling back to script spawn");
            }
            spawn_silent_script("start-hime.ps1");
        }
        HOTKEY_STOP_CSQ | HOTKEY_STOP_CAQ | HOTKEY_STOP_CWQ => {
            log("Stop HotKey match -> Signaling HimeQuit and running stop-hime.ps1");
            let _ = signal_named_event("HimeQuit");
            spawn_silent_script("stop-hime.ps1");
        }
        HOTKEY_TOGGLE_F12 | HOTKEY_TOGGLE_PAUSE | HOTKEY_TOGGLE_CAF12 => {
            if is_island_running() {
                log("hime-island.exe is running -> Signaling HimeToggleAssistant / HimeShowIsland (Single-Key)");
                if signal_named_event("HimeToggleAssistant") || signal_named_event("HimeShowIsland") {
                    log("Single-key toggle event signaled successfully");
                    return;
                }
                log("Failed to signal named event, falling back to script spawn");
            }
            spawn_silent_script("start-hime.ps1");
        }
        HOTKEY_SHORTCUTS_CSO | HOTKEY_SHORTCUTS_F1 | HOTKEY_SHORTCUTS_CAO => {
            if is_island_running() {
                log("hime-island.exe is running -> Signaling HimeShowShortcuts event");
                if signal_named_event("HimeShowShortcuts") {
                    log("HimeShowShortcuts event signaled successfully");
                    return;
                }
                log("Failed to signal HimeShowShortcuts event, falling back to script spawn");
            }
            spawn_silent_script("start-hime.ps1");
        }
        _ => {}
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_HOTKEY {
        let id = wparam.0 as i32;
        handle_hotkey(id);
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn main() {
    log("=== hime_hotkey daemon starting ===");

    // --- Single-instance guard: at most one hime_hotkey may run at a time.
    // A second instance exits immediately so it cannot steal half the hotkeys
    // from the first (Windows ERROR_HOTKEY_ALREADY_REGISTERED race).
    // CreateMutexW returns Ok even when the mutex already exists; the caller
    // must check GetLastError() == ERROR_ALREADY_EXISTS (0x183) to detect it.
    let _mutex_handle = unsafe {
        match CreateMutexW(None, true, w!("HimeHotkeyDaemonMutex")) {
            Ok(handle) => {
                let err = windows::Win32::Foundation::GetLastError();
                if err.0 == 0x183 {
                    // ERROR_ALREADY_EXISTS: another hime_hotkey is running.
                    log("Another hime_hotkey instance is already running. Exiting.");
                    return;
                }
                Some(handle)
            }
            Err(_) => {
                log("Failed to create single-instance mutex; proceeding anyway.");
                None
            }
        }
    };

    unsafe {
        let hinstance = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
        let class_name = w!("HimeHotkeyListenerClass");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            log("Notice: RegisterClassW returned 0 (class may already exist)");
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("HimeHotkeyListenerWindow"),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(HINSTANCE(hinstance.0)),
            None,
        );

        let window_hwnd = match hwnd {
            Ok(h) => {
                log(&format!("Created message window handle: 0x{:X}", h.0 as usize));
                Some(h)
            }
            Err(e) => {
                log(&format!("Failed to create message window: {}. Falling back to thread queue.", e));
                None
            }
        };

        let reg = |id: i32, name: &str, mods: HOT_KEY_MODIFIERS, vk: u32| {
            let res = RegisterHotKey(window_hwnd, id, HOT_KEY_MODIFIERS(mods.0 | MOD_NOREPEAT.0), vk);
            match res {
                Ok(_) => log(&format!("Registered {}: SUCCESS", name)),
                Err(e) => log(&format!("Registered {}: FAILED ({})", name, e)),
            }
        };

        // --- 1. TOGGLE HIME ASSISTANT ---
        reg(
            HOTKEY_ASSISTANT_CSH,
            "Ctrl + Shift + H (Toggle Assistant)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0),
            VK_H,
        );
        reg(
            HOTKEY_ASSISTANT_CAH,
            "Ctrl + Alt + H (Toggle Assistant Fallback)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0),
            VK_H,
        );
        reg(
            HOTKEY_ASSISTANT_CWH,
            "Ctrl + Win + H (Toggle Assistant Fallback)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_WIN.0),
            VK_H,
        );

        // --- 2. TOGGLE NOTCH / COMMAND CENTER ---
        reg(
            HOTKEY_NOTCH_CAS,
            "Ctrl + Alt + Space (Toggle Notch)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0),
            VK_SPACE,
        );
        reg(
            HOTKEY_NOTCH_CSS,
            "Ctrl + Shift + Space (Toggle Notch)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0),
            VK_SPACE,
        );

        // --- 3. TOGGLE MUSIC / LYRICS PANEL ---
        reg(
            HOTKEY_MUSIC_CSM,
            "Ctrl + Shift + M (Toggle Music Panel)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0),
            VK_M,
        );
        reg(
            HOTKEY_MUSIC_CAM,
            "Ctrl + Alt + M (Toggle Music Panel Fallback)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0),
            VK_M,
        );

        // --- 4. TOGGLE DOCK ---
        reg(
            HOTKEY_DOCK_CSD,
            "Ctrl + Shift + D (Toggle Dock)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0),
            VK_D,
        );
        reg(
            HOTKEY_DOCK_CAD,
            "Ctrl + Alt + D (Toggle Dock Fallback)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0),
            VK_D,
        );

        // --- 5. QUIT / STOP HIME ECOSYSTEM ---
        reg(
            HOTKEY_STOP_CSQ,
            "Ctrl + Shift + Q (Quit Hime)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0),
            VK_Q,
        );
        reg(
            HOTKEY_STOP_CAQ,
            "Ctrl + Alt + Q (Quit Hime Fallback)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0),
            VK_Q,
        );
        reg(
            HOTKEY_STOP_CWQ,
            "Ctrl + Win + Q (Quit Hime Fallback)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_WIN.0),
            VK_Q,
        );

        // --- 6. SINGLE-KEY ACTIVATION (F12, Pause, Ctrl+Alt+F12 fallback) ---
        reg(
            HOTKEY_TOGGLE_F12,
            "F12 (Single-Key Toggle Hime)",
            HOT_KEY_MODIFIERS(0),
            VK_F12,
        );
        reg(
            HOTKEY_TOGGLE_PAUSE,
            "Pause (Single-Key Toggle Hime)",
            HOT_KEY_MODIFIERS(0),
            VK_PAUSE,
        );
        reg(
            HOTKEY_TOGGLE_CAF12,
            "Ctrl + Alt + F12 (Toggle Hime Fallback)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0),
            VK_F12,
        );

        // --- 7. KEYBIND CHEATSHEET (Ctrl + Shift + /, F1, Ctrl + Alt + / fallback) ---
        reg(
            HOTKEY_SHORTCUTS_CSO,
            "Ctrl + Shift + / (Shortcuts Cheatsheet)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0),
            VK_OEM_2,
        );
        reg(
            HOTKEY_SHORTCUTS_F1,
            "F1 (Shortcuts Cheatsheet)",
            HOT_KEY_MODIFIERS(0),
            VK_F1,
        );
        reg(
            HOTKEY_SHORTCUTS_CAO,
            "Ctrl + Alt + / (Shortcuts Cheatsheet Fallback)",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0),
            VK_OEM_2,
        );

        log("Entering message pump loop...");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_HOTKEY && msg.hwnd.0.is_null() {
                // Thread-queue fallback when window creation failed (wnd_proc is not invoked).
                let id = msg.wParam.0 as i32;
                handle_hotkey(id);
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        log("Message loop exited.");
    }
}

#[cfg(test)]
mod tests {
    use super::should_trigger_hotkey;

    #[test]
    fn debounces_per_hotkey_id_independently() {
        assert!(should_trigger_hotkey(9001, 1000));
        assert!(!should_trigger_hotkey(9001, 1100));
        assert!(should_trigger_hotkey(9002, 1100));
        assert!(should_trigger_hotkey(9001, 1300));
    }
}
