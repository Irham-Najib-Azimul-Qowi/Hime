//! OS Shell Integration & Mouse Interaction for Hime Island Native (ADR-017 Phase 4).
//!
//! Provides:
//! - Safe Windows taskbar (`Shell_TrayWnd` & `Shell_SecondaryTrayWnd`) hide/restore with
//!   crash-recovery marker at `%APPDATA%\ai.hime.desktop\native_taskbar_hidden.flag`.
//! - 3-edge (`Top`, `Left`, `Right`) drag snap calculation, screen bounds, and edge hover
//!   wake / hit-testing helpers.
//! - Win32 `WM_DROPFILES` (`DragAcceptFiles` + `HDROP`) extraction into [`FileShelf`].

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

use windows::core::{BOOL, PCSTR};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::Shell::{
    DragAcceptFiles, DragFinish, DragQueryFileW, SHAppBarMessage, ABM_GETSTATE, ABM_GETTASKBARPOS,
    ABM_SETSTATE, APPBARDATA, HDROP,
};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowA, GetClassNameA, GetCursorPos, GetSystemMetrics, GetWindowLongA,
    GetWindowLongW, GetWindowRect, IsWindowVisible, SetForegroundWindow,
    SetLayeredWindowAttributes, SetWindowLongA, SetWindowLongW, SetWindowPos, ShowWindow,
    GWL_EXSTYLE, HWND_TOPMOST, LWA_ALPHA, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SWP_HIDEWINDOW, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TRANSPARENT,
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

/// Re-hides `Shell_TrayWnd` if Windows Explorer made it visible again while Hime Dock is active.
pub fn ensure_taskbar_hidden() -> bool {
    unsafe {
        let tray_class = PCSTR(c"Shell_TrayWnd".as_ptr() as *const u8);
        let Ok(tray_hwnd) = FindWindowA(tray_class, PCSTR::null()) else {
            return false;
        };
        if IsWindowVisible(tray_hwnd).as_bool() {
            return hide_taskbar();
        }
        for sec_hwnd in secondary_taskbar_windows() {
            if IsWindowVisible(sec_hwnd).as_bool() {
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
    let rx = cursor_x - screen.x as f32;
    let ry = cursor_y - screen.y as f32;
    let sw = screen.width.max(1) as f32;
    let sh = screen.height.max(1) as f32;
    let rect = notch.computed_rect(sw, sh);

    if notch.is_hidden {
        let margin = wake_margin_px.max(1.0);
        let wake_rect = match notch.edge {
            NotchEdge::Top => Rect::new(rect.x, 0.0, rect.width, rect.height + margin),
            NotchEdge::Left => Rect::new(0.0, rect.y, rect.width + margin, rect.height),
            NotchEdge::Right => Rect::new(sw - rect.width - margin, rect.y, rect.width + margin, rect.height),
        };
        is_point_in_rect(rx, ry, wake_rect)
    } else {
        is_point_in_rect(rx, ry, rect)
    }
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

/// Enables or disables Win32 `WM_DROPFILES` acceptance on `hwnd`.
pub fn enable_drag_drop(hwnd: HWND, accept: bool) {
    unsafe {
        DragAcceptFiles(hwnd, accept);
    }
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

        // Hidden top notch is centered horizontally; hovering top edge inside its span wakes it.
        assert!(is_notch_edge_hovered(960.0, 3.0, &notch, screen, 6.0));
        // Hovering far away from the notch or at bottom does not wake it.
        assert!(!is_notch_edge_hovered(960.0, 40.0, &notch, screen, 6.0));
        assert!(!is_notch_edge_hovered(960.0, 1078.0, &notch, screen, 6.0));

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
}
