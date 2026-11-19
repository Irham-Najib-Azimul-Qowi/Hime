#![windows_subsystem = "windows"]

use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
    TranslateMessage, MSG, WINDOW_EX_STYLE, WM_HOTKEY, WNDCLASSW, WS_OVERLAPPED,
};

const HOTKEY_START_CSH: i32 = 201; // Ctrl + Shift + H
const HOTKEY_START_CAS: i32 = 202; // Ctrl + Alt + Space
const HOTKEY_START_CAH: i32 = 203; // Ctrl + Alt + H (fallback attempt)

const HOTKEY_STOP_CSQ: i32 = 301;  // Ctrl + Shift + Q
const HOTKEY_STOP_CAQ: i32 = 302;  // Ctrl + Alt + Q

const VK_SPACE: u32 = 0x20;
const VK_H: u32 = 0x48;
const VK_Q: u32 = 0x51;

fn log(msg: &str) {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let log_path = format!("{}\\ai.hime.desktop\\hotkey.log", appdata);
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(file, "[hotkey] {}", msg);
        }
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_HOTKEY {
        let id = wparam.0 as i32;
        log(&format!("WM_HOTKEY received in WndProc: ID {}", id));

        match id {
            HOTKEY_START_CSH | HOTKEY_START_CAS | HOTKEY_START_CAH => {
                log("Executing start-hime.ps1");
                let res = Command::new("powershell.exe")
                    .args([
                        "-WindowStyle",
                        "Hidden",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-File",
                        "X:\\folder_ai\\start-hime.ps1",
                    ])
                    .spawn();
                match res {
                    Ok(_) => log("start-hime.ps1 spawned successfully"),
                    Err(e) => log(&format!("Failed to spawn start-hime: {}", e)),
                }
            }
            HOTKEY_STOP_CSQ | HOTKEY_STOP_CAQ => {
                log("Executing stop-hime.ps1");
                let res = Command::new("powershell.exe")
                    .args([
                        "-WindowStyle",
                        "Hidden",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-File",
                        "X:\\folder_ai\\stop-hime.ps1",
                    ])
                    .spawn();
                match res {
                    Ok(_) => log("stop-hime.ps1 spawned successfully"),
                    Err(e) => log(&format!("Failed to spawn stop-hime: {}", e)),
                }
            }
            _ => {}
        }
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn main() {
    log("=== hime_hotkey daemon (HWND listener) starting ===");

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
            log("Failed to register window class");
            return;
        }

        // Create a hidden message window
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

        if hwnd.is_err() {
            log("Failed to create message window");
            return;
        }
        let hwnd = hwnd.unwrap();

        let reg = |id: i32, name: &str, mods: HOT_KEY_MODIFIERS, vk: u32| {
            let res = RegisterHotKey(Some(hwnd), id, mods, vk);
            match res {
                Ok(_) => log(&format!("Registered {} to HWND: SUCCESS", name)),
                Err(e) => log(&format!("Registered {} to HWND: FAILED ({})", name, e)),
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

        log("Entering HWND message loop...");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        log("HWND Message loop exited.");
    }
}
