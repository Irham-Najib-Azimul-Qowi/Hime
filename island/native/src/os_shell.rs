//! OS Shell Integration & Mouse Interaction for Hime Island Native (ADR-017 Phase 4).
//!
//! Provides:
//! - Safe Windows taskbar (`Shell_TrayWnd` & `Shell_SecondaryTrayWnd`) hide/restore with
//!   crash-recovery marker at `%APPDATA%\ai.hime.desktop\native_taskbar_hidden.flag`.
//! - 3-edge (`Top`, `Left`, `Right`) drag snap calculation, screen bounds, and edge hover
//!   wake / hit-testing helpers.
//! - Win32 `WM_DROPFILES` (`DragAcceptFiles` + `HDROP`) extraction into [`FileShelf`].

use std::collections::HashMap;
use std::ffi::{c_void, OsStr, OsString};
use std::fs;
use std::io::Write;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use windows::core::{w, BOOL, PCSTR, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS,
};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, CreateSolidBrush, DeleteDC, DeleteObject, FillRect,
    GetDC, GetMonitorInfoW, GetSysColor, MonitorFromPoint, ReleaseDC, SelectObject,
    SetStretchBltMode, StretchBlt, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, COLOR_BACKGROUND,
    DIB_RGB_COLORS, HALFTONE, HBITMAP, HDC, HGDIOBJ, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    SRCCOPY,
};
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, TerminateProcess, PROCESS_TERMINATE,
};
use windows::Win32::UI::Shell::{
    DragAcceptFiles, DragFinish, DragQueryFileW, SHAppBarMessage, ShellExecuteW, ABM_GETSTATE,
    ABM_GETTASKBARPOS, ABM_SETSTATE, APPBARDATA, HDROP,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, SetFocus, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_LEFT, VK_LWIN, VK_RIGHT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, FindWindowA, GetClassNameA, GetCursorPos, GetSystemMetrics,
    GetWindowLongA, GetWindowLongW, GetWindowRect, GetWindowThreadProcessId, IsIconic,
    IsWindow, IsWindowVisible, PostMessageW, SetForegroundWindow, SetLayeredWindowAttributes,
    SetWindowLongA, SetWindowLongW, SetWindowPos, ShowWindow, GWL_EXSTYLE, HWND_TOPMOST,
    LWA_ALPHA, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SW_HIDE, SW_SHOW, SW_SHOWNORMAL, WM_CLOSE, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
};

use crate::d2d::Rect;
use crate::dock::DockBar;
use crate::notch::{NotchEdge, NotchView};
use crate::shelf::FileShelf;

/// Default hover margin (in pixels/DIPs) along the anchored screen edge to wake a hidden Notch.
pub const DEFAULT_EDGE_WAKE_THRESHOLD_PX: f32 = 6.0;

const TASKBAR_MARKER_DIR: &str = "ai.hime.desktop";
const TASKBAR_MARKER_FILE: &str = "native_taskbar_hidden.flag";

static ORIGINAL_TASKBAR_STATE: AtomicI32 = AtomicI32::new(-1);
static ORIGINAL_TRAY_RECT: Mutex<Option<RECT>> = Mutex::new(None);
static ORIGINAL_SEC_TRAY_RECTS: Mutex<Option<HashMap<isize, RECT>>> = Mutex::new(None);

/// Screen/monitor rectangle in virtual desktop coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ScreenBounds {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

// ---------------------------------------------------------------------------
// 1. Safe Taskbar Hide / Restore + Crash-Recovery Marker
// ---------------------------------------------------------------------------

/// Returns `%APPDATA%\ai.hime.desktop\native_taskbar_hidden.flag`.
pub fn taskbar_marker_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|dir| {
        PathBuf::from(dir)
            .join(TASKBAR_MARKER_DIR)
            .join(TASKBAR_MARKER_FILE)
    })
}

/// Returns `true` if a crash-recovery taskbar marker exists on disk.
pub fn has_taskbar_crash_marker() -> bool {
    taskbar_marker_path().is_some_and(|p| p.exists())
}

fn is_placed_rect(rect: &RECT) -> bool {
    rect.right > rect.left && rect.bottom > rect.top && rect.left > -9000 && rect.top > -9000
}

fn placed_taskbar_rect(hwnd: HWND) -> Option<RECT> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) }.ok()?;
    is_placed_rect(&rect).then_some(rect)
}

fn secondary_taskbar_windows() -> Vec<HWND> {
    unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let windows = &mut *(lparam.0 as *mut Vec<HWND>);
        let mut class = [0u8; 64];
        let length = GetClassNameA(hwnd, &mut class);
        if length > 0 && &class[..length as usize] == b"Shell_SecondaryTrayWnd" {
            windows.push(hwnd);
        }
        BOOL(1)
    }

    let mut windows = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(collect),
            LPARAM(&mut windows as *mut Vec<HWND> as isize),
        );
    }
    windows
}

unsafe fn restore_taskbar_exstyle(hwnd: HWND) {
    let ex = GetWindowLongA(hwnd, GWL_EXSTYLE);
    let cleaned = ex & !(WS_EX_LAYERED.0 as i32) & !(WS_EX_TRANSPARENT.0 as i32);
    if cleaned != ex {
        let _ = SetLayeredWindowAttributes(hwnd, windows::Win32::Foundation::COLORREF(0), 255, LWA_ALPHA);
        let _ = SetWindowLongA(hwnd, GWL_EXSTYLE, cleaned);
    }
}

fn format_marker_contents(
    original_state: i32,
    pid: u32,
    primary: Option<RECT>,
    secondaries: &HashMap<isize, RECT>,
) -> String {
    let mut contents = format!("abstate={original_state}\npid={pid}\n");
    if let Some(rect) = primary {
        contents.push_str(&format!(
            "primary={},{},{},{}\n",
            rect.left, rect.top, rect.right, rect.bottom
        ));
    }
    let mut sorted: Vec<_> = secondaries.iter().collect();
    sorted.sort_by_key(|(hwnd, _)| **hwnd);
    for (hwnd, rect) in sorted {
        contents.push_str(&format!(
            "secondary={hwnd},{},{},{},{}\n",
            rect.left, rect.top, rect.right, rect.bottom
        ));
    }
    contents
}

fn parse_rect<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<RECT> {
    let mut next = || parts.next()?.trim().parse::<i32>().ok();
    let rect = RECT {
        left: next()?,
        top: next()?,
        right: next()?,
        bottom: next()?,
    };
    parts.next().is_none().then_some(rect)
}

fn parse_marker_state(contents: &str) -> Option<i32> {
    contents
        .lines()
        .find_map(|line| line.trim().strip_prefix("abstate=")?.parse().ok())
}

fn parse_marker_primary(contents: &str) -> Option<RECT> {
    contents
        .lines()
        .find_map(|line| parse_rect(line.trim().strip_prefix("primary=")?.split(',')))
}

fn parse_marker_secondaries(contents: &str) -> HashMap<isize, RECT> {
    contents
        .lines()
        .filter_map(|line| {
            let mut parts = line.trim().strip_prefix("secondary=")?.split(',');
            let hwnd = parts.next()?.trim().parse::<isize>().ok()?;
            Some((hwnd, parse_rect(parts)?))
        })
        .collect()
}

/// Hides the Windows primary (`Shell_TrayWnd`) and secondary (`Shell_SecondaryTrayWnd`)
/// taskbars when the native Dock is active, persisting a crash-recovery marker at
/// `%APPDATA%\ai.hime.desktop\native_taskbar_hidden.flag`.
pub fn hide_taskbar() -> bool {
    let Some(marker_path) = taskbar_marker_path() else {
        return false;
    };

    unsafe {
        let tray_class = PCSTR(c"Shell_TrayWnd".as_ptr() as *const u8);
        let Ok(tray_hwnd) = FindWindowA(tray_class, PCSTR::null()) else {
            return false;
        };
        let secondary_trays = secondary_taskbar_windows();

        if let Ok(mut guard) = ORIGINAL_TRAY_RECT.lock() {
            if guard.is_none() {
                *guard = placed_taskbar_rect(tray_hwnd);
            }
        }
        if let Ok(mut guard) = ORIGINAL_SEC_TRAY_RECTS.lock() {
            let rects = guard.get_or_insert_with(HashMap::new);
            for &hwnd in &secondary_trays {
                if let Some(rect) = placed_taskbar_rect(hwnd) {
                    rects.entry(hwnd.0 as isize).or_insert(rect);
                }
            }
        }

        if ORIGINAL_TASKBAR_STATE.load(Ordering::Relaxed) == -1 {
            let mut abd = APPBARDATA {
                cbSize: std::mem::size_of::<APPBARDATA>() as u32,
                hWnd: tray_hwnd,
                ..Default::default()
            };
            let state = SHAppBarMessage(ABM_GETSTATE, &mut abd) as i32;
            ORIGINAL_TASKBAR_STATE.store(state, Ordering::Relaxed);
        }

        // Write crash marker before hiding so any abnormal termination can recover.
        if let Some(dir) = marker_path.parent() {
            if fs::create_dir_all(dir).is_err() {
                return false;
            }
        }
        let original_state = ORIGINAL_TASKBAR_STATE.load(Ordering::Relaxed);
        let primary = ORIGINAL_TRAY_RECT.lock().ok().and_then(|g| *g);
        let secondaries = ORIGINAL_SEC_TRAY_RECTS
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .unwrap_or_default();
        let contents = format_marker_contents(
            original_state,
            std::process::id(),
            primary,
            &secondaries,
        );
        let written = fs::File::create(&marker_path).and_then(|mut f| {
            f.write_all(contents.as_bytes())?;
            f.sync_all()
        });
        if written.is_err() {
            return false;
        }

        // Force auto-hide only when not already set to avoid poking Win11 state machine.
        let mut get_abd = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: tray_hwnd,
            ..Default::default()
        };
        let current_state = SHAppBarMessage(ABM_GETSTATE, &mut get_abd) as isize;
        if (current_state & 1) == 0 {
            let mut set_abd = APPBARDATA {
                cbSize: std::mem::size_of::<APPBARDATA>() as u32,
                hWnd: tray_hwnd,
                lParam: LPARAM(1),
                ..Default::default()
            };
            SHAppBarMessage(ABM_SETSTATE, &mut set_abd);
        }

        let _ = ShowWindow(tray_hwnd, SW_HIDE);
        let _ = SetWindowPos(
            tray_hwnd,
            None,
            -10000,
            -10000,
            0,
            0,
            SWP_HIDEWINDOW | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );

        for &sec_hwnd in &secondary_trays {
            let _ = ShowWindow(sec_hwnd, SW_HIDE);
            let _ = SetWindowPos(
                sec_hwnd,
                None,
                -10000,
                -10000,
                0,
                0,
                SWP_HIDEWINDOW | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }

        !IsWindowVisible(tray_hwnd).as_bool()
    }
}

/// Restores `Shell_TrayWnd` and `Shell_SecondaryTrayWnd` to their original state and
/// position, recovering from `%APPDATA%\ai.hime.desktop\native_taskbar_hidden.flag` if needed,
/// and removes the crash marker upon success.
///
/// Must be called on startup crash-recovery and on application exit.
pub fn restore_taskbar() -> bool {
    let marker_path = taskbar_marker_path();
    if let Some(ref path) = marker_path {
        if let Ok(contents) = fs::read_to_string(path) {
            if let Some(state) = parse_marker_state(&contents) {
                ORIGINAL_TASKBAR_STATE.store(state, Ordering::Relaxed);
            }
            if let Some(rect) = parse_marker_primary(&contents).filter(is_placed_rect) {
                if let Ok(mut guard) = ORIGINAL_TRAY_RECT.lock() {
                    *guard = Some(rect);
                }
            }
            let mut secs = parse_marker_secondaries(&contents);
            secs.retain(|_, r| is_placed_rect(r));
            if !secs.is_empty() {
                if let Ok(mut guard) = ORIGINAL_SEC_TRAY_RECTS.lock() {
                    guard.get_or_insert_with(HashMap::new).extend(secs);
                }
            }
        }
    }

    unsafe {
        let tray_class = PCSTR(c"Shell_TrayWnd".as_ptr() as *const u8);
        let Ok(tray_hwnd) = FindWindowA(tray_class, PCSTR::null()) else {
            return false;
        };
        let secondary_trays = secondary_taskbar_windows();

        if let Ok(mut guard) = ORIGINAL_TRAY_RECT.lock() {
            if guard.is_none() {
                let mut pos = APPBARDATA {
                    cbSize: std::mem::size_of::<APPBARDATA>() as u32,
                    hWnd: tray_hwnd,
                    ..Default::default()
                };
                if SHAppBarMessage(ABM_GETTASKBARPOS, &mut pos) != 0 && is_placed_rect(&pos.rc) {
                    *guard = Some(pos.rc);
                }
            }
        }

        let orig = ORIGINAL_TASKBAR_STATE.load(Ordering::Relaxed);
        let target_state = if orig != -1 { orig as isize } else { 2 };

        let mut get_abd = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: tray_hwnd,
            ..Default::default()
        };
        let current_state = SHAppBarMessage(ABM_GETSTATE, &mut get_abd) as isize;
        if current_state != target_state {
            let mut set_abd = APPBARDATA {
                cbSize: std::mem::size_of::<APPBARDATA>() as u32,
                hWnd: tray_hwnd,
                lParam: LPARAM(target_state),
                ..Default::default()
            };
            SHAppBarMessage(ABM_SETSTATE, &mut set_abd);
        }

        restore_taskbar_exstyle(tray_hwnd);
        let mut primary_positioned = true;
        if let Ok(guard) = ORIGINAL_TRAY_RECT.lock() {
            if let Some(rect) = *guard {
                primary_positioned = SetWindowPos(
                    tray_hwnd,
                    None,
                    rect.left,
                    rect.top,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                )
                .is_ok();
            }
        }
        let _ = ShowWindow(tray_hwnd, SW_SHOW);

        let mut secondary_positioned = true;
        for &sec_hwnd in &secondary_trays {
            restore_taskbar_exstyle(sec_hwnd);
            let saved = ORIGINAL_SEC_TRAY_RECTS.lock().ok().and_then(|g| {
                g.as_ref()
                    .and_then(|rects| rects.get(&(sec_hwnd.0 as isize)).copied())
            });
            if let Some(rect) = saved {
                secondary_positioned &= SetWindowPos(
                    sec_hwnd,
                    None,
                    rect.left,
                    rect.top,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                )
                .is_ok();
            }
            let _ = ShowWindow(sec_hwnd, SW_SHOW);
        }

        let on_screen = |hwnd: HWND| {
            let mut r = RECT::default();
            GetWindowRect(hwnd, &mut r).is_ok() && r.left > -9000 && r.top > -9000
        };
        let restored = primary_positioned
            && secondary_positioned
            && IsWindowVisible(tray_hwnd).as_bool()
            && on_screen(tray_hwnd)
            && secondary_trays
                .iter()
                .all(|&h| IsWindowVisible(h).as_bool() && on_screen(h));

        if restored {
            if let Some(ref path) = marker_path {
                if path.exists() {
                    let _ = fs::remove_file(path);
                }
            }
            if let Ok(mut guard) = ORIGINAL_SEC_TRAY_RECTS.lock() {
                if let Some(rects) = guard.as_mut() {
                    for h in &secondary_trays {
                        rects.remove(&(h.0 as isize));
                    }
                }
            }
        }

        restored
    }
}

/// Checks for `%APPDATA%\ai.hime.desktop\native_taskbar_hidden.flag` on startup
/// and restores the taskbar if a previous session exited abnormally.
pub fn restore_taskbar_if_crashed() -> bool {
    if has_taskbar_crash_marker() {
        restore_taskbar()
    } else {
        false
    }
}

/// Convenience wrapper to show (`true`) or hide (`false`) the native Windows taskbar.
pub fn set_taskbar_visible(visible: bool) -> bool {
    if visible {
        restore_taskbar()
    } else {
        hide_taskbar()
    }
}

/// Fast path: re-hides primary `Shell_TrayWnd` immediately if Windows Explorer un-hid or moved it back onto the screen.
pub fn ensure_primary_taskbar_hidden() -> bool {
    unsafe {
        let tray_class = PCSTR(c"Shell_TrayWnd".as_ptr() as *const u8);
        let Ok(tray_hwnd) = FindWindowA(tray_class, PCSTR::null()) else {
            return false;
        };
        if IsWindowVisible(tray_hwnd).as_bool() || placed_taskbar_rect(tray_hwnd).is_some() {
            return hide_taskbar();
        }
        true
    }
}

/// Re-hides `Shell_TrayWnd` and secondary taskbars if Windows Explorer made them visible or moved them onscreen.
pub fn ensure_taskbar_hidden() -> bool {
    unsafe {
        let tray_class = PCSTR(c"Shell_TrayWnd".as_ptr() as *const u8);
        let Ok(tray_hwnd) = FindWindowA(tray_class, PCSTR::null()) else {
            return false;
        };
        if IsWindowVisible(tray_hwnd).as_bool() || placed_taskbar_rect(tray_hwnd).is_some() {
            return hide_taskbar();
        }
        for sec_hwnd in secondary_taskbar_windows() {
            if IsWindowVisible(sec_hwnd).as_bool() || placed_taskbar_rect(sec_hwnd).is_some() {
                return hide_taskbar();
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// 2. 3-Edge Drag Snap & Mouse Hit-Testing Helpers
// ---------------------------------------------------------------------------

/// Returns the primary monitor dimensions via `GetSystemMetrics(SM_CXSCREEN / SM_CYSCREEN)`.
pub fn primary_screen_bounds() -> ScreenBounds {
    unsafe {
        let w = GetSystemMetrics(SM_CXSCREEN).max(1);
        let h = GetSystemMetrics(SM_CYSCREEN).max(1);
        ScreenBounds::new(0, 0, w, h)
    }
}

/// Returns the combined virtual desktop bounds across all monitors.
pub fn virtual_screen_bounds() -> ScreenBounds {
    unsafe {
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let w = GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1);
        let h = GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1);
        ScreenBounds::new(x, y, w, h)
    }
}

/// Returns the monitor bounds containing `(x, y)`, falling back to [`primary_screen_bounds`].
pub fn screen_bounds_at_point(x: i32, y: i32) -> ScreenBounds {
    unsafe {
        let hmon = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(hmon, &mut info).as_bool() {
            let r = info.rcMonitor;
            return ScreenBounds::new(
                r.left,
                r.top,
                (r.right - r.left).max(1),
                (r.bottom - r.top).max(1),
            );
        }
    }
    primary_screen_bounds()
}

/// Returns the current global cursor position in screen coordinates.
pub fn cursor_position() -> Option<(i32, i32)> {
    let mut pt = POINT::default();
    unsafe { GetCursorPos(&mut pt) }
        .ok()
        .map(|()| (pt.x, pt.y))
}

/// Determines the nearest allowed [`NotchEdge`] (`Top`, `Left`, or `Right` — never `Bottom`)
/// from drag cursor coordinates `(cursor_x, cursor_y)` within `screen`.
pub fn snap_notch_edge(cursor_x: f32, cursor_y: f32, screen: ScreenBounds) -> NotchEdge {
    let sw = (screen.width.max(1)) as f32;
    let rx = (cursor_x - screen.x as f32).clamp(0.0, sw);
    let ry = (cursor_y - screen.y as f32).max(0.0);

    let dist_top = ry;
    let dist_left = rx;
    let dist_right = (sw - rx).max(0.0);

    if dist_left < dist_top && dist_left <= dist_right {
        NotchEdge::Left
    } else if dist_right < dist_top && dist_right < dist_left {
        NotchEdge::Right
    } else {
        NotchEdge::Top
    }
}

/// Samples the live cursor position and monitor bounds to compute the 3-edge snap target.
pub fn snap_notch_edge_from_cursor() -> NotchEdge {
    match cursor_position() {
        Some((x, y)) => {
            let screen = screen_bounds_at_point(x, y);
            snap_notch_edge(x as f32, y as f32, screen)
        }
        None => NotchEdge::Top,
    }
}

/// Returns `true` if `(x, y)` lies inside `rect` (inclusive).
pub fn is_point_in_rect(x: f32, y: f32, rect: Rect) -> bool {
    x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height
}

/// Returns `true` when `(cursor_x, cursor_y)` is within `threshold_px` of `edge`
/// along `screen`, used to wake a hidden (`is_hidden`) Notch on edge hover.
pub fn is_cursor_at_notch_wake_edge(
    cursor_x: f32,
    cursor_y: f32,
    edge: NotchEdge,
    screen: ScreenBounds,
    threshold_px: f32,
) -> bool {
    let threshold = threshold_px.max(1.0);
    let rx = cursor_x - screen.x as f32;
    let ry = cursor_y - screen.y as f32;
    let sw = screen.width as f32;
    let sh = screen.height as f32;

    if rx < 0.0 || rx > sw || ry < 0.0 || ry > sh {
        return false;
    }

    match edge {
        NotchEdge::Top => ry <= threshold,
        NotchEdge::Left => rx <= threshold,
        NotchEdge::Right => (sw - rx) <= threshold,
    }
}

/// Hit-tests `(x, y)` (in surface-local coordinates) against `notch.computed_rect(...)`.
pub fn hit_test_notch(
    x: f32,
    y: f32,
    notch: &NotchView,
    surface_width: f32,
    surface_height: f32,
) -> bool {
    let rect = notch.computed_rect(surface_width, surface_height);
    is_point_in_rect(x, y, rect)
}

/// Returns `true` if the cursor at `(cursor_x, cursor_y)` (screen coordinates) should wake
/// a hidden `NotchView` or hits an active `NotchView` along its anchored edge.
pub fn is_notch_edge_hovered(
    cursor_x: f32,
    cursor_y: f32,
    notch: &NotchView,
    screen: ScreenBounds,
    wake_margin_px: f32,
) -> bool {
    is_notch_edge_hovered_scaled(cursor_x, cursor_y, notch, screen, wake_margin_px, 1.0)
}

/// DPI-aware variant of [`is_notch_edge_hovered`] converting physical cursor/screen pixels
/// into DIP coordinates before comparing with `NotchView` spring dimensions.
pub fn is_notch_edge_hovered_scaled(
    cursor_x: f32,
    cursor_y: f32,
    notch: &NotchView,
    screen: ScreenBounds,
    wake_margin_px: f32,
    dip_scale: f32,
) -> bool {
    let scale = dip_scale.max(0.5);
    let rx = (cursor_x - screen.x as f32) / scale;
    let ry = (cursor_y - screen.y as f32) / scale;
    let sw = (screen.width.max(1) as f32) / scale;
    let sh = (screen.height.max(1) as f32) / scale;
    let margin_dip = wake_margin_px / scale;
    if rx < 0.0 || rx > sw || ry < 0.0 || ry > sh {
        return false;
    }
    let rect = notch.computed_rect(sw, sh);

    if notch.is_hidden {
        // When hidden, strictly match the minimized 48x3 target bar span unless dragging files
        // externally (wake_margin_px > 20.0) so top-edge hover never leaks across wide empty space.
        let is_drag = wake_margin_px > 20.0;
        let margin = if is_drag {
            margin_dip
        } else {
            margin_dip.clamp(3.0, 6.0)
        };
        let wake_rect = match notch.edge {
            NotchEdge::Top => {
                let base_w = if rect.width > 1.0 { rect.width } else { 48.0 };
                let span = if is_drag { base_w.max(220.0) } else { base_w.min(64.0) };
                let x = (sw - span) * 0.5;
                Rect::new(x, 0.0, span, margin.max(rect.height))
            }
            NotchEdge::Left => {
                let base_h = if rect.height > 1.0 { rect.height } else { 48.0 };
                let span = if is_drag { base_h.max(200.0) } else { base_h.min(64.0) };
                let y = (sh - span) * 0.5;
                Rect::new(0.0, y, margin.max(rect.width), span)
            }
            NotchEdge::Right => {
                let base_h = if rect.height > 1.0 { rect.height } else { 48.0 };
                let span = if is_drag { base_h.max(200.0) } else { base_h.min(64.0) };
                let y = (sh - span) * 0.5;
                let w = margin.max(rect.width);
                Rect::new((sw - w).max(0.0), y, w, span)
            }
        };
        is_point_in_rect(rx, ry, wake_rect)
    } else {
        // Use at least the collapsed pill dimensions with 4px hysteresis so mid-animation spring
        // frames after waking never drop below the cursor and cause open/close flicker.
        let hyst = 4.0_f32;
        let stable_rect = match notch.edge {
            NotchEdge::Top => {
                let w = rect.width.max(276.0) + hyst * 2.0;
                let h = rect.height.max(36.0) + hyst;
                Rect::new(((sw - w) * 0.5).max(0.0), 0.0, w.min(sw), h)
            }
            NotchEdge::Left => {
                let w = rect.width.max(46.0) + hyst;
                let h = rect.height.max(200.0) + hyst * 2.0;
                Rect::new(0.0, ((sh - h) * 0.5).max(0.0), w, h.min(sh))
            }
            NotchEdge::Right => {
                let w = rect.width.max(46.0) + hyst;
                let h = rect.height.max(200.0) + hyst * 2.0;
                Rect::new((sw - w).max(0.0), ((sh - h) * 0.5).max(0.0), w, h.min(sh))
            }
        };
        is_point_in_rect(rx, ry, stable_rect)
    }
}

/// Returns `true` when `(cursor_x, cursor_y)` in screen coordinates lies inside the bottom-edge
/// wake zone for a hidden [`DockBar`] (so 100%-invisible AutoHide/Smart Docks wake smoothly).
pub fn is_dock_edge_hovered(
    cursor_x: f32,
    cursor_y: f32,
    dock: &DockBar,
    screen: ScreenBounds,
    wake_margin_px: f32,
) -> bool {
    let rx = cursor_x - screen.x as f32;
    let ry = cursor_y - screen.y as f32;
    let sw = screen.width.max(1) as f32;
    let sh = screen.height.max(1) as f32;
    if rx < 0.0 || rx > sw || ry < 0.0 || ry > sh {
        return false;
    }

    let surf_rect = Rect::new(0.0, 0.0, sw, sh);
    let dock_rect = dock.dock_rect(surf_rect);
    if dock.is_hidden {
        let margin = wake_margin_px.max(4.0);
        let span = dock_rect.width.max(240.0).min(sw);
        let wake_x = ((sw - span) * 0.5).max(0.0);
        let wake_rect = Rect::new(wake_x, (sh - margin).max(0.0), span, margin);
        is_point_in_rect(rx, ry, wake_rect)
    } else {
        is_point_in_rect(rx, ry, dock_rect)
    }
}

/// Returns `true` when `(cursor_x, cursor_y)` in screen coordinates lies in the vertical
/// edge-wake zone for the left (Volume) or right (Brightness) invisible edge overlay.
pub fn is_overlay_edge_hovered(
    cursor_x: f32,
    cursor_y: f32,
    is_left: bool,
    screen: ScreenBounds,
    wake_margin_px: f32,
    band_height: f32,
) -> bool {
    let rx = cursor_x - screen.x as f32;
    let ry = cursor_y - screen.y as f32;
    let sw = screen.width.max(1) as f32;
    let sh = screen.height.max(1) as f32;
    let margin = wake_margin_px.max(4.0);
    let band_h = band_height.clamp(48.0, sh);
    let top = (sh - band_h) * 0.5;
    let wake_rect = if is_left {
        Rect::new(0.0, top, margin, band_h)
    } else {
        Rect::new((sw - margin).max(0.0), top, margin, band_h)
    };
    is_point_in_rect(rx, ry, wake_rect)
}

/// Returns `true` when pointer events should pass through (`WS_EX_TRANSPARENT`)
/// because the cursor is outside both the Notch (or its wake strip) and the visible Dock.
pub fn should_enable_click_through(
    cursor_x: f32,
    cursor_y: f32,
    notch: &NotchView,
    dock: Option<&DockBar>,
    screen: ScreenBounds,
    wake_margin_px: f32,
) -> bool {
    if is_notch_edge_hovered(cursor_x, cursor_y, notch, screen, wake_margin_px) {
        return false;
    }

    if let Some(dock_bar) = dock {
        if dock_bar.visible || dock_bar.visibility.value > 0.01 {
            let rx = cursor_x - screen.x as f32;
            let ry = cursor_y - screen.y as f32;
            let surface_rect = Rect::new(0.0, 0.0, screen.width as f32, screen.height as f32);
            let dock_rect = dock_bar.dock_rect(surface_rect);
            if is_point_in_rect(rx, ry, dock_rect) {
                return false;
            }
        }
    }

    true
}

// ---------------------------------------------------------------------------
// 3. Drag-and-Drop (`WM_DROPFILES`)
// ---------------------------------------------------------------------------

/// Enables or disables Win32 `WM_DROPFILES` acceptance on `hwnd`, including UIPI message filter
/// allowance for Explorer drag-and-drop (`WM_DROPFILES`, `WM_COPYGLOBALDATA`, `WM_COPYDATA`).
pub fn enable_drag_drop(hwnd: HWND, accept: bool) {
    #[link(name = "user32")]
    extern "system" {
        fn ChangeWindowMessageFilterEx(
            hwnd: isize,
            message: u32,
            action: u32,
            pchangefilterstruct: *mut c_void,
        ) -> i32;
    }
    const MSGFLT_ALLOW: u32 = 1;
    const MSGFLT_RESET: u32 = 0;
    const WM_DROPFILES_MSG: u32 = 0x0233;
    const WM_COPYDATA_MSG: u32 = 0x004A;
    const WM_COPYGLOBALDATA_MSG: u32 = 0x0049;

    unsafe {
        DragAcceptFiles(hwnd, accept);
        let action = if accept { MSGFLT_ALLOW } else { MSGFLT_RESET };
        for msg in [WM_DROPFILES_MSG, WM_COPYGLOBALDATA_MSG, WM_COPYDATA_MSG] {
            let _ = ChangeWindowMessageFilterEx(hwnd.0 as isize, msg, action, std::ptr::null_mut());
        }
    }
}

/// Alias for [`enable_drag_drop`] to make Notch/Drawer file-drop wiring explicit.
pub fn enable_file_drop(hwnd: HWND, accept: bool) {
    enable_drag_drop(hwnd, accept);
}

/// Toggles `WS_EX_NOACTIVATE` so an interactive surface (Drawer / Settings) can accept keyboard focus.
pub fn set_window_focusable(hwnd: HWND, focusable: bool) {
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let no_activate = WS_EX_NOACTIVATE.0 as i32;
        let new_style = if focusable {
            style & !no_activate
        } else {
            style | no_activate
        };
        if new_style != style {
            let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
        }
    }
}

/// Brings `hwnd` to the foreground and gives it keyboard focus for `WM_KEYDOWN` / `WM_CHAR`.
pub fn focus_window(hwnd: HWND) {
    unsafe {
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(Some(hwnd));
    }
}

/// Re-asserts `HWND_TOPMOST` Z-order without activating or stealing focus.
pub fn ensure_topmost(hwnd: HWND) {
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

/// Extracts all dropped file/folder paths from a Win32 `HDROP` handle and calls `DragFinish`.
pub fn extract_dropped_files(hdrop: HDROP) -> Vec<PathBuf> {
    if hdrop.0.is_null() {
        return Vec::new();
    }

    let mut paths = Vec::new();
    unsafe {
        let count = DragQueryFileW(hdrop, u32::MAX, None);
        for index in 0..count {
            let len = DragQueryFileW(hdrop, index, None);
            if len == 0 {
                continue;
            }
            let mut buf = vec![0u16; len as usize + 1];
            let written = DragQueryFileW(hdrop, index, Some(&mut buf));
            if written > 0 {
                let os_str = OsString::from_wide(&buf[..written as usize]);
                paths.push(PathBuf::from(os_str));
            }
        }
        DragFinish(hdrop);
    }
    paths
}

/// Handles `WM_DROPFILES` (`wparam` holds `HDROP`), extracts paths, and adds them to `shelf`.
/// Returns the extracted list of `PathBuf`s.
pub fn handle_wm_dropfiles(wparam: WPARAM, shelf: &mut FileShelf) -> Vec<PathBuf> {
    let hdrop = HDROP(wparam.0 as *mut std::ffi::c_void);
    let paths = extract_dropped_files(hdrop);
    if !paths.is_empty() {
        shelf.add_paths(&paths);
    }
    paths
}

/// Returns `true` if `hwnd` is the Windows Explorer Desktop shell window (`Progman` / `WorkerW`)
/// so `DockMode::Smart` and `notch_mode == "smart"` do not treat an empty desktop as overlapping.
pub fn is_desktop_shell_hwnd(hwnd: HWND) -> bool {
    if hwnd.0.is_null() {
        return false;
    }
    let mut class = [0u8; 64];
    let len = unsafe { GetClassNameA(hwnd, &mut class) };
    if len <= 0 {
        return false;
    }
    matches!(&class[..len as usize], b"Progman" | b"WorkerW")
}

/// Pure classifier for transient Windows shell/flyout/OSD window classes (Taskbar, Quick Settings,
/// Volume/Brightness OSD, tray overflow) that should not trigger `DockMode::Smart` visibility toggles.
pub fn is_transient_system_flyout_class(class_name: &str) -> bool {
    matches!(
        class_name,
        "Shell_TrayWnd"
            | "Shell_SecondaryTrayWnd"
            | "NotifyIconOverflowWindow"
            | "TopLevelWindowForOverflowXamlIsland"
            | "ControlCenterWindow"
            | "XamlExplorerHostIslandWindow"
            | "Windows.UI.Core.CoreWindow"
    )
}

/// Returns `true` if `hwnd` belongs to a transient Windows shell flyout or hardware Volume/Brightness OSD.
pub fn is_transient_system_flyout_hwnd(hwnd: HWND) -> bool {
    if hwnd.0.is_null() {
        return false;
    }
    let mut class = [0u8; 96];
    let len = unsafe { GetClassNameA(hwnd, &mut class) };
    if len <= 0 {
        return false;
    }
    let Ok(class_name) = std::str::from_utf8(&class[..len as usize]) else {
        return false;
    };
    is_transient_system_flyout_class(class_name)
}

// ---------------------------------------------------------------------------
// 4. Windows 10/11 Virtual Desktops ("Spaces")
// ---------------------------------------------------------------------------

/// Metadata for a Windows 10/11 Virtual Desktop ("Space").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualDesktopInfo {
    /// Canonical lowercase GUID string (`"{...}"`) or fallback `"desktop-1"`.
    pub id: String,
    /// Display name from Registry (`Name` value) or fallback `"Desktop N"`.
    pub name: String,
    /// Zero-based desktop index matching `VirtualDesktopIDs` order.
    pub index: usize,
    /// True if this is the active desktop (`CurrentVirtualDesktop`).
    pub active: bool,
}

const VIRTUAL_DESKTOPS_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\VirtualDesktops";

#[link(name = "advapi32")]
extern "system" {
    fn RegOpenKeyExW(
        hKey: isize,
        lpSubKey: windows::core::PCWSTR,
        ulOptions: u32,
        samDesired: u32,
        phkResult: *mut isize,
    ) -> i32;
    fn RegQueryValueExW(
        hKey: isize,
        lpValueName: windows::core::PCWSTR,
        lpReserved: *mut u32,
        lpType: *mut u32,
        lpData: *mut u8,
        lpcbData: *mut u32,
    ) -> i32;
    fn RegCloseKey(hKey: isize) -> i32;
}

fn to_wide_null(s: &str) -> Vec<u16> {
    OsString::from(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn read_hkcu_value(subkey: &str, value_name: &str) -> Option<(u32, Vec<u8>)> {
    const HKEY_CURRENT_USER: isize = 0x8000_0001_u32 as i32 as isize;
    const KEY_READ: u32 = 0x0002_0019;

    let subkey_w = to_wide_null(subkey);
    let val_w = to_wide_null(value_name);
    let mut hkey: isize = 0;
    unsafe {
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            windows::core::PCWSTR(subkey_w.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        ) != 0
        {
            return None;
        }
        let mut reg_type: u32 = 0;
        let mut size: u32 = 0;
        let rc_size = RegQueryValueExW(
            hkey,
            windows::core::PCWSTR(val_w.as_ptr()),
            std::ptr::null_mut(),
            &mut reg_type,
            std::ptr::null_mut(),
            &mut size,
        );
        if rc_size != 0 || size == 0 || size > 65_536 {
            let _ = RegCloseKey(hkey);
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        let rc_read = RegQueryValueExW(
            hkey,
            windows::core::PCWSTR(val_w.as_ptr()),
            std::ptr::null_mut(),
            &mut reg_type,
            buf.as_mut_ptr(),
            &mut size,
        );
        let _ = RegCloseKey(hkey);
        if rc_read != 0 {
            return None;
        }
        buf.truncate(size as usize);
        Some((reg_type, buf))
    }
}

/// Returns `true` when Windows Personalization (`SystemUsesLightTheme` / `AppsUseLightTheme`)
/// is configured for Light mode (zero screen capture overhead for GPU Acrylic `Auto` polarity).
pub fn system_uses_light_theme() -> bool {
    const PERSONALIZE_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    const REG_DWORD: u32 = 4;
    for val_name in ["SystemUsesLightTheme", "AppsUseLightTheme"] {
        if let Some((kind, bytes)) = read_hkcu_value(PERSONALIZE_KEY, val_name) {
            if kind == REG_DWORD && bytes.len() >= 4 {
                return u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) != 0;
            }
        }
    }
    false
}

/// Returns `true` when Windows Personalization `EnableTransparency` is enabled (`!= 0`).
/// When `false`, Windows DWM turns `ACCENT_ENABLE_ACRYLICBLURBEHIND` solid black, so Hime falls
/// back cleanly to pure-Rust `LiquidGlass` backdrop compositing.
pub fn dwm_transparency_enabled() -> bool {
    const PERSONALIZE_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    const REG_DWORD: u32 = 4;
    if let Some((kind, bytes)) = read_hkcu_value(PERSONALIZE_KEY, "EnableTransparency") {
        if kind == REG_DWORD && bytes.len() >= 4 {
            return u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) != 0;
        }
    }
    false
}

fn read_hkcu_binary(subkey: &str, value_name: &str) -> Option<Vec<u8>> {
    const REG_BINARY: u32 = 3;
    let (ty, data) = read_hkcu_value(subkey, value_name)?;
    (ty == REG_BINARY).then_some(data)
}

fn read_hkcu_string(subkey: &str, value_name: &str) -> Option<String> {
    const REG_SZ: u32 = 1;
    let (ty, data) = read_hkcu_value(subkey, value_name)?;
    if ty != REG_SZ || data.len() < 2 {
        return None;
    }
    let u16s: Vec<u16> = data
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&w| w != 0)
        .collect();
    let s = String::from_utf16_lossy(&u16s).trim().to_string();
    (!s.is_empty()).then_some(s)
}

/// Formats a 16-byte Windows GUID binary blob as uppercase `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}`.
pub fn format_windows_guid_upper(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 16 {
        return None;
    }
    let d1 = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
    let d2 = u16::from_le_bytes(bytes[4..6].try_into().ok()?);
    let d3 = u16::from_le_bytes(bytes[6..8].try_into().ok()?);
    Some(format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        d1,
        d2,
        d3,
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    ))
}

/// Pure helper that converts raw `VirtualDesktopIDs` + `CurrentVirtualDesktop` registry bytes
/// and a desktop-name lookup closure into ordered [`VirtualDesktopInfo`] entries.
pub fn parse_virtual_desktops_from_bytes<F>(
    ids_bytes: Option<&[u8]>,
    current_bytes: Option<&[u8]>,
    mut lookup_name: F,
) -> Vec<VirtualDesktopInfo>
where
    F: FnMut(&str) -> Option<String>,
{
    let current_upper = current_bytes
        .filter(|b| b.len() >= 16)
        .and_then(|b| format_windows_guid_upper(&b[..16]));

    let mut guids: Vec<String> = ids_bytes
        .unwrap_or(&[])
        .chunks_exact(16)
        .filter_map(format_windows_guid_upper)
        .collect();

    if guids.is_empty() {
        if let Some(ref cur) = current_upper {
            guids.push(cur.clone());
        } else {
            return vec![VirtualDesktopInfo {
                id: "desktop-1".to_string(),
                name: "Desktop 1".to_string(),
                index: 0,
                active: true,
            }];
        }
    }

    let mut any_active = false;
    let mut out: Vec<VirtualDesktopInfo> = guids
        .into_iter()
        .enumerate()
        .map(|(index, guid_upper)| {
            let active = current_upper.as_deref() == Some(guid_upper.as_str());
            any_active |= active;
            let name = lookup_name(&guid_upper)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("Desktop {}", index + 1));
            VirtualDesktopInfo {
                id: guid_upper.to_ascii_lowercase(),
                name,
                index,
                active,
            }
        })
        .collect();

    if !any_active {
        if let Some(first) = out.first_mut() {
            first.active = true;
        }
    }
    out
}

/// Lists Windows 10/11 Virtual Desktops safely from `HKCU\...\Explorer\VirtualDesktops`.
/// Never panics; falls back to a single `"Desktop 1"` entry if Registry keys are absent.
pub fn list_virtual_desktops() -> Vec<VirtualDesktopInfo> {
    let ids = read_hkcu_binary(VIRTUAL_DESKTOPS_KEY, "VirtualDesktopIDs");
    let current = read_hkcu_binary(VIRTUAL_DESKTOPS_KEY, "CurrentVirtualDesktop");
    parse_virtual_desktops_from_bytes(ids.as_deref(), current.as_deref(), |guid_upper| {
        let desktop_key = format!(r"{VIRTUAL_DESKTOPS_KEY}\Desktops\{guid_upper}");
        read_hkcu_string(&desktop_key, "Name")
    })
}

fn make_key_input(vk: VIRTUAL_KEY, extended: bool, key_up: bool) -> INPUT {
    let mut flags = windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0);
    if extended {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_win_ctrl_combo(key: VIRTUAL_KEY, extended_key: bool, repeats: usize) -> bool {
    if repeats == 0 {
        return true;
    }
    let steps = repeats.min(32);
    for step in 0..steps {
        let inputs = [
            make_key_input(VK_LWIN, true, false),
            make_key_input(VK_CONTROL, false, false),
            make_key_input(key, extended_key, false),
            make_key_input(key, extended_key, true),
            make_key_input(VK_CONTROL, false, true),
            make_key_input(VK_LWIN, true, true),
        ];
        let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
        if sent != inputs.len() as u32 {
            return false;
        }
        if step + 1 < steps {
            thread::sleep(Duration::from_millis(35));
        }
    }
    true
}

/// Switches to the Virtual Desktop at `target_index` using `Win + Ctrl + Left/Right`.
pub fn switch_virtual_desktop(target_index: usize) -> bool {
    let list = list_virtual_desktops();
    if target_index >= list.len() {
        return false;
    }
    let current_idx = list.iter().position(|d| d.active).unwrap_or(0);
    if target_index == current_idx {
        return true;
    }
    if target_index > current_idx {
        send_win_ctrl_combo(VK_RIGHT, true, target_index - current_idx)
    } else {
        send_win_ctrl_combo(VK_LEFT, true, current_idx - target_index)
    }
}

/// Creates a new Windows Virtual Desktop using `Win + Ctrl + D`.
pub fn create_virtual_desktop() -> bool {
    const VK_D: VIRTUAL_KEY = VIRTUAL_KEY(0x44);
    send_win_ctrl_combo(VK_D, false, 1)
}

/// Opens the native Windows 11 Notification & Calendar Center using `Win + N`.
pub fn open_notification_center() -> bool {
    const VK_N: VIRTUAL_KEY = VIRTUAL_KEY(0x4E);
    let inputs = [
        make_key_input(VK_LWIN, true, false),
        make_key_input(VK_N, false, false),
        make_key_input(VK_N, false, true),
        make_key_input(VK_LWIN, true, true),
    ];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    sent == inputs.len() as u32
}

/// Launches an executable or shell target with optional command-line arguments.
pub fn launch_executable_with_args(target: &str, args: &str) {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return;
    }
    let wide_target: Vec<u16> = OsStr::new(trimmed).encode_wide().chain(Some(0)).collect();
    let trimmed_args = args.trim();
    let wide_args: Option<Vec<u16>> = if trimmed_args.is_empty() {
        None
    } else {
        Some(OsStr::new(trimmed_args).encode_wide().chain(Some(0)).collect())
    };
    unsafe {
        let params_ptr = wide_args
            .as_ref()
            .map_or(PCWSTR::null(), |v| PCWSTR(v.as_ptr()));
        let _ = ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(wide_target.as_ptr()),
            params_ptr,
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
    }
}

/// Opens Windows Explorer highlighting `path` on disk.
pub fn open_file_location(path: &str) {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return;
    }
    let args = format!("/select,\"{}\"", trimmed);
    launch_executable_with_args("explorer.exe", &args);
}

/// Requests graceful close (`WM_CLOSE`) for every valid HWND in `hwnds`.
pub fn close_windows_by_hwnd(hwnds: &[isize]) {
    for &raw in hwnds {
        if raw == 0 {
            continue;
        }
        let hwnd = HWND(raw as *mut c_void);
        unsafe {
            if IsWindow(Some(hwnd)).as_bool() {
                let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
    }
}

fn terminate_pid_guarded(pid: u32, own_pid: u32, terminated: &mut Vec<u32>) {
    if pid == 0 || pid == own_pid || terminated.contains(&pid) {
        return;
    }
    unsafe {
        if let Ok(proc_h) = OpenProcess(PROCESS_TERMINATE, false, pid) {
            let _ = TerminateProcess(proc_h, 1);
            let _ = CloseHandle(proc_h);
            terminated.push(pid);
        }
    }
}

/// Terminates unique non-self processes owning `hwnds` (matching Windows 11 "End task").
pub fn terminate_processes_by_hwnd(hwnds: &[isize]) {
    let own_pid = unsafe { GetCurrentProcessId() };
    let mut terminated_pids = Vec::new();
    for &raw in hwnds {
        if raw == 0 {
            continue;
        }
        let hwnd = HWND(raw as *mut c_void);
        let mut pid = 0u32;
        unsafe {
            if !IsWindow(Some(hwnd)).as_bool() {
                continue;
            }
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
        }
        terminate_pid_guarded(pid, own_pid, &mut terminated_pids);
    }
}

/// Terminates a list of non-self process IDs safely (used when tray apps have no top-level HWND).
pub fn terminate_processes_by_pid(pids: &[u32]) {
    let own_pid = unsafe { GetCurrentProcessId() };
    let mut terminated_pids = Vec::new();
    for &pid in pids {
        terminate_pid_guarded(pid, own_pid, &mut terminated_pids);
    }
}

// ---------------------------------------------------------------------------
// 5. Lightweight Desktop & Below-Z-Order Window Backdrop Capture
// ---------------------------------------------------------------------------

/// Premultiplied opaque BGRA backdrop buffer for a native shell surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedBackdrop {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

#[derive(Debug, Default)]
struct GdiBackdropScratch {
    cap_dc: HDC,
    cap_dib: HBITMAP,
    cap_old: HGDIOBJ,
    cap_bits: *mut c_void,
    cap_w: u32,
    cap_h: u32,
    win_dc: HDC,
    win_dib: HBITMAP,
    win_old: HGDIOBJ,
    win_w: i32,
    win_h: i32,
}

impl Drop for GdiBackdropScratch {
    fn drop(&mut self) {
        unsafe {
            if !self.cap_dc.0.is_null() {
                if !self.cap_dib.0.is_null() {
                    let _ = SelectObject(self.cap_dc, self.cap_old);
                    let _ = DeleteObject(self.cap_dib.into());
                }
                let _ = DeleteDC(self.cap_dc);
            }
            if !self.win_dc.0.is_null() {
                if !self.win_dib.0.is_null() {
                    let _ = SelectObject(self.win_dc, self.win_old);
                    let _ = DeleteObject(self.win_dib.into());
                }
                let _ = DeleteDC(self.win_dc);
            }
        }
    }
}

impl GdiBackdropScratch {
    unsafe fn ensure_cap_surface(&mut self, w: u32, h: u32) -> Option<(HDC, *mut u8, usize)> {
        if self.cap_dc.0.is_null() {
            self.cap_dc = CreateCompatibleDC(None);
            if self.cap_dc.0.is_null() {
                return None;
            }
            let _ = SetStretchBltMode(self.cap_dc, HALFTONE);
        }
        if self.cap_dib.0.is_null() || self.cap_w != w || self.cap_h != h {
            if !self.cap_dib.0.is_null() {
                let _ = SelectObject(self.cap_dc, self.cap_old);
                let _ = DeleteObject(self.cap_dib.into());
                self.cap_dib = HBITMAP::default();
                self.cap_bits = std::ptr::null_mut();
            }
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w as i32,
                    biHeight: -(h as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits: *mut c_void = std::ptr::null_mut();
            let dib =
                CreateDIBSection(Some(self.cap_dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
                    .ok()?;
            self.cap_old = SelectObject(self.cap_dc, dib.into());
            self.cap_dib = dib;
            self.cap_bits = bits;
            self.cap_w = w;
            self.cap_h = h;
        }
        if self.cap_bits.is_null() {
            return None;
        }
        let byte_len = (w as usize).saturating_mul(h as usize).saturating_mul(4);
        Some((self.cap_dc, self.cap_bits as *mut u8, byte_len))
    }

    unsafe fn ensure_win_surface(&mut self, w: i32, h: i32) -> Option<HDC> {
        if self.win_dc.0.is_null() {
            self.win_dc = CreateCompatibleDC(None);
            if self.win_dc.0.is_null() {
                return None;
            }
        }
        if self.win_dib.0.is_null() || self.win_w < w || self.win_h < h {
            if !self.win_dib.0.is_null() {
                let _ = SelectObject(self.win_dc, self.win_old);
                let _ = DeleteObject(self.win_dib.into());
                self.win_dib = HBITMAP::default();
            }
            let alloc_w = w.max(self.win_w);
            let alloc_h = h.max(self.win_h);
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: alloc_w,
                    biHeight: -alloc_h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits: *mut c_void = std::ptr::null_mut();
            let dib = CreateDIBSection(
                Some(self.win_dc),
                &bmi,
                DIB_RGB_COLORS,
                &mut bits,
                None,
                0,
            )
            .ok()?;
            self.win_old = SelectObject(self.win_dc, dib.into());
            self.win_dib = dib;
            self.win_w = alloc_w;
            self.win_h = alloc_h;
        }
        Some(self.win_dc)
    }
}

/// Captures desktop wallpaper + visible non-Hime windows underneath a surface without
/// `WDA_EXCLUDEFROMCAPTURE` (keeping screenshots intact and steady-state RAM tiny).
#[derive(Debug, Default)]
pub struct BackdropCapturer {
    cached_wallpaper_path: Option<String>,
    cached_wallpaper_color: u32,
    cached_wallpaper_screen: Option<ScreenBounds>,
    wallpaper_checked_at: Option<Instant>,
    wallpaper_w: u32,
    wallpaper_h: u32,
    wallpaper_bgra: Vec<u8>,
    scratch: GdiBackdropScratch,
}

unsafe impl Send for BackdropCapturer {}

impl BackdropCapturer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Captures a downsampled opaque BGRA backdrop for `(screen_x, screen_y, width, height)`
    /// excluding Hime's own HWNDs. When `is_surface_hidden` is `true`, uses a single `<0.5ms`
    /// screen `StretchBlt` before the surface becomes visible.
    pub fn capture_region(
        &mut self,
        screen_x: i32,
        screen_y: i32,
        width: u32,
        height: u32,
        exclude_hwnds: &[isize],
        is_surface_hidden: bool,
    ) -> Option<CapturedBackdrop> {
        if width == 0 || height == 0 {
            return None;
        }
        let cap_w = (width / 2).clamp(16, 480);
        let cap_h = (height / 2).clamp(16, 280);

        if is_surface_hidden {
            if let Some(bgra) = capture_screen_region_bgra(
                screen_x,
                screen_y,
                width as i32,
                height as i32,
                cap_w,
                cap_h,
            ) {
                return Some(CapturedBackdrop {
                    width: cap_w,
                    height: cap_h,
                    bgra,
                });
            }
        }

        let primary = primary_screen_bounds();
        self.ensure_wallpaper_cache(primary);
        let bgra = composite_windows_below(
            &mut self.scratch,
            screen_x,
            screen_y,
            width as i32,
            height as i32,
            cap_w,
            cap_h,
            exclude_hwnds,
            primary,
            (self.wallpaper_w, self.wallpaper_h, &self.wallpaper_bgra),
        )?;
        Some(CapturedBackdrop {
            width: cap_w,
            height: cap_h,
            bgra,
        })
    }

    fn ensure_wallpaper_cache(&mut self, screen: ScreenBounds) {
        let now = Instant::now();
        if self.cached_wallpaper_screen == Some(screen)
            && !self.wallpaper_bgra.is_empty()
            && self
                .wallpaper_checked_at
                .is_some_and(|t| now.duration_since(t) < Duration::from_millis(500))
        {
            return;
        }
        self.wallpaper_checked_at = Some(now);

        let wp_path = query_desktop_wallpaper_path();
        let bg_color = unsafe { GetSysColor(COLOR_BACKGROUND) };
        if self.cached_wallpaper_screen == Some(screen)
            && self.cached_wallpaper_path == wp_path
            && self.cached_wallpaper_color == bg_color
            && !self.wallpaper_bgra.is_empty()
        {
            return;
        }

        let target_w = ((screen.width.max(1) as u32) / 4).clamp(120, 480);
        let target_h = ((screen.height.max(1) as u32) / 4).clamp(68, 270);
        let pixels = wp_path
            .as_deref()
            .and_then(|p| decode_wallpaper_scaled_bgra(p, target_w, target_h))
            .unwrap_or_else(|| solid_colorref_bgra(bg_color, target_w, target_h));

        self.cached_wallpaper_path = wp_path;
        self.cached_wallpaper_color = bg_color;
        self.cached_wallpaper_screen = Some(screen);
        self.wallpaper_w = target_w;
        self.wallpaper_h = target_h;
        self.wallpaper_bgra = pixels;
    }
}

fn solid_colorref_bgra(colorref: u32, width: u32, height: u32) -> Vec<u8> {
    let r = (colorref & 0xFF) as u8;
    let g = ((colorref >> 8) & 0xFF) as u8;
    let b = ((colorref >> 16) & 0xFF) as u8;
    let count = (width as usize).saturating_mul(height as usize);
    let mut out = Vec::with_capacity(count * 4);
    for _ in 0..count {
        out.extend_from_slice(&[b, g, r, 255]);
    }
    out
}

fn force_opaque_alpha(bgra: &mut [u8]) {
    for px in bgra.chunks_exact_mut(4) {
        px[3] = 255;
    }
}

fn query_desktop_wallpaper_path() -> Option<String> {
    use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETDESKWALLPAPER};
    let mut buf = [0u16; 520];
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            buf.len() as u32,
            Some(buf.as_mut_ptr() as *mut c_void),
            Default::default(),
        )
    }
    .is_ok();
    if !ok {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    let path = String::from_utf16_lossy(&buf[..len]).trim().to_string();
    (!path.is_empty() && std::path::Path::new(&path).is_file()).then_some(path)
}

fn decode_wallpaper_scaled_bgra(path: &str, target_w: u32, target_h: u32) -> Option<Vec<u8>> {
    #[repr(C)]
    struct GdiplusStartupInput {
        gdiplus_version: u32,
        debug_event_callback: usize,
        suppress_background_threads: i32,
        suppress_external_codecs: i32,
    }
    #[link(name = "gdiplus")]
    extern "system" {
        fn GdiplusStartup(
            token: *mut usize,
            input: *const GdiplusStartupInput,
            output: *mut c_void,
        ) -> i32;
        fn GdiplusShutdown(token: usize);
        fn GdipLoadImageFromFile(filename: *const u16, image: *mut *mut c_void) -> i32;
        fn GdipDisposeImage(image: *mut c_void) -> i32;
        fn GdipCreateFromHDC(
            hdc: windows::Win32::Graphics::Gdi::HDC,
            graphics: *mut *mut c_void,
        ) -> i32;
        fn GdipDeleteGraphics(graphics: *mut c_void) -> i32;
        fn GdipDrawImageRectI(
            graphics: *mut c_void,
            image: *mut c_void,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        ) -> i32;
    }

    unsafe {
        let mut token: usize = 0;
        let input = GdiplusStartupInput {
            gdiplus_version: 1,
            debug_event_callback: 0,
            suppress_background_threads: 0,
            suppress_external_codecs: 0,
        };
        if GdiplusStartup(&mut token, &input, std::ptr::null_mut()) != 0 {
            return None;
        }

        let wide = to_wide_null(path);
        let mut image: *mut c_void = std::ptr::null_mut();
        if GdipLoadImageFromFile(wide.as_ptr(), &mut image) != 0 || image.is_null() {
            GdiplusShutdown(token);
            return None;
        }

        let mem_dc = CreateCompatibleDC(None);
        if mem_dc.0.is_null() {
            GdipDisposeImage(image);
            GdiplusShutdown(token);
            return None;
        }
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: target_w as i32,
                biHeight: -(target_h as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut c_void = std::ptr::null_mut();
        let Ok(dib) = CreateDIBSection(Some(mem_dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
        else {
            let _ = DeleteDC(mem_dc);
            GdipDisposeImage(image);
            GdiplusShutdown(token);
            return None;
        };
        let old = SelectObject(mem_dc, dib.into());
        let byte_len = (target_w as usize)
            .saturating_mul(target_h as usize)
            .saturating_mul(4);
        std::ptr::write_bytes(bits as *mut u8, 0, byte_len);

        let mut graphics: *mut c_void = std::ptr::null_mut();
        let ok = GdipCreateFromHDC(mem_dc, &mut graphics) == 0 && !graphics.is_null();
        let drawn = ok
            && GdipDrawImageRectI(graphics, image, 0, 0, target_w as i32, target_h as i32) == 0;
        if !graphics.is_null() {
            GdipDeleteGraphics(graphics);
        }

        let mut out = if drawn && !bits.is_null() {
            let mut slice = std::slice::from_raw_parts(bits as *const u8, byte_len).to_vec();
            force_opaque_alpha(&mut slice);
            Some(slice)
        } else {
            None
        };

        let _ = SelectObject(mem_dc, old);
        let _ = DeleteObject(dib.into());
        let _ = DeleteDC(mem_dc);
        GdipDisposeImage(image);
        GdiplusShutdown(token);
        out.take()
    }
}

fn capture_screen_region_bgra(
    sx: i32,
    sy: i32,
    sw: i32,
    sh: i32,
    cap_w: u32,
    cap_h: u32,
) -> Option<Vec<u8>> {
    unsafe {
        let screen_dc = GetDC(None);
        if screen_dc.0.is_null() {
            return None;
        }
        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        if mem_dc.0.is_null() {
            let _ = ReleaseDC(None, screen_dc);
            return None;
        }
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: cap_w as i32,
                biHeight: -(cap_h as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut c_void = std::ptr::null_mut();
        let Ok(dib) = CreateDIBSection(Some(mem_dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
        else {
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(None, screen_dc);
            return None;
        };
        let old = SelectObject(mem_dc, dib.into());
        let _ = SetStretchBltMode(mem_dc, HALFTONE);
        let ok = StretchBlt(
            mem_dc,
            0,
            0,
            cap_w as i32,
            cap_h as i32,
            Some(screen_dc),
            sx,
            sy,
            sw.max(1),
            sh.max(1),
            SRCCOPY,
        )
        .as_bool();

        let byte_len = (cap_w as usize).saturating_mul(cap_h as usize).saturating_mul(4);
        let out = if ok && !bits.is_null() {
            let mut buf = std::slice::from_raw_parts(bits as *const u8, byte_len).to_vec();
            force_opaque_alpha(&mut buf);
            Some(buf)
        } else {
            None
        };

        let _ = SelectObject(mem_dc, old);
        let _ = DeleteObject(dib.into());
        let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(None, screen_dc);
        out
    }
}

struct BackdropWinCandidate {
    hwnd: HWND,
    rect: RECT,
}

struct BackdropWinSearch<'a> {
    region: RECT,
    exclude_hwnds: &'a [isize],
    windows: Vec<BackdropWinCandidate>,
}

fn collect_live_wallpaper_children(desktop_hwnd: HWND, search: &mut BackdropWinSearch<'_>) {
    unsafe extern "system" fn child_proc(child: HWND, lparam: LPARAM) -> BOOL {
        let search = &mut *(lparam.0 as *mut BackdropWinSearch<'_>);
        let raw = child.0 as isize;
        if search.exclude_hwnds.contains(&raw) || !IsWindowVisible(child).as_bool() {
            return BOOL(1);
        }
        let mut class = [0u8; 64];
        let len = GetClassNameA(child, &mut class);
        if len > 0 {
            let cls = &class[..len as usize];
            if matches!(
                cls,
                b"SHELLDLL_DefView" | b"FolderView" | b"SysListView32" | b"WorkerW" | b"Progman"
            ) {
                return BOOL(1);
            }
        }
        let mut rect = RECT::default();
        if DwmGetWindowAttribute(
            child,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut _ as *mut c_void,
            std::mem::size_of::<RECT>() as u32,
        )
        .is_err()
            && GetWindowRect(child, &mut rect).is_err()
        {
            return BOOL(1);
        }
        if rect.right <= rect.left || rect.bottom <= rect.top {
            return BOOL(1);
        }
        let intersects = rect.left < search.region.right
            && rect.right > search.region.left
            && rect.top < search.region.bottom
            && rect.bottom > search.region.top;
        if intersects {
            search.windows.push(BackdropWinCandidate { hwnd: child, rect });
        }
        BOOL(1)
    }

    unsafe {
        let _ = EnumChildWindows(
            Some(desktop_hwnd),
            Some(child_proc),
            LPARAM(search as *mut BackdropWinSearch<'_> as isize),
        );
    }
}

fn collect_intersecting_windows(region: RECT, exclude_hwnds: &[isize]) -> Vec<BackdropWinCandidate> {
    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let search = &mut *(lparam.0 as *mut BackdropWinSearch<'_>);
        let raw = hwnd.0 as isize;
        if search.exclude_hwnds.contains(&raw) {
            return BOOL(1);
        }
        if is_desktop_shell_hwnd(hwnd) {
            if IsWindowVisible(hwnd).as_bool() {
                collect_live_wallpaper_children(hwnd, search);
            }
            return BOOL(1);
        }
        if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
            return BOOL(1);
        }
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if (ex_style & WS_EX_TOOLWINDOW.0) != 0 {
            return BOOL(1);
        }
        let mut cloaked = 0u32;
        if DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut c_void,
            std::mem::size_of::<u32>() as u32,
        )
        .is_ok()
            && cloaked != 0
        {
            return BOOL(1);
        }
        let mut rect = RECT::default();
        if DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut _ as *mut c_void,
            std::mem::size_of::<RECT>() as u32,
        )
        .is_err()
            && GetWindowRect(hwnd, &mut rect).is_err()
        {
            return BOOL(1);
        }
        if rect.right <= rect.left || rect.bottom <= rect.top {
            return BOOL(1);
        }
        let intersects = rect.left < search.region.right
            && rect.right > search.region.left
            && rect.top < search.region.bottom
            && rect.bottom > search.region.top;
        if intersects {
            search.windows.push(BackdropWinCandidate { hwnd, rect });
        }
        BOOL(1)
    }

    let mut search = BackdropWinSearch {
        region,
        exclude_hwnds,
        windows: Vec::new(),
    };
    unsafe {
        let _ = EnumWindows(
            Some(enum_proc),
            LPARAM(&mut search as *mut BackdropWinSearch<'_> as isize),
        );
    }
    // EnumWindows yields top-to-bottom Z-order; reverse to draw bottom-to-top.
    if search.windows.len() > 6 {
        search.windows.truncate(6);
    }
    search.windows.reverse();
    search.windows
}

/// Computes an FNV-1a signature of the visible non-Hime windows intersecting `bounds`
/// (in Z-order, with extended frame rects) so settled surfaces avoid redundant `PrintWindow`
/// composites until windows behind them actually open, close, move, or reorder.
pub fn visible_window_layout_hash(bounds: (i32, i32, u32, u32), exclude_hwnds: &[isize]) -> u64 {
    let region = RECT {
        left: bounds.0,
        top: bounds.1,
        right: bounds.0.saturating_add(bounds.2 as i32),
        bottom: bounds.1.saturating_add(bounds.3 as i32),
    };
    let wins = collect_intersecting_windows(region, exclude_hwnds);
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |v: u64| {
        h ^= v;
        h = h.wrapping_mul(0x1000_0000_01b3);
    };
    mix(bounds.0 as i64 as u64);
    mix(bounds.1 as i64 as u64);
    mix(bounds.2 as u64);
    mix(bounds.3 as u64);
    mix(wins.len() as u64);
    for w in wins {
        mix(w.hwnd.0 as isize as u64);
        mix(w.rect.left as i64 as u64);
        mix(w.rect.top as i64 as u64);
        mix(w.rect.right as i64 as u64);
        mix(w.rect.bottom as i64 as u64);
    }
    h
}

#[allow(clippy::too_many_arguments)]
fn composite_windows_below(
    scratch: &mut GdiBackdropScratch,
    sx: i32,
    sy: i32,
    sw: i32,
    sh: i32,
    cap_w: u32,
    cap_h: u32,
    exclude_hwnds: &[isize],
    screen: ScreenBounds,
    wallpaper: (u32, u32, &[u8]),
) -> Option<Vec<u8>> {
    #[link(name = "user32")]
    extern "system" {
        fn PrintWindow(hwnd: HWND, hdcblt: windows::Win32::Graphics::Gdi::HDC, nflags: u32) -> BOOL;
    }
    const PW_RENDERFULLCONTENT: u32 = 0x0000_0002;

    let (wp_w, wp_h, wp_bytes) = wallpaper;
    let region = RECT {
        left: sx,
        top: sy,
        right: sx + sw.max(1),
        bottom: sy + sh.max(1),
    };
    let windows_below = collect_intersecting_windows(region, exclude_hwnds);

    unsafe {
        let (cap_dc, cap_bits, byte_len) = scratch.ensure_cap_surface(cap_w, cap_h)?;

        // 1. Seed cap_dc with the matching slice of cached wallpaper (or desktop background color).
        if wp_w > 0
            && wp_h > 0
            && wp_bytes.len() == (wp_w as usize * wp_h as usize * 4)
        {
            sample_wallpaper_slice(
                wp_bytes,
                wp_w,
                wp_h,
                screen,
                region,
                cap_w,
                cap_h,
                std::slice::from_raw_parts_mut(cap_bits, byte_len),
            );
        } else {
            let brush = CreateSolidBrush(windows::Win32::Foundation::COLORREF(
                GetSysColor(COLOR_BACKGROUND),
            ));
            let fill_r = RECT {
                left: 0,
                top: 0,
                right: cap_w as i32,
                bottom: cap_h as i32,
            };
            let _ = FillRect(cap_dc, &fill_r, brush);
            let _ = DeleteObject(brush.into());
        }

        // 2. Composite visible non-Hime windows below Hime Z-order onto cap_dc reusing cached win_dib.
        if !windows_below.is_empty() {
            for cand in windows_below {
                let win_w = (cand.rect.right - cand.rect.left).clamp(1, screen.width.max(1));
                let win_h = (cand.rect.bottom - cand.rect.top).clamp(1, screen.height.max(1));
                let Some(win_dc) = scratch.ensure_win_surface(win_w, win_h) else {
                    continue;
                };
                let ok = PrintWindow(cand.hwnd, win_dc, PW_RENDERFULLCONTENT).as_bool()
                    || PrintWindow(cand.hwnd, win_dc, 0).as_bool();
                if ok {
                    let ix0 = cand.rect.left.max(region.left);
                    let iy0 = cand.rect.top.max(region.top);
                    let ix1 = cand.rect.right.min(region.right);
                    let iy1 = cand.rect.bottom.min(region.bottom);
                    let iw = (ix1 - ix0).max(0);
                    let ih = (iy1 - iy0).max(0);
                    if iw > 0 && ih > 0 {
                        let src_x = ix0 - cand.rect.left;
                        let src_y = iy0 - cand.rect.top;
                        let dst_x = (((ix0 - region.left) as f32 / sw.max(1) as f32)
                            * cap_w as f32)
                            .floor() as i32;
                        let dst_y = (((iy0 - region.top) as f32 / sh.max(1) as f32)
                            * cap_h as f32)
                            .floor() as i32;
                        let dst_w =
                            (((iw as f32 / sw.max(1) as f32) * cap_w as f32).ceil() as i32)
                                .max(1);
                        let dst_h =
                            (((ih as f32 / sh.max(1) as f32) * cap_h as f32).ceil() as i32)
                                .max(1);
                        let _ = StretchBlt(
                            cap_dc,
                            dst_x,
                            dst_y,
                            dst_w,
                            dst_h,
                            Some(win_dc),
                            src_x,
                            src_y,
                            iw,
                            ih,
                            SRCCOPY,
                        );
                    }
                }
            }
        }

        let mut buf = std::slice::from_raw_parts(cap_bits, byte_len).to_vec();
        force_opaque_alpha(&mut buf);
        Some(buf)
    }
}

#[allow(clippy::too_many_arguments)]
fn sample_wallpaper_slice(
    wp_bytes: &[u8],
    wp_w: u32,
    wp_h: u32,
    screen: ScreenBounds,
    region: RECT,
    dst_w: u32,
    dst_h: u32,
    dst: &mut [u8],
) {
    let sw = screen.width.max(1) as f32;
    let sh = screen.height.max(1) as f32;
    let reg_w = (region.right - region.left).max(1) as f32;
    let reg_h = (region.bottom - region.top).max(1) as f32;

    for dy in 0..dst_h {
        let world_y = region.top as f32 + (dy as f32 + 0.5) / dst_h as f32 * reg_h;
        let wy = (((world_y - screen.y as f32) / sh).clamp(0.0, 0.9999) * wp_h as f32) as usize;
        let src_row = wy * wp_w as usize * 4;
        let dst_row = dy as usize * dst_w as usize * 4;
        for dx in 0..dst_w {
            let world_x = region.left as f32 + (dx as f32 + 0.5) / dst_w as f32 * reg_w;
            let wx =
                (((world_x - screen.x as f32) / sw).clamp(0.0, 0.9999) * wp_w as f32) as usize;
            let s_idx = src_row + wx * 4;
            let d_idx = dst_row + dx as usize * 4;
            dst[d_idx..d_idx + 4].copy_from_slice(&wp_bytes[s_idx..s_idx + 4]);
        }
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::DockItem;

    #[test]
    fn snap_notch_edge_selects_top_left_right_and_never_bottom() {
        let screen = ScreenBounds::new(0, 0, 1920, 1080);

        // Top center -> Top
        assert_eq!(snap_notch_edge(960.0, 24.0, screen), NotchEdge::Top);
        // Left middle -> Left
        assert_eq!(snap_notch_edge(16.0, 540.0, screen), NotchEdge::Left);
        // Right middle -> Right
        assert_eq!(snap_notch_edge(1904.0, 540.0, screen), NotchEdge::Right);
        // Top-left corner tie -> Top
        assert_eq!(snap_notch_edge(0.0, 0.0, screen), NotchEdge::Top);
        // Bottom-right corner -> Right (never Bottom)
        assert_eq!(snap_notch_edge(1910.0, 1075.0, screen), NotchEdge::Right);
        // Bottom-left corner -> Left (never Bottom)
        assert_eq!(snap_notch_edge(10.0, 1075.0, screen), NotchEdge::Left);
    }

    #[test]
    fn snap_notch_edge_supports_secondary_monitor_offset() {
        // Secondary monitor positioned to the left of primary: [-1920, 0, 1920, 1080]
        let secondary = ScreenBounds::new(-1920, 0, 1920, 1080);

        assert_eq!(snap_notch_edge(-960.0, 15.0, secondary), NotchEdge::Top);
        assert_eq!(snap_notch_edge(-1905.0, 500.0, secondary), NotchEdge::Left);
        assert_eq!(snap_notch_edge(-10.0, 500.0, secondary), NotchEdge::Right);
    }

    #[test]
    fn edge_hover_wakes_hidden_notch_only_near_anchored_edge() {
        let screen = ScreenBounds::new(0, 0, 1920, 1080);
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_hidden(true);
        for _ in 0..120 {
            notch.tick(1.0 / 60.0);
        }

        // Hovering strictly inside the 48px minimized top-center bar wakes it.
        assert!(is_notch_edge_hovered(960.0, 3.0, &notch, screen, 6.0));
        // Hovering outside the 48px minimized span on the top edge MUST NOT leak/wake!
        assert!(!is_notch_edge_hovered(870.0, 3.0, &notch, screen, 6.0));
        assert!(!is_notch_edge_hovered(1050.0, 3.0, &notch, screen, 6.0));
        // Hovering far away from the notch or at bottom does not wake it.
        assert!(!is_notch_edge_hovered(960.0, 40.0, &notch, screen, 6.0));
        assert!(!is_notch_edge_hovered(960.0, 1078.0, &notch, screen, 6.0));

        // Immediately after waking (mid-animation before width_spring reaches 276), stable
        // target bounds + hysteresis keep cursor inside without open/close flicker.
        notch.set_hidden(false);
        notch.tick(1.0 / 60.0);
        assert!(is_notch_edge_hovered(960.0, 5.0, &notch, screen, 6.0));
        assert!(is_notch_edge_hovered(960.0, 34.0, &notch, screen, 6.0));

        // Full-edge wake helper checks edge threshold anywhere along the edge.
        assert!(is_cursor_at_notch_wake_edge(100.0, 4.0, NotchEdge::Top, screen, 6.0));
        assert!(is_cursor_at_notch_wake_edge(4.0, 500.0, NotchEdge::Left, screen, 6.0));
        assert!(is_cursor_at_notch_wake_edge(1916.0, 500.0, NotchEdge::Right, screen, 6.0));
        assert!(!is_cursor_at_notch_wake_edge(960.0, 1078.0, NotchEdge::Top, screen, 6.0));
    }

    #[test]
    fn hit_test_and_click_through_respect_notch_and_dock() {
        let screen = ScreenBounds::new(0, 0, 1920, 1080);
        let notch = NotchView::new(NotchEdge::Top);
        let mut dock = DockBar::new();
        dock.add_item(DockItem::new("files", "Files", "explorer.exe", true));

        // Over top notch -> hit_test true, click_through false
        assert!(hit_test_notch(960.0, 15.0, &notch, 1920.0, 1080.0));
        assert!(!should_enable_click_through(
            960.0,
            15.0,
            &notch,
            Some(&dock),
            screen,
            DEFAULT_EDGE_WAKE_THRESHOLD_PX,
        ));

        // Over bottom dock -> click_through false
        let dock_rect = dock.dock_rect(Rect::new(0.0, 0.0, 1920.0, 1080.0));
        assert!(!should_enable_click_through(
            dock_rect.x + dock_rect.width * 0.5,
            dock_rect.y + dock_rect.height * 0.5,
            &notch,
            Some(&dock),
            screen,
            DEFAULT_EDGE_WAKE_THRESHOLD_PX,
        ));

        // Empty desktop center -> click_through true
        assert!(should_enable_click_through(
            960.0,
            540.0,
            &notch,
            Some(&dock),
            screen,
            DEFAULT_EDGE_WAKE_THRESHOLD_PX,
        ));
    }

    #[test]
    fn taskbar_marker_roundtrip_preserves_state_and_rects() {
        let primary = RECT {
            left: 0,
            top: 1032,
            right: 1920,
            bottom: 1080,
        };
        let mut secs = HashMap::new();
        secs.insert(
            12345_isize,
            RECT {
                left: -1920,
                top: 1032,
                right: 0,
                bottom: 1080,
            },
        );

        let text = format_marker_contents(2, 9999, Some(primary), &secs);
        assert_eq!(parse_marker_state(&text), Some(2));
        let parsed_primary = parse_marker_primary(&text).unwrap();
        assert_eq!(parsed_primary.top, 1032);
        assert_eq!(parsed_primary.right, 1920);
        let parsed_secs = parse_marker_secondaries(&text);
        assert_eq!(parsed_secs.get(&12345_isize).unwrap().left, -1920);
    }

    #[test]
    fn transient_system_flyout_class_excludes_user_apps() {
        assert!(is_transient_system_flyout_class("Shell_TrayWnd"));
        assert!(is_transient_system_flyout_class("ControlCenterWindow"));
        assert!(is_transient_system_flyout_class("XamlExplorerHostIslandWindow"));
        assert!(is_transient_system_flyout_class("Windows.UI.Core.CoreWindow"));
        assert!(!is_transient_system_flyout_class("CabinetWClass"));
        assert!(!is_transient_system_flyout_class("Chrome_WidgetWin_1"));
        assert!(!is_transient_system_flyout_class("ApplicationFrameWindow"));
    }

    #[test]
    fn dock_and_overlay_edge_hover_wake_when_hidden() {
        use crate::dock::DockMode;

        let screen = ScreenBounds::new(0, 0, 1920, 1080);
        let mut dock = DockBar::new();
        dock.add_item(DockItem::new("files", "Files", "explorer.exe", true));
        dock.set_mode(DockMode::AutoHide);
        for _ in 0..90 {
            dock.tick(1.0 / 60.0);
        }
        assert!(dock.is_hidden);

        // Cursor at bottom center screen edge wakes hidden Dock
        assert!(is_dock_edge_hovered(960.0, 1078.0, &dock, screen, 8.0));
        // Cursor far above bottom edge or far side does not wake Dock
        assert!(!is_dock_edge_hovered(960.0, 900.0, &dock, screen, 8.0));
        assert!(!is_dock_edge_hovered(10.0, 1078.0, &dock, screen, 8.0));

        // Left & right overlay edge bands wake only near mid-height edges
        assert!(is_overlay_edge_hovered(2.0, 540.0, true, screen, 8.0, 240.0));
        assert!(!is_overlay_edge_hovered(2.0, 100.0, true, screen, 8.0, 240.0));
        assert!(is_overlay_edge_hovered(1918.0, 540.0, false, screen, 8.0, 240.0));
        assert!(!is_overlay_edge_hovered(1800.0, 540.0, false, screen, 8.0, 240.0));
    }

    #[test]
    fn virtual_desktop_registry_bytes_parse_guids_names_and_active() {
        // Real little-endian Windows GUID bytes: {DFD37548-7649-470E-99F3-0411EC22CABE}
        let g1: [u8; 16] = [
            0x48, 0x75, 0xD3, 0xDF, 0x49, 0x76, 0x0E, 0x47, 0x99, 0xF3, 0x04, 0x11, 0xEC, 0x22,
            0xCA, 0xBE,
        ];
        let g2: [u8; 16] = [
            0xBA, 0xC0, 0x73, 0x71, 0x12, 0x72, 0xD7, 0x4F, 0x85, 0x74, 0x62, 0x56, 0x3C, 0x3B,
            0x4B, 0xEE,
        ];
        assert_eq!(
            format_windows_guid_upper(&g1).as_deref(),
            Some("{DFD37548-7649-470E-99F3-0411EC22CABE}")
        );

        let mut both = Vec::new();
        both.extend_from_slice(&g1);
        both.extend_from_slice(&g2);

        let list = parse_virtual_desktops_from_bytes(Some(&both), Some(&g2), |guid| {
            if guid == "{7173C0BA-7212-4FD7-8574-62563C3B4BEE}" {
                Some("Work".to_string())
            } else {
                None
            }
        });
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "{dfd37548-7649-470e-99f3-0411ec22cabe}");
        assert_eq!(list[0].name, "Desktop 1");
        assert!(!list[0].active);
        assert_eq!(list[1].id, "{7173c0ba-7212-4fd7-8574-62563c3b4bee}");
        assert_eq!(list[1].name, "Work");
        assert!(list[1].active);

        // Missing registry blobs fall back safely to single Desktop 1
        let fallback = parse_virtual_desktops_from_bytes(None, None, |_| None);
        assert_eq!(fallback.len(), 1);
        assert_eq!(fallback[0].name, "Desktop 1");
        assert!(fallback[0].active);
    }

    #[test]
    fn backdrop_capturer_samples_wallpaper_slice_with_opaque_alpha() {
        let mut wp = vec![0u8; 4 * 4 * 4];
        // Bottom-right pixel (x=3, y=3) painted bright cyan, alpha=0 initially
        let idx = (3 * 4 + 3) * 4;
        wp[idx] = 240;
        wp[idx + 1] = 180;
        wp[idx + 2] = 20;
        let mut dst = vec![0u8; 2 * 2 * 4];
        sample_wallpaper_slice(
            &wp,
            4,
            4,
            ScreenBounds::new(0, 0, 100, 100),
            RECT {
                left: 75,
                top: 75,
                right: 100,
                bottom: 100,
            },
            2,
            2,
            &mut dst,
        );
        force_opaque_alpha(&mut dst);
        assert_eq!(&dst[0..4], &[240, 180, 20, 255]);

        let mut capturer = BackdropCapturer::new();
        let cap = capturer
            .capture_region(0, 0, 200, 100, &[], false)
            .expect("capture_region returns buffer");
        assert_eq!(cap.width, 100);
        assert_eq!(cap.height, 50);
        assert_eq!(cap.bgra.len(), 100 * 50 * 4);
        assert!(cap.bgra.chunks_exact(4).all(|px| px[3] == 255));

        let h1 = visible_window_layout_hash((0, 0, 200, 100), &[]);
        let h2 = visible_window_layout_hash((0, 0, 200, 100), &[]);
        let h3 = visible_window_layout_hash((10, 0, 200, 100), &[]);
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
}
