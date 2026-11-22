#![windows_subsystem = "windows"]

use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
    TranslateMessage, MSG, WINDOW_EX_STYLE, WM_HOTKEY, WNDCLASSW, WS_OVERLAPPED,
};

const HOTKEY_START_CSH: i32 = 201; // Ctrl + Shift + H
const HOTKEY_START_CAS: i32 = 202; // Ctrl + Alt + Space
const HOTKEY_START_CWH: i32 = 203; // Ctrl + Win + H
const HOTKEY_START_CAH: i32 = 204; // Ctrl + Alt + H (fallback)

const HOTKEY_STOP_CSQ: i32 = 301;  // Ctrl + Shift + Q
const HOTKEY_STOP_CAQ: i32 = 302;  // Ctrl + Alt + Q
const HOTKEY_STOP_CWQ: i32 = 303;  // Ctrl + Win + Q

const VK_SPACE: u32 = 0x20;
const VK_H: u32 = 0x48;
const VK_Q: u32 = 0x51;

static LAST_TRIGGER_MS: AtomicU64 = AtomicU64::new(0);

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
        use windows::Win32::System::Threading::{OpenMutexW, SYNCHRONIZATION_ACCESS_RIGHTS};
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

        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };
        if let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            let mut entry = PROCESSENTRY32W::default();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let end = entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    let name = String::from_utf16_lossy(&entry.szExeFile[..end]);
                    if name.eq_ignore_ascii_case("hime-island.exe") {
                        let _ = CloseHandle(snapshot);
                        return true;
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);
        }
    }
    false
}

fn signal_show_island() -> bool {
    unsafe {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenEventW, SetEvent, SYNCHRONIZATION_ACCESS_RIGHTS};
        let event_name: Vec<u16> = "HimeShowIsland"
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

fn handle_hotkey(id: i32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let prev = LAST_TRIGGER_MS.swap(now, Ordering::Relaxed);
    if now.saturating_sub(prev) < 400 {
        log(&format!("Ignoring duplicate trigger for ID {} (debounce)", id));
        return;
    }

    log(&format!("WM_HOTKEY triggered: ID {}", id));

    match id {
        HOTKEY_START_CSH | HOTKEY_START_CAS | HOTKEY_START_CWH | HOTKEY_START_CAH => {
            if is_island_running() {
                log("hime-island.exe is running -> Signaling HimeShowIsland named event");
                if signal_show_island() {
                    log("HimeShowIsland event signaled successfully");
                    return;
                }
                log("Failed to signal HimeShowIsland event, falling back to powershell");
            }

            log("HotKey match -> Spawning start-hime.ps1 with CREATE_NO_WINDOW");
            let mut cmd = Command::new("powershell.exe");
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW: prevent console window flash
            }
            let res = cmd
                .args([
                    "-WindowStyle",
                    "Hidden",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    "X:\\folder_ai\\Hime\\scripts\\start-hime.ps1",
                ])
                .spawn();
            match res {
                Ok(_) => log("start-hime.ps1 spawned successfully"),
                Err(e) => log(&format!("Failed to spawn start-hime: {}", e)),
            }
        }
        HOTKEY_STOP_CSQ | HOTKEY_STOP_CAQ | HOTKEY_STOP_CWQ => {
            log("HotKey match -> Spawning stop-hime.ps1 with CREATE_NO_WINDOW");
            let mut cmd = Command::new("powershell.exe");
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW: prevent console window flash
            }
            let res = cmd
                .args([
                    "-WindowStyle",
                    "Hidden",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    "X:\\folder_ai\\Hime\\scripts\\stop-hime.ps1",
                ])
                .spawn();
            match res {
                Ok(_) => log("stop-hime.ps1 spawned successfully"),
                Err(e) => log(&format!("Failed to spawn stop-hime: {}", e)),
            }
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
            let res = RegisterHotKey(window_hwnd, id, mods, vk);
            match res {
                Ok(_) => log(&format!("Registered {}: SUCCESS", name)),
                Err(e) => log(&format!("Registered {}: FAILED ({})", name, e)),
            }
        };

        // --- START HOTKEYS ---
        reg(
            HOTKEY_START_CSH,
            "Ctrl + Shift + H",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0),
            VK_H,
        );
        reg(
            HOTKEY_START_CAS,
            "Ctrl + Alt + Space",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0),
            VK_SPACE,
        );
        reg(
            HOTKEY_START_CWH,
            "Ctrl + Win + H",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_WIN.0),
            VK_H,
        );
        reg(
            HOTKEY_START_CAH,
            "Ctrl + Alt + H",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0),
            VK_H,
        );

        // --- STOP HOTKEYS ---
        reg(
            HOTKEY_STOP_CSQ,
            "Ctrl + Shift + Q",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0),
            VK_Q,
        );
        reg(
            HOTKEY_STOP_CAQ,
            "Ctrl + Alt + Q",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0),
            VK_Q,
        );
        reg(
            HOTKEY_STOP_CWQ,
            "Ctrl + Win + Q",
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_WIN.0),
            VK_Q,
        );

        log("Entering message pump loop...");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_HOTKEY {
                let id = msg.wParam.0 as i32;
                handle_hotkey(id);
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        log("Message loop exited.");
    }
}
