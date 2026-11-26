//! Hime Island Native Desktop Shell (`hime-island-native.exe`).
//! Pure-Rust Direct2D + DirectWrite + DirectComposition shell with live Win32 telemetry,
//! safe taskbar integration, 3-edge Dynamic Island, Dock, Dual Edge Overlays, Settings, and Hime AI IPC.

#![windows_subsystem = "windows"]

use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use windows::core::{w, GUID, PCWSTR, PWSTR};
use windows::Devices::Radios::{Radio, RadioKind, RadioState};
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
use windows::Win32::Foundation::{
    CloseHandle, HANDLE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM,
};
use windows::Win32::NetworkManagement::WiFi::{
    dot11_BSS_type_infrastructure, wlan_connection_mode_profile, wlan_interface_state_connected,
    WlanCloseHandle, WlanConnect, WlanEnumInterfaces, WlanFreeMemory, WlanOpenHandle,
    WLAN_CONNECTION_PARAMETERS, WLAN_INTERFACE_INFO, WLAN_INTERFACE_INFO_LIST,
};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{
    CreateEventW, OpenProcess, QueryFullProcessImageNameW, WaitForSingleObject,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT,
    VK_SHIFT,
};
use windows::Win32::UI::Shell::{
    ShellExecuteW, Shell_NotifyIconGetRect, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP,
    NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, NOTIFYICONIDENTIFIER,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, BringWindowToTop, CallWindowProcW, CreatePopupMenu, CreateWindowExW,
    DefWindowProcW, DestroyMenu, DispatchMessageW, EnumWindows, FindWindowExW, FindWindowW,
    GetClassNameA, GetCursorPos, GetForegroundWindow, GetMessageW, GetWindowLongA, GetWindowRect,
    GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, KillTimer,
    LoadIconW, PostMessageW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
    SetLayeredWindowAttributes, SetTimer, SetWindowLongA, SetWindowLongPtrW, ShowWindow,
    TrackPopupMenu, TranslateMessage, GWL_EXSTYLE, GWLP_WNDPROC, HWND_MESSAGE, IDI_APPLICATION,
    LWA_ALPHA, MF_STRING, MSG, SW_HIDE, SW_RESTORE, SW_SHOW, SW_SHOWNOACTIVATE, SW_SHOWNORMAL,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD, WINDOW_EX_STYLE, WM_APP, WM_CHAR, WM_DESTROY,
    WM_HOTKEY, WM_KEYDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONUP, WM_TIMER,
    WNDCLASSW, WNDPROC, WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_OVERLAPPED,
};

use hime_island_native::ai_drawer::{
    AiDrawer, ApprovalCard, ApprovalStatus, DrawerAction, DrawerAttachment,
};
use hime_island_native::d2d::{
    self, HimeExpression, NativeSurface, Rect, SurfaceEvent, SurfaceEventKind, WM_SURFACE_EVENT,
};
use hime_island_native::dock::{DockAction, DockBar, DockItem, DockMode, TrayEntry};
use hime_island_native::hotkey::{self, HotkeyAction, HotkeyManager};
use hime_island_native::ipc_client::{
    AiProfileUpsertParams, ApprovalResolution, ChatTurnParams, ChatTurnResult, HimeIpcClient,
    IpcError, IPC_CONTRACT_VERSION,
};
use hime_island_native::module_mgr::{HimeAiModuleManager, RouterProbeResult};
use hime_island_native::notch::{
    NotchAction, NotchEdge, NotchPanel, NotchView, ShelfSummaryItem, WifiNetworkItem,
};
use hime_island_native::os_shell;
use hime_island_native::overlay::{sanitize_device_name, DualEdgeOverlays};
use hime_island_native::settings_store::{NativeSettings, SettingsStore};
use hime_island_native::settings_ui::{
    ApiProfileConfig, NotchMode, ProviderPreset, SettingsAction, SettingsControl, SettingsPanel,
    SETTINGS_WINDOW_HEIGHT, SETTINGS_WINDOW_WIDTH,
};
use hime_island_native::shelf::{FileShelf, ShelfItemKind};
use hime_island_native::telemetry::{
    self, NetworkKind, RunningWindow, TelemetryHandle, TelemetrySnapshot, WifiNetworkInfo,
};

const CONTROLLER_CLASS: PCWSTR = w!("HimeIslandNativeController");
const TRAY_ICON_ID: u32 = 1;
const WM_TRAY_CALLBACK: u32 = WM_APP + 1;
const WM_EXTRA_SURFACE_EVENT: u32 = WM_APP + 3;
const WM_MOUSELEAVE: u32 = 0x02A3;
const TIMER_ANIM_ID: usize = 1;
const ANIM_INTERVAL_ACTIVE_MS: u32 = 16;
const ANIM_INTERVAL_IDLE_MS: u32 = 250;

const OVERLAY_DIP_WIDTH: f32 = 80.0;
const OVERLAY_DIP_HEIGHT: f32 = 240.0;
const DOCK_DIP_WIDTH: f32 = 960.0;
const DOCK_DIP_HEIGHT: f32 = 124.0;

const MENU_TOGGLE_NOTCH: usize = 1001;
const MENU_TOGGLE_DOCK: usize = 1002;
const MENU_TOGGLE_AI_DRAWER: usize = 1003;
const MENU_TOGGLE_SETTINGS: usize = 1004;
const MENU_EXIT: usize = 1005;

static ORIG_SURFACE_WNDPROC: AtomicIsize = AtomicIsize::new(0);
static EXTRA_EVENT_TARGET: AtomicIsize = AtomicIsize::new(0);

#[derive(Debug, Clone, Copy)]
enum ExtraSurfaceEventKind {
    RightButtonUp { x: f32, y: f32 },
    MouseWheel { delta_steps: f32 },
    MouseLeave,
}

#[derive(Debug, Clone, Copy)]
struct ExtraSurfaceEvent {
    hwnd: isize,
    kind: ExtraSurfaceEventKind,
}

thread_local! {
    static EXTRA_SURFACE_EVENTS: RefCell<Vec<ExtraSurfaceEvent>> = const { RefCell::new(Vec::new()) };
    static TRACKED_MOUSE_HWNDS: RefCell<HashSet<isize>> = RefCell::new(HashSet::new());
}

fn subclass_surface(hwnd: HWND) {
    unsafe {
        let prev = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, extra_surface_wnd_proc as *const () as isize);
        if prev != 0 {
            let _ = ORIG_SURFACE_WNDPROC.compare_exchange(
                0,
                prev,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
        }
    }
}

fn drain_extra_surface_events() -> Vec<ExtraSurfaceEvent> {
    EXTRA_SURFACE_EVENTS.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

unsafe extern "system" fn extra_surface_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let key = hwnd.0 as isize;
    match msg {
        WM_MOUSEMOVE => {
            let need_track = TRACKED_MOUSE_HWNDS.with(|set| set.borrow_mut().insert(key));
            if need_track {
                let mut tme = TRACKMOUSEEVENT {
                    cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                let _ = TrackMouseEvent(&mut tme);
            }
        }
        WM_MOUSELEAVE => {
            TRACKED_MOUSE_HWNDS.with(|set| {
                set.borrow_mut().remove(&key);
            });
            push_extra_event(key, ExtraSurfaceEventKind::MouseLeave);
            return LRESULT(0);
        }
        WM_RBUTTONUP => {
            let x = (lparam.0 & 0xFFFF) as i16 as f32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;
            push_extra_event(key, ExtraSurfaceEventKind::RightButtonUp { x, y });
            return LRESULT(0);
        }
        WM_MOUSEWHEEL => {
            let raw_delta = ((wparam.0 >> 16) & 0xFFFF) as i16;
            let delta_steps = raw_delta as f32 / 120.0;
            push_extra_event(key, ExtraSurfaceEventKind::MouseWheel { delta_steps });
            return LRESULT(0);
        }
        _ => {}
    }

    let orig_ptr = ORIG_SURFACE_WNDPROC.load(Ordering::Relaxed);
    if orig_ptr != 0 {
        let orig: WNDPROC = std::mem::transmute(orig_ptr);
        CallWindowProcW(orig, hwnd, msg, wparam, lparam)
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

fn push_extra_event(hwnd: isize, kind: ExtraSurfaceEventKind) {
    EXTRA_SURFACE_EVENTS.with(|cell| {
        cell.borrow_mut().push(ExtraSurfaceEvent { hwnd, kind });
    });
    let target = EXTRA_EVENT_TARGET.load(Ordering::Relaxed);
    if target != 0 {
        unsafe {
            let _ = PostMessageW(
                Some(HWND(target as *mut std::ffi::c_void)),
                WM_EXTRA_SURFACE_EVENT,
                WPARAM(0),
                LPARAM(0),
            );
        }
    }
}

enum IpcBgResult {
    ChatTurn(Result<ChatTurnResult, IpcError>),
    ProfileTest(Result<String, IpcError>),
    ApprovalResolved(Result<(), IpcError>),
    RouterProbe(RouterProbeResult),
    WifiScan(Vec<WifiNetworkInfo>),
    WifiConnect(String, Result<(), String>),
    TrayEntries(Vec<TrayEntry>),
}

enum OsCommand {
    SetVolume { percent: u8, unmute: bool },
    ToggleMute,
    SetBrightness(u8),
    MediaPlayPause,
    MediaNext,
    MediaPrev,
    MediaSeek(u64),
    SetWifiRadio(bool),
    SetBluetoothRadio(bool),
    ScanWifi,
    ConnectWifi(String),
    Probe9Router,
    ScanTrayEntries,
}

struct OsWorker {
    tx: mpsc::Sender<OsCommand>,
}

impl OsWorker {
    fn spawn(result_tx: mpsc::Sender<IpcBgResult>) -> Self {
        let (tx, rx) = mpsc::channel::<OsCommand>();
        let _ = std::thread::Builder::new()
            .name("hime-os-worker".to_string())
            .spawn(move || run_os_worker(rx, result_tx));
        Self { tx }
    }

    fn send(&self, cmd: OsCommand) {
        let _ = self.tx.send(cmd);
    }
}

fn run_os_worker(rx: mpsc::Receiver<OsCommand>, result_tx: mpsc::Sender<IpcBgResult>) {
    let com_ok = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
    let mut smtc_mgr: Option<GlobalSystemMediaTransportControlsSessionManager> =
        GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .and_then(|op| op.get())
            .ok();

    while let Ok(first) = rx.recv() {
        let mut cmds = vec![first];
        while let Ok(more) = rx.try_recv() {
            cmds.push(more);
        }

        // Coalesce rapid Volume/Brightness drag updates to the latest value.
        let mut latest_volume: Option<(u8, bool)> = None;
        let mut latest_brightness: Option<u8> = None;
        let mut ordered = Vec::with_capacity(cmds.len());
        for cmd in cmds {
            match cmd {
                OsCommand::SetVolume { percent, unmute } => {
                    latest_volume = Some((percent, unmute));
                }
                OsCommand::SetBrightness(pct) => {
                    latest_brightness = Some(pct);
                }
                other => ordered.push(other),
            }
        }

        if let Some((pct, unmute)) = latest_volume {
            if unmute {
                let _ = telemetry::toggle_mute();
            }
            let _ = telemetry::set_master_volume_percent(pct);
        }
        if let Some(pct) = latest_brightness {
            let _ = telemetry::set_brightness(pct);
        }

        for cmd in ordered {
            match cmd {
                OsCommand::ToggleMute => {
                    let _ = telemetry::toggle_mute();
                }
                OsCommand::MediaPlayPause => {
                    if let Some(session) = get_smtc_session(&mut smtc_mgr) {
                        let _ = session.TryTogglePlayPauseAsync();
                    }
                }
                OsCommand::MediaNext => {
                    if let Some(session) = get_smtc_session(&mut smtc_mgr) {
                        let _ = session.TrySkipNextAsync();
                    }
                }
                OsCommand::MediaPrev => {
                    if let Some(session) = get_smtc_session(&mut smtc_mgr) {
                        let _ = session.TrySkipPreviousAsync();
                    }
                }
                OsCommand::MediaSeek(pos_ms) => {
                    if let Some(session) = get_smtc_session(&mut smtc_mgr) {
                        let ticks = (pos_ms as i64).saturating_mul(10_000);
                        let _ = session.TryChangePlaybackPositionAsync(ticks);
                    }
                }
                OsCommand::SetWifiRadio(enabled) => {
                    let _ = set_radio_state(RadioKind::WiFi, enabled);
                }
                OsCommand::SetBluetoothRadio(enabled) => {
                    let _ = set_radio_state(RadioKind::Bluetooth, enabled);
                }
                OsCommand::ScanWifi => {
                    let list = telemetry::scan_available_wifi_networks();
                    let _ = result_tx.send(IpcBgResult::WifiScan(list));
                }
                OsCommand::ConnectWifi(ssid) => {
                    let res = connect_wifi_ssid(&ssid);
                    let _ = result_tx.send(IpcBgResult::WifiConnect(ssid, res));
                }
                OsCommand::Probe9Router => {
                    let mgr = HimeAiModuleManager::with_default_dir();
                    let probe = mgr.probe_9router(Duration::from_millis(250));
                    let _ = result_tx.send(IpcBgResult::RouterProbe(probe));
                }
                OsCommand::ScanTrayEntries => {
                    let entries = collect_live_tray_entries();
                    let _ = result_tx.send(IpcBgResult::TrayEntries(entries));
                }
                OsCommand::SetVolume { .. } | OsCommand::SetBrightness(_) => {}
            }
        }
    }

    if com_ok {
        unsafe { CoUninitialize() };
    }
}

fn get_smtc_session(
    mgr_slot: &mut Option<GlobalSystemMediaTransportControlsSessionManager>,
) -> Option<windows::Media::Control::GlobalSystemMediaTransportControlsSession> {
    if mgr_slot.is_none() {
        *mgr_slot = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .and_then(|op| op.get())
            .ok();
    }
    match mgr_slot.as_ref().and_then(|m| m.GetCurrentSession().ok()) {
        Some(sess) => Some(sess),
        None => {
            *mgr_slot = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .and_then(|op| op.get())
                .ok();
            mgr_slot.as_ref().and_then(|m| m.GetCurrentSession().ok())
        }
    }
}

fn set_radio_state(kind: RadioKind, enabled: bool) -> Result<(), String> {
    let radios = Radio::GetRadiosAsync()
        .and_then(|op| op.get())
        .map_err(|e| e.to_string())?;
    for i in 0..radios.Size().unwrap_or(0) {
        if let Ok(radio) = radios.GetAt(i) {
            if radio.Kind().ok() == Some(kind) {
                let target = if enabled {
                    RadioState::On
                } else {
                    RadioState::Off
                };
                let _ = radio.SetStateAsync(target).and_then(|op| op.get());
                return Ok(());
            }
        }
    }
    Err("Radio device not found".to_string())
}

fn connect_wifi_ssid(ssid: &str) -> Result<(), String> {
    unsafe {
        let mut version = 0u32;
        let mut handle = HANDLE::default();
        let rc = WlanOpenHandle(2, None, &mut version, &mut handle);
        if rc != 0 {
            open_shell_uri("ms-availablenetworks:");
            return Err(format!("WlanOpenHandle failed ({rc})"));
        }

        let mut list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        let enum_rc = WlanEnumInterfaces(handle, None, &mut list);
        let mut target_guid = None;
        if enum_rc == 0 && !list.is_null() {
            if let Some(header) = list.as_ref() {
                let count = (header.dwNumberOfItems as usize).min(64);
                let entries: &[WLAN_INTERFACE_INFO] =
                    std::slice::from_raw_parts(header.InterfaceInfo.as_ptr(), count);
                target_guid = entries
                    .iter()
                    .find(|i| i.isState == wlan_interface_state_connected)
                    .or_else(|| entries.first())
                    .map(|i| i.InterfaceGuid);
            }
            WlanFreeMemory(list as *const _);
        }

        let Some(guid) = target_guid else {
            WlanCloseHandle(handle, None);
            open_shell_uri("ms-availablenetworks:");
            return Err("No Wi-Fi adapter available".to_string());
        };

        let wide_ssid: Vec<u16> = OsStr::new(ssid).encode_wide().chain(Some(0)).collect();
        let params = WLAN_CONNECTION_PARAMETERS {
            wlanConnectionMode: wlan_connection_mode_profile,
            strProfile: PCWSTR(wide_ssid.as_ptr()),
            pDot11Ssid: std::ptr::null_mut(),
            pDesiredBssidList: std::ptr::null_mut(),
            dot11BssType: dot11_BSS_type_infrastructure,
            dwFlags: 0,
        };
        let conn_rc = WlanConnect(handle, &guid, &params, None);
        WlanCloseHandle(handle, None);

        if conn_rc == 0 {
            Ok(())
        } else {
            open_shell_uri("ms-availablenetworks:");
            Err(format!("Profile unavailable or failed ({conn_rc})"))
        }
    }
}

fn open_shell_uri(uri: &str) {
    let wide: Vec<u16> = OsStr::new(uri).encode_wide().chain(Some(0)).collect();
    unsafe {
        let _ = ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(wide.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
    }
}

struct AppShell {
    notch: NotchView,
    dock: DockBar,
    overlays: DualEdgeOverlays,
    settings: SettingsPanel,
    native_settings: NativeSettings,
    ai_drawer: AiDrawer,
    shelf: FileShelf,
    telemetry: TelemetryHandle,
    module_mgr: HimeAiModuleManager,
    ipc: Arc<HimeIpcClient>,
    ipc_tx: mpsc::Sender<IpcBgResult>,
    ipc_rx: mpsc::Receiver<IpcBgResult>,
    os_worker: OsWorker,
    notch_surface: NativeSurface,
    dock_surface: NativeSurface,
    overlay_left_surface: NativeSurface,
    overlay_right_surface: NativeSurface,
    settings_surface: NativeSurface,
    drawer_surface: NativeSurface,
    last_tick: Instant,
    fast_timer_active: bool,
    last_snap_volume: Option<(u8, bool)>,
    last_snap_brightness: Option<u8>,
    last_volume_write_at: Option<Instant>,
    last_brightness_write_at: Option<Instant>,
    last_audio_device: Option<String>,
    last_running_windows: Vec<RunningWindow>,
    dragging_notch: bool,
    dragging_notch_slider: bool,
    drag_moved: bool,
    drag_start_cursor: (i32, i32),
    suppressed_notif_hwnds: Vec<isize>,
    quit_event: Option<HANDLE>,
    tick_count: u64,
}

impl AppShell {
    fn new(ctrl_hwnd: HWND) -> windows::core::Result<Self> {
        let screen = os_shell::primary_screen_bounds();
        let native_settings = SettingsStore::load().normalized();
        let initial_edge = parse_notch_edge(&native_settings.notch_edge);

        let (nx, ny, nw, nh) = notch_surface_bounds(initial_edge, screen, 1.0);
        let mut notch_surface = NativeSurface::new("HimeNotch", nx, ny, nw, nh)?;
        let notch_scale = (notch_surface.dpi() / 96.0).max(0.5);
        if (notch_scale - 1.0).abs() > 0.01 {
            let (sx, sy, sw, sh) = notch_surface_bounds(initial_edge, screen, notch_scale);
            let _ = notch_surface.set_bounds(sx, sy, sw, sh);
        }
        subclass_surface(notch_surface.hwnd());
        notch_surface.set_click_through(true);
        notch_surface.show();
        os_shell::enable_drag_drop(notch_surface.hwnd(), true);

        let (dx, dy, dock_w, dock_h) = dock_surface_bounds(screen, 1.0);
        let mut dock_surface = NativeSurface::new("HimeDock", dx, dy, dock_w, dock_h)?;
        let dock_scale = (dock_surface.dpi() / 96.0).max(0.5);
        if (dock_scale - 1.0).abs() > 0.01 {
            let (sx, sy, sw, sh) = dock_surface_bounds(screen, dock_scale);
            let _ = dock_surface.set_bounds(sx, sy, sw, sh);
        }
        subclass_surface(dock_surface.hwnd());

        let (olx, oly, olw, olh) = left_overlay_bounds(screen, 1.0);
        let mut overlay_left_surface = NativeSurface::new("HimeOverlayLeft", olx, oly, olw, olh)?;
        let ol_scale = (overlay_left_surface.dpi() / 96.0).max(0.5);
        if (ol_scale - 1.0).abs() > 0.01 {
            let (sx, sy, sw, sh) = left_overlay_bounds(screen, ol_scale);
            let _ = overlay_left_surface.set_bounds(sx, sy, sw, sh);
        }
        subclass_surface(overlay_left_surface.hwnd());

        let (orx, ory, orw, orh) = right_overlay_bounds(screen, 1.0);
        let mut overlay_right_surface = NativeSurface::new("HimeOverlayRight", orx, ory, orw, orh)?;
        let or_scale = (overlay_right_surface.dpi() / 96.0).max(0.5);
        if (or_scale - 1.0).abs() > 0.01 {
            let (sx, sy, sw, sh) = right_overlay_bounds(screen, or_scale);
            let _ = overlay_right_surface.set_bounds(sx, sy, sw, sh);
        }
        subclass_surface(overlay_right_surface.hwnd());

        let mut settings_surface = NativeSurface::new(
            "HimeSettings",
            screen.x + (screen.width - SETTINGS_WINDOW_WIDTH as i32) / 2,
            screen.y + (screen.height - SETTINGS_WINDOW_HEIGHT as i32) / 2,
            SETTINGS_WINDOW_WIDTH,
            SETTINGS_WINDOW_HEIGHT,
        )?;
        let set_scale = (settings_surface.dpi() / 96.0).max(0.5);
        if (set_scale - 1.0).abs() > 0.01 {
            let sw = (SETTINGS_WINDOW_WIDTH as f32 * set_scale).round() as u32;
            let sh = (SETTINGS_WINDOW_HEIGHT as f32 * set_scale).round() as u32;
            let _ = settings_surface.set_bounds(
                screen.x + (screen.width - sw as i32) / 2,
                screen.y + (screen.height - sh as i32) / 2,
                sw,
                sh,
            );
        }
        subclass_surface(settings_surface.hwnd());

        let mut drawer_surface = NativeSurface::new(
            "HimeAiDrawer",
            screen.x + (screen.width - 440) / 2,
            screen.y + 84,
            440,
            500,
        )?;
        let drw_scale = (drawer_surface.dpi() / 96.0).max(0.5);
        if (drw_scale - 1.0).abs() > 0.01 {
            let dw = (440.0 * drw_scale).round() as u32;
            let dh = (500.0 * drw_scale).round() as u32;
            let _ = drawer_surface.set_bounds(
                screen.x + (screen.width - dw as i32) / 2,
                screen.y + (84.0 * drw_scale).round() as i32,
                dw,
                dh,
            );
        }
        subclass_surface(drawer_surface.hwnd());
        os_shell::enable_drag_drop(drawer_surface.hwnd(), true);

        d2d::set_surface_event_target(ctrl_hwnd);
        EXTRA_EVENT_TARGET.store(ctrl_hwnd.0 as isize, Ordering::Relaxed);

        let shelf = FileShelf::load().unwrap_or_else(|_| FileShelf::new());
        let mut module_mgr = HimeAiModuleManager::with_default_dir();
        let router_probe = module_mgr.probe_9router(Duration::from_millis(120));

        let mut settings = SettingsPanel::new();
        settings.state.router_9_detected = router_probe.available;
        apply_native_settings_to_panel(&native_settings, &mut settings);

        if native_settings.hime_ai_enabled {
            let _ = module_mgr.spawn();
        }

        let telemetry = TelemetryHandle::start();
        let (ipc_tx, ipc_rx) = mpsc::channel();
        let os_worker = OsWorker::spawn(ipc_tx.clone());
        let ipc = Arc::new(HimeIpcClient::with_default_pipe());

        let quit_event = unsafe { CreateEventW(None, true, false, w!("HimeQuit")) }.ok();

        let mut notch = NotchView::new(initial_edge);
        notch.ai_status_text = if router_probe.available {
            "9Router Connected".to_string()
        } else {
            "Ready (Local)".to_string()
        };

        let dock = DockBar::new();
        let overlays = DualEdgeOverlays::new();

        let mut shell = Self {
            notch,
            dock,
            overlays,
            settings,
            native_settings: native_settings.clone(),
            ai_drawer: AiDrawer::new(),
            shelf,
            telemetry,
            module_mgr,
            ipc,
            ipc_tx,
            ipc_rx,
            os_worker,
            notch_surface,
            dock_surface,
            overlay_left_surface,
            overlay_right_surface,
            settings_surface,
            drawer_surface,
            last_tick: Instant::now(),
            fast_timer_active: false,
            last_snap_volume: None,
            last_snap_brightness: None,
            last_volume_write_at: None,
            last_brightness_write_at: None,
            last_audio_device: None,
            last_running_windows: Vec::new(),
            dragging_notch: false,
            dragging_notch_slider: false,
            drag_moved: false,
            drag_start_cursor: (0, 0),
            suppressed_notif_hwnds: Vec::new(),
            quit_event,
            tick_count: 0,
        };

        shell.apply_native_settings(&native_settings, false);
        shell.sync_shelf_to_views();
        shell.os_worker.send(OsCommand::ScanTrayEntries);

        let initial_snap = shell.telemetry.snapshot();
        shell.apply_telemetry(&initial_snap, false);
        shell.sync_overlay_visibility();
        shell.notch_surface.show();
        if shell.dock.visible {
            shell.dock_surface.show();
        }
        shell.update_surface_click_through();
        shell.render_all();
        shell.log_startup_diag();
        Ok(shell)
    }

    fn log_startup_diag(&self) {
        self.log_tick_diag("startup");
    }

    fn log_tick_diag(&self, label: &str) {
        let (nw, nh) = self.notch_surface.dip_size();
        let nrect = self.notch.computed_rect(nw, nh);
        let nvis = unsafe { IsWindowVisible(self.notch_surface.hwnd()).as_bool() };
        let mut nwin = RECT::default();
        let _ = unsafe { GetWindowRect(self.notch_surface.hwnd(), &mut nwin) };
        let (dw, dh) = self.dock_surface.dip_size();
        let drect = self.dock.dock_rect(Rect::new(0.0, 0.0, dw, dh));
        let dvis = unsafe { IsWindowVisible(self.dock_surface.hwnd()).as_bool() };
        let mut dwin = RECT::default();
        let _ = unsafe { GetWindowRect(self.dock_surface.hwnd(), &mut dwin) };
        log_diag(&format!(
            "{label}: notch(hwnd={:?}, vis={}, win=({},{},{},{}), dip=({nw:.1},{nh:.1}), rect=({:.1},{:.1},{:.1},{:.1}), hidden={}) dock(hwnd={:?}, vis={}, enabled={}, win=({},{},{},{}), dip=({dw:.1},{dh:.1}), rect=({:.1},{:.1},{:.1},{:.1}), spring={:.2}, items={})",
            self.notch_surface.hwnd(),
            nvis,
            nwin.left, nwin.top, nwin.right, nwin.bottom,
            nrect.x, nrect.y, nrect.width, nrect.height,
            self.notch.is_hidden,
            self.dock_surface.hwnd(),
            dvis,
            self.dock.visible,
            dwin.left, dwin.top, dwin.right, dwin.bottom,
            drect.x, drect.y, drect.width, drect.height,
            self.dock.visibility.value,
            self.dock.items().len(),
        ));
    }

    fn apply_native_settings(&mut self, ns: &NativeSettings, save: bool) {
        let norm = ns.normalized();
        self.native_settings = norm.clone();
        apply_native_settings_to_panel(&norm, &mut self.settings);
        if save {
            let _ = SettingsStore::save(&norm);
        }

        // 1. Notch settings
        let target_edge = parse_notch_edge(&norm.notch_edge);
        self.apply_notch_edge(target_edge);
        self.notch.notch_mode_label = capitalize_label(&norm.notch_mode);
        self.notch.free_drag_enabled = norm.free_drag_enabled;
        self.notch.time_format_24h = norm.time_format_24h;
        self.notch.dock_fixed = norm.dock_mode == "fixed";
        self.notch.volume_overlay_enabled = norm.volume_overlay_enabled;
        self.notch.brightness_overlay_enabled = norm.brightness_overlay_enabled;
        self.notch.notif_suppression_enabled = norm.suppress_native_notifications;
        self.notch.set_ai_enabled(norm.hime_ai_enabled);
        if norm.notch_mode == "fixed" {
            self.notch.set_hidden(false);
        } else if norm.notch_mode == "peek" && !self.notch.is_expanded {
            self.notch.set_hidden(true);
        }

        // 2. Dock settings
        self.dock.set_ai_enabled(norm.hime_ai_enabled);
        self.dock.set_mode(parse_dock_mode(&norm.dock_mode));
        let pinned_items = build_pinned_dock_items(&norm.dock_pinned);
        self.dock.set_pinned_apps(pinned_items);
        if !self.last_running_windows.is_empty() {
            self.dock.sync_running_windows(&self.last_running_windows);
        }
        if self.dock.visible != norm.dock_enabled {
            self.dock.set_visible(norm.dock_enabled);
            if norm.dock_enabled {
                self.dock_surface.show();
                os_shell::hide_taskbar();
            } else {
                self.dock_surface.hide();
                os_shell::restore_taskbar();
            }
        } else if norm.dock_enabled {
            self.dock_surface.show();
            os_shell::hide_taskbar();
        } else {
            self.dock_surface.hide();
            os_shell::restore_taskbar();
        }

        // 3. Dual Edge Overlays
        self.overlays.volume.set_enabled(norm.volume_overlay_enabled);
        self.overlays
            .brightness
            .set_enabled(norm.brightness_overlay_enabled);
        self.sync_overlay_visibility();

        // 4. Hotkeys (Win+1..9)
        hotkey::set_ll_features(norm.dock_win_number_enabled, true);

        // 5. Notification suppression
        if !norm.suppress_native_notifications {
            self.restore_suppressed_notifications();
        } else {
            self.scan_and_suppress_notifications();
        }

        // 6. Hime AI sidecar
        if norm.hime_ai_enabled {
            let _ = self.module_mgr.spawn();
        } else {
            let _ = self.module_mgr.stop();
            if self.ai_drawer.visible {
                self.ai_drawer.close();
                os_shell::set_window_focusable(self.drawer_surface.hwnd(), false);
                self.drawer_surface.hide();
            }
        }
    }

    fn sync_overlay_visibility(&self) {
        if self.overlays.volume.is_visible() {
            self.overlay_left_surface.show();
            self.overlay_left_surface.set_click_through(false);
        } else {
            self.overlay_left_surface.set_click_through(true);
            self.overlay_left_surface.hide();
        }
        if self.overlays.brightness.is_visible() {
            self.overlay_right_surface.show();
            self.overlay_right_surface.set_click_through(false);
        } else {
            self.overlay_right_surface.set_click_through(true);
            self.overlay_right_surface.hide();
        }
    }

    fn sync_shelf_to_views(&mut self) {
        let attachments: Vec<DrawerAttachment> = self
            .shelf
            .items()
            .iter()
            .map(|item| DrawerAttachment {
                id: item.id.clone(),
                name: item.name.clone(),
                path: item.path.clone(),
                kind: format!("{:?}", item.kind),
            })
            .collect();
        self.ai_drawer.clear_attachments();
        self.ai_drawer.attach_items(attachments);

        let summary: Vec<ShelfSummaryItem> = self
            .shelf
            .items()
            .iter()
            .map(|item| ShelfSummaryItem {
                name: item.name.clone(),
                kind: shelf_kind_tag(item.kind).to_string(),
                size_label: format_bytes(item.size_bytes),
            })
            .collect();
        self.notch.set_shelf_items(summary);
    }

    fn apply_telemetry(&mut self, snap: &TelemetrySnapshot, notify_hud: bool) {
        self.notch.set_clock(snap.hours, snap.minutes);
        self.notch.set_net_stats(snap.network.stats);
        self.notch.set_battery(snap.battery);

        let show_music =
            self.native_settings.music_mode_enabled || self.notch.active_panel == NotchPanel::Music;
        if show_music {
            self.notch.set_music(snap.music.clone());
        } else {
            self.notch.set_music(None);
        }

        let wifi_items: Vec<WifiNetworkItem> = snap
            .available_wifi
            .iter()
            .map(|w| WifiNetworkItem {
                ssid: w.ssid.clone(),
                signal: w.signal_percent,
                secured: w.secured,
                connected: w.connected,
            })
            .collect();
        if snap.network.kind == NetworkKind::Wifi && snap.network.ssid.is_some() {
            self.notch.wifi_enabled = true;
        }
        self.notch.set_wifi_status(
            self.notch.wifi_enabled,
            snap.network.ssid.clone(),
            wifi_items,
        );

        let expr = if snap.network.stats.is_internet_down {
            HimeExpression::Offline
        } else if show_music && snap.music.as_ref().is_some_and(|m| m.is_playing) {
            HimeExpression::Senang
        } else {
            HimeExpression::Netral
        };
        self.notch.set_expression(expr);

        // Sync running windows & tray count into DockBar
        self.last_running_windows = snap.running_windows.clone();
        self.dock.sync_running_windows(&snap.running_windows);
        let effective_tray = snap.tray_count.max(self.dock.tray_items().len() as u32);
        self.dock.set_system_tray_count(effective_tray);

        self.last_audio_device = sanitize_device_name(snap.volume.device_name.as_deref());

        let vol_pair = (snap.volume.percent, snap.volume.muted);
        let recent_vol_write = self
            .last_volume_write_at
            .is_some_and(|t| t.elapsed() < Duration::from_millis(1300));
        if !self.overlays.volume.is_dragging && !recent_vol_write {
            if !notify_hud || self.last_snap_volume.is_none() {
                // Quietly seed initial volume without expanding idle pill
                self.overlays.volume.percent = snap.volume.percent.min(100);
                self.overlays.volume.muted = snap.volume.muted;
                self.overlays.volume.device_name = self.last_audio_device.clone();
                self.overlays.volume.bar_fill.snap_to(if snap.volume.muted {
                    0.0
                } else {
                    snap.volume.percent.min(100) as f32 / 100.0
                });
                self.notch
                    .set_volume(snap.volume.percent as f32 / 100.0, snap.volume.muted);
            } else if self.last_snap_volume != Some(vol_pair) {
                self.overlays.show_volume(
                    snap.volume.percent,
                    snap.volume.muted,
                    snap.volume.device_name.clone(),
                );
                self.notch
                    .set_volume(snap.volume.percent as f32 / 100.0, snap.volume.muted);
            }
        }
        self.last_snap_volume = Some(vol_pair);

        let bri = snap.brightness.min(100);
        let recent_bri_write = self
            .last_brightness_write_at
            .is_some_and(|t| t.elapsed() < Duration::from_millis(1300));
        if !self.overlays.brightness.is_dragging && !recent_bri_write {
            if !notify_hud || self.last_snap_brightness.is_none() {
                self.overlays.brightness.percent = bri;
                self.overlays
                    .brightness
                    .bar_fill
                    .snap_to(bri as f32 / 100.0);
                self.notch.set_brightness(bri);
            } else if self.last_snap_brightness != Some(bri) {
                self.overlays.show_brightness(bri);
                self.notch.set_brightness(bri);
            }
        }
        self.last_snap_brightness = Some(bri);
    }

    fn apply_notch_edge(&mut self, edge: NotchEdge) {
        if self.notch.edge == edge {
            return;
        }
        self.notch.set_edge(edge);
        let screen = os_shell::primary_screen_bounds();
        let scale = (self.notch_surface.dpi() / 96.0).max(0.5);
        let (x, y, w, h) = notch_surface_bounds(edge, screen, scale);
        let _ = self.notch_surface.set_bounds(x, y, w, h);
    }

    fn update_surface_click_through(&mut self) {
        let primary = os_shell::primary_screen_bounds();
        if let Some((cx, cy)) = os_shell::cursor_position() {
            // Notch click-through & edge-wake
            let notch_scale = (self.notch_surface.dpi() / 96.0).max(0.5);
            let (nx, ny, _, _) = notch_surface_bounds(self.notch.edge, primary, notch_scale);
            let (nw_dip, nh_dip) = self.notch_surface.dip_size();
            let local_nx = (cx - nx) as f32 / notch_scale;
            let local_ny = (cy - ny) as f32 / notch_scale;
            let over_notch =
                os_shell::hit_test_notch(local_nx, local_ny, &self.notch, nw_dip, nh_dip);
            let near_wake = os_shell::is_notch_edge_hovered(
                cx as f32,
                cy as f32,
                &self.notch,
                primary,
                os_shell::DEFAULT_EDGE_WAKE_THRESHOLD_PX,
            );
            if near_wake && self.notch.is_hidden {
                self.notch.set_hidden(false);
            } else if !over_notch
                && !near_wake
                && !self.notch.is_expanded
                && !self.dragging_notch
                && !self.dragging_notch_slider
            {
                if self.native_settings.notch_mode == "peek" {
                    self.notch.set_hidden(true);
                } else if self.native_settings.notch_mode == "smart" {
                    let overlapped = self.is_foreground_overlapping_notch(primary);
                    self.notch.set_hidden(overlapped);
                }
            }
            self.notch_surface.set_click_through(
                !(over_notch || near_wake || self.dragging_notch || self.dragging_notch_slider),
            );

            // Dock click-through, overlap detection & bottom-edge wake
            if self.dock.visible {
                let dock_scale = (self.dock_surface.dpi() / 96.0).max(0.5);
                let (dx, dy, _, _) = dock_surface_bounds(primary, dock_scale);
                let (dw_dip, dh_dip) = self.dock_surface.dip_size();
                let surf_rect = Rect::new(0.0, 0.0, dw_dip, dh_dip);
                let local_dx = (cx - dx) as f32 / dock_scale;
                let local_dy = (cy - dy) as f32 / dock_scale;

                let overlapped = self.is_foreground_overlapping_dock(primary);
                self.dock.set_overlapped(overlapped);

                if self.dock.is_hidden && self.dock.show_progress.value <= 0.2 {
                    let wake = self.dock.wake_hit_rect(surf_rect);
                    let over_wake = os_shell::is_point_in_rect(local_dx, local_dy, wake);
                    self.dock.set_edge_hovered(over_wake);
                    self.dock_surface.set_click_through(!over_wake);
                } else {
                    let dock_rect = self.dock.dock_rect(surf_rect);
                    let over_dock = os_shell::is_point_in_rect(local_dx, local_dy, dock_rect);
                    let over_flyout = self
                        .dock
                        .tray_flyout_rect(surf_rect, dock_rect)
                        .is_some_and(|r| os_shell::is_point_in_rect(local_dx, local_dy, r));
                    let over_any = over_dock || over_flyout;
                    if !over_any && self.dock.has_hover() {
                        self.dock.clear_hover();
                    }
                    self.dock.set_edge_hovered(over_any);
                    self.dock_surface.set_click_through(!over_any);
                }
            } else {
                self.dock_surface.set_click_through(true);
            }

            // Left & Right Edge Overlay hover guard (safety net in case cursor leaves quickly)
            self.sync_overlay_hover_cursor(true, cx, cy, primary);
            self.sync_overlay_hover_cursor(false, cx, cy, primary);
        }

        self.drawer_surface
            .set_click_through(!self.ai_drawer.visible);
        self.settings_surface
            .set_click_through(!self.settings.visible);
    }

    fn sync_overlay_hover_cursor(
        &mut self,
        is_left: bool,
        cx: i32,
        cy: i32,
        screen: os_shell::ScreenBounds,
    ) {
        let (scale, sw, sh) = {
            let surface = if is_left {
                &self.overlay_left_surface
            } else {
                &self.overlay_right_surface
            };
            let (w, h) = surface.dip_size();
            ((surface.dpi() / 96.0).max(0.5), w, h)
        };
        let (ox, oy, _, _) = if is_left {
            left_overlay_bounds(screen, scale)
        } else {
            right_overlay_bounds(screen, scale)
        };
        let surf_rect = Rect::new(0.0, 0.0, sw, sh);
        let lx = (cx - ox) as f32 / scale;
        let ly = (cy - oy) as f32 / scale;
        let hud = if is_left {
            &mut self.overlays.volume
        } else {
            &mut self.overlays.brightness
        };
        if !hud.is_visible() {
            return;
        }
        let over = hud.hit_test_pill(lx, ly, surf_rect);
        if over != hud.is_hovered && (over || !hud.is_dragging) {
            hud.set_hovered(over);
        }
    }

    fn is_foreground_overlapping_dock(&self, screen: os_shell::ScreenBounds) -> bool {
        let fg = unsafe { GetForegroundWindow() };
        if fg.0.is_null() || self.is_own_hwnd(fg) {
            return false;
        }
        unsafe {
            if !IsWindowVisible(fg).as_bool() || IsIconic(fg).as_bool() {
                return false;
            }
            let mut r = RECT::default();
            if GetWindowRect(fg, &mut r).is_err() {
                return false;
            }
            let scale = (self.dock_surface.dpi() / 96.0).max(0.5);
            let dock_band_top = screen.y + screen.height - (68.0 * scale).round() as i32;
            r.bottom > dock_band_top
                && r.right > screen.x
                && r.left < screen.x + screen.width
                && r.top < screen.y + screen.height
        }
    }

    fn is_foreground_overlapping_notch(&self, screen: os_shell::ScreenBounds) -> bool {
        let fg = unsafe { GetForegroundWindow() };
        if fg.0.is_null() || self.is_own_hwnd(fg) {
            return false;
        }
        unsafe {
            if !IsWindowVisible(fg).as_bool() || IsIconic(fg).as_bool() {
                return false;
            }
            let mut r = RECT::default();
            if GetWindowRect(fg, &mut r).is_err() {
                return false;
            }
            let scale = (self.notch_surface.dpi() / 96.0).max(0.5);
            let (nw, nh) = self.notch_surface.dip_size();
            let rect = self.notch.computed_rect(nw, nh);
            let (nx, ny, _, _) = notch_surface_bounds(self.notch.edge, screen, scale);
            let n_left = nx + (rect.x * scale).round() as i32;
            let n_top = ny + (rect.y * scale).round() as i32;
            let n_right = n_left + (rect.width * scale).round() as i32;
            let n_bottom = n_top + (rect.height * scale).round() as i32;
            r.left < n_right && r.right > n_left && r.top < n_bottom && r.bottom > n_top
        }
    }

    fn is_own_hwnd(&self, hwnd: HWND) -> bool {
        let raw = hwnd.0 as isize;
        raw == self.notch_surface.hwnd().0 as isize
            || raw == self.dock_surface.hwnd().0 as isize
            || raw == self.overlay_left_surface.hwnd().0 as isize
            || raw == self.overlay_right_surface.hwnd().0 as isize
            || raw == self.settings_surface.hwnd().0 as isize
            || raw == self.drawer_surface.hwnd().0 as isize
    }

    fn handle_surface_event(&mut self, ev: SurfaceEvent, ctrl_hwnd: HWND) {
        let raw_hwnd = ev.hwnd;
        if raw_hwnd == self.notch_surface.hwnd().0 as isize {
            self.handle_notch_surface_event(ev.kind, ctrl_hwnd);
            return;
        }
        if raw_hwnd == self.dock_surface.hwnd().0 as isize {
            self.handle_dock_surface_event(ev.kind, ctrl_hwnd);
            return;
        }
        if raw_hwnd == self.overlay_left_surface.hwnd().0 as isize {
            self.handle_overlay_surface_event(true, ev.kind, ctrl_hwnd);
            return;
        }
        if raw_hwnd == self.overlay_right_surface.hwnd().0 as isize {
            self.handle_overlay_surface_event(false, ev.kind, ctrl_hwnd);
            return;
        }
        if raw_hwnd == self.settings_surface.hwnd().0 as isize {
            self.handle_settings_surface_event(ev.kind, ctrl_hwnd);
            return;
        }
        if raw_hwnd == self.drawer_surface.hwnd().0 as isize {
            self.handle_drawer_surface_event(ev.kind, ctrl_hwnd);
        }
    }

    fn handle_extra_surface_event(&mut self, ev: ExtraSurfaceEvent, ctrl_hwnd: HWND) {
        if ev.hwnd == self.overlay_left_surface.hwnd().0 as isize {
            self.handle_overlay_extra_event(true, ev.kind, ctrl_hwnd);
            return;
        }
        if ev.hwnd == self.overlay_right_surface.hwnd().0 as isize {
            self.handle_overlay_extra_event(false, ev.kind, ctrl_hwnd);
            return;
        }
        if ev.hwnd == self.dock_surface.hwnd().0 as isize {
            self.handle_dock_extra_event(ev.kind, ctrl_hwnd);
            return;
        }
        if ev.hwnd == self.notch_surface.hwnd().0 as isize {
            self.handle_notch_extra_event(ev.kind, ctrl_hwnd);
        }
    }

    fn handle_notch_surface_event(&mut self, kind: SurfaceEventKind, ctrl_hwnd: HWND) {
        let scale = (self.notch_surface.dpi() / 96.0).max(0.5);
        let (nw, nh) = self.notch_surface.dip_size();
        let surf_rect = Rect::new(0.0, 0.0, nw, nh);
        let mut render_now = false;

        match kind {
            SurfaceEventKind::LeftButtonDown { x, y } => {
                let lx = x / scale;
                let ly = y / scale;
                if self.notch.is_hidden {
                    self.notch.set_hidden(false);
                    render_now = true;
                }
                if self.notch.is_expanded {
                    if let Some(act @ (NotchAction::SetVolume(_)
                    | NotchAction::SetBrightness(_)
                    | NotchAction::MusicSeek(_))) = self.notch.hit_test_click(lx, ly, surf_rect)
                    {
                        self.dragging_notch_slider = true;
                        unsafe {
                            let _ = SetCapture(self.notch_surface.hwnd());
                        }
                        self.execute_notch_action(act, ctrl_hwnd);
                        render_now = true;
                    } else {
                        self.dragging_notch = true;
                        self.drag_moved = false;
                        self.drag_start_cursor = os_shell::cursor_position().unwrap_or((0, 0));
                    }
                } else {
                    self.dragging_notch = true;
                    self.drag_moved = false;
                    self.drag_start_cursor = os_shell::cursor_position().unwrap_or((0, 0));
                }
            }
            SurfaceEventKind::MouseMove { x, y } => {
                let lx = x / scale;
                let ly = y / scale;
                if self.notch.is_hidden {
                    self.notch.set_hidden(false);
                    render_now = true;
                }
                if self.dragging_notch_slider {
                    if let Some(act @ (NotchAction::SetVolume(_)
                    | NotchAction::SetBrightness(_)
                    | NotchAction::MusicSeek(_))) = self.notch.hit_test_click(lx, ly, surf_rect)
                    {
                        self.execute_notch_action(act, ctrl_hwnd);
                        render_now = true;
                    }
                } else if self.dragging_notch && self.notch.free_drag_enabled {
                    if let Some((cx, cy)) = os_shell::cursor_position() {
                        let dx = (cx - self.drag_start_cursor.0).abs();
                        let dy = (cy - self.drag_start_cursor.1).abs();
                        if dx > 12 || dy > 12 {
                            self.drag_moved = true;
                            let next_edge = os_shell::snap_notch_edge_from_cursor();
                            self.apply_notch_edge(next_edge);
                            render_now = true;
                        }
                    }
                }
            }
            SurfaceEventKind::LeftButtonUp { x, y } => {
                if self.dragging_notch_slider {
                    self.dragging_notch_slider = false;
                    unsafe {
                        let _ = ReleaseCapture();
                    }
                    render_now = true;
                } else if self.dragging_notch {
                    self.dragging_notch = false;
                    if self.drag_moved {
                        self.native_settings.notch_edge = notch_edge_to_str(self.notch.edge).to_string();
                        apply_native_settings_to_panel(&self.native_settings, &mut self.settings);
                        let _ = SettingsStore::save(&self.native_settings);
                        render_now = true;
                    } else {
                        let lx = x / scale;
                        let ly = y / scale;
                        if let Some(action) = self.notch.hit_test_click(lx, ly, surf_rect) {
                            self.execute_notch_action(action, ctrl_hwnd);
                            render_now = true;
                        }
                    }
                }
            }
            SurfaceEventKind::DropFiles { wparam } => {
                let added = os_shell::handle_wm_dropfiles(WPARAM(wparam), &mut self.shelf);
                if !added.is_empty() {
                    let _ = self.shelf.save();
                    self.sync_shelf_to_views();
                    self.notch
                        .set_ncm_center_alert(Some(format!("+{} file(s)", added.len())));
                    self.notch.set_panel(NotchPanel::Shelf);
                    render_now = true;
                }
            }
        }

        if render_now {
            self.update_surface_click_through();
            self.render_all();
        }
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn handle_notch_extra_event(&mut self, kind: ExtraSurfaceEventKind, ctrl_hwnd: HWND) {
        if let ExtraSurfaceEventKind::MouseWheel { delta_steps, .. } = kind {
            if self.notch.is_expanded && delta_steps.abs() > 0.01 {
                let shift_held = unsafe { (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0 };
                if shift_held || self.notch.active_panel != NotchPanel::CommandCenter {
                    let act = if delta_steps > 0.0 {
                        NotchAction::PrevPanel
                    } else {
                        NotchAction::NextPanel
                    };
                    self.execute_notch_action(act, ctrl_hwnd);
                    self.update_surface_click_through();
                    self.render_all();
                    self.ensure_fast_timer(ctrl_hwnd);
                }
            }
        }
    }

    fn execute_notch_action(&mut self, action: NotchAction, ctrl_hwnd: HWND) {
        let was_vol_muted = self.notch.volume_muted;
        self.notch.apply_action(&action);

        match action {
            NotchAction::ExpandPanel(NotchPanel::Wifi) | NotchAction::OpenWifiList | NotchAction::WifiRefresh => {
                self.os_worker.send(OsCommand::ScanWifi);
            }
            NotchAction::ToggleWifi => {
                self.os_worker
                    .send(OsCommand::SetWifiRadio(self.notch.wifi_enabled));
            }
            NotchAction::ToggleBluetooth => {
                self.os_worker
                    .send(OsCommand::SetBluetoothRadio(self.notch.bluetooth_enabled));
            }
            NotchAction::WifiConnect(ssid) => {
                self.notch
                    .set_ncm_center_alert(Some(format!("Connecting {ssid}...")));
                self.os_worker.send(OsCommand::ConnectWifi(ssid));
            }
            NotchAction::CycleNotchMode => {
                let mut ns = self.native_settings.clone();
                ns.cycle_notch_mode();
                self.apply_native_settings(&ns, true);
            }
            NotchAction::ToggleFreeDrag => {
                let mut ns = self.native_settings.clone();
                ns.free_drag_enabled = self.notch.free_drag_enabled;
                self.apply_native_settings(&ns, true);
            }
            NotchAction::ToggleDockMode => {
                let mut ns = self.native_settings.clone();
                ns.dock_mode = if self.notch.dock_fixed {
                    "fixed".to_string()
                } else {
                    "auto-hide".to_string()
                };
                self.apply_native_settings(&ns, true);
            }
            NotchAction::ToggleVolumeOverlay => {
                let mut ns = self.native_settings.clone();
                ns.volume_overlay_enabled = self.notch.volume_overlay_enabled;
                self.apply_native_settings(&ns, true);
            }
            NotchAction::ToggleBrightnessOverlay => {
                let mut ns = self.native_settings.clone();
                ns.brightness_overlay_enabled = self.notch.brightness_overlay_enabled;
                self.apply_native_settings(&ns, true);
            }
            NotchAction::ToggleNotifSuppression => {
                let mut ns = self.native_settings.clone();
                ns.suppress_native_notifications = self.notch.notif_suppression_enabled;
                self.apply_native_settings(&ns, true);
            }
            NotchAction::ToggleHimeAi => {
                let mut ns = self.native_settings.clone();
                ns.hime_ai_enabled = self.notch.ai_enabled;
                self.apply_native_settings(&ns, true);
            }
            NotchAction::SetVolume(ratio) => {
                let pct = (ratio.clamp(0.0, 1.0) * 100.0).round() as u8;
                let muted = pct == 0;
                self.last_volume_write_at = Some(Instant::now());
                self.os_worker.send(OsCommand::SetVolume {
                    percent: pct,
                    unmute: was_vol_muted && pct > 0,
                });
                if self.overlays.volume.enabled {
                    self.overlays
                        .show_volume(pct, muted, self.last_audio_device.clone());
                } else {
                    self.overlays.volume.percent = pct;
                    self.overlays.volume.muted = muted;
                }
            }
            NotchAction::SetBrightness(pct) => {
                let clamped = pct.min(100);
                self.last_brightness_write_at = Some(Instant::now());
                self.os_worker.send(OsCommand::SetBrightness(clamped));
                if self.overlays.brightness.enabled {
                    self.overlays.show_brightness(clamped);
                } else {
                    self.overlays.brightness.percent = clamped;
                }
            }
            NotchAction::MusicPlayPause => {
                self.os_worker.send(OsCommand::MediaPlayPause);
            }
            NotchAction::MusicNext => {
                self.os_worker.send(OsCommand::MediaNext);
            }
            NotchAction::MusicPrev => {
                self.os_worker.send(OsCommand::MediaPrev);
            }
            NotchAction::MusicSeek(ratio) => {
                let pos_ms = ((self.notch.music_duration_ms as f32) * ratio.clamp(0.0, 1.0)) as u64;
                self.os_worker.send(OsCommand::MediaSeek(pos_ms));
            }
            NotchAction::OpenSettings => {
                self.open_settings(ctrl_hwnd);
            }
            NotchAction::OpenAiDrawer => {
                if !self.ai_drawer.visible {
                    self.handle_action(HotkeyAction::ToggleAssistant, ctrl_hwnd);
                }
            }
            NotchAction::ClearShelf => {
                self.shelf.clear();
                let _ = self.shelf.save();
                self.sync_shelf_to_views();
            }
            NotchAction::ExpandPanel(_)
            | NotchAction::Collapse
            | NotchAction::PrevPanel
            | NotchAction::NextPanel
            | NotchAction::OpenShortcuts
            | NotchAction::CalendarPrevMonth
            | NotchAction::CalendarNextMonth
            | NotchAction::CalendarToday
            | NotchAction::PomodoroStartPause
            | NotchAction::PomodoroReset
            | NotchAction::PomodoroPreset(_)
            | NotchAction::MusicToggleLyrics => {}
        }
    }

    fn handle_dock_surface_event(&mut self, kind: SurfaceEventKind, ctrl_hwnd: HWND) {
        let scale = (self.dock_surface.dpi() / 96.0).max(0.5);
        let (dw, dh) = self.dock_surface.dip_size();
        let rect = Rect::new(0.0, 0.0, dw, dh);
        let mut render_now = false;

        match kind {
            SurfaceEventKind::MouseMove { x, y } => {
                let prev_hidden = self.dock.is_hidden;
                let prev_hover = self.dock.hovered_index();
                let next_hover = self.dock.set_hovered_at(x / scale, y / scale, rect);
                if prev_hidden != self.dock.is_hidden || prev_hover != next_hover || self.dock.has_hover()
                {
                    render_now = true;
                }
            }
            SurfaceEventKind::LeftButtonDown { x, y } => {
                if let Some(act) = self.dock.hit_test_action(x / scale, y / scale, rect) {
                    self.execute_dock_action(act, ctrl_hwnd);
                    render_now = true;
                }
            }
            SurfaceEventKind::LeftButtonUp { .. } | SurfaceEventKind::DropFiles { .. } => {}
        }

        if render_now {
            self.update_surface_click_through();
            self.render_all();
        }
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn handle_dock_extra_event(&mut self, kind: ExtraSurfaceEventKind, ctrl_hwnd: HWND) {
        let scale = (self.dock_surface.dpi() / 96.0).max(0.5);
        let (dw, dh) = self.dock_surface.dip_size();
        let rect = Rect::new(0.0, 0.0, dw, dh);
        match kind {
            ExtraSurfaceEventKind::MouseLeave => {
                self.dock.clear_hover();
                self.dock.set_edge_hovered(false);
                self.update_surface_click_through();
                self.render_all();
                self.ensure_fast_timer(ctrl_hwnd);
            }
            ExtraSurfaceEventKind::RightButtonUp { x, y } => {
                if let Some(act) = self.dock.hit_test_context_action(x / scale, y / scale, rect) {
                    self.execute_dock_action(act, ctrl_hwnd);
                    self.update_surface_click_through();
                    self.render_all();
                    self.ensure_fast_timer(ctrl_hwnd);
                }
            }
            ExtraSurfaceEventKind::MouseWheel { .. } => {}
        }
    }

    fn execute_dock_action(&mut self, action: DockAction, ctrl_hwnd: HWND) {
        match action {
            DockAction::App(idx) => {
                if let Some(item) = self.dock.items().get(idx).cloned() {
                    focus_or_launch_dock_item(&item);
                }
            }
            DockAction::TogglePin(idx) => {
                if self.dock.toggle_pin_at(idx).is_some() {
                    let pinned: Vec<String> = self
                        .dock
                        .items()
                        .iter()
                        .filter(|it| it.pinned)
                        .map(|it| {
                            if !it.exe_path.trim().is_empty() {
                                it.exe_path.clone()
                            } else {
                                it.executable.clone()
                            }
                        })
                        .collect();
                    self.native_settings.dock_pinned = pinned;
                    apply_native_settings_to_panel(&self.native_settings, &mut self.settings);
                    let _ = SettingsStore::save(&self.native_settings);
                }
            }
            DockAction::ToggleAiDrawer => {
                self.handle_action(HotkeyAction::ToggleAssistant, ctrl_hwnd);
            }
            DockAction::ToggleSettings => {
                self.toggle_settings(ctrl_hwnd);
            }
            DockAction::ToggleTray => {
                if self.dock.tray_items().is_empty() {
                    unsafe {
                        show_tray_menu(ctrl_hwnd);
                    }
                } else {
                    self.dock.toggle_tray();
                }
            }
            DockAction::ActivateTrayItem(idx) => {
                if let Some(entry) = self.dock.tray_items().get(idx).cloned() {
                    self.dock.set_tray_open(false);
                    if entry.hwnd != 0 {
                        focus_hwnd(entry.hwnd);
                    } else if !entry.exe_path.trim().is_empty() {
                        launch_executable(&entry.exe_path);
                    }
                }
            }
            DockAction::WakeHiddenDock => {
                self.dock.set_edge_hovered(true);
            }
        }
    }

    fn handle_overlay_surface_event(
        &mut self,
        is_left: bool,
        kind: SurfaceEventKind,
        ctrl_hwnd: HWND,
    ) {
        let (surface_hwnd, scale, sw, sh) = {
            let surf = if is_left {
                &self.overlay_left_surface
            } else {
                &self.overlay_right_surface
            };
            let (w, h) = surf.dip_size();
            (surf.hwnd(), (surf.dpi() / 96.0).max(0.5), w, h)
        };
        let surf_rect = Rect::new(0.0, 0.0, sw, sh);
        let mut render_now = false;

        match kind {
            SurfaceEventKind::MouseMove { x, y } => {
                let lx = x / scale;
                let ly = y / scale;
                let hud = if is_left {
                    &mut self.overlays.volume
                } else {
                    &mut self.overlays.brightness
                };
                if !hud.is_hovered {
                    hud.set_hovered(true);
                    render_now = true;
                }
                if hud.is_dragging {
                    if let Some(ratio) = hud.hit_test_slider(lx, ly, surf_rect) {
                        self.apply_overlay_slider_ratio(is_left, ratio);
                        render_now = true;
                    }
                }
            }
            SurfaceEventKind::LeftButtonDown { x, y } => {
                let lx = x / scale;
                let ly = y / scale;
                if is_left && self.overlays.volume.hit_test_mute_button(lx, ly, surf_rect) {
                    let muted = self.overlays.volume.toggle_mute();
                    self.last_volume_write_at = Some(Instant::now());
                    self.os_worker.send(OsCommand::ToggleMute);
                    self.notch
                        .set_volume(self.overlays.volume.percent as f32 / 100.0, muted);
                    render_now = true;
                } else {
                    let hit_ratio = if is_left {
                        self.overlays.volume.hit_test_slider(lx, ly, surf_rect)
                    } else {
                        self.overlays.brightness.hit_test_slider(lx, ly, surf_rect)
                    };
                    if let Some(ratio) = hit_ratio {
                        if is_left {
                            self.overlays.volume.set_dragging(true);
                        } else {
                            self.overlays.brightness.set_dragging(true);
                        }
                        unsafe {
                            let _ = SetCapture(surface_hwnd);
                        }
                        self.apply_overlay_slider_ratio(is_left, ratio);
                        render_now = true;
                    } else {
                        let hud = if is_left {
                            &mut self.overlays.volume
                        } else {
                            &mut self.overlays.brightness
                        };
                        hud.set_hovered(true);
                        render_now = true;
                    }
                }
            }
            SurfaceEventKind::LeftButtonUp { .. } => {
                let was_dragging = if is_left {
                    self.overlays.volume.is_dragging
                } else {
                    self.overlays.brightness.is_dragging
                };
                if was_dragging {
                    if is_left {
                        self.overlays.volume.set_dragging(false);
                    } else {
                        self.overlays.brightness.set_dragging(false);
                    }
                    unsafe {
                        let _ = ReleaseCapture();
                    }
                    render_now = true;
                }
            }
            SurfaceEventKind::DropFiles { .. } => {}
        }

        if render_now {
            self.render_all();
        }
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn handle_overlay_extra_event(
        &mut self,
        is_left: bool,
        kind: ExtraSurfaceEventKind,
        ctrl_hwnd: HWND,
    ) {
        let mut render_now = false;
        match kind {
            ExtraSurfaceEventKind::MouseLeave => {
                let hud = if is_left {
                    &mut self.overlays.volume
                } else {
                    &mut self.overlays.brightness
                };
                if !hud.is_dragging {
                    hud.set_hovered(false);
                    render_now = true;
                }
            }
            ExtraSurfaceEventKind::MouseWheel { delta_steps, .. } => {
                if delta_steps.abs() > 0.01 {
                    if is_left {
                        let was_muted = self.overlays.volume.muted;
                        let pct = self.overlays.volume.on_scroll(delta_steps);
                        let muted = self.overlays.volume.muted;
                        self.last_volume_write_at = Some(Instant::now());
                        self.os_worker.send(OsCommand::SetVolume {
                            percent: pct,
                            unmute: was_muted && pct > 0,
                        });
                        self.notch.set_volume(pct as f32 / 100.0, muted);
                    } else {
                        let pct = self.overlays.brightness.on_scroll(delta_steps);
                        self.last_brightness_write_at = Some(Instant::now());
                        self.os_worker.send(OsCommand::SetBrightness(pct));
                        self.notch.set_brightness(pct);
                    }
                    render_now = true;
                }
            }
            ExtraSurfaceEventKind::RightButtonUp { .. } => {}
        }
        if render_now {
            self.render_all();
        }
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn apply_overlay_slider_ratio(&mut self, is_left: bool, ratio: f32) {
        if is_left {
            let was_muted = self.overlays.volume.muted;
            let pct = self.overlays.volume.set_percent_from_ratio(ratio);
            let muted = self.overlays.volume.muted;
            self.last_volume_write_at = Some(Instant::now());
            self.os_worker.send(OsCommand::SetVolume {
                percent: pct,
                unmute: was_muted && pct > 0,
            });
            self.notch.set_volume(pct as f32 / 100.0, muted);
        } else {
            let pct = self.overlays.brightness.set_percent_from_ratio(ratio);
            self.last_brightness_write_at = Some(Instant::now());
            self.os_worker.send(OsCommand::SetBrightness(pct));
            self.notch.set_brightness(pct);
        }
    }

    fn handle_settings_surface_event(&mut self, kind: SurfaceEventKind, ctrl_hwnd: HWND) {
        let scale = (self.settings_surface.dpi() / 96.0).max(0.5);
        match kind {
            SurfaceEventKind::MouseMove { x, y } => {
                if self.settings.set_hovered_at(x / scale, y / scale) {
                    self.render_all();
                }
            }
            SurfaceEventKind::LeftButtonDown { x, y } => {
                if self.settings.visible {
                    os_shell::focus_window(self.settings_surface.hwnd());
                }
                if let Some(action) = self.settings.handle_click(x / scale, y / scale) {
                    self.apply_settings_action(action, ctrl_hwnd);
                    self.render_all();
                }
                self.ensure_fast_timer(ctrl_hwnd);
            }
            SurfaceEventKind::LeftButtonUp { .. } | SurfaceEventKind::DropFiles { .. } => {}
        }
    }

    fn apply_settings_action(&mut self, action: SettingsAction, ctrl_hwnd: HWND) {
        match action {
            SettingsAction::SelectTab(_) => {}
            SettingsAction::UpdateSettings
            | SettingsAction::ToggleDockMode
            | SettingsAction::ToggleNotchMode
            | SettingsAction::ToggleFreeDrag
            | SettingsAction::ToggleHimeAi
            | SettingsAction::ResetToDefaults => {
                let ns = extract_native_settings_from_panel(&self.settings);
                self.apply_native_settings(&ns, true);
            }
            SettingsAction::TestAndSaveProfile => {
                let ns = extract_native_settings_from_panel(&self.settings);
                self.apply_native_settings(&ns, true);
                self.spawn_profile_test_and_probe();
            }
            SettingsAction::Close => {
                os_shell::set_window_focusable(self.settings_surface.hwnd(), false);
                self.settings_surface.hide();
                self.update_surface_click_through();
            }
        }
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn spawn_profile_test_and_probe(&self) {
        self.os_worker.send(OsCommand::Probe9Router);
        let cfg = self.settings.state.active_profile.clone();
        let ipc = Arc::clone(&self.ipc);
        let tx = self.ipc_tx.clone();
        std::thread::spawn(move || {
            let upsert = AiProfileUpsertParams {
                contract_version: IPC_CONTRACT_VERSION.to_string(),
                id: cfg.id.clone(),
                name: cfg.name,
                adapter_kind: cfg.preset.adapter_kind().to_string(),
                base_url: cfg.base_url,
                default_model: cfg.model,
                priority: 100,
                enabled: true,
                api_key: None,
            };
            let res = ipc
                .upsert_profile(upsert)
                .and_then(|_| ipc.test_profile(&cfg.id))
                .map(|r| r.message);
            let _ = tx.send(IpcBgResult::ProfileTest(res));
        });
    }

    fn open_settings(&mut self, ctrl_hwnd: HWND) {
        if !self.settings.visible {
            self.settings.open();
        }
        self.settings_surface.set_click_through(false);
        os_shell::set_window_focusable(self.settings_surface.hwnd(), true);
        self.settings_surface.show();
        os_shell::focus_window(self.settings_surface.hwnd());
        self.os_worker.send(OsCommand::Probe9Router);
        self.render_all();
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn toggle_settings(&mut self, ctrl_hwnd: HWND) {
        if self.settings.visible {
            self.settings.close();
            os_shell::set_window_focusable(self.settings_surface.hwnd(), false);
            self.settings_surface.hide();
            self.update_surface_click_through();
            self.render_all();
            self.ensure_fast_timer(ctrl_hwnd);
        } else {
            self.open_settings(ctrl_hwnd);
        }
    }

    fn handle_drawer_surface_event(&mut self, kind: SurfaceEventKind, ctrl_hwnd: HWND) {
        let scale = (self.drawer_surface.dpi() / 96.0).max(0.5);
        match kind {
            SurfaceEventKind::LeftButtonDown { .. } => {
                if self.ai_drawer.visible {
                    os_shell::focus_window(self.drawer_surface.hwnd());
                }
            }
            SurfaceEventKind::LeftButtonUp { x, y } => {
                if let Some(action) = self.ai_drawer.hit_test(x / scale, y / scale) {
                    self.apply_drawer_action(action, ctrl_hwnd);
                    self.render_all();
                } else if self.ai_drawer.visible {
                    os_shell::focus_window(self.drawer_surface.hwnd());
                }
            }
            SurfaceEventKind::DropFiles { wparam } => {
                let added = os_shell::handle_wm_dropfiles(WPARAM(wparam), &mut self.shelf);
                if !added.is_empty() {
                    let _ = self.shelf.save();
                    self.sync_shelf_to_views();
                    self.render_all();
                    self.ensure_fast_timer(ctrl_hwnd);
                }
            }
            SurfaceEventKind::MouseMove { .. } => {}
        }
    }

    fn apply_drawer_action(&mut self, action: DrawerAction, ctrl_hwnd: HWND) {
        match action {
            DrawerAction::Close => {
                self.ai_drawer.close();
                os_shell::set_window_focusable(self.drawer_surface.hwnd(), false);
                self.drawer_surface.hide();
            }
            DrawerAction::ClearAttachments => {
                self.shelf.clear();
                let _ = self.shelf.save();
                self.sync_shelf_to_views();
            }
            DrawerAction::SendPrompt => {
                self.dispatch_drawer_prompt();
            }
            DrawerAction::Approve(id) => {
                self.ai_drawer.resolve_approval(&id, true);
                self.dispatch_approval(id, true);
            }
            DrawerAction::Deny(id) => {
                self.ai_drawer.resolve_approval(&id, false);
                self.dispatch_approval(id, false);
            }
        }
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn handle_window_key_message(&mut self, msg: &MSG, ctrl_hwnd: HWND) -> bool {
        if msg.hwnd == self.drawer_surface.hwnd() && self.ai_drawer.visible {
            let action = match msg.message {
                WM_KEYDOWN => self.ai_drawer.handle_key_down(msg.wParam.0 as u32),
                WM_CHAR => {
                    char::from_u32(msg.wParam.0 as u32).and_then(|ch| self.ai_drawer.handle_char(ch))
                }
                _ => None,
            };
            if let Some(act) = action {
                self.apply_drawer_action(act, ctrl_hwnd);
            } else {
                self.ensure_fast_timer(ctrl_hwnd);
            }
            self.render_all();
            return true;
        }

        if msg.hwnd == self.settings_surface.hwnd() && self.settings.visible {
            // Note: SettingsPanel::handle_key_down already invokes apply_row_control internally!
            // Do NOT call settings.apply_control(ctrl) again or toggles will double-flip.
            let ctrl = match msg.message {
                WM_KEYDOWN => self.settings.handle_key_down(msg.wParam.0 as u32),
                WM_CHAR => {
                    char::from_u32(msg.wParam.0 as u32).and_then(|ch| self.settings.handle_char(ch))
                }
                _ => None,
            };
            match ctrl {
                Some(SettingsControl::Close) => {
                    self.settings.close();
                    self.apply_settings_action(SettingsAction::Close, ctrl_hwnd);
                }
                Some(SettingsControl::TestAndSaveProfile) => {
                    self.apply_settings_action(SettingsAction::TestAndSaveProfile, ctrl_hwnd);
                }
                Some(_) | None => {
                    let ns = extract_native_settings_from_panel(&self.settings);
                    if ns != self.native_settings {
                        self.apply_native_settings(&ns, true);
                    }
                    self.ensure_fast_timer(ctrl_hwnd);
                }
            }
            self.render_all();
            return true;
        }

        false
    }

    fn dispatch_drawer_prompt(&mut self) {
        let prompt = if self.ai_drawer.draft_prompt.trim().is_empty() {
            if self.ai_drawer.attachments.is_empty() {
                return;
            }
            "Analyze the attached shelf items.".to_string()
        } else {
            std::mem::take(&mut self.ai_drawer.draft_prompt)
        };

        let msg_id = format!("msg-{}", self.ai_drawer.messages.len() + 1);
        self.ai_drawer.push_user_message(&msg_id, &prompt);
        self.ai_drawer
            .begin_assistant_stream(format!("reply-{}", self.ai_drawer.messages.len() + 1));

        let attachments: Vec<String> = self
            .ai_drawer
            .attachments
            .iter()
            .map(|a| a.path.clone())
            .collect();
        let profile_id = Some(self.settings.state.active_profile.id.clone());
        let ipc = Arc::clone(&self.ipc);
        let tx = self.ipc_tx.clone();
        std::thread::spawn(move || {
            let res = ipc.chat_turn(ChatTurnParams {
                session_id: "native-drawer".to_string(),
                prompt,
                attachments,
                profile_id,
            });
            let _ = tx.send(IpcBgResult::ChatTurn(res));
        });
    }

    fn dispatch_approval(&self, approval_id: String, approved: bool) {
        let ipc = Arc::clone(&self.ipc);
        let tx = self.ipc_tx.clone();
        std::thread::spawn(move || {
            let res = ipc
                .resolve_approval(ApprovalResolution {
                    approval_id,
                    approved,
                    reason: None,
                })
                .map(|_| ());
            let _ = tx.send(IpcBgResult::ApprovalResolved(res));
        });
    }

    fn handle_action(&mut self, action: HotkeyAction, ctrl_hwnd: HWND) {
        match action {
            HotkeyAction::ToggleAssistant => {
                if !self.native_settings.hime_ai_enabled && !self.ai_drawer.visible {
                    self.notch
                        .set_ncm_center_alert(Some("Hime AI is disabled".to_string()));
                    self.render_all();
                    self.ensure_fast_timer(ctrl_hwnd);
                    return;
                }
                if !self.ai_drawer.visible {
                    self.sync_shelf_to_views();
                }
                self.ai_drawer.toggle();
                if self.ai_drawer.visible {
                    self.drawer_surface.set_click_through(false);
                    os_shell::set_window_focusable(self.drawer_surface.hwnd(), true);
                    self.drawer_surface.show();
                    os_shell::focus_window(self.drawer_surface.hwnd());
                    self.render_all();
                } else {
                    os_shell::set_window_focusable(self.drawer_surface.hwnd(), false);
                    self.drawer_surface.hide();
                    self.render_all();
                }
            }
            HotkeyAction::ToggleNotch => {
                let next = !self.notch.is_expanded;
                self.notch.set_expanded(next);
                self.update_surface_click_through();
                self.render_all();
            }
            HotkeyAction::ToggleCalendar => {
                if self.notch.is_expanded && self.notch.active_panel == NotchPanel::Calendar {
                    self.notch.set_expanded(false);
                } else {
                    self.notch.set_panel(NotchPanel::Calendar);
                }
                self.update_surface_click_through();
                self.render_all();
            }
            HotkeyAction::ToggleMusic => {
                if self.notch.is_expanded && self.notch.active_panel == NotchPanel::Music {
                    self.notch.set_expanded(false);
                } else {
                    self.notch.set_panel(NotchPanel::Music);
                }
                self.update_surface_click_through();
                self.render_all();
            }
            HotkeyAction::ToggleDock => {
                let mut ns = self.native_settings.clone();
                ns.dock_enabled = !ns.dock_enabled;
                self.apply_native_settings(&ns, true);
                self.update_surface_click_through();
                self.render_all();
            }
            HotkeyAction::ToggleShortcuts => {
                if self.notch.is_expanded && self.notch.active_panel == NotchPanel::Shortcuts {
                    self.notch.set_expanded(false);
                } else {
                    self.notch.set_panel(NotchPanel::Shortcuts);
                }
                self.update_surface_click_through();
                self.render_all();
            }
            HotkeyAction::FocusDockSlot(slot) => {
                self.dock.set_hovered(Some(slot as usize));
                if let Some(item) = self.dock.items().get(slot as usize).cloned() {
                    focus_or_launch_dock_item(&item);
                }
                self.render_all();
            }
            HotkeyAction::StopHime => unsafe {
                PostQuitMessage(0);
            },
        }
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn tick_and_render(&mut self, ctrl_hwnd: HWND) {
        if let Some(evt) = self.quit_event {
            if unsafe { WaitForSingleObject(evt, 0) }.0 == 0 {
                unsafe {
                    PostQuitMessage(0);
                }
                return;
            }
        }

        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32().min(0.05);
        self.last_tick = now;

        for action in hotkey::drain_ll_actions() {
            self.handle_action(action, ctrl_hwnd);
        }

        for ev in d2d::drain_surface_events() {
            self.handle_surface_event(ev, ctrl_hwnd);
        }

        for ev in drain_extra_surface_events() {
            self.handle_extra_surface_event(ev, ctrl_hwnd);
        }

        while let Ok(bg) = self.ipc_rx.try_recv() {
            match bg {
                IpcBgResult::ChatTurn(Ok(res)) => {
                    self.ai_drawer.append_stream_chunk(&res.reply);
                    self.ai_drawer.finish_assistant_stream();
                    if let Some(app_id) = res.pending_approval_id {
                        self.ai_drawer.push_approval_card(ApprovalCard {
                            id: app_id,
                            tool_name: "Tool Execution".to_string(),
                            summary: "Core requested approval".to_string(),
                            risk_level: "medium".to_string(),
                            status: ApprovalStatus::Pending,
                        });
                    }
                }
                IpcBgResult::ChatTurn(Err(err)) => {
                    self.ai_drawer.append_stream_chunk(&format!("[IPC] {err}"));
                    self.ai_drawer.finish_assistant_stream();
                }
                IpcBgResult::ProfileTest(Ok(msg)) => {
                    self.settings.set_status_notice(Some(msg.clone()));
                    self.notch.set_ncm_center_alert(Some(msg));
                }
                IpcBgResult::ProfileTest(Err(err)) => {
                    let msg = format!("IPC: {err}");
                    self.settings.set_status_notice(Some(msg.clone()));
                    self.notch.set_ncm_center_alert(Some(msg));
                }
                IpcBgResult::ApprovalResolved(Ok(())) => {}
                IpcBgResult::ApprovalResolved(Err(err)) => {
                    self.notch
                        .set_ncm_center_alert(Some(format!("Approval: {err}")));
                }
                IpcBgResult::RouterProbe(probe) => {
                    self.settings.state.router_9_detected = probe.available;
                    self.notch.ai_status_text = if probe.available {
                        format!("9Router Connected ({} models)", probe.models.len())
                    } else {
                        "Gateway Standby".to_string()
                    };
                }
                IpcBgResult::WifiScan(list) => {
                    let items = list
                        .into_iter()
                        .map(|w| WifiNetworkItem {
                            ssid: w.ssid,
                            signal: w.signal_percent,
                            secured: w.secured,
                            connected: w.connected,
                        })
                        .collect();
                    self.notch.set_wifi_status(
                        self.notch.wifi_enabled,
                        self.notch.wifi_ssid.clone(),
                        items,
                    );
                }
                IpcBgResult::WifiConnect(ssid, Ok(())) => {
                    self.notch
                        .set_ncm_center_alert(Some(format!("Connected: {ssid}")));
                    self.os_worker.send(OsCommand::ScanWifi);
                }
                IpcBgResult::WifiConnect(_, Err(err)) => {
                    self.notch.set_ncm_center_alert(Some(err));
                }
                IpcBgResult::TrayEntries(entries) => {
                    let count = entries.len() as u32;
                    self.dock.set_tray_items(entries);
                    if count == 0 {
                        let snap = self.telemetry.snapshot();
                        self.dock.set_system_tray_count(snap.tray_count);
                    }
                }
            }
        }

        let snap = self.telemetry.snapshot();
        self.apply_telemetry(&snap, true);
        self.update_surface_click_through();

        if self.native_settings.suppress_native_notifications && self.tick_count % 2 == 0 {
            self.scan_and_suppress_notifications();
        }
        if self.tick_count > 0 && self.tick_count % 20 == 0 {
            self.os_worker.send(OsCommand::ScanTrayEntries);
        }

        let mut animating = false;
        animating |= self.notch.tick(dt);
        animating |= self.dock.tick(dt);
        animating |= self.overlays.tick(dt);
        animating |= self.settings.tick(dt);
        animating |= self.ai_drawer.tick(dt);

        self.sync_overlay_visibility();
        self.render_all();

        self.tick_count = self.tick_count.saturating_add(1);
        if self.tick_count == 12 {
            self.log_tick_diag("tick_3s");
            os_shell::ensure_topmost(self.notch_surface.hwnd());
            if self.dock.visible {
                os_shell::ensure_topmost(self.dock_surface.hwnd());
            }
            if self.overlays.volume.is_visible() {
                os_shell::ensure_topmost(self.overlay_left_surface.hwnd());
            }
            if self.overlays.brightness.is_visible() {
                os_shell::ensure_topmost(self.overlay_right_surface.hwnd());
            }
        }

        if animating {
            self.ensure_fast_timer(ctrl_hwnd);
        } else if self.fast_timer_active {
            self.set_idle_timer(ctrl_hwnd);
        }
    }

    fn scan_and_suppress_notifications(&mut self) {
        let found = find_native_toast_hwnds();
        for hwnd in found {
            unsafe {
                let raw = hwnd.0 as isize;
                if IsWindowVisible(hwnd).as_bool() {
                    let ex = GetWindowLongA(hwnd, GWL_EXSTYLE);
                    let _ = SetWindowLongA(
                        hwnd,
                        GWL_EXSTYLE,
                        ex | WS_EX_LAYERED.0 as i32 | WS_EX_TRANSPARENT.0 as i32,
                    );
                    let _ = SetLayeredWindowAttributes(
                        hwnd,
                        windows::Win32::Foundation::COLORREF(0),
                        0,
                        LWA_ALPHA,
                    );
                    let _ = ShowWindow(hwnd, SW_HIDE);
                    if !self.suppressed_notif_hwnds.contains(&raw) {
                        self.suppressed_notif_hwnds.push(raw);
                    }
                }
            }
        }
    }

    fn restore_suppressed_notifications(&mut self) {
        for raw in self.suppressed_notif_hwnds.drain(..) {
            if raw == 0 {
                continue;
            }
            let hwnd = HWND(raw as *mut std::ffi::c_void);
            unsafe {
                if IsWindow(Some(hwnd)).as_bool() {
                    let ex = GetWindowLongA(hwnd, GWL_EXSTYLE);
                    let cleaned = ex & !(WS_EX_TRANSPARENT.0 as i32) & !(WS_EX_LAYERED.0 as i32);
                    let _ = SetLayeredWindowAttributes(
                        hwnd,
                        windows::Win32::Foundation::COLORREF(0),
                        255,
                        LWA_ALPHA,
                    );
                    let _ = SetWindowLongA(hwnd, GWL_EXSTYLE, cleaned);
                    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                }
            }
        }
    }

    fn ensure_fast_timer(&mut self, ctrl_hwnd: HWND) {
        if !self.fast_timer_active {
            unsafe {
                let _ = SetTimer(Some(ctrl_hwnd), TIMER_ANIM_ID, ANIM_INTERVAL_ACTIVE_MS, None);
            }
            self.fast_timer_active = true;
        }
    }

    fn set_idle_timer(&mut self, ctrl_hwnd: HWND) {
        unsafe {
            let _ = SetTimer(Some(ctrl_hwnd), TIMER_ANIM_ID, ANIM_INTERVAL_IDLE_MS, None);
        }
        self.fast_timer_active = false;
    }

    fn render_surface<F>(&self, name: &str, surface: &NativeSurface, draw: F)
    where
        F: FnOnce(&NativeSurface) -> windows::core::Result<()>,
    {
        surface.begin_draw();
        if let Err(err) = draw(surface) {
            log_render_error(name, "render", &err);
        }
        if let Err(err) = surface.end_draw_and_present() {
            log_render_error(name, "present", &err);
        }
    }

    fn render_all(&self) {
        self.render_surface("HimeNotch", &self.notch_surface, |s| self.notch.render(s));

        if self.dock.visible {
            self.render_surface("HimeDock", &self.dock_surface, |s| self.dock.render(s));
        }

        if self.overlays.volume.is_visible() {
            self.render_surface("HimeOverlayLeft", &self.overlay_left_surface, |s| {
                self.overlays.volume.render(s)
            });
        }

        if self.overlays.brightness.is_visible() {
            self.render_surface("HimeOverlayRight", &self.overlay_right_surface, |s| {
                self.overlays.brightness.render(s)
            });
        }

        if self.settings.visible || self.settings.is_animating() {
            self.render_surface("HimeSettings", &self.settings_surface, |s| {
                self.settings.render(s)
            });
        }

        if self.ai_drawer.visible || self.ai_drawer.is_animating() {
            self.render_surface("HimeAiDrawer", &self.drawer_surface, |s| {
                self.ai_drawer.render(s)
            });
        }
    }

    fn shutdown(&mut self) {
        self.restore_suppressed_notifications();
        self.telemetry.stop();
        let _ = self.shelf.save();
        let _ = self.module_mgr.stop();
        os_shell::restore_taskbar();
        if let Some(evt) = self.quit_event.take() {
            unsafe {
                let _ = CloseHandle(evt);
            }
        }
    }
}

fn collect_live_tray_entries() -> Vec<TrayEntry> {
    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            hKey: isize,
            lpSubKey: PCWSTR,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut isize,
        ) -> i32;
        fn RegEnumKeyExW(
            hKey: isize,
            dwIndex: u32,
            lpName: PWSTR,
            lpcchName: *mut u32,
            lpReserved: *mut u32,
            lpClass: PWSTR,
            lpcchClass: *mut u32,
            lpftLastWriteTime: *mut c_void,
        ) -> i32;
        fn RegQueryValueExW(
            hKey: isize,
            lpValueName: PCWSTR,
            lpReserved: *mut u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> i32;
        fn RegCloseKey(hKey: isize) -> i32;
    }

    const HKEY_CURRENT_USER: isize = -2147483647isize;
    const KEY_READ: u32 = 0x20019;
    const REG_SZ: u32 = 1;
    const REG_DWORD: u32 = 4;

    unsafe fn reg_open(parent: isize, subkey: &str) -> Option<isize> {
        let wide: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();
        let mut out = 0isize;
        if RegOpenKeyExW(parent, PCWSTR(wide.as_ptr()), 0, KEY_READ, &mut out) == 0 {
            Some(out)
        } else {
            None
        }
    }

    unsafe fn reg_string(hkey: isize, name: &str) -> Option<String> {
        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let mut kind = 0u32;
        let mut size = 0u32;
        if RegQueryValueExW(
            hkey,
            PCWSTR(wide.as_ptr()),
            std::ptr::null_mut(),
            &mut kind,
            std::ptr::null_mut(),
            &mut size,
        ) != 0
            || kind != REG_SZ
            || size == 0
            || size > 65536
        {
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        if RegQueryValueExW(
            hkey,
            PCWSTR(wide.as_ptr()),
            std::ptr::null_mut(),
            &mut kind,
            buf.as_mut_ptr(),
            &mut size,
        ) != 0
        {
            return None;
        }
        let chars: Vec<u16> = buf[..size as usize]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .take_while(|&ch| ch != 0)
            .collect();
        String::from_utf16(&chars).ok()
    }

    unsafe fn reg_dword(hkey: isize, name: &str) -> Option<u32> {
        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let mut kind = 0u32;
        let mut val = 0u32;
        let mut size = std::mem::size_of::<u32>() as u32;
        if RegQueryValueExW(
            hkey,
            PCWSTR(wide.as_ptr()),
            std::ptr::null_mut(),
            &mut kind,
            (&mut val as *mut u32).cast::<u8>(),
            &mut size,
        ) == 0
            && kind == REG_DWORD
        {
            Some(val)
        } else {
            None
        }
    }

    unsafe extern "system" fn collect_hwnd_exe(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
        let out = &mut *(lparam.0 as *mut Vec<(isize, String)>);
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid != 0 {
            if let Ok(proc_h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut path_buf = [0u16; 1024];
                let mut len = path_buf.len() as u32;
                if QueryFullProcessImageNameW(
                    proc_h,
                    PROCESS_NAME_WIN32,
                    PWSTR(path_buf.as_mut_ptr()),
                    &mut len,
                )
                .is_ok()
                {
                    let exe = String::from_utf16_lossy(&path_buf[..len as usize]);
                    out.push((hwnd.0 as isize, exe));
                }
                let _ = CloseHandle(proc_h);
            }
        }
        true.into()
    }

    unsafe {
        let Some(root) = reg_open(HKEY_CURRENT_USER, "Control Panel\\NotifyIconSettings") else {
            return Vec::new();
        };

        let mut windows_list: Vec<(isize, String)> = Vec::new();
        let param = LPARAM(&mut windows_list as *mut Vec<(isize, String)> as isize);
        let _ = EnumWindows(Some(collect_hwnd_exe), param);
        let mut prev = None;
        while let Ok(hwnd) = FindWindowExW(Some(HWND_MESSAGE), prev, PCWSTR::null(), PCWSTR::null())
        {
            let _ = collect_hwnd_exe(hwnd, param);
            prev = Some(hwnd);
        }

        let own_exe = std::env::current_exe()
            .ok()
            .map(|p| p.to_string_lossy().to_lowercase());
        let mut seen: HashSet<String> = HashSet::new();
        let mut entries = Vec::new();

        for idx in 0..1024u32 {
            let mut name_buf = [0u16; 256];
            let mut name_len = name_buf.len() as u32;
            if RegEnumKeyExW(
                root,
                idx,
                PWSTR(name_buf.as_mut_ptr()),
                &mut name_len,
                std::ptr::null_mut(),
                PWSTR::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) != 0
            {
                break;
            }
            let subkey = String::from_utf16_lossy(&name_buf[..name_len as usize]);
            let Some(item_key) = reg_open(root, &subkey) else {
                continue;
            };
            let exe_path = reg_string(item_key, "ExecutablePath");
            let icon_guid = reg_string(item_key, "IconGuid");
            let uid = reg_dword(item_key, "UID");
            let _ = RegCloseKey(item_key);

            let Some(path) = exe_path else {
                continue;
            };
            let lower = path.to_lowercase();
            if lower.ends_with("\\explorer.exe") || own_exe.as_deref() == Some(lower.as_str()) {
                continue;
            }
            if seen.contains(&lower) {
                continue;
            }

            let mut nid = NOTIFYICONIDENTIFIER {
                cbSize: std::mem::size_of::<NOTIFYICONIDENTIFIER>() as u32,
                ..Default::default()
            };
            let matched_hwnd = if let Some(guid_str) = icon_guid {
                if let Ok(guid) =
                    GUID::try_from(guid_str.trim_start_matches('{').trim_end_matches('}'))
                {
                    nid.guidItem = guid;
                    if Shell_NotifyIconGetRect(&nid).is_ok() {
                        windows_list
                            .iter()
                            .find(|(_, p)| p.eq_ignore_ascii_case(&path))
                            .map(|(h, _)| *h)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else if let Some(u) = uid {
                nid.uID = u;
                windows_list
                    .iter()
                    .find(|(hwnd, p)| {
                        if !p.eq_ignore_ascii_case(&path) {
                            return false;
                        }
                        nid.hWnd = HWND(*hwnd as *mut _);
                        Shell_NotifyIconGetRect(&nid).is_ok()
                    })
                    .map(|(h, _)| *h)
            } else {
                None
            };

            if let Some(hwnd) = matched_hwnd {
                seen.insert(lower.clone());
                let exe_name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("App");
                let label = telemetry::friendly_app_name(exe_name);
                entries.push(TrayEntry::new(lower, label, path, hwnd));
            }
        }

        let _ = RegCloseKey(root);
        entries
    }
}

fn find_native_toast_hwnds() -> Vec<HWND> {
    unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
        if !IsWindowVisible(hwnd).as_bool() {
            return true.into();
        }
        let mut class_buf = [0u8; 64];
        let clen = GetClassNameA(hwnd, &mut class_buf);
        if clen <= 0 || &class_buf[..clen as usize] != b"Windows.UI.Core.CoreWindow" {
            return true.into();
        }
        let mut title_buf = [0u16; 128];
        let tlen = GetWindowTextW(hwnd, &mut title_buf);
        let title = String::from_utf16_lossy(&title_buf[..tlen.max(0) as usize]);
        if !title.to_ascii_lowercase().contains("notification") {
            return true.into();
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return true.into();
        }
        if let Ok(proc_h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            let mut path_buf = [0u16; 512];
            let mut plen = path_buf.len() as u32;
            let ok = QueryFullProcessImageNameW(
                proc_h,
                PROCESS_NAME_WIN32,
                windows::core::PWSTR(path_buf.as_mut_ptr()),
                &mut plen,
            )
            .is_ok();
            let _ = CloseHandle(proc_h);
            if ok {
                let exe = String::from_utf16_lossy(&path_buf[..plen as usize]).to_ascii_lowercase();
                if exe.ends_with("\\shellexperiencehost.exe") {
                    let out = &mut *(lparam.0 as *mut Vec<HWND>);
                    out.push(hwnd);
                }
            }
        }
        true.into()
    }

    let mut out = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut out as *mut Vec<HWND> as isize));
    }
    out
}

fn parse_notch_edge(raw: &str) -> NotchEdge {
    match raw.trim().to_ascii_lowercase().as_str() {
        "left" => NotchEdge::Left,
        "right" => NotchEdge::Right,
        _ => NotchEdge::Top,
    }
}

fn notch_edge_to_str(edge: NotchEdge) -> &'static str {
    match edge {
        NotchEdge::Top => "top",
        NotchEdge::Left => "left",
        NotchEdge::Right => "right",
    }
}

fn parse_dock_mode(raw: &str) -> DockMode {
    match raw.trim().to_ascii_lowercase().as_str() {
        "auto-hide" | "autohide" => DockMode::AutoHide,
        "smart" => DockMode::Smart,
        _ => DockMode::Fixed,
    }
}

fn capitalize_label(mode: &str) -> String {
    match mode.trim().to_ascii_lowercase().as_str() {
        "smart" => "Smart".to_string(),
        "peek" => "Peek".to_string(),
        _ => "Fixed".to_string(),
    }
}

fn apply_native_settings_to_panel(ns: &NativeSettings, panel: &mut SettingsPanel) {
    let norm = ns.normalized();
    panel.state.notch_behavior = norm.notch_mode.clone();
    panel.state.notch_edge = norm.notch_edge.clone();
    panel.state.notch_mode = NotchMode::from_edge_and_behavior(&norm.notch_edge, &norm.notch_mode);
    panel.state.free_drag_enabled = norm.free_drag_enabled;
    panel.state.time_format_24h = norm.time_format_24h;
    panel.state.music_mode_enabled = norm.music_mode_enabled;
    panel.state.suppress_native_notifications = norm.suppress_native_notifications;
    panel.state.dock_enabled = norm.dock_enabled;
    panel.state.dock_mode = norm.dock_mode.clone();
    panel.state.dock_pinned = norm.dock_pinned.clone();
    panel.state.dock_win_number_enabled = norm.dock_win_number_enabled;
    panel.state.volume_overlay_enabled = norm.volume_overlay_enabled;
    panel.state.brightness_overlay_enabled = norm.brightness_overlay_enabled;
    panel.state.hime_ai_enabled = norm.hime_ai_enabled;
    panel.state.hime_router_url = norm.hime_router_url.clone();
    panel.state.hime_router_model = norm.hime_router_model.clone();

    let preset = if norm.hime_router_url.contains("127.0.0.1:20128") {
        ProviderPreset::NineRouter
    } else if norm.hime_router_url.contains("generativelanguage.googleapis.com") {
        ProviderPreset::GeminiApi
    } else if norm.hime_router_url.contains("api.openai.com") {
        ProviderPreset::OpenAi
    } else {
        ProviderPreset::Custom
    };
    let mut profile = ApiProfileConfig::from_preset(preset);
    profile.base_url = norm.hime_router_url;
    profile.model = norm.hime_router_model;
    panel.state.active_profile = profile;
}

fn extract_native_settings_from_panel(panel: &SettingsPanel) -> NativeSettings {
    NativeSettings {
        notch_mode: panel.state.notch_behavior.clone(),
        free_drag_enabled: panel.state.free_drag_enabled,
        notch_edge: panel.state.notch_edge.clone(),
        time_format_24h: panel.state.time_format_24h,
        music_mode_enabled: panel.state.music_mode_enabled,
        suppress_native_notifications: panel.state.suppress_native_notifications,
        dock_enabled: panel.state.dock_enabled,
        dock_mode: panel.state.dock_mode.clone(),
        dock_pinned: panel.state.dock_pinned.clone(),
        dock_win_number_enabled: panel.state.dock_win_number_enabled,
        volume_overlay_enabled: panel.state.volume_overlay_enabled,
        brightness_overlay_enabled: panel.state.brightness_overlay_enabled,
        hime_ai_enabled: panel.state.hime_ai_enabled,
        hime_router_url: panel.state.hime_router_url.clone(),
        hime_router_model: panel.state.hime_router_model.clone(),
    }
    .normalized()
}

fn build_pinned_dock_items(entries: &[String]) -> Vec<DockItem> {
    let mut out = Vec::with_capacity(entries.len());
    for raw in entries {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let exe_name = Path::new(trimmed)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(trimmed)
            .to_string();
        let id = exe_name.to_lowercase();
        let label = telemetry::friendly_app_name(&exe_name);
        let mut item = DockItem::new(id, label, exe_name, true);
        if trimmed.contains('\\') || trimmed.contains('/') {
            item.exe_path = trimmed.to_string();
        }
        out.push(item);
    }
    out
}

fn shelf_kind_tag(kind: ShelfItemKind) -> &'static str {
    match kind {
        ShelfItemKind::File => "FILE",
        ShelfItemKind::Folder => "DIR",
        ShelfItemKind::Image => "IMG",
        ShelfItemKind::Audio => "AUD",
        ShelfItemKind::Video => "VID",
        ShelfItemKind::Document => "DOC",
        ShelfItemKind::Archive => "ZIP",
        ShelfItemKind::Other => "OBJ",
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn log_diag(msg: &str) {
    use std::io::Write;
    let Some(base) = std::env::var_os("LOCALAPPDATA") else {
        return;
    };
    let dir = std::path::PathBuf::from(base).join("Hime");
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("native-render.log"))
    {
        let _ = writeln!(file, "{msg}");
    }
}

fn log_render_error(surface: &str, stage: &str, err: &windows::core::Error) {
    log_diag(&format!("[{surface}] {stage} failed: {err:?}"));
}

fn dock_surface_bounds(screen: os_shell::ScreenBounds, scale: f32) -> (i32, i32, u32, u32) {
    let w = (DOCK_DIP_WIDTH * scale).round().max(320.0) as u32;
    let w = w.min(screen.width.max(320) as u32);
    let h = (DOCK_DIP_HEIGHT * scale).round().max(96.0) as u32;
    (
        screen.x + (screen.width - w as i32) / 2,
        screen.y + screen.height - h as i32,
        w,
        h,
    )
}

fn left_overlay_bounds(screen: os_shell::ScreenBounds, scale: f32) -> (i32, i32, u32, u32) {
    let w = (OVERLAY_DIP_WIDTH * scale).round() as u32;
    let h = (OVERLAY_DIP_HEIGHT * scale).round() as u32;
    (screen.x, screen.y + (screen.height - h as i32) / 2, w, h)
}

fn right_overlay_bounds(screen: os_shell::ScreenBounds, scale: f32) -> (i32, i32, u32, u32) {
    let w = (OVERLAY_DIP_WIDTH * scale).round() as u32;
    let h = (OVERLAY_DIP_HEIGHT * scale).round() as u32;
    (
        screen.x + screen.width - w as i32,
        screen.y + (screen.height - h as i32) / 2,
        w,
        h,
    )
}

fn notch_surface_bounds(
    edge: NotchEdge,
    screen: os_shell::ScreenBounds,
    scale: f32,
) -> (i32, i32, u32, u32) {
    match edge {
        NotchEdge::Top => {
            let w = (540.0 * scale).round() as u32;
            let h = (320.0 * scale).round() as u32;
            (screen.x + (screen.width - w as i32) / 2, screen.y, w, h)
        }
        NotchEdge::Left => {
            let w = (500.0 * scale).round() as u32;
            let h = (360.0 * scale).round() as u32;
            (screen.x, screen.y + (screen.height - h as i32) / 2, w, h)
        }
        NotchEdge::Right => {
            let w = (500.0 * scale).round() as u32;
            let h = (360.0 * scale).round() as u32;
            (
                screen.x + screen.width - w as i32,
                screen.y + (screen.height - h as i32) / 2,
                w,
                h,
            )
        }
    }
}

fn focus_hwnd(hwnd_raw: isize) -> bool {
    unsafe {
        let wnd = HWND(hwnd_raw as *mut std::ffi::c_void);
        if IsWindow(Some(wnd)).as_bool() {
            if IsIconic(wnd).as_bool() {
                let _ = ShowWindow(wnd, SW_RESTORE);
            } else if !IsWindowVisible(wnd).as_bool() {
                let _ = ShowWindow(wnd, SW_SHOW);
            }
            if !SetForegroundWindow(wnd).as_bool() {
                let _ = BringWindowToTop(wnd);
                let _ = SetForegroundWindow(wnd);
            }
            return true;
        }
    }
    false
}

fn focus_or_launch_dock_item(item: &DockItem) {
    if !item.hwnds.is_empty() {
        let fg = unsafe { GetForegroundWindow().0 as isize };
        let target_raw = if item.hwnds.len() > 1 && item.hwnds[0] == fg {
            item.hwnds[1]
        } else {
            item.hwnds[0]
        };
        if focus_hwnd(target_raw) {
            return;
        }
    }
    let target = if !item.exe_path.trim().is_empty() {
        item.exe_path.trim()
    } else {
        item.executable.trim()
    };
    launch_executable(target);
}

fn launch_executable(target: &str) {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return;
    }
    let wide: Vec<u16> = OsStr::new(trimmed).encode_wide().chain(Some(0)).collect();
    unsafe {
        let _ = ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(wide.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
    }
}

thread_local! {
    static APP_SHELL: RefCell<Option<AppShell>> = const { RefCell::new(None) };
    static HOTKEY_MGR: RefCell<Option<HotkeyManager>> = const { RefCell::new(None) };
}

fn main() {
    if os_shell::has_taskbar_crash_marker() {
        os_shell::restore_taskbar();
    }

    if unsafe { FindWindowW(CONTROLLER_CLASS, w!("Hime Island Native Controller")) }.is_ok() {
        return;
    }

    let hinstance = unsafe { GetModuleHandleW(None) }.expect("GetModuleHandleW");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(controller_wnd_proc),
        hInstance: hinstance.into(),
        lpszClassName: CONTROLLER_CLASS,
        ..Default::default()
    };
    unsafe {
        RegisterClassW(&wc);
    }

    let ctrl_hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CONTROLLER_CLASS,
            w!("Hime Island Native Controller"),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(hinstance.into()),
            None,
        )
    }
    .expect("CreateWindowExW controller");

    add_tray_icon(ctrl_hwnd);

    let shell = AppShell::new(ctrl_hwnd).expect("initialize AppShell");
    APP_SHELL.with(|cell| *cell.borrow_mut() = Some(shell));

    let hk_mgr = HotkeyManager::register(ctrl_hwnd, true);
    HOTKEY_MGR.with(|cell| *cell.borrow_mut() = Some(hk_mgr));

    unsafe {
        let _ = SetTimer(Some(ctrl_hwnd), TIMER_ANIM_ID, ANIM_INTERVAL_IDLE_MS, None);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            if msg.message == WM_KEYDOWN || msg.message == WM_CHAR {
                let handled = APP_SHELL.with(|cell| {
                    cell.borrow_mut()
                        .as_mut()
                        .is_some_and(|shell| shell.handle_window_key_message(&msg, ctrl_hwnd))
                });
                if handled {
                    continue;
                }
            }
            DispatchMessageW(&msg);
        }
        let _ = KillTimer(Some(ctrl_hwnd), TIMER_ANIM_ID);
    }

    HOTKEY_MGR.with(|cell| {
        cell.borrow_mut().take();
    });
    APP_SHELL.with(|cell| {
        if let Some(mut s) = cell.borrow_mut().take() {
            s.shutdown();
        }
    });
    os_shell::restore_taskbar();
    remove_tray_icon(ctrl_hwnd);
}

unsafe extern "system" fn controller_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            let action = HOTKEY_MGR.with(|cell| {
                cell.borrow()
                    .as_ref()
                    .and_then(|mgr| mgr.handle_wm_hotkey(wparam))
            });
            if let Some(act) = action {
                APP_SHELL.with(|cell| {
                    if let Some(shell) = cell.borrow_mut().as_mut() {
                        shell.handle_action(act, hwnd);
                    }
                });
            }
            LRESULT(0)
        }
        WM_SURFACE_EVENT => {
            APP_SHELL.with(|cell| {
                if let Some(shell) = cell.borrow_mut().as_mut() {
                    for ev in d2d::drain_surface_events() {
                        shell.handle_surface_event(ev, hwnd);
                    }
                }
            });
            LRESULT(0)
        }
        WM_EXTRA_SURFACE_EVENT => {
            APP_SHELL.with(|cell| {
                if let Some(shell) = cell.borrow_mut().as_mut() {
                    for ev in drain_extra_surface_events() {
                        shell.handle_extra_surface_event(ev, hwnd);
                    }
                }
            });
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_ANIM_ID {
                APP_SHELL.with(|cell| {
                    if let Some(shell) = cell.borrow_mut().as_mut() {
                        shell.tick_and_render(hwnd);
                    }
                });
            }
            LRESULT(0)
        }
        WM_TRAY_CALLBACK => {
            let event = (lparam.0 as u32) & 0xFFFF;
            if event == WM_RBUTTONUP || event == WM_LBUTTONUP {
                show_tray_menu(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn add_tray_icon(hwnd: HWND) {
    let mut tip = [0u16; 128];
    let text: Vec<u16> = "Hime Island Native".encode_utf16().collect();
    let len = text.len().min(tip.len() - 1);
    tip[..len].copy_from_slice(&text[..len]);

    let icon = unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap_or_default();
    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAY_CALLBACK,
        hIcon: icon,
        szTip: tip,
        ..Default::default()
    };
    unsafe {
        let _ = Shell_NotifyIconW(NIM_ADD, &nid);
    }
}

fn remove_tray_icon(hwnd: HWND) {
    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        ..Default::default()
    };
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}

unsafe fn show_tray_menu(hwnd: HWND) {
    let Ok(menu) = CreatePopupMenu() else {
        return;
    };
    let _ = AppendMenuW(menu, MF_STRING, MENU_TOGGLE_NOTCH, w!("Toggle Notch"));
    let _ = AppendMenuW(menu, MF_STRING, MENU_TOGGLE_DOCK, w!("Toggle Dock"));
    let _ = AppendMenuW(menu, MF_STRING, MENU_TOGGLE_AI_DRAWER, w!("AI Drawer"));
    let _ = AppendMenuW(menu, MF_STRING, MENU_TOGGLE_SETTINGS, w!("Settings"));
    let _ = AppendMenuW(menu, MF_STRING, MENU_EXIT, w!("Exit"));

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    let _ = SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(
        menu,
        TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD,
        pt.x,
        pt.y,
        Some(0),
        hwnd,
        None,
    );
    let _ = DestroyMenu(menu);

    match cmd.0 as usize {
        MENU_TOGGLE_NOTCH => {
            APP_SHELL.with(|cell| {
                if let Some(shell) = cell.borrow_mut().as_mut() {
                    shell.handle_action(HotkeyAction::ToggleNotch, hwnd);
                }
            });
        }
        MENU_TOGGLE_DOCK => {
            APP_SHELL.with(|cell| {
                if let Some(shell) = cell.borrow_mut().as_mut() {
                    shell.handle_action(HotkeyAction::ToggleDock, hwnd);
                }
            });
        }
        MENU_TOGGLE_AI_DRAWER => {
            APP_SHELL.with(|cell| {
                if let Some(shell) = cell.borrow_mut().as_mut() {
                    shell.handle_action(HotkeyAction::ToggleAssistant, hwnd);
                }
            });
        }
        MENU_TOGGLE_SETTINGS => {
            APP_SHELL.with(|cell| {
                if let Some(shell) = cell.borrow_mut().as_mut() {
                    shell.toggle_settings(hwnd);
                }
            });
        }
        MENU_EXIT => {
            APP_SHELL.with(|cell| {
                if let Some(shell) = cell.borrow_mut().as_mut() {
                    shell.handle_action(HotkeyAction::StopHime, hwnd);
                }
            });
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_roundtrip_between_native_settings_and_panel() {
        let mut ns = NativeSettings::default();
        ns.notch_mode = "smart".to_string();
        ns.notch_edge = "left".to_string();
        ns.free_drag_enabled = true;
        ns.time_format_24h = false;
        ns.music_mode_enabled = false;
        ns.suppress_native_notifications = false;
        ns.dock_enabled = false;
        ns.dock_mode = "auto-hide".to_string();
        ns.dock_pinned = vec!["wt.exe".to_string(), r"C:\Apps\Code.exe".to_string()];
        ns.dock_win_number_enabled = false;
        ns.volume_overlay_enabled = false;
        ns.brightness_overlay_enabled = true;
        ns.hime_ai_enabled = false;
        ns.hime_router_url = "https://api.openai.com/v1".to_string();
        ns.hime_router_model = "gpt-4o-mini".to_string();

        let mut panel = SettingsPanel::new();
        apply_native_settings_to_panel(&ns, &mut panel);
        assert_eq!(panel.state.active_profile.preset, ProviderPreset::OpenAi);

        let extracted = extract_native_settings_from_panel(&panel);
        assert_eq!(extracted, ns.normalized());
    }

    #[test]
    fn surface_bounds_fit_expanded_notch_dual_overlays_and_dock() {
        let screen = os_shell::ScreenBounds::new(0, 0, 1920, 1080);

        let (_, _, tw, th) = notch_surface_bounds(NotchEdge::Top, screen, 1.0);
        assert!(tw >= 540);
        assert!(th >= 320);

        let (lx, _, lw, lh) = notch_surface_bounds(NotchEdge::Left, screen, 1.0);
        assert_eq!(lx, 0);
        assert!(lw >= 480);
        assert!(lh >= 320);

        let (rx, _, rw, rh) = notch_surface_bounds(NotchEdge::Right, screen, 1.0);
        assert_eq!(rx + rw as i32, 1920);
        assert!(rw >= 480);
        assert!(rh >= 320);

        let (olx, oly, olw, olh) = left_overlay_bounds(screen, 1.0);
        assert_eq!(olx, 0);
        assert_eq!(olw, 80);
        assert_eq!(olh, 240);
        assert_eq!(oly, (1080 - 240) / 2);

        let (orx, ory, orw, orh) = right_overlay_bounds(screen, 1.0);
        assert_eq!(orx, 1920 - 80);
        assert_eq!(orw, 80);
        assert_eq!(orh, 240);
        assert_eq!(ory, (1080 - 240) / 2);

        let (_, dy, dw, dh) = dock_surface_bounds(screen, 1.0);
        assert_eq!(dw, 960);
        assert_eq!(dh, 124);
        assert_eq!(dy + dh as i32, 1080);
    }

    #[test]
    fn pinned_dock_items_parse_filenames_and_paths() {
        let items = build_pinned_dock_items(&[
            "wt.exe".to_string(),
            r"C:\Program Files\Microsoft VS Code\Code.exe".to_string(),
        ]);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "wt.exe");
        assert!(items[0].pinned);
        assert!(items[0].exe_path.is_empty());
        assert_eq!(items[1].id, "code.exe");
        assert_eq!(items[1].exe_path, r"C:\Program Files\Microsoft VS Code\Code.exe");
    }
}
