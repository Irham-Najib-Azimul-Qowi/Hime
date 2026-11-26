//! Global hotkey manager (`RegisterHotKey`) and low-level keyboard hook
//! (`WH_KEYBOARD_LL`) for Hime Island Native.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use windows::core::Result;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, RegisterHotKey, SendInput, UnregisterHotKey, HOT_KEY_MODIFIERS, INPUT,
    INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, VIRTUAL_KEY, VK_LWIN, VK_RWIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, LLKHF_INJECTED,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

// --- Registered Hotkey IDs ---
pub const HOTKEY_ASSISTANT_CSH: i32 = 201; // Ctrl + Shift + H
pub const HOTKEY_ASSISTANT_CAH: i32 = 202; // Ctrl + Alt + H (fallback)
pub const HOTKEY_NOTCH_CSS: i32 = 211; // Ctrl + Shift + Space
pub const HOTKEY_NOTCH_CAS: i32 = 212; // Ctrl + Alt + Space (fallback)
pub const HOTKEY_MUSIC_CSM: i32 = 221; // Ctrl + Shift + M
pub const HOTKEY_DOCK_CSD: i32 = 231; // Ctrl + Shift + D
pub const HOTKEY_SHORTCUTS_F1: i32 = 251; // F1
pub const HOTKEY_SHORTCUTS_CSO: i32 = 252; // Ctrl + Shift + /
pub const HOTKEY_STOP_CSQ: i32 = 301; // Ctrl + Shift + Q

// Virtual key codes
const VK_SPACE: u32 = 0x20;
const VK_D: u32 = 0x44;
const VK_H: u32 = 0x48;
const VK_M: u32 = 0x4D;
const VK_N: u16 = 0x4E;
const VK_Q: u32 = 0x51;
const VK_F1: u32 = 0x70;
const VK_OEM_2: u32 = 0xBF;
const MASK_VK: u16 = 0xE8; // Unassigned VK used to mask Start menu activation on Win release.

const DEBOUNCE_MS: u64 = 250;

/// Actions emitted by global hotkeys or the low-level Win+N / Win+1..9 hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    ToggleAssistant,
    ToggleNotch,
    ToggleMusic,
    ToggleDock,
    ToggleShortcuts,
    StopHime,
    ToggleCalendar,
    FocusDockSlot(u8),
}

// Low-level hook global state (Win32 WH_KEYBOARD_LL callback has no user-data pointer).
static KEYBOARD_HOOK: AtomicIsize = AtomicIsize::new(0);
static WIN_KEY_DOWN: AtomicBool = AtomicBool::new(false);
static WIN_NUMBER_HELD: AtomicU8 = AtomicU8::new(0);
static WIN_N_HELD: AtomicBool = AtomicBool::new(false);
static LL_ACTION_QUEUE: OnceLock<Mutex<VecDeque<HotkeyAction>>> = OnceLock::new();
static LAST_TRIGGER_BY_ID: OnceLock<Mutex<HashMap<i32, u64>>> = OnceLock::new();
static LL_WIN_NUMBER_ENABLED: AtomicBool = AtomicBool::new(true);
static LL_WIN_N_ENABLED: AtomicBool = AtomicBool::new(true);

fn action_queue() -> &'static Mutex<VecDeque<HotkeyAction>> {
    LL_ACTION_QUEUE.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn push_ll_action(action: HotkeyAction) {
    if let Ok(mut q) = action_queue().lock() {
        q.push_back(action);
    }
}

/// Drains all pending actions captured by the `WH_KEYBOARD_LL` hook.
pub fn drain_ll_actions() -> Vec<HotkeyAction> {
    let Ok(mut q) = action_queue().lock() else {
        return Vec::new();
    };
    q.drain(..).collect()
}

/// Enables or disables `Win+1..9` and `Win+N` interception at runtime.
pub fn set_ll_features(win_number: bool, win_n: bool) {
    LL_WIN_NUMBER_ENABLED.store(win_number, Ordering::Relaxed);
    LL_WIN_N_ENABLED.store(win_n, Ordering::Relaxed);
}

/// Maps top-row digit virtual key codes (`'1'..='9'`) to zero-based dock slots `0..=8`.
pub fn win_number_index(vk: u16) -> Option<u8> {
    match vk {
        0x31..=0x39 => Some((vk - 0x31) as u8),
        _ => None,
    }
}

/// Maps a registered `WM_HOTKEY` ID to its `HotkeyAction`.
pub fn hotkey_id_to_action(id: i32) -> Option<HotkeyAction> {
    match id {
        HOTKEY_ASSISTANT_CSH | HOTKEY_ASSISTANT_CAH => Some(HotkeyAction::ToggleAssistant),
        HOTKEY_NOTCH_CSS | HOTKEY_NOTCH_CAS => Some(HotkeyAction::ToggleNotch),
        HOTKEY_MUSIC_CSM => Some(HotkeyAction::ToggleMusic),
        HOTKEY_DOCK_CSD => Some(HotkeyAction::ToggleDock),
        HOTKEY_SHORTCUTS_F1 | HOTKEY_SHORTCUTS_CSO => Some(HotkeyAction::ToggleShortcuts),
        HOTKEY_STOP_CSQ => Some(HotkeyAction::StopHime),
        _ => None,
    }
}

/// Debounces hotkey triggers within `DEBOUNCE_MS` to suppress auto-repeat / double fires.
pub fn should_trigger_hotkey(id: i32, now_ms: u64) -> bool {
    let map_lock = LAST_TRIGGER_BY_ID.get_or_init(|| Mutex::new(HashMap::new()));
    let Ok(mut map) = map_lock.lock() else {
        return true;
    };
    should_trigger_with_map(&mut map, id, now_ms)
}

fn should_trigger_with_map(map: &mut HashMap<i32, u64>, id: i32, now_ms: u64) -> bool {
    if let Some(&prev) = map.get(&id) {
        if now_ms.saturating_sub(prev) < DEBOUNCE_MS {
            return false;
        }
    }
    map.insert(id, now_ms);
    true
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Manages Win32 global hotkeys and the low-level `WH_KEYBOARD_LL` hook.
pub struct HotkeyManager {
    hwnd: HWND,
    registered_ids: Vec<i32>,
    ll_hook_installed: bool,
}

impl HotkeyManager {
    /// Registers global hotkeys on `hwnd` and optionally installs `WH_KEYBOARD_LL`.
    pub fn register(hwnd: HWND, install_ll_hook: bool) -> Self {
        let mut registered_ids = Vec::new();
        let specs: &[(i32, HOT_KEY_MODIFIERS, u32)] = &[
            (HOTKEY_ASSISTANT_CSH, MOD_CONTROL | MOD_SHIFT, VK_H),
            (HOTKEY_ASSISTANT_CAH, MOD_CONTROL | MOD_ALT, VK_H),
            (HOTKEY_NOTCH_CSS, MOD_CONTROL | MOD_SHIFT, VK_SPACE),
            (HOTKEY_NOTCH_CAS, MOD_CONTROL | MOD_ALT, VK_SPACE),
            (HOTKEY_MUSIC_CSM, MOD_CONTROL | MOD_SHIFT, VK_M),
            (HOTKEY_DOCK_CSD, MOD_CONTROL | MOD_SHIFT, VK_D),
            (HOTKEY_SHORTCUTS_F1, HOT_KEY_MODIFIERS(0), VK_F1),
            (HOTKEY_SHORTCUTS_CSO, MOD_CONTROL | MOD_SHIFT, VK_OEM_2),
            (HOTKEY_STOP_CSQ, MOD_CONTROL | MOD_SHIFT, VK_Q),
        ];

        unsafe {
            for &(id, mods, vk) in specs {
                let flags = HOT_KEY_MODIFIERS(mods.0 | MOD_NOREPEAT.0);
                if RegisterHotKey(Some(hwnd), id, flags, vk).is_ok() {
                    registered_ids.push(id);
                }
            }
        }

        let ll_hook_installed = if install_ll_hook {
            install_keyboard_hook().is_ok()
        } else {
            false
        };

        Self {
            hwnd,
            registered_ids,
            ll_hook_installed,
        }
    }

    /// Returns the number of successfully registered global hotkeys.
    pub fn registered_count(&self) -> usize {
        self.registered_ids.len()
    }

    /// Returns whether the low-level keyboard hook is currently active.
    pub fn has_ll_hook(&self) -> bool {
        self.ll_hook_installed
    }

    /// Processes a `WM_HOTKEY` `wparam` and returns an action if debounced.
    pub fn handle_wm_hotkey(&self, wparam: WPARAM) -> Option<HotkeyAction> {
        let id = wparam.0 as i32;
        if !should_trigger_hotkey(id, now_epoch_ms()) {
            return None;
        }
        hotkey_id_to_action(id)
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        unsafe {
            for &id in &self.registered_ids {
                let _ = UnregisterHotKey(Some(self.hwnd), id);
            }
        }
        if self.ll_hook_installed {
            uninstall_keyboard_hook();
        }
    }
}

/// Installs the `WH_KEYBOARD_LL` hook on the current thread (must pump a Win32 message loop).
pub fn install_keyboard_hook() -> Result<()> {
    if KEYBOARD_HOOK.load(Ordering::Relaxed) != 0 {
        return Ok(());
    }
    unsafe {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0)?;
        KEYBOARD_HOOK.store(hook.0 as isize, Ordering::Relaxed);
        Ok(())
    }
}

/// Uninstalls the `WH_KEYBOARD_LL` hook if currently installed.
pub fn uninstall_keyboard_hook() {
    let raw = KEYBOARD_HOOK.swap(0, Ordering::Relaxed);
    if raw != 0 {
        unsafe {
            let _ = UnhookWindowsHookEx(HHOOK(raw as *mut _));
        }
    }
    WIN_KEY_DOWN.store(false, Ordering::Relaxed);
    WIN_NUMBER_HELD.store(0, Ordering::Relaxed);
    WIN_N_HELD.store(false, Ordering::Relaxed);
}

fn win_key_physically_down() -> bool {
    unsafe {
        (GetAsyncKeyState(VK_LWIN.0 as i32) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_RWIN.0 as i32) as u16 & 0x8000) != 0
    }
}

fn send_start_menu_mask() {
    let mask = VIRTUAL_KEY(MASK_VK);
    let inputs = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: mask,
                    wScan: 0,
                    dwFlags: Default::default(),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: mask,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];
    unsafe {
        let _ = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
    if (kb.flags.0 & LLKHF_INJECTED.0) != 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let msg = wparam.0 as u32;
    let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
    let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
    let vk = kb.vkCode as u16;

    if vk == VK_LWIN.0 || vk == VK_RWIN.0 {
        if is_down {
            WIN_KEY_DOWN.store(true, Ordering::Relaxed);
        } else if is_up {
            WIN_KEY_DOWN.store(false, Ordering::Relaxed);
            WIN_NUMBER_HELD.store(0, Ordering::Relaxed);
            WIN_N_HELD.store(false, Ordering::Relaxed);
        }
        return CallNextHookEx(None, code, wparam, lparam);
    }

    if WIN_KEY_DOWN.load(Ordering::Relaxed) && !win_key_physically_down() {
        WIN_KEY_DOWN.store(false, Ordering::Relaxed);
    }

    // Win + N -> Calendar toggle
    if LL_WIN_N_ENABLED.load(Ordering::Relaxed) && vk == VK_N {
        if is_down && WIN_KEY_DOWN.load(Ordering::Relaxed) {
            if !WIN_N_HELD.swap(true, Ordering::Relaxed) {
                send_start_menu_mask();
                push_ll_action(HotkeyAction::ToggleCalendar);
            }
            return LRESULT(1);
        } else if is_up && WIN_N_HELD.swap(false, Ordering::Relaxed) {
            return LRESULT(1);
        }
    }

    // Win + 1..9 -> Dock slot focus
    if LL_WIN_NUMBER_ENABLED.load(Ordering::Relaxed) {
        if let Some(slot) = win_number_index(vk) {
            let digit = slot + 1;
            if is_down && WIN_KEY_DOWN.load(Ordering::Relaxed) {
                if WIN_NUMBER_HELD.load(Ordering::Relaxed) != digit {
                    WIN_NUMBER_HELD.store(digit, Ordering::Relaxed);
                    send_start_menu_mask();
                    push_ll_action(HotkeyAction::FocusDockSlot(slot));
                }
                return LRESULT(1);
            } else if is_up && WIN_NUMBER_HELD.load(Ordering::Relaxed) == digit {
                WIN_NUMBER_HELD.store(0, Ordering::Relaxed);
                return LRESULT(1);
            }
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win_number_index_maps_digits_1_to_9() {
        assert_eq!(win_number_index(0x31), Some(0));
        assert_eq!(win_number_index(0x35), Some(4));
        assert_eq!(win_number_index(0x39), Some(8));
        assert_eq!(win_number_index(0x30), None);
        assert_eq!(win_number_index(VK_N), None);
    }

    #[test]
    fn hotkey_id_mapping_covers_all_registered_ids() {
        assert_eq!(
            hotkey_id_to_action(HOTKEY_ASSISTANT_CSH),
            Some(HotkeyAction::ToggleAssistant)
        );
        assert_eq!(
            hotkey_id_to_action(HOTKEY_NOTCH_CSS),
            Some(HotkeyAction::ToggleNotch)
        );
        assert_eq!(
            hotkey_id_to_action(HOTKEY_MUSIC_CSM),
            Some(HotkeyAction::ToggleMusic)
        );
        assert_eq!(
            hotkey_id_to_action(HOTKEY_DOCK_CSD),
            Some(HotkeyAction::ToggleDock)
        );
        assert_eq!(
            hotkey_id_to_action(HOTKEY_SHORTCUTS_F1),
            Some(HotkeyAction::ToggleShortcuts)
        );
        assert_eq!(
            hotkey_id_to_action(HOTKEY_SHORTCUTS_CSO),
            Some(HotkeyAction::ToggleShortcuts)
        );
        assert_eq!(
            hotkey_id_to_action(HOTKEY_STOP_CSQ),
            Some(HotkeyAction::StopHime)
        );
        assert_eq!(hotkey_id_to_action(9999), None);
    }

    #[test]
    fn debounce_suppresses_rapid_repeats_and_recovers() {
        let mut map = HashMap::new();
        assert!(should_trigger_with_map(&mut map, HOTKEY_NOTCH_CSS, 1000));
        assert!(!should_trigger_with_map(&mut map, HOTKEY_NOTCH_CSS, 1100));
        assert!(should_trigger_with_map(&mut map, HOTKEY_NOTCH_CSS, 1260));
        assert!(should_trigger_with_map(&mut map, HOTKEY_DOCK_CSD, 1100));
    }
}
