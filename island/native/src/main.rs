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
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use windows::core::{w, PCWSTR};
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
use windows::Win32::System::SystemInformation::GetLocalTime;
use windows::Win32::System::Threading::{
    CreateEventW, OpenProcess, QueryFullProcessImageNameW, WaitForSingleObject,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, RegisterHotKey, ReleaseCapture, SetCapture, TrackMouseEvent,
    UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, TME_LEAVE,
    TRACKMOUSEEVENT, VK_LBUTTON, VK_RBUTTON, VK_SHIFT,
};
use windows::Win32::UI::Shell::{
    ShellExecuteW, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, BringWindowToTop, CallWindowProcW, CreatePopupMenu, CreateWindowExW,
    DefWindowProcW, DestroyMenu, DispatchMessageW, EnumWindows, FindWindowW, GetAncestor,
    GetClassNameA, GetCursorPos, GetForegroundWindow, GetMessageW, GetWindowLongA, GetWindowRect,
    GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, KillTimer,
    LoadCursorW, LoadIconW, PeekMessageW, PostMessageW, PostQuitMessage, RegisterClassW, SetCursor,
    SetForegroundWindow, SetLayeredWindowAttributes, SetTimer, SetWindowLongA, SetWindowLongPtrW,
    ShowWindow, TrackPopupMenu, TranslateMessage, GA_ROOT, GWL_EXSTYLE, GWLP_WNDPROC, HTCLIENT,
    IDC_ARROW, IDI_APPLICATION, LWA_ALPHA, MF_STRING, MSG, PM_REMOVE, SW_HIDE, SW_RESTORE,
    SW_SHOW, SW_SHOWNOACTIVATE, SW_SHOWNORMAL, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD,
    WINDOW_EX_STYLE, WM_APP, WM_CHAR, WM_DESTROY, WM_HOTKEY, WM_KEYDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONUP, WM_SETCURSOR, WM_TIMER, WNDCLASSW, WNDPROC,
    WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_OVERLAPPED,
};

use hime_island_native::ai_drawer::{
    AiDrawer, ApprovalCard, ApprovalStatus, DeveloperWorkspaceSummary, DocumentsWorkspaceSummary,
    DrawerAction, DrawerAttachment, DrawerTab,
};
use hime_island_native::d2d::{
    self, HimeExpression, NativeSurface, Rect, SurfaceEvent, SurfaceEventKind, WM_SURFACE_EVENT,
};
use hime_island_native::dock::{
    ContextMenuOp, DockAction, DockBar, DockItem, DockMode, SpaceEntry, TrayEntry,
};
use hime_island_native::hotkey::{self, HotkeyAction, HotkeyManager};
use hime_island_native::ipc_client::{
    AiProfileUpsertParams, ApprovalResolution, ChatTurnParams, ChatTurnResult,
    CreateProductivityTaskParams, DocumentJobRecord, EmergencyPauseRecord, HimeIpcClient, IpcError,
    LyricsGetParams, LyricsResult, LyricsSetOffsetResponse, ProactiveActionResponse,
    ProactiveSettings, ProactiveSettingsSetParams, ProactiveSuggestionItem, ProductivityTaskRecord,
    Routine, RoutineSessionOutcomeResponse, RoutineStartSessionResponse, ScanSecretsParams,
    ScreenTimeSessionRecord, TranslateOptInStatus, TranslateSelectionParams,
    TranslateSetOptInResponse, TranslationResult, IPC_CONTRACT_VERSION,
};
use hime_island_native::module_mgr::{HimeAiModuleManager, RouterProbeResult};
use hime_island_native::notch::{
    NotchAction, NotchEdge, NotchPanel, NotchView, ShelfSummaryItem, WifiNetworkItem,
};
use hime_island_native::notch_workspace::{
    parse_lrc_lines, FocusSessionStatus, LrcLine, ProactiveLevel, TaskCardItem, TaskPriority,
    TaskStatus, TaskTabFilter,
};
use hime_island_native::os_shell;
use hime_island_native::overlay::{sanitize_device_name, DualEdgeOverlays};
use hime_island_native::settings_store::{NativeSettings, SettingsStore};
use hime_island_native::settings_ui::{
    ApiProfileConfig, NotchMode, ProviderPreset, SettingsAction, SettingsControl, SettingsPanel,
    SettingsTab, SETTINGS_WINDOW_HEIGHT, SETTINGS_WINDOW_WIDTH,
};
use hime_island_native::shelf::{FileShelf, ShelfItemKind};
use hime_island_native::telemetry::{
    self, NetworkKind, RunningWindow, TelemetryHandle, TelemetrySnapshot, WifiNetworkInfo,
};
use hime_island_native::theme::ColorThemeMode;

const CONTROLLER_CLASS: PCWSTR = w!("HimeIslandNativeController");
const TRAY_ICON_ID: u32 = 1;
const WM_TRAY_CALLBACK: u32 = WM_APP + 1;
const WM_EXTRA_SURFACE_EVENT: u32 = WM_APP + 3;
const WM_BACKDROP_READY: u32 = WM_APP + 4;
const WM_MOUSELEAVE: u32 = 0x02A3;
const TIMER_ANIM_ID: usize = 1;
const ANIM_INTERVAL_ACTIVE_MS: u32 = 16;
const ANIM_INTERVAL_EDGE_POLL_MS: u32 = 50;
const ANIM_INTERVAL_IDLE_MS: u32 = 250;

const OVERLAY_DIP_WIDTH: f32 = 80.0;
const OVERLAY_DIP_HEIGHT: f32 = 240.0;
const DOCK_DIP_WIDTH: f32 = 1080.0;
const DOCK_DIP_HEIGHT: f32 = 304.0;

const MENU_TOGGLE_NOTCH: usize = 1001;
const MENU_TOGGLE_DOCK: usize = 1002;
const MENU_TOGGLE_AI_DRAWER: usize = 1003;
const MENU_TOGGLE_SETTINGS: usize = 1004;
const MENU_EXIT: usize = 1005;

const HOTKEY_TRANSLATE_CST: i32 = 241; // Ctrl + Shift + T
const VK_T: u32 = 0x54;
const PROACTIVE_EVAL_INTERVAL: Duration = Duration::from_secs(60);
const CORE_BOOTSTRAP_RETRY_INTERVAL: Duration = Duration::from_secs(5);
const MEDIA_TIMELINE_SYNC_INTERVAL: Duration = Duration::from_secs(1);
const BACKDROP_REFRESH_INTERVAL: Duration = Duration::from_millis(33);

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

fn remove_tracked_hwnd(hwnd: HWND) {
    let key = hwnd.0 as isize;
    TRACKED_MOUSE_HWNDS.with(|set| {
        set.borrow_mut().remove(&key);
    });
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
    VirtualDesktops(Vec<os_shell::VirtualDesktopInfo>),
    MediaTimeline {
        title: String,
        artist: String,
        album: Option<String>,
        position_ms: u64,
        duration_ms: u64,
    },
    LyricsLoaded {
        title: String,
        artist: String,
        result: Result<LyricsResult, IpcError>,
    },
    LyricsOffsetSaved(Result<LyricsSetOffsetResponse, IpcError>),
    TasksLoaded(Result<Vec<ProductivityTaskRecord>, IpcError>),
    FocusDataLoaded {
        routines: Result<Vec<Routine>, IpcError>,
        screentime: Result<Vec<ScreenTimeSessionRecord>, IpcError>,
    },
    RoutineStarted(Result<RoutineStartSessionResponse, IpcError>),
    RoutineOutcome(Result<RoutineSessionOutcomeResponse, IpcError>),
    EmergencyPaused(Result<EmergencyPauseRecord, IpcError>),
    ProactiveEvaluated(Result<Vec<ProactiveSuggestionItem>, IpcError>),
    ProactiveResponded(Result<ProactiveActionResponse, IpcError>),
    TranslateOptInLoaded(Result<TranslateOptInStatus, IpcError>),
    TranslateOptInSaved(Result<TranslateSetOptInResponse, IpcError>),
    TranslationDone(Result<TranslationResult, IpcError>),
    ProactiveSettingsLoaded(Result<ProactiveSettings, IpcError>),
    ProactiveBudgetSaved(Result<String, IpcError>),
    MemoryCountLoaded(Result<usize, IpcError>),
    SkillsCountLoaded(Result<usize, IpcError>),
    McpCountLoaded(Result<usize, IpcError>),
    DevCheckLoaded(Result<DeveloperWorkspaceSummary, IpcError>),
    DocsLoaded(Result<DocumentsWorkspaceSummary, IpcError>),
}

enum IpcCommand {
    BootstrapWorkspace,
    RefreshAiSettingsAndCounts,
    GetLyrics {
        title: String,
        artist: String,
        album: Option<String>,
        duration_sec: Option<u32>,
    },
    SetLyricsOffset {
        cache_key: String,
        offset_ms: i32,
    },
    RefreshTasks,
    CreateTask(CreateProductivityTaskParams),
    MoveTask {
        task_id: String,
        column_id: String,
        status: String,
    },
    DeleteTask(String),
    RefreshFocusAndScreenTime,
    FocusStart {
        routine_id: String,
    },
    FocusComplete {
        session_id: String,
    },
    FocusSkip {
        session_id: String,
    },
    FocusEmergencyPause {
        duration_min: i64,
        reason: String,
    },
    EvaluateProactive,
    RespondProactive {
        suggestion_id: String,
        action: String,
    },
    SetTranslateOptIn(bool),
    SetProactiveBudget(String),
    RefreshMemory,
    ReloadSkills,
    ReloadMcp,
    RunDevCheck {
        workspace_path: String,
    },
    RefreshDocs,
}

struct IpcWorker {
    tx: mpsc::Sender<IpcCommand>,
}

impl IpcWorker {
    fn spawn(ipc: Arc<HimeIpcClient>, result_tx: mpsc::Sender<IpcBgResult>) -> Self {
        let (tx, rx) = mpsc::channel::<IpcCommand>();
        let _ = std::thread::Builder::new()
            .name("hime-ipc-worker".to_string())
            .spawn(move || run_ipc_worker(ipc, rx, result_tx));
        Self { tx }
    }

    fn send(&self, cmd: IpcCommand) {
        let _ = self.tx.send(cmd);
    }
}

fn run_ipc_worker(
    ipc: Arc<HimeIpcClient>,
    rx: mpsc::Receiver<IpcCommand>,
    result_tx: mpsc::Sender<IpcBgResult>,
) {
    while let Ok(cmd) = rx.recv() {
        match cmd {
            IpcCommand::BootstrapWorkspace => {
                let _ = result_tx.send(IpcBgResult::TasksLoaded(
                    ipc.list_productivity_tasks(None),
                ));
                let _ = result_tx.send(IpcBgResult::FocusDataLoaded {
                    routines: ipc.list_routines(false),
                    screentime: ipc.list_screentime_sessions(Some(100)),
                });
                send_ai_settings_and_counts(&ipc, &result_tx);
            }
            IpcCommand::RefreshAiSettingsAndCounts => {
                send_ai_settings_and_counts(&ipc, &result_tx);
            }
            IpcCommand::GetLyrics {
                title,
                artist,
                album,
                duration_sec,
            } => {
                let res = ipc.get_lyrics(LyricsGetParams {
                    title: title.clone(),
                    artist: artist.clone(),
                    album,
                    duration_sec,
                    allow_online: true,
                });
                let _ = result_tx.send(IpcBgResult::LyricsLoaded {
                    title,
                    artist,
                    result: res,
                });
            }
            IpcCommand::SetLyricsOffset {
                cache_key,
                offset_ms,
            } => {
                let res = ipc.set_lyrics_offset(&cache_key, offset_ms);
                let _ = result_tx.send(IpcBgResult::LyricsOffsetSaved(res));
            }
            IpcCommand::RefreshTasks => {
                let _ = result_tx.send(IpcBgResult::TasksLoaded(
                    ipc.list_productivity_tasks(None),
                ));
            }
            IpcCommand::CreateTask(params) => {
                let res = ipc
                    .create_productivity_task(params)
                    .and_then(|_| ipc.list_productivity_tasks(None));
                let _ = result_tx.send(IpcBgResult::TasksLoaded(res));
            }
            IpcCommand::MoveTask {
                task_id,
                column_id,
                status,
            } => {
                let res = ipc
                    .move_productivity_task(&task_id, &column_id, &status)
                    .and_then(|_| ipc.list_productivity_tasks(None));
                let _ = result_tx.send(IpcBgResult::TasksLoaded(res));
            }
            IpcCommand::DeleteTask(task_id) => {
                let res = ipc
                    .delete_productivity_task(&task_id)
                    .and_then(|_| ipc.list_productivity_tasks(None));
                let _ = result_tx.send(IpcBgResult::TasksLoaded(res));
            }
            IpcCommand::RefreshFocusAndScreenTime => {
                let _ = result_tx.send(IpcBgResult::FocusDataLoaded {
                    routines: ipc.list_routines(false),
                    screentime: ipc.list_screentime_sessions(Some(100)),
                });
            }
            IpcCommand::FocusStart { routine_id } => {
                let res = ipc.start_routine_session(&routine_id, None);
                let _ = result_tx.send(IpcBgResult::RoutineStarted(res));
            }
            IpcCommand::FocusComplete { session_id } => {
                let res = ipc.complete_routine_session(&session_id);
                let _ = result_tx.send(IpcBgResult::RoutineOutcome(res));
                let _ = result_tx.send(IpcBgResult::FocusDataLoaded {
                    routines: ipc.list_routines(false),
                    screentime: ipc.list_screentime_sessions(Some(100)),
                });
            }
            IpcCommand::FocusSkip { session_id } => {
                let res = ipc.skip_routine_session(&session_id, Some("User skipped"));
                let _ = result_tx.send(IpcBgResult::RoutineOutcome(res));
                let _ = result_tx.send(IpcBgResult::FocusDataLoaded {
                    routines: ipc.list_routines(false),
                    screentime: ipc.list_screentime_sessions(Some(100)),
                });
            }
            IpcCommand::FocusEmergencyPause {
                duration_min,
                reason,
            } => {
                let res = ipc.trigger_emergency_pause(duration_min, &reason);
                let _ = result_tx.send(IpcBgResult::EmergencyPaused(res));
            }
            IpcCommand::EvaluateProactive => {
                let res = ipc.evaluate_proactive_suggestions();
                let _ = result_tx.send(IpcBgResult::ProactiveEvaluated(res));
            }
            IpcCommand::RespondProactive {
                suggestion_id,
                action,
            } => {
                let res = ipc.respond_proactive_suggestion(&suggestion_id, &action);
                let _ = result_tx.send(IpcBgResult::ProactiveResponded(res));
            }
            IpcCommand::SetTranslateOptIn(enabled) => {
                let res = ipc.set_translate_opt_in(enabled);
                let _ = result_tx.send(IpcBgResult::TranslateOptInSaved(res));
            }
            IpcCommand::SetProactiveBudget(budget) => {
                let current = ipc.get_proactive_settings().ok();
                let params = ProactiveSettingsSetParams {
                    is_enabled: current.as_ref().is_none_or(|s| s.is_enabled),
                    attention_mode: budget_to_attention_mode(&budget).to_string(),
                    quiet_hour_start: current.as_ref().map_or(23, |s| s.quiet_hour_start),
                    quiet_hour_end: current.as_ref().map_or(6, |s| s.quiet_hour_end),
                    cooldown_minutes: current.as_ref().map_or(30, |s| s.cooldown_minutes),
                    mute_hours: None,
                };
                let res = ipc.set_proactive_settings(params).map(|_| budget);
                let _ = result_tx.send(IpcBgResult::ProactiveBudgetSaved(res));
            }
            IpcCommand::RefreshMemory => {
                let res = ipc.list_memory(None, None, Some(200)).map(|m| m.len());
                let _ = result_tx.send(IpcBgResult::MemoryCountLoaded(res));
            }
            IpcCommand::ReloadSkills => {
                let res = ipc.reload_skills().map(|s| s.len());
                let _ = result_tx.send(IpcBgResult::SkillsCountLoaded(res));
            }
            IpcCommand::ReloadMcp => {
                let res = ipc
                    .reload_mcp()
                    .and_then(|_| ipc.list_mcp())
                    .map(|s| s.len());
                let _ = result_tx.send(IpcBgResult::McpCountLoaded(res));
            }
            IpcCommand::RunDevCheck { workspace_path } => {
                let res = run_dev_workspace_check(&ipc, &workspace_path);
                let _ = result_tx.send(IpcBgResult::DevCheckLoaded(res));
            }
            IpcCommand::RefreshDocs => {
                let res = ipc.list_document_jobs().map(|jobs| summarize_document_jobs(&jobs));
                let _ = result_tx.send(IpcBgResult::DocsLoaded(res));
            }
        }
    }
}

fn send_ai_settings_and_counts(ipc: &HimeIpcClient, result_tx: &mpsc::Sender<IpcBgResult>) {
    let _ = result_tx.send(IpcBgResult::TranslateOptInLoaded(
        ipc.get_translate_opt_in(),
    ));
    let _ = result_tx.send(IpcBgResult::ProactiveSettingsLoaded(
        ipc.get_proactive_settings(),
    ));
    let _ = result_tx.send(IpcBgResult::MemoryCountLoaded(
        ipc.list_memory(None, None, Some(200)).map(|m| m.len()),
    ));
    let _ = result_tx.send(IpcBgResult::SkillsCountLoaded(
        ipc.list_skills().map(|s| s.len()),
    ));
    let _ = result_tx.send(IpcBgResult::McpCountLoaded(
        ipc.list_mcp().map(|s| s.len()),
    ));
}

fn run_dev_workspace_check(
    ipc: &HimeIpcClient,
    workspace_path: &str,
) -> Result<DeveloperWorkspaceSummary, IpcError> {
    let git = ipc.developer_git_status(workspace_path)?;
    let mut candidate_files: Vec<String> = Vec::new();
    for path in git
        .staged_files
        .iter()
        .chain(git.modified_files.iter())
        .chain(git.untracked_files.iter())
    {
        if !candidate_files.contains(path) {
            candidate_files.push(path.clone());
        }
        if candidate_files.len() >= 20 {
            break;
        }
    }

    let mut total_findings = 0usize;
    let mut scan_error: Option<String> = None;
    for rel in &candidate_files {
        let full = Path::new(workspace_path).join(rel);
        if !full.is_file() {
            continue;
        }
        match ipc.developer_scan_secrets(ScanSecretsParams {
            file_path: Some(rel.clone()),
            path: Some(full.to_string_lossy().into_owned()),
            content: None,
        }) {
            Ok(scan) => total_findings = total_findings.saturating_add(scan.count),
            Err(err) => {
                scan_error = Some(err.to_string());
                break;
            }
        }
    }

    let status_text = if let Some(err) = scan_error {
        format!("Git ok; scan err: {err}")
    } else if total_findings > 0 {
        format!("{total_findings} secret finding(s) detected")
    } else if git.is_clean {
        "Clean • 0 secret findings".to_string()
    } else {
        format!("{} file(s) scanned • 0 secrets", candidate_files.len())
    };

    Ok(DeveloperWorkspaceSummary {
        branch: git.current_branch,
        staged_count: git.staged_files.len(),
        unstaged_count: git.modified_files.len(),
        untracked_count: git.untracked_files.len(),
        is_clean: git.is_clean,
        secret_findings_count: total_findings,
        last_check_status: Some(status_text),
    })
}

enum OsCommand {
    SetVolume { percent: u8, unmute: bool },
    ToggleMute,
    SetBrightness(u8),
    MediaPlayPause,
    MediaNext,
    MediaPrev,
    MediaSeek(u64),
    SyncMediaTimeline,
    SetWifiRadio(bool),
    SetBluetoothRadio(bool),
    ScanWifi,
    ConnectWifi(String),
    Probe9Router,
    ScanTrayEntries,
    RefreshVirtualDesktops,
    SwitchVirtualDesktop(usize),
    CreateVirtualDesktop,
    OpenNotificationCenter,
    CloseWindows(Vec<isize>),
    EndTaskWindows(Vec<isize>),
    EndTaskPids(Vec<u32>),
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
                OsCommand::SyncMediaTimeline => {
                    if let Some(session) = get_smtc_session(&mut smtc_mgr) {
                        if let Ok(props) = session.TryGetMediaPropertiesAsync().and_then(|op| op.get()) {
                            let title = props.Title().unwrap_or_default().to_string();
                            let artist = props.Artist().unwrap_or_default().to_string();
                            let album = props
                                .AlbumTitle()
                                .ok()
                                .map(|s| s.to_string())
                                .filter(|s| !s.trim().is_empty());
                            if let Ok(tl) = session.GetTimelineProperties() {
                                let pos_ticks = tl.Position().map(|t| t.Duration).unwrap_or(0).max(0);
                                let end_ticks = tl.EndTime().map(|t| t.Duration).unwrap_or(0).max(0);
                                let _ = result_tx.send(IpcBgResult::MediaTimeline {
                                    title,
                                    artist,
                                    album,
                                    position_ms: (pos_ticks / 10_000) as u64,
                                    duration_ms: (end_ticks / 10_000) as u64,
                                });
                            }
                        }
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
                OsCommand::RefreshVirtualDesktops => {
                    let spaces = os_shell::list_virtual_desktops();
                    let _ = result_tx.send(IpcBgResult::VirtualDesktops(spaces));
                }
                OsCommand::SwitchVirtualDesktop(index) => {
                    let _ = os_shell::switch_virtual_desktop(index);
                    std::thread::sleep(Duration::from_millis(120));
                    let spaces = os_shell::list_virtual_desktops();
                    let _ = result_tx.send(IpcBgResult::VirtualDesktops(spaces));
                }
                OsCommand::CreateVirtualDesktop => {
                    let _ = os_shell::create_virtual_desktop();
                    std::thread::sleep(Duration::from_millis(150));
                    let spaces = os_shell::list_virtual_desktops();
                    let _ = result_tx.send(IpcBgResult::VirtualDesktops(spaces));
                }
                OsCommand::OpenNotificationCenter => {
                    let _ = os_shell::open_notification_center();
                }
                OsCommand::CloseWindows(hwnds) => {
                    os_shell::close_windows_by_hwnd(&hwnds);
                }
                OsCommand::EndTaskWindows(hwnds) => {
                    os_shell::terminate_processes_by_hwnd(&hwnds);
                }
                OsCommand::EndTaskPids(pids) => {
                    os_shell::terminate_processes_by_pid(&pids);
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

const STATIC_COMPOSITE_REFRESH_INTERVAL: Duration = Duration::from_millis(33);
const BACKDROP_TARGET_DOCK: usize = 0;
const BACKDROP_TARGET_SETTINGS: usize = 1;
const BACKDROP_TARGET_DRAWER: usize = 2;

#[derive(Default)]
struct BackdropSlot {
    last_at: Option<Instant>,
    last_hash: Option<u64>,
    layout_hash: Option<u64>,
    from_wake: bool,
}

fn should_skip_settled_backdrop_refresh(
    slot: &BackdropSlot,
    now: Instant,
    cur_layout: u64,
    force: bool,
) -> bool {
    if force || slot.last_hash.is_none() {
        return false;
    }
    if slot.layout_hash == Some(cur_layout) {
        return slot
            .last_at
            .is_some_and(|t| now.duration_since(t) < STATIC_COMPOSITE_REFRESH_INTERVAL);
    }
    false
}

struct BackdropShared {
    shutdown: bool,
    ctrl_hwnd: isize,
    exclude_hwnds: Vec<isize>,
    targets: [Option<(i32, i32, u32, u32)>; 3],
    last_hashes: [Option<u64>; 3],
    pending_frames: [Option<(u64, os_shell::CapturedBackdrop)>; 3],
}

impl BackdropShared {
    fn new(ctrl_hwnd: isize) -> Self {
        Self {
            shutdown: false,
            ctrl_hwnd,
            exclude_hwnds: Vec::new(),
            targets: [None, None, None],
            last_hashes: [None, None, None],
            pending_frames: [None, None, None],
        }
    }
}

struct BackdropWorker {
    shared: Arc<(Mutex<BackdropShared>, Condvar)>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl BackdropWorker {
    fn spawn(ctrl_hwnd: HWND) -> Self {
        let shared = Arc::new((Mutex::new(BackdropShared::new(ctrl_hwnd.0 as isize)), Condvar::new()));
        let bg = Arc::clone(&shared);
        let handle = std::thread::Builder::new()
            .name("hime-backdrop-worker".to_string())
            .spawn(move || run_backdrop_worker(bg))
            .ok();
        Self { shared, handle }
    }

    fn sync_targets(&self, targets: [Option<(i32, i32, u32, u32)>; 3], exclude_hwnds: Vec<isize>) {
        let (lock, cvar) = &*self.shared;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        let was_idle = state.targets.iter().all(Option::is_none);
        if state.targets != targets || state.exclude_hwnds != exclude_hwnds {
            state.targets = targets;
            state.exclude_hwnds = exclude_hwnds;
            for idx in 0..3 {
                if state.targets[idx].is_none() {
                    state.last_hashes[idx] = None;
                    state.pending_frames[idx] = None;
                }
            }
            if was_idle && state.targets.iter().any(Option::is_some) {
                cvar.notify_one();
            }
        }
    }

    fn seed_hash(&self, idx: usize, hash: u64) {
        let (lock, _) = &*self.shared;
        if let Ok(mut state) = lock.lock() {
            state.last_hashes[idx] = Some(hash);
            state.pending_frames[idx] = None;
        }
    }

    fn take_pending(&self) -> [Option<(u64, os_shell::CapturedBackdrop)>; 3] {
        let (lock, _) = &*self.shared;
        let Ok(mut state) = lock.lock() else {
            return [None, None, None];
        };
        [
            state.pending_frames[0].take(),
            state.pending_frames[1].take(),
            state.pending_frames[2].take(),
        ]
    }

    fn shutdown(&mut self) {
        let (lock, cvar) = &*self.shared;
        if let Ok(mut state) = lock.lock() {
            state.shutdown = true;
            cvar.notify_all();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_backdrop_worker(shared: Arc<(Mutex<BackdropShared>, Condvar)>) {
    let mut capturer = os_shell::BackdropCapturer::new();
    let (lock, cvar) = &*shared;

    loop {
        let (targets, exclude_hwnds, prev_hashes, ctrl_hwnd) = {
            let mut state = match lock.lock() {
                Ok(g) => g,
                Err(_) => break,
            };
            while !state.shutdown && state.targets.iter().all(Option::is_none) {
                state = match cvar.wait(state) {
                    Ok(g) => g,
                    Err(_) => return,
                };
            }
            if state.shutdown {
                break;
            }
            (
                state.targets,
                state.exclude_hwnds.clone(),
                state.last_hashes,
                state.ctrl_hwnd,
            )
        };

        let cycle_start = Instant::now();
        let mut updates: [Option<(u64, os_shell::CapturedBackdrop)>; 3] = [None, None, None];
        let mut any_changed = false;

        for (idx, target) in targets.into_iter().enumerate() {
            let Some((x, y, w, h)) = target else {
                continue;
            };
            let Some(cap) = capturer.capture_region(x, y, w, h, &exclude_hwnds, false) else {
                continue;
            };
            let h_val = hash_backdrop(cap.width, cap.height, &cap.bgra);
            if prev_hashes[idx] != Some(h_val) {
                updates[idx] = Some((h_val, cap));
                any_changed = true;
            }
        }

        if any_changed {
            let mut post_notify = false;
            if let Ok(mut state) = lock.lock() {
                if !state.shutdown {
                    for idx in 0..3 {
                        if let Some((h_val, cap)) = updates[idx].take() {
                            if state.targets[idx].is_some() {
                                state.last_hashes[idx] = Some(h_val);
                                state.pending_frames[idx] = Some((h_val, cap));
                                post_notify = true;
                            }
                        }
                    }
                }
            }
            if post_notify && ctrl_hwnd != 0 {
                unsafe {
                    let _ = PostMessageW(
                        Some(HWND(ctrl_hwnd as *mut std::ffi::c_void)),
                        WM_BACKDROP_READY,
                        WPARAM(0),
                        LPARAM(0),
                    );
                }
            }
        }

        let elapsed = cycle_start.elapsed();
        if elapsed < BACKDROP_REFRESH_INTERVAL {
            let rem = BACKDROP_REFRESH_INTERVAL - elapsed;
            if let Ok(state) = lock.lock() {
                if !state.shutdown {
                    let _ = cvar.wait_timeout(state, rem);
                }
            }
        }
    }
}

fn hash_backdrop(width: u32, height: u32, bgra: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325 ^ ((width as u64) << 32) ^ (height as u64);
    for chunk in bgra.chunks_exact(16) {
        let word = u64::from_le_bytes([
            chunk[0], chunk[1], chunk[2], chunk[3], chunk[8], chunk[9], chunk[10], chunk[11],
        ]);
        h ^= word;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

fn update_surface_backdrop(
    capturer: &mut os_shell::BackdropCapturer,
    surface: &NativeSurface,
    slot: &mut BackdropSlot,
    bounds: (i32, i32, u32, u32),
    exclude_hwnds: &[isize],
    is_surface_hidden: bool,
    force: bool,
) -> bool {
    let now = Instant::now();
    if !force
        && slot
            .last_at
            .is_some_and(|t| now.duration_since(t) < BACKDROP_REFRESH_INTERVAL)
    {
        return false;
    }
    let cur_layout = os_shell::visible_window_layout_hash(bounds, exclude_hwnds);
    if !is_surface_hidden && should_skip_settled_backdrop_refresh(slot, now, cur_layout, force) {
        slot.last_at = Some(now);
        return false;
    }
    slot.last_at = Some(now);
    let Some(cap) = capturer.capture_region(
        bounds.0,
        bounds.1,
        bounds.2,
        bounds.3,
        exclude_hwnds,
        is_surface_hidden,
    ) else {
        return false;
    };
    slot.layout_hash = Some(cur_layout);
    slot.from_wake = is_surface_hidden;
    let h = hash_backdrop(cap.width, cap.height, &cap.bgra);
    if slot.last_hash == Some(h) {
        return false;
    }
    slot.last_hash = Some(h);
    surface.set_backdrop_pixels(cap.width, cap.height, &cap.bgra);
    true
}

struct AppShell {
    notch: NotchView,
    dock: DockBar,
    overlays: DualEdgeOverlays,
    settings: SettingsPanel,
    native_settings: NativeSettings,
    ai_drawer: AiDrawer,
    shelf: FileShelf,
    telemetry: std::mem::ManuallyDrop<TelemetryHandle>,
    module_mgr: HimeAiModuleManager,
    ipc: Option<Arc<HimeIpcClient>>,
    ipc_worker: Option<IpcWorker>,
    ipc_tx: mpsc::Sender<IpcBgResult>,
    ipc_rx: mpsc::Receiver<IpcBgResult>,
    os_worker: OsWorker,
    notch_surface: NativeSurface,
    dock_surface: NativeSurface,
    dock_tooltip_surface: Option<NativeSurface>,
    last_dock_tooltip: Option<(i32, i32, u32, u32, String, u8)>,
    overlay_left_surface: NativeSurface,
    overlay_right_surface: NativeSurface,
    settings_surface: Option<NativeSurface>,
    drawer_surface: Option<NativeSurface>,
    backdrop_capturer: Option<os_shell::BackdropCapturer>,
    backdrop_worker: Option<BackdropWorker>,
    backdrop_dock: BackdropSlot,
    backdrop_settings: BackdropSlot,
    backdrop_drawer: BackdropSlot,
    last_tick: Instant,
    startup_at: Instant,
    startup_trim_done: bool,
    fast_timer_active: bool,
    dirty_notch: bool,
    dirty_dock: bool,
    dirty_overlay_left: bool,
    dirty_overlay_right: bool,
    dirty_settings: bool,
    dirty_drawer: bool,
    last_telemetry_snap: Option<TelemetrySnapshot>,
    last_snap_volume: Option<(u8, bool)>,
    last_snap_brightness: Option<u8>,
    last_volume_write_at: Option<Instant>,
    last_brightness_write_at: Option<Instant>,
    last_audio_device: Option<String>,
    last_running_windows: Vec<RunningWindow>,
    last_music_track: Option<(String, String)>,
    current_lyrics_cache_key: Option<String>,
    active_routine_session_id: Option<String>,
    last_proactive_id: Option<String>,
    dismissed_proactive_ids: HashSet<String>,
    last_proactive_eval_at: Instant,
    last_media_timeline_sync_at: Instant,
    last_core_bootstrap_at: Option<Instant>,
    core_workspace_seeded: bool,
    workspace_checked_once: bool,
    dragging_notch: bool,
    dragging_notch_slider: bool,
    drag_moved: bool,
    drag_start_cursor: (i32, i32),
    notch_leave_at: Option<Instant>,
    was_mouse_down: bool,
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
        os_shell::enable_file_drop(notch_surface.hwnd(), true);

        let (dx, dy, dock_w, dock_h) = dock_surface_bounds(screen, 1.0);
        let mut dock_surface = NativeSurface::new_gpu_acrylic("HimeDock", dx, dy, dock_w, dock_h)?;
        dock_surface.set_system_light_theme(os_shell::system_uses_light_theme());
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

        d2d::set_surface_event_target(ctrl_hwnd);
        EXTRA_EVENT_TARGET.store(ctrl_hwnd.0 as isize, Ordering::Relaxed);

        let shelf = FileShelf::load().unwrap_or_else(|_| FileShelf::new());
        let mut module_mgr = HimeAiModuleManager::with_default_dir();

        let mut settings = SettingsPanel::new();
        settings.state.router_9_detected = false;
        apply_native_settings_to_panel(&native_settings, &mut settings);

        let telemetry = std::mem::ManuallyDrop::new(TelemetryHandle::start());
        let (ipc_tx, ipc_rx) = mpsc::channel();
        let os_worker = OsWorker::spawn(ipc_tx.clone());

        let (ipc, ipc_worker) = if native_settings.hime_ai_enabled {
            let _ = module_mgr.spawn();
            os_worker.send(OsCommand::Probe9Router);
            let client = Arc::new(HimeIpcClient::with_default_pipe());
            let worker = IpcWorker::spawn(Arc::clone(&client), ipc_tx.clone());
            (Some(client), Some(worker))
        } else {
            (None, None)
        };

        let quit_event = unsafe { CreateEventW(None, true, false, w!("HimeQuit")) }.ok();

        let mut notch = NotchView::new(initial_edge);
        notch.ai_status_text = "Ready (Local)".to_string();

        let dock = DockBar::new();
        let overlays = DualEdgeOverlays::new();
        let now = Instant::now();

        let need_sw_backdrop = !dock_surface.is_gpu_acrylic();
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
            ipc_worker,
            ipc_tx,
            ipc_rx,
            os_worker,
            notch_surface,
            dock_surface,
            dock_tooltip_surface: None,
            last_dock_tooltip: None,
            overlay_left_surface,
            overlay_right_surface,
            settings_surface: None,
            drawer_surface: None,
            backdrop_capturer: need_sw_backdrop.then(os_shell::BackdropCapturer::new),
            backdrop_worker: need_sw_backdrop.then(|| BackdropWorker::spawn(ctrl_hwnd)),
            backdrop_dock: BackdropSlot::default(),
            backdrop_settings: BackdropSlot::default(),
            backdrop_drawer: BackdropSlot::default(),
            last_tick: now,
            startup_at: now,
            startup_trim_done: false,
            fast_timer_active: false,
            dirty_notch: true,
            dirty_dock: true,
            dirty_overlay_left: true,
            dirty_overlay_right: true,
            dirty_settings: false,
            dirty_drawer: false,
            last_telemetry_snap: None,
            last_snap_volume: None,
            last_snap_brightness: None,
            last_volume_write_at: None,
            last_brightness_write_at: None,
            last_audio_device: None,
            last_running_windows: Vec::new(),
            last_music_track: None,
            current_lyrics_cache_key: None,
            active_routine_session_id: None,
            last_proactive_id: None,
            dismissed_proactive_ids: HashSet::new(),
            last_proactive_eval_at: now,
            last_media_timeline_sync_at: now,
            last_core_bootstrap_at: if native_settings.hime_ai_enabled {
                Some(now)
            } else {
                None
            },
            core_workspace_seeded: false,
            workspace_checked_once: false,
            dragging_notch: false,
            dragging_notch_slider: false,
            drag_moved: false,
            drag_start_cursor: (0, 0),
            notch_leave_at: None,
            was_mouse_down: false,
            suppressed_notif_hwnds: Vec::new(),
            quit_event,
            tick_count: 0,
        };

        shell.apply_native_settings(&native_settings, false);
        shell.sync_shelf_to_views();
        shell.os_worker.send(OsCommand::ScanTrayEntries);
        shell.os_worker.send(OsCommand::RefreshVirtualDesktops);
        if native_settings.hime_ai_enabled {
            shell.send_ipc(IpcCommand::BootstrapWorkspace);
        }

        let initial_snap = shell.telemetry.snapshot();
        shell.apply_telemetry(&initial_snap, false);
        shell.refresh_all_backdrops_startup();
        shell.sync_overlay_visibility();
        shell.notch_surface.show();
        if shell.dock.visible {
            shell.dock_surface.show();
        }
        shell.update_surface_click_through();
        shell.mark_all_dirty();
        shell.render_all();
        shell.sync_backdrop_worker_targets();
        shell.log_startup_diag();
        Ok(shell)
    }

    fn apply_color_theme_to_surfaces(&self) {
        let mode = ColorThemeMode::from_str_lossy(&self.native_settings.color_theme);
        self.dock_surface.set_color_theme_mode(mode);
        if let Some(ref surf) = self.settings_surface {
            surf.set_color_theme_mode(mode);
        }
        if let Some(ref surf) = self.drawer_surface {
            surf.set_color_theme_mode(mode);
        }
    }

    fn own_hwnds(&self) -> Vec<isize> {
        let mut hwnds = vec![
            self.notch_surface.hwnd().0 as isize,
            self.dock_surface.hwnd().0 as isize,
            self.overlay_left_surface.hwnd().0 as isize,
            self.overlay_right_surface.hwnd().0 as isize,
        ];
        if let Some(ref s) = self.settings_surface {
            hwnds.push(s.hwnd().0 as isize);
        }
        if let Some(ref s) = self.drawer_surface {
            hwnds.push(s.hwnd().0 as isize);
        }
        if let Some(ref s) = self.dock_tooltip_surface {
            hwnds.push(s.hwnd().0 as isize);
        }
        hwnds
    }

    fn refresh_all_backdrops_startup(&mut self) {
        if self.dock_surface.is_gpu_acrylic() {
            return;
        }
        let exclude = self.own_hwnds();
        let Some(ref mut capturer) = self.backdrop_capturer else {
            return;
        };
        let screen = os_shell::primary_screen_bounds();
        let d_scale = (self.dock_surface.dpi() / 96.0).max(0.5);
        let d_bounds = dock_surface_bounds(screen, d_scale);
        let _ = update_surface_backdrop(
            capturer,
            &self.dock_surface,
            &mut self.backdrop_dock,
            d_bounds,
            &exclude,
            true,
            true,
        );
    }

    fn capture_dock_backdrop(&mut self, force: bool, is_hidden: bool) {
        if self.dock_surface.is_gpu_acrylic() {
            return;
        }
        let exclude = self.own_hwnds();
        let Some(ref mut capturer) = self.backdrop_capturer else {
            return;
        };
        let screen = os_shell::primary_screen_bounds();
        let scale = (self.dock_surface.dpi() / 96.0).max(0.5);
        let bounds = dock_surface_bounds(screen, scale);
        if update_surface_backdrop(
            capturer,
            &self.dock_surface,
            &mut self.backdrop_dock,
            bounds,
            &exclude,
            is_hidden,
            force,
        ) {
            if let (Some(h), Some(ref worker)) = (self.backdrop_dock.last_hash, &self.backdrop_worker) {
                worker.seed_hash(BACKDROP_TARGET_DOCK, h);
            }
            self.dirty_dock = true;
        }
    }

    /// Ensures the Dock has a crisp backdrop before waking from auto-hide/smart-hide or enabling,
    /// using `StretchBlt` only while completely invisible (`show_progress <= 0.02`) and keeping the
    /// cached wake capture during mid-recede re-hovers so no semi-transparent dock pixels bleed in.
    fn ensure_dock_wake_backdrop(&mut self) {
        if self.dock_surface.is_gpu_acrylic() {
            return;
        }
        if self.dock.show_progress.value <= 0.02 {
            self.capture_dock_backdrop(true, true);
        } else if self.backdrop_dock.last_hash.is_none() {
            self.capture_dock_backdrop(true, false);
        }
    }

    fn capture_settings_backdrop(&mut self, force: bool, is_hidden: bool) {
        let exclude = self.own_hwnds();
        let Some(ref surf) = self.settings_surface else {
            return;
        };
        if surf.is_gpu_acrylic() {
            return;
        }
        let Some(ref mut capturer) = self.backdrop_capturer else {
            return;
        };
        let mut r = RECT::default();
        if unsafe { GetWindowRect(surf.hwnd(), &mut r).is_err() } {
            return;
        }
        let bounds = (
            r.left,
            r.top,
            (r.right - r.left).max(1) as u32,
            (r.bottom - r.top).max(1) as u32,
        );
        if update_surface_backdrop(
            capturer,
            surf,
            &mut self.backdrop_settings,
            bounds,
            &exclude,
            is_hidden,
            force,
        ) {
            if let (Some(h), Some(ref worker)) =
                (self.backdrop_settings.last_hash, &self.backdrop_worker)
            {
                worker.seed_hash(BACKDROP_TARGET_SETTINGS, h);
            }
            self.dirty_settings = true;
        }
    }

    fn capture_drawer_backdrop(&mut self, force: bool, is_hidden: bool) {
        let exclude = self.own_hwnds();
        let Some(ref surf) = self.drawer_surface else {
            return;
        };
        if surf.is_gpu_acrylic() {
            return;
        }
        let Some(ref mut capturer) = self.backdrop_capturer else {
            return;
        };
        let mut r = RECT::default();
        if unsafe { GetWindowRect(surf.hwnd(), &mut r).is_err() } {
            return;
        }
        let bounds = (
            r.left,
            r.top,
            (r.right - r.left).max(1) as u32,
            (r.bottom - r.top).max(1) as u32,
        );
        if update_surface_backdrop(
            capturer,
            surf,
            &mut self.backdrop_drawer,
            bounds,
            &exclude,
            is_hidden,
            force,
        ) {
            if let (Some(h), Some(ref worker)) =
                (self.backdrop_drawer.last_hash, &self.backdrop_worker)
            {
                worker.seed_hash(BACKDROP_TARGET_DRAWER, h);
            }
            self.dirty_drawer = true;
        }
    }

    fn sync_backdrop_worker_targets(&self) {
        let Some(ref worker) = self.backdrop_worker else {
            return;
        };
        let screen = os_shell::primary_screen_bounds();
        let dock_bounds = if !self.dock_surface.is_gpu_acrylic()
            && self.dock.visible
            && !self.dock.is_hidden
            && self.dock.show_progress.value > 0.02
        {
            let scale = (self.dock_surface.dpi() / 96.0).max(0.5);
            Some(dock_surface_bounds(screen, scale))
        } else {
            None
        };
        let settings_bounds = if self.settings.visible {
            self.settings_surface
                .as_ref()
                .filter(|surf| !surf.is_gpu_acrylic())
                .and_then(|surf| {
                    let mut r = RECT::default();
                    unsafe { GetWindowRect(surf.hwnd(), &mut r).is_ok() }.then(|| {
                        (
                            r.left,
                            r.top,
                            (r.right - r.left).max(1) as u32,
                            (r.bottom - r.top).max(1) as u32,
                        )
                    })
                })
        } else {
            None
        };
        let drawer_bounds = if self.ai_drawer.visible {
            self.drawer_surface
                .as_ref()
                .filter(|surf| !surf.is_gpu_acrylic())
                .and_then(|surf| {
                    let mut r = RECT::default();
                    unsafe { GetWindowRect(surf.hwnd(), &mut r).is_ok() }.then(|| {
                        (
                            r.left,
                            r.top,
                            (r.right - r.left).max(1) as u32,
                            (r.bottom - r.top).max(1) as u32,
                        )
                    })
                })
        } else {
            None
        };
        worker.sync_targets([dock_bounds, settings_bounds, drawer_bounds], self.own_hwnds());
    }

    fn drain_dynamic_backdrops(&mut self) {
        let Some(ref worker) = self.backdrop_worker else {
            return;
        };
        let frames = worker.take_pending();
        let now = Instant::now();
        if let Some((hash, cap)) = frames[BACKDROP_TARGET_DOCK].as_ref() {
            if self.dock.visible && !self.dock.is_hidden {
                self.dock_surface
                    .set_backdrop_pixels(cap.width, cap.height, &cap.bgra);
                self.backdrop_dock.last_hash = Some(*hash);
                self.backdrop_dock.last_at = Some(now);
                self.backdrop_dock.from_wake = false;
                self.dirty_dock = true;
            }
        }
        if let Some((hash, cap)) = frames[BACKDROP_TARGET_SETTINGS].as_ref() {
            if self.settings.visible {
                if let Some(ref surf) = self.settings_surface {
                    surf.set_backdrop_pixels(cap.width, cap.height, &cap.bgra);
                    self.backdrop_settings.last_hash = Some(*hash);
                    self.backdrop_settings.last_at = Some(now);
                    self.backdrop_settings.from_wake = false;
                    self.dirty_settings = true;
                }
            }
        }
        if let Some((hash, cap)) = frames[BACKDROP_TARGET_DRAWER].as_ref() {
            if self.ai_drawer.visible {
                if let Some(ref surf) = self.drawer_surface {
                    surf.set_backdrop_pixels(cap.width, cap.height, &cap.bgra);
                    self.backdrop_drawer.last_hash = Some(*hash);
                    self.backdrop_drawer.last_at = Some(now);
                    self.backdrop_drawer.from_wake = false;
                    self.dirty_drawer = true;
                }
            }
        }
        self.render_all();
    }

    fn ensure_ipc_client(&mut self) -> Arc<HimeIpcClient> {
        if let Some(ref client) = self.ipc {
            return Arc::clone(client);
        }
        let client = Arc::new(HimeIpcClient::with_default_pipe());
        self.ipc = Some(Arc::clone(&client));
        client
    }

    fn send_ipc(&mut self, cmd: IpcCommand) {
        if !self.native_settings.hime_ai_enabled {
            return;
        }
        if self.ipc_worker.is_none() {
            let client = self.ensure_ipc_client();
            self.ipc_worker = Some(IpcWorker::spawn(client, self.ipc_tx.clone()));
        }
        if let Some(ref worker) = self.ipc_worker {
            worker.send(cmd);
        }
    }

    fn ensure_settings_surface(&mut self) -> Option<HWND> {
        if let Some(ref surf) = self.settings_surface {
            return Some(surf.hwnd());
        }
        let screen = os_shell::primary_screen_bounds();
        let mut settings_surface = NativeSurface::new_gpu_acrylic(
            "HimeSettings",
            screen.x + (screen.width - SETTINGS_WINDOW_WIDTH as i32) / 2,
            screen.y + (screen.height - SETTINGS_WINDOW_HEIGHT as i32) / 2,
            SETTINGS_WINDOW_WIDTH,
            SETTINGS_WINDOW_HEIGHT,
        )
        .ok()?;
        settings_surface.set_system_light_theme(os_shell::system_uses_light_theme());
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
        let hwnd = settings_surface.hwnd();
        subclass_surface(hwnd);
        settings_surface.set_color_theme_mode(ColorThemeMode::from_str_lossy(
            &self.native_settings.color_theme,
        ));
        self.settings_surface = Some(settings_surface);
        self.backdrop_settings = BackdropSlot::default();
        self.capture_settings_backdrop(true, true);
        self.dirty_settings = true;
        Some(hwnd)
    }

    fn ensure_drawer_surface(&mut self) -> Option<HWND> {
        if let Some(ref surf) = self.drawer_surface {
            return Some(surf.hwnd());
        }
        let screen = os_shell::primary_screen_bounds();
        let mut drawer_surface = NativeSurface::new_gpu_acrylic(
            "HimeAiDrawer",
            screen.x + (screen.width - 440) / 2,
            screen.y + 84,
            440,
            500,
        )
        .ok()?;
        drawer_surface.set_system_light_theme(os_shell::system_uses_light_theme());
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
        let hwnd = drawer_surface.hwnd();
        subclass_surface(hwnd);
        os_shell::enable_file_drop(hwnd, true);
        drawer_surface.set_color_theme_mode(ColorThemeMode::from_str_lossy(
            &self.native_settings.color_theme,
        ));
        self.drawer_surface = Some(drawer_surface);
        self.backdrop_drawer = BackdropSlot::default();
        self.capture_drawer_backdrop(true, true);
        self.dirty_drawer = true;
        Some(hwnd)
    }

    fn release_settings_surface_if_idle(&mut self) {
        if !self.settings.visible && !self.settings.is_animating() {
            if let Some(surf) = self.settings_surface.take() {
                remove_tracked_hwnd(surf.hwnd());
                drop(surf);
                d2d::trim_working_set();
            }
            self.backdrop_settings = BackdropSlot::default();
            self.dirty_settings = false;
        }
    }

    fn release_drawer_surface_if_idle(&mut self) {
        if !self.ai_drawer.visible && !self.ai_drawer.is_animating() {
            if let Some(surf) = self.drawer_surface.take() {
                remove_tracked_hwnd(surf.hwnd());
                drop(surf);
                d2d::trim_working_set();
            }
            self.backdrop_drawer = BackdropSlot::default();
            self.dirty_drawer = false;
        }
    }

    fn mark_all_dirty(&mut self) {
        self.dirty_notch = true;
        self.dirty_dock = true;
        self.dirty_overlay_left = true;
        self.dirty_overlay_right = true;
        if self.settings_surface.is_some() {
            self.dirty_settings = true;
        }
        if self.drawer_surface.is_some() {
            self.dirty_drawer = true;
        }
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
            "{label}: notch(hwnd={:?}, vis={}, win=({},{},{},{}), dip=({nw:.1},{nh:.1}), rect=({:.1},{:.1},{:.1},{:.1}), hidden={}) dock(hwnd={:?}, vis={}, enabled={}, mode={:?}, hidden={}, show={:.2}, grace={:.2}, edge_hov={}, has_hov={}, win=({},{},{},{}), dip=({dw:.1},{dh:.1}), rect=({:.1},{:.1},{:.1},{:.1}), spring={:.2}, items={})",
            self.notch_surface.hwnd(),
            nvis,
            nwin.left, nwin.top, nwin.right, nwin.bottom,
            nrect.x, nrect.y, nrect.width, nrect.height,
            self.notch.is_hidden,
            self.dock_surface.hwnd(),
            dvis,
            self.dock.visible,
            self.dock.mode,
            self.dock.is_hidden,
            self.dock.show_progress.value,
            self.dock.hide_grace_timer(),
            self.dock.is_edge_hovered,
            self.dock.has_hover(),
            dwin.left, dwin.top, dwin.right, dwin.bottom,
            drect.x, drect.y, drect.width, drect.height,
            self.dock.visibility.value,
            self.dock.items().len(),
        ));
    }

    fn apply_native_settings(&mut self, ns: &NativeSettings, save: bool) {
        let norm = ns.normalized();
        self.native_settings = norm.clone();
        self.apply_color_theme_to_surfaces();
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
            if norm.dock_enabled {
                self.capture_dock_backdrop(true, true);
                self.dock.set_visible(true);
                self.dock_surface.show();
                os_shell::hide_taskbar();
            } else {
                self.dock.set_visible(false);
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
            if !self.core_workspace_seeded && save {
                self.send_ipc(IpcCommand::BootstrapWorkspace);
                self.last_core_bootstrap_at = Some(Instant::now());
            }
        } else {
            let _ = self.module_mgr.stop();
            self.ipc_worker = None;
            self.ipc = None;
            self.core_workspace_seeded = false;
            self.settings.state.router_9_detected = false;
            if self.ai_drawer.visible {
                self.ai_drawer.close();
                if let Some(ref surf) = self.drawer_surface {
                    os_shell::set_window_focusable(surf.hwnd(), false);
                    surf.hide();
                }
            }
        }
        self.mark_all_dirty();
    }

    fn sync_overlay_visibility(&mut self) {
        if self.overlays.volume.is_visible() {
            self.overlay_left_surface.show();
            let interactive = self.overlays.volume.is_expanded()
                || self.overlays.volume.is_hovered
                || self.overlays.volume.is_dragging;
            self.overlay_left_surface.set_click_through(!interactive);
        } else {
            self.overlay_left_surface.set_click_through(true);
            self.overlay_left_surface.hide();
        }
        if self.overlays.brightness.is_visible() {
            self.overlay_right_surface.show();
            let interactive = self.overlays.brightness.is_expanded()
                || self.overlays.brightness.is_hovered
                || self.overlays.brightness.is_dragging;
            self.overlay_right_surface.set_click_through(!interactive);
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
        self.dirty_notch = true;
        if self.drawer_surface.is_some() {
            self.dirty_drawer = true;
        }
    }

    fn apply_telemetry(&mut self, snap: &TelemetrySnapshot, notify_hud: bool) {
        let notch_data_changed = self.last_telemetry_snap.as_ref().is_none_or(|prev| {
            prev.hours != snap.hours
                || prev.minutes != snap.minutes
                || prev.network != snap.network
                || prev.battery != snap.battery
                || prev.music != snap.music
                || prev.available_wifi != snap.available_wifi
        });
        let tray_count_changed = self
            .last_telemetry_snap
            .as_ref()
            .is_some_and(|prev| prev.tray_count != snap.tray_count);
        let dock_data_changed = self.last_telemetry_snap.as_ref().is_none_or(|prev| {
            prev.running_windows != snap.running_windows || prev.tray_count != snap.tray_count
        });

        self.notch.set_clock(snap.hours, snap.minutes);
        self.notch.set_net_stats(snap.network.stats);
        self.notch.set_battery(snap.battery);

        let show_music =
            self.native_settings.music_mode_enabled || self.notch.active_panel == NotchPanel::Music;
        if show_music {
            let next_track = snap
                .music
                .as_ref()
                .map(|m| (m.title.clone(), m.artist.clone()));
            if next_track != self.last_music_track {
                self.last_music_track = next_track.clone();
                self.current_lyrics_cache_key = None;
                self.notch.music_album = None;
                self.notch.music_position_ms = 0;
                self.notch.music_duration_ms = 0;
                self.notch.music_lyric_offset_ms = 0;
                self.notch.set_music_synced_lrc(Vec::new());
                self.dirty_notch = true;
                if let Some((title, artist)) = next_track {
                    self.os_worker.send(OsCommand::SyncMediaTimeline);
                    if self.native_settings.music_mode_enabled
                        && self.native_settings.hime_ai_enabled
                    {
                        self.send_ipc(IpcCommand::GetLyrics {
                            title,
                            artist,
                            album: None,
                            duration_sec: None,
                        });
                    }
                }
            }
            self.notch.set_music(snap.music.clone());
        } else {
            if self.last_music_track.is_some() {
                self.last_music_track = None;
                self.current_lyrics_cache_key = None;
                self.notch.set_music_synced_lrc(Vec::new());
                self.dirty_notch = true;
            }
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
        if dock_data_changed {
            self.last_running_windows = snap.running_windows.clone();
            self.dock.sync_running_windows(&snap.running_windows);
            let effective_tray = snap.tray_count.max(self.dock.tray_items().len() as u32);
            self.dock.set_system_tray_count(effective_tray);
            if tray_count_changed {
                self.os_worker.send(OsCommand::ScanTrayEntries);
            }
            self.dirty_dock = true;
        }

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
                self.dirty_notch = true;
                self.dirty_overlay_left = true;
            } else if self.last_snap_volume != Some(vol_pair) {
                self.overlays.show_volume(
                    snap.volume.percent,
                    snap.volume.muted,
                    snap.volume.device_name.clone(),
                );
                self.notch
                    .set_volume(snap.volume.percent as f32 / 100.0, snap.volume.muted);
                self.dirty_notch = true;
                self.dirty_overlay_left = true;
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
                self.dirty_notch = true;
                self.dirty_overlay_right = true;
            } else if self.last_snap_brightness != Some(bri) {
                self.overlays.show_brightness(bri);
                self.notch.set_brightness(bri);
                self.dirty_notch = true;
                self.dirty_overlay_right = true;
            }
        }
        self.last_snap_brightness = Some(bri);

        if notch_data_changed {
            self.dirty_notch = true;
        }
        self.last_telemetry_snap = Some(snap.clone());
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
        self.dirty_notch = true;
    }

    fn update_surface_click_through(&mut self) {
        let primary = os_shell::primary_screen_bounds();
        if let Some((cx, cy)) = os_shell::cursor_position() {
            let left_btn_down =
                unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0 };
            let external_file_drag = left_btn_down
                && !self.dragging_notch
                && !self.dragging_notch_slider
                && !self.overlays.volume.is_dragging
                && !self.overlays.brightness.is_dragging;

            // Notch click-through, edge-wake & drag-and-drop wake
            let notch_scale = (self.notch_surface.dpi() / 96.0).max(0.5);
            let (nx, ny, _, _) = notch_surface_bounds(self.notch.edge, primary, notch_scale);
            let (nw_dip, nh_dip) = self.notch_surface.dip_size();
            let local_nx = (cx - nx) as f32 / notch_scale;
            let local_ny = (cy - ny) as f32 / notch_scale;
            let over_notch =
                os_shell::hit_test_notch(local_nx, local_ny, &self.notch, nw_dip, nh_dip);
            let wake_margin = if external_file_drag {
                44.0
            } else {
                os_shell::DEFAULT_EDGE_WAKE_THRESHOLD_PX
            };
            let near_wake = os_shell::is_notch_edge_hovered_scaled(
                cx as f32,
                cy as f32,
                &self.notch,
                primary,
                wake_margin,
                notch_scale,
            );
            let drag_near_notch = external_file_drag
                && is_cursor_in_notch_drop_band(cx as f32, cy as f32, &self.notch, primary);
            let prev_notch_hidden = self.notch.is_hidden;
            let prev_drag_hover = self.notch.drag_hover;
            let dragging_over_notch =
                external_file_drag && (over_notch || near_wake || drag_near_notch);
            self.notch.set_drag_hover(dragging_over_notch);
            let keep_notch_visible = over_notch
                || near_wake
                || drag_near_notch
                || self.notch.is_expanded
                || self.dragging_notch
                || self.dragging_notch_slider
                || self.notch.has_active_banner()
                || self.notch.ncm_center_alert.is_some();

            if keep_notch_visible {
                self.notch_leave_at = None;
                if (near_wake || over_notch || drag_near_notch) && self.notch.is_hidden {
                    self.notch.set_hidden(false);
                }
            } else if self.native_settings.notch_mode == "peek" {
                let left_at = *self.notch_leave_at.get_or_insert_with(Instant::now);
                if left_at.elapsed() >= Duration::from_millis(800) {
                    self.notch.set_hidden(true);
                }
            } else if self.native_settings.notch_mode == "smart" {
                self.notch_leave_at = None;
                let overlapped = self.is_foreground_overlapping_notch(primary);
                self.notch.set_hidden(overlapped);
            } else {
                self.notch_leave_at = None;
            }
            if self.notch.is_hidden != prev_notch_hidden || self.notch.drag_hover != prev_drag_hover
            {
                self.dirty_notch = true;
            }
            if !over_notch && self.notch.pointer_pos.is_some() {
                self.notch.set_pointer_hover(None);
                self.dirty_notch = true;
            }
            self.notch_surface.set_click_through(
                !(over_notch
                    || near_wake
                    || drag_near_notch
                    || self.dragging_notch
                    || self.dragging_notch_slider),
            );

            // Dock click-through, overlap detection & bottom-edge wake
            if self.dock.visible {
                let prev_dock_hidden = self.dock.is_hidden;
                let prev_dock_edge_hov = self.dock.is_edge_hovered;
                let prev_dock_has_hov = self.dock.has_hover();
                let dock_scale = (self.dock_surface.dpi() / 96.0).max(0.5);
                let (dx, dy, _, _) = dock_surface_bounds(primary, dock_scale);
                let (dw_dip, dh_dip) = self.dock_surface.dip_size();
                let surf_rect = Rect::new(0.0, 0.0, dw_dip, dh_dip);
                let local_dx = (cx - dx) as f32 / dock_scale;
                let local_dy = (cy - dy) as f32 / dock_scale;

                let overlapped = self.is_foreground_overlapping_dock(primary);
                self.dock.set_overlapped(overlapped);

                let wake = self.dock.wake_hit_rect(surf_rect);
                let over_wake = os_shell::is_dock_edge_hovered(
                    cx as f32,
                    cy as f32,
                    &self.dock,
                    primary,
                    os_shell::DEFAULT_EDGE_WAKE_THRESHOLD_PX,
                ) || os_shell::is_point_in_rect(local_dx, local_dy, wake);

                if self.dock.is_hidden && self.dock.show_progress.value <= 0.2 {
                    if over_wake {
                        self.ensure_dock_wake_backdrop();
                    }
                    self.dock.set_edge_hovered(over_wake);
                    self.dock_surface.set_click_through(true);
                } else {
                    if self.dock.is_hidden && over_wake {
                        self.ensure_dock_wake_backdrop();
                    }
                    let dock_rect = self.dock.dock_rect(surf_rect);
                    let over_dock = os_shell::is_point_in_rect(local_dx, local_dy, dock_rect);
                    let over_tray_flyout = self
                        .dock
                        .tray_flyout_rect(surf_rect, dock_rect)
                        .is_some_and(|r| os_shell::is_point_in_rect(local_dx, local_dy, r));
                    let over_spaces_flyout = self
                        .dock
                        .spaces_flyout_rect(surf_rect, dock_rect)
                        .is_some_and(|r| os_shell::is_point_in_rect(local_dx, local_dy, r));
                    let over_context_menu = self
                        .dock
                        .context_menu_rect(surf_rect, dock_rect)
                        .is_some_and(|r| os_shell::is_point_in_rect(local_dx, local_dy, r));
                    let over_flyout = over_tray_flyout || over_spaces_flyout || over_context_menu;
                    let over_any = over_dock || over_flyout;
                    let keep_awake =
                        over_any || (self.dock.mode != DockMode::Fixed && over_wake);
                    if !over_any && self.dock.has_hover() {
                        self.dock.clear_hover();
                    }
                    self.dock.set_edge_hovered(keep_awake);
                    self.dock_surface.set_click_through(!over_any);
                }
                if self.dock.is_hidden != prev_dock_hidden
                    || self.dock.is_edge_hovered != prev_dock_edge_hov
                    || self.dock.has_hover() != prev_dock_has_hov
                {
                    self.dirty_dock = true;
                }
            } else {
                self.dock_surface.set_click_through(true);
            }

            // Left & Right Edge Overlay edge-wake + hover guard
            let over_left_overlay = self.sync_overlay_hover_cursor(true, cx, cy, primary);
            let over_right_overlay = self.sync_overlay_hover_cursor(false, cx, cy, primary);

            // Click-outside immediate collapse for awake Notch, Dock & Edge Overlays
            let right_btn_down =
                unsafe { (GetAsyncKeyState(VK_RBUTTON.0 as i32) as u16 & 0x8000) != 0 };
            let mouse_down = left_btn_down || right_btn_down;
            let clicked_now = mouse_down && !self.was_mouse_down;
            self.was_mouse_down = mouse_down;
            if clicked_now {
                let over_settings = self.settings.visible
                    && self.settings_surface.as_ref().is_some_and(|s| {
                        let mut r = RECT::default();
                        let ok = unsafe { GetWindowRect(s.hwnd(), &mut r).is_ok() };
                        ok && cx >= r.left && cx <= r.right && cy >= r.top && cy <= r.bottom
                    });
                let over_drawer = self.ai_drawer.visible
                    && self.drawer_surface.as_ref().is_some_and(|s| {
                        let mut r = RECT::default();
                        let ok = unsafe { GetWindowRect(s.hwnd(), &mut r).is_ok() };
                        ok && cx >= r.left && cx <= r.right && cy >= r.top && cy <= r.bottom
                    });

                if !over_settings && !over_drawer {
                    if !over_notch && !near_wake && !self.dragging_notch && !self.dragging_notch_slider {
                        if self.notch.is_expanded {
                            self.notch.set_expanded(false);
                            self.dirty_notch = true;
                        }
                        if self.native_settings.notch_mode != "fixed"
                            && !self.notch.is_hidden
                            && !self.notch.has_active_banner()
                            && self.notch.ncm_center_alert.is_none()
                        {
                            self.notch.set_hidden(true);
                            self.notch_leave_at = None;
                            self.dirty_notch = true;
                            self.notch_surface.set_click_through(true);
                        }
                    }

                    if self.dock.visible {
                        let dock_scale = (self.dock_surface.dpi() / 96.0).max(0.5);
                        let (dx, dy, _, _) = dock_surface_bounds(primary, dock_scale);
                        let (dw_dip, dh_dip) = self.dock_surface.dip_size();
                        let surf_rect = Rect::new(0.0, 0.0, dw_dip, dh_dip);
                        let local_dx = (cx - dx) as f32 / dock_scale;
                        let local_dy = (cy - dy) as f32 / dock_scale;
                        let dock_rect = self.dock.dock_rect(surf_rect);
                        let wake = self.dock.wake_hit_rect(surf_rect);
                        let over_dock = os_shell::is_point_in_rect(local_dx, local_dy, dock_rect);
                        let over_wake = os_shell::is_point_in_rect(local_dx, local_dy, wake);
                        let over_tray_flyout = self
                            .dock
                            .tray_flyout_rect(surf_rect, dock_rect)
                            .is_some_and(|r| os_shell::is_point_in_rect(local_dx, local_dy, r));
                        let over_spaces_flyout = self
                            .dock
                            .spaces_flyout_rect(surf_rect, dock_rect)
                            .is_some_and(|r| os_shell::is_point_in_rect(local_dx, local_dy, r));
                        let over_context_menu = self
                            .dock
                            .context_menu_rect(surf_rect, dock_rect)
                            .is_some_and(|r| os_shell::is_point_in_rect(local_dx, local_dy, r));
                        let over_flyout = over_tray_flyout || over_spaces_flyout || over_context_menu;

                        if !over_dock && !over_flyout && !over_wake {
                            if self.dock.tray_open {
                                self.dock.set_tray_open(false);
                                self.dirty_dock = true;
                            }
                            if self.dock.spaces_open {
                                self.dock.set_spaces_open(false);
                                self.dirty_dock = true;
                            }
                            if self.dock.context_menu.is_some() {
                                self.dock.close_context_menu();
                                self.dirty_dock = true;
                            }
                            if self.dock.mode != DockMode::Fixed && !self.dock.is_hidden {
                                self.dock.force_hide_now();
                                self.dirty_dock = true;
                                self.dock_surface.set_click_through(true);
                            }
                        }
                    }

                    if !over_left_overlay
                        && !self.overlays.volume.is_dragging
                        && (self.overlays.volume.is_expanded() || self.overlays.volume.is_hovered)
                    {
                        self.overlays.volume.hide();
                        self.dirty_overlay_left = true;
                        self.overlay_left_surface.set_click_through(true);
                    }
                    if !over_right_overlay
                        && !self.overlays.brightness.is_dragging
                        && (self.overlays.brightness.is_expanded()
                            || self.overlays.brightness.is_hovered)
                    {
                        self.overlays.brightness.hide();
                        self.dirty_overlay_right = true;
                        self.overlay_right_surface.set_click_through(true);
                    }
                }
            }
        }

        if let Some(ref surf) = self.drawer_surface {
            surf.set_click_through(!self.ai_drawer.visible);
        }
        if let Some(ref surf) = self.settings_surface {
            surf.set_click_through(!self.settings.visible);
        }
    }

    fn sync_overlay_hover_cursor(
        &mut self,
        is_left: bool,
        cx: i32,
        cy: i32,
        screen: os_shell::ScreenBounds,
    ) -> bool {
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
        let (enabled, was_visible, is_expanded, is_hovered, is_dragging, over_pill, over_wake_rect) = {
            let hud = if is_left {
                &self.overlays.volume
            } else {
                &self.overlays.brightness
            };
            (
                hud.enabled,
                hud.is_visible(),
                hud.is_expanded(),
                hud.is_hovered,
                hud.is_dragging,
                hud.is_expanded() && hud.hit_test_pill(lx, ly, surf_rect),
                os_shell::is_point_in_rect(lx, ly, hud.wake_hit_rect(surf_rect)),
            )
        };
        if !enabled {
            let surf = if is_left {
                &self.overlay_left_surface
            } else {
                &self.overlay_right_surface
            };
            surf.set_click_through(true);
            return false;
        }
        let over_edge = os_shell::is_overlay_edge_hovered(
            cx as f32,
            cy as f32,
            is_left,
            screen,
            os_shell::DEFAULT_EDGE_WAKE_THRESHOLD_PX,
            OVERLAY_DIP_HEIGHT * scale,
        );
        let over = over_pill || over_wake_rect || over_edge;
        let _ = was_visible;
        let (hud, surf) = if is_left {
            (&mut self.overlays.volume, &self.overlay_left_surface)
        } else {
            (&mut self.overlays.brightness, &self.overlay_right_surface)
        };
        if over != is_hovered && (over || !is_dragging) {
            hud.set_hovered(over);
            if is_left {
                self.dirty_overlay_left = true;
            } else {
                self.dirty_overlay_right = true;
            }
        }
        let interactive = is_expanded || hud.is_expanded() || hud.is_hovered || is_dragging;
        surf.set_click_through(!interactive);
        over
    }

    fn is_foreground_overlapping_dock(&self, screen: os_shell::ScreenBounds) -> bool {
        let fg = unsafe { GetForegroundWindow() };
        if fg.0.is_null() || self.is_own_hwnd(fg) || os_shell::is_desktop_shell_hwnd(fg) {
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
        if fg.0.is_null() || self.is_own_hwnd(fg) || os_shell::is_desktop_shell_hwnd(fg) {
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
            || self
                .settings_surface
                .as_ref()
                .is_some_and(|s| raw == s.hwnd().0 as isize)
            || self
                .drawer_surface
                .as_ref()
                .is_some_and(|s| raw == s.hwnd().0 as isize)
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
        if self
            .settings_surface
            .as_ref()
            .is_some_and(|s| raw_hwnd == s.hwnd().0 as isize)
        {
            self.handle_settings_surface_event(ev.kind, ctrl_hwnd);
            return;
        }
        if self
            .drawer_surface
            .as_ref()
            .is_some_and(|s| raw_hwnd == s.hwnd().0 as isize)
        {
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
                self.notch.set_pointer_hover(Some((lx, ly)));
                self.notch.set_pointer_pressed(true);
                render_now = true;
                if self.notch.is_hidden {
                    self.notch.set_hidden(false);
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
                let prev_hover = self.notch.pointer_pos;
                self.notch.set_pointer_hover(Some((lx, ly)));
                let hover_transition = prev_hover.is_none() || self.notch.is_expanded;
                if hover_transition && prev_hover != self.notch.pointer_pos {
                    render_now = true;
                }
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
                } else if self.notch.is_expanded {
                    render_now = true;
                }
            }
            SurfaceEventKind::LeftButtonUp { x, y } => {
                self.notch.set_pointer_pressed(false);
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
                    self.notch.handle_files_dropped(added.len());
                    self.notch_surface.set_click_through(false);
                    render_now = true;
                }
            }
        }

        if render_now {
            self.dirty_notch = true;
            self.update_surface_click_through();
            self.render_all();
        }
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn handle_notch_extra_event(&mut self, kind: ExtraSurfaceEventKind, ctrl_hwnd: HWND) {
        match kind {
            ExtraSurfaceEventKind::MouseLeave => {
                self.notch.set_pointer_hover(None);
                self.notch.set_pointer_pressed(false);
                self.notch.set_drag_hover(false);
                self.dirty_notch = true;
                self.update_surface_click_through();
                self.render_all();
                self.ensure_fast_timer(ctrl_hwnd);
            }
            ExtraSurfaceEventKind::MouseWheel { delta_steps, .. } => {
                if self.notch.is_expanded && delta_steps.abs() > 0.01 {
                    let shift_held =
                        unsafe { (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0 };
                    if shift_held || self.notch.active_panel != NotchPanel::CommandCenter {
                        let act = if delta_steps > 0.0 {
                            NotchAction::PrevPanel
                        } else {
                            NotchAction::NextPanel
                        };
                        self.execute_notch_action(act, ctrl_hwnd);
                        self.dirty_notch = true;
                        self.update_surface_click_through();
                        self.render_all();
                        self.ensure_fast_timer(ctrl_hwnd);
                    }
                }
            }
            ExtraSurfaceEventKind::RightButtonUp { .. } => {}
        }
    }

    fn refresh_active_notch_panel(&mut self) {
        match self.notch.active_panel {
            NotchPanel::Wifi => {
                self.os_worker.send(OsCommand::ScanWifi);
            }
            NotchPanel::Music => {
                self.os_worker.send(OsCommand::SyncMediaTimeline);
                if self.current_lyrics_cache_key.is_none()
                    && self.native_settings.music_mode_enabled
                    && self.native_settings.hime_ai_enabled
                {
                    if let Some(m) = self.notch.music.clone() {
                        let album = self.notch.music_album.clone();
                        let duration_sec = (self.notch.music_duration_ms > 0)
                            .then_some((self.notch.music_duration_ms / 1000) as u32);
                        self.send_ipc(IpcCommand::GetLyrics {
                            title: m.title,
                            artist: m.artist,
                            album,
                            duration_sec,
                        });
                    }
                }
            }
            NotchPanel::Tasks => {
                if self.native_settings.hime_ai_enabled {
                    self.notch.tasks_ws.loading = true;
                    self.notch.tasks_ws.error = None;
                    self.send_ipc(IpcCommand::RefreshTasks);
                }
            }
            NotchPanel::Focus => {
                if self.native_settings.hime_ai_enabled {
                    self.send_ipc(IpcCommand::RefreshFocusAndScreenTime);
                }
            }
            NotchPanel::Status
            | NotchPanel::CommandCenter
            | NotchPanel::Calendar
            | NotchPanel::Shortcuts
            | NotchPanel::Shelf
            | NotchPanel::HimeAi => {}
        }
    }

    fn execute_notch_action(&mut self, action: NotchAction, ctrl_hwnd: HWND) {
        let was_vol_muted = self.notch.volume_muted;
        let quick_add_prio = self.notch.tasks_ws.quick_add_priority;
        let quick_add_tab = self.notch.tasks_ws.active_tab;
        self.notch.apply_action(&action);
        self.dirty_notch = true;

        match action {
            NotchAction::ExpandPanel(_)
            | NotchAction::OpenWifiList
            | NotchAction::PrevPanel
            | NotchAction::NextPanel => {
                self.refresh_active_notch_panel();
            }
            NotchAction::WifiRefresh => {
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
                    self.dirty_overlay_left = true;
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
                    self.dirty_overlay_right = true;
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
            NotchAction::LyricsAdjustOffset(_) | NotchAction::LyricsResetOffset => {
                if self.native_settings.hime_ai_enabled {
                    if let Some(cache_key) = self.current_lyrics_cache_key.clone() {
                        let offset_ms = self.notch.music_lyric_offset_ms as i32;
                        self.send_ipc(IpcCommand::SetLyricsOffset {
                            cache_key,
                            offset_ms,
                        });
                    }
                }
            }
            NotchAction::TasksQuickAdd(title) => {
                let trimmed = title.trim().to_string();
                if !trimmed.is_empty() && self.native_settings.hime_ai_enabled {
                    let due_date = match quick_add_tab {
                        TaskTabFilter::Upcoming => Some(now_unix_secs() + 86_400),
                        _ => Some(now_unix_secs()),
                    };
                    let column_id = match quick_add_tab {
                        TaskTabFilter::Upcoming => "col-upcoming".to_string(),
                        _ => "col-todo".to_string(),
                    };
                    self.send_ipc(IpcCommand::CreateTask(CreateProductivityTaskParams {
                        title: trimmed,
                        board_id: Some("board-default".to_string()),
                        column_id: Some(column_id),
                        priority: Some(task_priority_to_str(quick_add_prio).to_string()),
                        due_date,
                        ..Default::default()
                    }));
                }
            }
            NotchAction::TasksToggleComplete(id) => {
                if !id.starts_with("local-task-") && self.native_settings.hime_ai_enabled {
                    let is_done = self
                        .notch
                        .tasks_ws
                        .tasks
                        .iter()
                        .find(|t| t.id == id)
                        .is_some_and(|t| t.status.is_done());
                    let (column_id, status) = if is_done {
                        ("col-done".to_string(), "done".to_string())
                    } else {
                        ("col-todo".to_string(), "todo".to_string())
                    };
                    self.send_ipc(IpcCommand::MoveTask {
                        task_id: id,
                        column_id,
                        status,
                    });
                }
            }
            NotchAction::TasksDelete(id) => {
                if !id.starts_with("local-task-") && self.native_settings.hime_ai_enabled {
                    self.send_ipc(IpcCommand::DeleteTask(id));
                }
            }
            NotchAction::TasksSelectTab(_) | NotchAction::TasksCyclePriority => {}
            NotchAction::FocusStart => {
                if self.native_settings.hime_ai_enabled {
                    let routine_id = self.notch.focus_ws.routine_id.clone();
                    self.send_ipc(IpcCommand::FocusStart { routine_id });
                }
            }
            NotchAction::FocusComplete => {
                if self.native_settings.hime_ai_enabled {
                    if let Some(session_id) = self.active_routine_session_id.take() {
                        self.send_ipc(IpcCommand::FocusComplete { session_id });
                    }
                }
            }
            NotchAction::FocusSkip => {
                if self.native_settings.hime_ai_enabled {
                    if let Some(session_id) = self.active_routine_session_id.take() {
                        self.send_ipc(IpcCommand::FocusSkip { session_id });
                    }
                }
            }
            NotchAction::FocusEmergencyPause => {
                if self.notch.focus_ws.emergency_paused && self.native_settings.hime_ai_enabled {
                    self.send_ipc(IpcCommand::FocusEmergencyPause {
                        duration_min: 15,
                        reason: "Notch Emergency Pause".to_string(),
                    });
                }
            }
            NotchAction::ProactiveAccept(id) => {
                self.dismissed_proactive_ids.insert(id.clone());
                if self.native_settings.hime_ai_enabled {
                    self.send_ipc(IpcCommand::RespondProactive {
                        suggestion_id: id,
                        action: "accept".to_string(),
                    });
                }
            }
            NotchAction::ProactiveDismiss(id) => {
                self.dismissed_proactive_ids.insert(id.clone());
                if self.native_settings.hime_ai_enabled {
                    self.send_ipc(IpcCommand::RespondProactive {
                        suggestion_id: id,
                        action: "dismiss".to_string(),
                    });
                }
            }
            NotchAction::ProactiveSnooze(id) => {
                self.dismissed_proactive_ids.insert(id.clone());
                if self.native_settings.hime_ai_enabled {
                    self.send_ipc(IpcCommand::RespondProactive {
                        suggestion_id: id,
                        action: "mute".to_string(),
                    });
                }
            }
            NotchAction::ProactiveOpenDrawer => {
                if let Some(id) = self.last_proactive_id.clone() {
                    self.dismissed_proactive_ids.insert(id);
                }
                self.open_drawer_tab(DrawerTab::Chat, ctrl_hwnd);
            }
            NotchAction::TranslateClose => {}
            NotchAction::TranslateOpenDrawer => {
                self.open_drawer_tab(DrawerTab::Translate, ctrl_hwnd);
            }
            NotchAction::OpenSettings => {
                self.open_settings(ctrl_hwnd);
            }
            NotchAction::OpenAiDrawer => {
                self.open_drawer_tab(DrawerTab::Chat, ctrl_hwnd);
            }
            NotchAction::ClearShelf => {
                self.shelf.clear();
                let _ = self.shelf.save();
                self.sync_shelf_to_views();
            }
            NotchAction::Collapse
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
                let prev_has_hover = self.dock.has_hover();
                let next_hover = self.dock.set_hovered_at(x / scale, y / scale, rect);
                if prev_hidden != self.dock.is_hidden
                    || prev_hover != next_hover
                    || prev_has_hover != self.dock.has_hover()
                    || self.dock.has_hover()
                {
                    render_now = true;
                }
            }
            SurfaceEventKind::LeftButtonDown { x, y } => {
                if let Some(act) = self.dock.hit_test_action(x / scale, y / scale, rect) {
                    self.execute_dock_action(act, ctrl_hwnd);
                    render_now = true;
                } else if self.dock.context_menu.is_some() {
                    self.dock.close_context_menu();
                    render_now = true;
                }
            }
            SurfaceEventKind::LeftButtonUp { .. } | SurfaceEventKind::DropFiles { .. } => {}
        }

        if render_now {
            self.dirty_dock = true;
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
                self.dirty_dock = true;
                self.update_surface_click_through();
                self.render_all();
                self.ensure_fast_timer(ctrl_hwnd);
            }
            ExtraSurfaceEventKind::RightButtonUp { x, y } => {
                if let Some(act) = self.dock.hit_test_context_action(x / scale, y / scale, rect) {
                    self.execute_dock_action(act, ctrl_hwnd);
                    self.dirty_dock = true;
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
                self.dock.trigger_bounce(idx);
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
            DockAction::ToggleSpaces => {
                let opened = self.dock.toggle_spaces();
                if opened {
                    self.os_worker.send(OsCommand::RefreshVirtualDesktops);
                }
                self.dirty_dock = true;
            }
            DockAction::SwitchDesktop(idx) => {
                self.dock.set_spaces_open(false);
                self.os_worker.send(OsCommand::SwitchVirtualDesktop(idx));
                self.dirty_dock = true;
            }
            DockAction::NewDesktop => {
                self.dock.set_spaces_open(false);
                self.os_worker.send(OsCommand::CreateVirtualDesktop);
                self.dirty_dock = true;
            }
            DockAction::OpenNotifications => {
                self.dock.set_tray_open(false);
                self.dock.set_spaces_open(false);
                self.dock.close_context_menu();
                self.os_worker.send(OsCommand::OpenNotificationCenter);
                self.dirty_dock = true;
            }
            DockAction::ToggleTray => {
                let opened = self.dock.toggle_tray();
                if opened {
                    self.os_worker.send(OsCommand::ScanTrayEntries);
                }
                self.dirty_dock = true;
            }
            DockAction::ActivateTrayItem(idx) => {
                if let Some(entry) = self.dock.tray_items().get(idx).cloned() {
                    self.dock.set_tray_open(false);
                    if let Some(ref uri) = entry.launch_uri {
                        open_shell_uri(uri);
                    } else if (entry.hwnd == 0 || !focus_hwnd(entry.hwnd))
                        && !entry.exe_path.trim().is_empty()
                    {
                        launch_executable(&entry.exe_path);
                    }
                    self.dirty_dock = true;
                }
            }
            DockAction::OpenAppContextMenu(idx) => {
                let (dw, dh) = self.dock_surface.dip_size();
                self.dock
                    .open_app_context_menu(idx, Rect::new(0.0, 0.0, dw, dh));
                self.dirty_dock = true;
            }
            DockAction::OpenTrayContextMenu(idx) => {
                let (dw, dh) = self.dock_surface.dip_size();
                self.dock
                    .open_tray_context_menu(idx, Rect::new(0.0, 0.0, dw, dh));
                self.dirty_dock = true;
            }
            DockAction::ExecuteContextMenuItem(row_idx) => {
                let op = self
                    .dock
                    .context_menu
                    .as_ref()
                    .and_then(|menu| menu.items.get(row_idx).map(|it| it.op.clone()));
                self.dock.close_context_menu();
                self.dirty_dock = true;
                if let Some(op) = op {
                    match op {
                        ContextMenuOp::Launch { exe_path, args } => {
                            os_shell::launch_executable_with_args(&exe_path, &args);
                        }
                        ContextMenuOp::FocusOrLaunchApp(idx) => {
                            if let Some(item) = self.dock.items().get(idx).cloned() {
                                self.dock.trigger_bounce(idx);
                                focus_or_launch_dock_item(&item);
                            }
                        }
                        ContextMenuOp::TogglePin(idx) => {
                            self.execute_dock_action(DockAction::TogglePin(idx), ctrl_hwnd);
                        }
                        ContextMenuOp::OpenFileLocation(path) => {
                            os_shell::open_file_location(&path);
                        }
                        ContextMenuOp::CloseWindows(hwnds) => {
                            self.os_worker.send(OsCommand::CloseWindows(hwnds));
                        }
                        ContextMenuOp::EndTask(hwnds) => {
                            self.os_worker.send(OsCommand::EndTaskWindows(hwnds));
                        }
                        ContextMenuOp::EndTaskPids(pids) => {
                            self.os_worker.send(OsCommand::EndTaskPids(pids));
                        }
                        ContextMenuOp::FocusOrLaunchTray(idx) => {
                            self.execute_dock_action(DockAction::ActivateTrayItem(idx), ctrl_hwnd);
                        }
                        ContextMenuOp::OpenTaskbarSettings => {
                            open_shell_uri("ms-settings:taskbar");
                        }
                    }
                }
            }
            DockAction::CloseContextMenu => {
                self.dock.close_context_menu();
                self.dirty_dock = true;
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
                    self.dirty_notch = true;
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
            if is_left {
                self.dirty_overlay_left = true;
            } else {
                self.dirty_overlay_right = true;
            }
            self.sync_overlay_visibility();
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
                    self.dirty_notch = true;
                    render_now = true;
                }
            }
            ExtraSurfaceEventKind::RightButtonUp { .. } => {}
        }
        if render_now {
            if is_left {
                self.dirty_overlay_left = true;
            } else {
                self.dirty_overlay_right = true;
            }
            self.sync_overlay_visibility();
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
            self.dirty_overlay_left = true;
        } else {
            let pct = self.overlays.brightness.set_percent_from_ratio(ratio);
            self.last_brightness_write_at = Some(Instant::now());
            self.os_worker.send(OsCommand::SetBrightness(pct));
            self.notch.set_brightness(pct);
            self.dirty_overlay_right = true;
        }
        self.dirty_notch = true;
    }

    fn handle_settings_surface_event(&mut self, kind: SurfaceEventKind, ctrl_hwnd: HWND) {
        let Some((scale, surf_hwnd)) = self
            .settings_surface
            .as_ref()
            .map(|s| ((s.dpi() / 96.0).max(0.5), s.hwnd()))
        else {
            return;
        };
        match kind {
            SurfaceEventKind::MouseMove { x, y } => {
                if self.settings.set_hovered_at(x / scale, y / scale) {
                    self.dirty_settings = true;
                    self.render_all();
                    self.ensure_fast_timer(ctrl_hwnd);
                }
            }
            SurfaceEventKind::LeftButtonDown { x, y } => {
                if self.settings.visible {
                    os_shell::focus_window(surf_hwnd);
                }
                if let Some(action) = self.settings.handle_click(x / scale, y / scale) {
                    self.dirty_settings = true;
                    self.apply_settings_action(action, ctrl_hwnd);
                    self.render_all();
                }
                self.ensure_fast_timer(ctrl_hwnd);
            }
            SurfaceEventKind::LeftButtonUp { .. } | SurfaceEventKind::DropFiles { .. } => {}
        }
    }

    fn apply_settings_action(&mut self, action: SettingsAction, ctrl_hwnd: HWND) {
        self.dirty_settings = true;
        match action {
            SettingsAction::SelectTab(tab) => {
                if tab == SettingsTab::HimeAi && self.native_settings.hime_ai_enabled {
                    self.send_ipc(IpcCommand::RefreshAiSettingsAndCounts);
                }
            }
            SettingsAction::UpdateSettings
            | SettingsAction::ToggleDockMode
            | SettingsAction::ToggleNotchMode
            | SettingsAction::ToggleFreeDrag
            | SettingsAction::ToggleHimeAi
            | SettingsAction::CycleColorTheme
            | SettingsAction::ResetToDefaults => {
                let ns = extract_native_settings_from_panel(&self.settings);
                self.apply_native_settings(&ns, true);
            }
            SettingsAction::ToggleTranslateOptIn => {
                if self.native_settings.hime_ai_enabled {
                    let enabled = self.settings.state.translate_opt_in;
                    self.send_ipc(IpcCommand::SetTranslateOptIn(enabled));
                }
            }
            SettingsAction::CycleProactiveBudget => {
                if self.native_settings.hime_ai_enabled {
                    let budget = self.settings.state.proactive_budget.clone();
                    self.send_ipc(IpcCommand::SetProactiveBudget(budget));
                }
            }
            SettingsAction::RefreshMemory => {
                if self.native_settings.hime_ai_enabled {
                    self.send_ipc(IpcCommand::RefreshMemory);
                } else {
                    self.settings
                        .set_status_notice(Some("Hime AI is disabled".to_string()));
                }
            }
            SettingsAction::ReloadSkills => {
                if self.native_settings.hime_ai_enabled {
                    self.send_ipc(IpcCommand::ReloadSkills);
                } else {
                    self.settings
                        .set_status_notice(Some("Hime AI is disabled".to_string()));
                }
            }
            SettingsAction::ReloadMcp => {
                if self.native_settings.hime_ai_enabled {
                    self.send_ipc(IpcCommand::ReloadMcp);
                } else {
                    self.settings
                        .set_status_notice(Some("Hime AI is disabled".to_string()));
                }
            }
            SettingsAction::TestAndSaveProfile => {
                let ns = extract_native_settings_from_panel(&self.settings);
                self.apply_native_settings(&ns, true);
                self.spawn_profile_test_and_probe();
            }
            SettingsAction::Close => {
                if let Some(ref surf) = self.settings_surface {
                    os_shell::set_window_focusable(surf.hwnd(), false);
                    surf.hide();
                }
                self.update_surface_click_through();
                self.release_settings_surface_if_idle();
            }
        }
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn spawn_profile_test_and_probe(&mut self) {
        if !self.native_settings.hime_ai_enabled {
            self.settings
                .set_status_notice(Some("Hime AI is disabled".to_string()));
            return;
        }
        self.os_worker.send(OsCommand::Probe9Router);
        let cfg = self.settings.state.active_profile.clone();
        let ipc = self.ensure_ipc_client();
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
        let had_surface = self.settings_surface.is_some();
        if self.ensure_settings_surface().is_none() {
            return;
        }
        if !self.settings.visible {
            if had_surface
                && self
                    .settings_surface
                    .as_ref()
                    .is_some_and(|s| !unsafe { IsWindowVisible(s.hwnd()).as_bool() })
            {
                self.capture_settings_backdrop(true, true);
            }
            self.settings.open();
        }
        if let Some(ref surf) = self.settings_surface {
            surf.set_click_through(false);
            os_shell::set_window_focusable(surf.hwnd(), true);
            surf.show();
            os_shell::focus_window(surf.hwnd());
        }
        if self.native_settings.hime_ai_enabled {
            self.os_worker.send(OsCommand::Probe9Router);
            self.send_ipc(IpcCommand::RefreshAiSettingsAndCounts);
        }
        self.dirty_settings = true;
        self.render_all();
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn toggle_settings(&mut self, ctrl_hwnd: HWND) {
        if self.settings.visible {
            self.settings.close();
            if let Some(ref surf) = self.settings_surface {
                os_shell::set_window_focusable(surf.hwnd(), false);
                surf.hide();
            }
            self.update_surface_click_through();
            self.release_settings_surface_if_idle();
            self.render_all();
            self.ensure_fast_timer(ctrl_hwnd);
        } else {
            self.open_settings(ctrl_hwnd);
        }
    }

    fn handle_drawer_surface_event(&mut self, kind: SurfaceEventKind, ctrl_hwnd: HWND) {
        let Some((scale, surf_hwnd)) = self
            .drawer_surface
            .as_ref()
            .map(|s| ((s.dpi() / 96.0).max(0.5), s.hwnd()))
        else {
            return;
        };
        match kind {
            SurfaceEventKind::LeftButtonDown { x, y } => {
                if self.ai_drawer.visible {
                    os_shell::focus_window(surf_hwnd);
                }
                self.ai_drawer.set_pointer_hover(Some((x / scale, y / scale)));
                self.ai_drawer.set_pointer_pressed(true);
                self.dirty_drawer = true;
                self.render_all();
            }
            SurfaceEventKind::LeftButtonUp { x, y } => {
                self.ai_drawer.set_pointer_pressed(false);
                if let Some(action) = self.ai_drawer.hit_test(x / scale, y / scale) {
                    self.dirty_drawer = true;
                    self.apply_drawer_action(action, ctrl_hwnd);
                    self.render_all();
                } else {
                    if self.ai_drawer.visible {
                        os_shell::focus_window(surf_hwnd);
                    }
                    self.dirty_drawer = true;
                    self.render_all();
                }
            }
            SurfaceEventKind::DropFiles { wparam } => {
                let added = os_shell::handle_wm_dropfiles(WPARAM(wparam), &mut self.shelf);
                if !added.is_empty() {
                    let _ = self.shelf.save();
                    self.sync_shelf_to_views();
                    self.dirty_drawer = true;
                    self.render_all();
                    self.ensure_fast_timer(ctrl_hwnd);
                }
            }
            SurfaceEventKind::MouseMove { x, y } => {
                if self.ai_drawer.visible {
                    self.ai_drawer.set_pointer_hover(Some((x / scale, y / scale)));
                    self.dirty_drawer = true;
                    self.render_all();
                    self.ensure_fast_timer(ctrl_hwnd);
                }
            }
        }
    }

    fn apply_drawer_action(&mut self, action: DrawerAction, ctrl_hwnd: HWND) {
        self.dirty_drawer = true;
        match action {
            DrawerAction::Close => {
                self.ai_drawer.close();
                if let Some(ref surf) = self.drawer_surface {
                    os_shell::set_window_focusable(surf.hwnd(), false);
                    surf.hide();
                }
                self.release_drawer_surface_if_idle();
            }
            DrawerAction::SelectTab(tab) => {
                self.ai_drawer.select_tab(tab);
                if tab == DrawerTab::Workspace
                    && self.native_settings.hime_ai_enabled
                    && !self.workspace_checked_once
                {
                    self.workspace_checked_once = true;
                    self.trigger_workspace_checks();
                }
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
            DrawerAction::ClearTranslation => {
                self.ai_drawer.clear_translation();
            }
            DrawerAction::CopyTranslation => {
                let text = self.ai_drawer.translation.translated_text.clone();
                if text.trim().is_empty() {
                    self.ai_drawer.translation.status_notice = Some("Nothing to copy".to_string());
                } else if copy_text_to_clipboard(&text) {
                    self.ai_drawer.mark_translation_copied();
                } else {
                    self.ai_drawer.translation.status_notice =
                        Some("Clipboard unavailable".to_string());
                }
            }
            DrawerAction::RunDevCheck => {
                if self.native_settings.hime_ai_enabled {
                    self.ai_drawer.developer.last_check_status =
                        Some("Running git & secret scan...".to_string());
                    self.send_ipc(IpcCommand::RunDevCheck {
                        workspace_path: current_workspace_path(),
                    });
                } else {
                    self.ai_drawer.developer.last_check_status =
                        Some("Hime AI is disabled".to_string());
                }
            }
            DrawerAction::RefreshDocs => {
                if self.native_settings.hime_ai_enabled {
                    self.ai_drawer.documents.latest_job_label =
                        Some("Refreshing document jobs...".to_string());
                    self.send_ipc(IpcCommand::RefreshDocs);
                } else {
                    self.ai_drawer.documents.latest_job_label =
                        Some("Hime AI is disabled".to_string());
                }
            }
        }
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn trigger_workspace_checks(&mut self) {
        self.ai_drawer.developer.last_check_status = Some("Checking workspace...".to_string());
        self.send_ipc(IpcCommand::RunDevCheck {
            workspace_path: current_workspace_path(),
        });
        self.send_ipc(IpcCommand::RefreshDocs);
    }

    fn open_drawer_tab(&mut self, tab: DrawerTab, ctrl_hwnd: HWND) {
        if !self.native_settings.hime_ai_enabled {
            self.notch
                .set_ncm_center_alert(Some("Hime AI is disabled".to_string()));
            self.dirty_notch = true;
            self.render_all();
            self.ensure_fast_timer(ctrl_hwnd);
            return;
        }
        let had_surface = self.drawer_surface.is_some();
        if self.ensure_drawer_surface().is_none() {
            return;
        }
        self.ai_drawer.select_tab(tab);
        if !self.ai_drawer.visible {
            if had_surface
                && self
                    .drawer_surface
                    .as_ref()
                    .is_some_and(|s| !unsafe { IsWindowVisible(s.hwnd()).as_bool() })
            {
                self.capture_drawer_backdrop(true, true);
            }
            self.sync_shelf_to_views();
            self.ai_drawer.open();
        }
        if tab == DrawerTab::Workspace && !self.workspace_checked_once {
            self.workspace_checked_once = true;
            self.trigger_workspace_checks();
        }
        if let Some(ref surf) = self.drawer_surface {
            surf.set_click_through(false);
            os_shell::set_window_focusable(surf.hwnd(), true);
            surf.show();
            os_shell::focus_window(surf.hwnd());
        }
        self.dirty_drawer = true;
        self.update_surface_click_through();
        self.render_all();
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn trigger_translate_selection(&mut self, ctrl_hwnd: HWND) {
        if !self.native_settings.hime_ai_enabled {
            self.notch
                .set_ncm_center_alert(Some("Hime AI is disabled".to_string()));
            self.dirty_notch = true;
            self.render_all();
            self.ensure_fast_timer(ctrl_hwnd);
            return;
        }
        let target_lang = if self.ai_drawer.translation.target_lang.trim().is_empty() {
            "id".to_string()
        } else {
            self.ai_drawer.translation.target_lang.to_ascii_lowercase()
        };
        self.notch
            .translate_banner
            .show_loading("Selected text", &target_lang);
        if !self.notch.is_expanded {
            self.notch.set_hidden(false);
        }
        self.dirty_notch = true;
        self.update_surface_click_through();
        self.render_all();
        self.ensure_fast_timer(ctrl_hwnd);

        let ipc = self.ensure_ipc_client();
        let tx = self.ipc_tx.clone();
        std::thread::spawn(move || {
            let res = ipc.translate_selection(TranslateSelectionParams {
                target_lang,
                text: None,
            });
            let _ = tx.send(IpcBgResult::TranslationDone(res));
        });
    }

    fn handle_window_key_message(&mut self, msg: &MSG, ctrl_hwnd: HWND) -> bool {
        if self
            .drawer_surface
            .as_ref()
            .is_some_and(|s| msg.hwnd == s.hwnd())
            && self.ai_drawer.visible
        {
            let action = match msg.message {
                WM_KEYDOWN => self.ai_drawer.handle_key_down(msg.wParam.0 as u32),
                WM_CHAR => {
                    char::from_u32(msg.wParam.0 as u32).and_then(|ch| self.ai_drawer.handle_char(ch))
                }
                _ => None,
            };
            self.dirty_drawer = true;
            if let Some(act) = action {
                self.apply_drawer_action(act, ctrl_hwnd);
            } else {
                self.ensure_fast_timer(ctrl_hwnd);
            }
            self.render_all();
            return true;
        }

        if self
            .settings_surface
            .as_ref()
            .is_some_and(|s| msg.hwnd == s.hwnd())
            && self.settings.visible
        {
            // Note: SettingsPanel::handle_key_down already invokes apply_row_control internally!
            // Do NOT call settings.apply_control(ctrl) again or toggles will double-flip.
            let ctrl = match msg.message {
                WM_KEYDOWN => self.settings.handle_key_down(msg.wParam.0 as u32),
                WM_CHAR => {
                    char::from_u32(msg.wParam.0 as u32).and_then(|ch| self.settings.handle_char(ch))
                }
                _ => None,
            };
            self.dirty_settings = true;
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
        if !self.native_settings.hime_ai_enabled {
            return;
        }
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
        self.dirty_drawer = true;

        let attachments: Vec<String> = self
            .ai_drawer
            .attachments
            .iter()
            .map(|a| a.path.clone())
            .collect();
        let profile_id = Some(self.settings.state.active_profile.id.clone());
        let ipc = self.ensure_ipc_client();
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

    fn dispatch_approval(&mut self, approval_id: String, approved: bool) {
        if !self.native_settings.hime_ai_enabled {
            return;
        }
        let ipc = self.ensure_ipc_client();
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
                    self.dirty_notch = true;
                    self.render_all();
                    self.ensure_fast_timer(ctrl_hwnd);
                    return;
                }
                if !self.ai_drawer.visible {
                    let had_surface = self.drawer_surface.is_some();
                    if self.ensure_drawer_surface().is_none() {
                        return;
                    }
                    if had_surface
                        && self
                            .drawer_surface
                            .as_ref()
                            .is_some_and(|s| !unsafe { IsWindowVisible(s.hwnd()).as_bool() })
                    {
                        self.capture_drawer_backdrop(true, true);
                    }
                    self.sync_shelf_to_views();
                }
                self.ai_drawer.toggle();
                self.dirty_drawer = true;
                if self.ai_drawer.visible {
                    if let Some(ref surf) = self.drawer_surface {
                        surf.set_click_through(false);
                        os_shell::set_window_focusable(surf.hwnd(), true);
                        surf.show();
                        os_shell::focus_window(surf.hwnd());
                    }
                    self.render_all();
                } else {
                    if let Some(ref surf) = self.drawer_surface {
                        os_shell::set_window_focusable(surf.hwnd(), false);
                        surf.hide();
                    }
                    self.render_all();
                    self.release_drawer_surface_if_idle();
                }
            }
            HotkeyAction::ToggleNotch => {
                let next = !self.notch.is_expanded;
                self.notch.set_expanded(next);
                self.dirty_notch = true;
                self.update_surface_click_through();
                self.render_all();
            }
            HotkeyAction::ToggleCalendar => {
                if self.notch.is_expanded && self.notch.active_panel == NotchPanel::Calendar {
                    self.notch.set_expanded(false);
                } else {
                    self.notch.set_panel(NotchPanel::Calendar);
                }
                self.dirty_notch = true;
                self.update_surface_click_through();
                self.render_all();
            }
            HotkeyAction::ToggleMusic => {
                if self.notch.is_expanded && self.notch.active_panel == NotchPanel::Music {
                    self.notch.set_expanded(false);
                } else {
                    self.notch.set_panel(NotchPanel::Music);
                }
                self.dirty_notch = true;
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
                self.dirty_notch = true;
                self.update_surface_click_through();
                self.render_all();
            }
            HotkeyAction::FocusDockSlot(slot) => {
                self.dock.set_hovered(Some(slot as usize));
                self.dock.trigger_bounce(slot as usize);
                self.dirty_dock = true;
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
                    self.dirty_drawer = true;
                }
                IpcBgResult::ChatTurn(Err(err)) => {
                    self.ai_drawer.append_stream_chunk(&format!("[IPC] {err}"));
                    self.ai_drawer.finish_assistant_stream();
                    self.dirty_drawer = true;
                }
                IpcBgResult::ProfileTest(Ok(msg)) => {
                    self.settings.set_status_notice(Some(msg.clone()));
                    self.notch.set_ncm_center_alert(Some(msg));
                    self.dirty_settings = true;
                    self.dirty_notch = true;
                }
                IpcBgResult::ProfileTest(Err(err)) => {
                    let msg = format!("IPC: {err}");
                    self.settings.set_status_notice(Some(msg.clone()));
                    self.notch.set_ncm_center_alert(Some(msg));
                    self.dirty_settings = true;
                    self.dirty_notch = true;
                }
                IpcBgResult::ApprovalResolved(Ok(())) => {}
                IpcBgResult::ApprovalResolved(Err(err)) => {
                    self.notch
                        .set_ncm_center_alert(Some(format!("Approval: {err}")));
                    self.dirty_notch = true;
                }
                IpcBgResult::RouterProbe(probe) => {
                    self.settings.state.router_9_detected = probe.available;
                    self.notch.ai_status_text = if probe.available {
                        format!("9Router Connected ({} models)", probe.models.len())
                    } else {
                        "Gateway Standby".to_string()
                    };
                    self.dirty_settings = true;
                    self.dirty_notch = true;
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
                    self.dirty_notch = true;
                }
                IpcBgResult::WifiConnect(ssid, Ok(())) => {
                    self.notch
                        .set_ncm_center_alert(Some(format!("Connected: {ssid}")));
                    self.os_worker.send(OsCommand::ScanWifi);
                    self.dirty_notch = true;
                }
                IpcBgResult::WifiConnect(_, Err(err)) => {
                    self.notch.set_ncm_center_alert(Some(err));
                    self.dirty_notch = true;
                }
                IpcBgResult::TrayEntries(entries) => {
                    let count = entries.len() as u32;
                    self.dock.set_tray_items(entries);
                    if count == 0 {
                        let snap = self.telemetry.snapshot();
                        self.dock.set_system_tray_count(snap.tray_count);
                    }
                    self.dirty_dock = true;
                }
                IpcBgResult::VirtualDesktops(spaces) => {
                    let mapped: Vec<SpaceEntry> = spaces
                        .into_iter()
                        .map(|s| SpaceEntry::new(s.id, s.name, s.index, s.active))
                        .collect();
                    if self.dock.spaces() != mapped.as_slice() {
                        self.dock.set_spaces(mapped);
                        self.dirty_dock = true;
                    }
                }
                IpcBgResult::MediaTimeline {
                    title,
                    artist,
                    album,
                    position_ms,
                    duration_ms,
                } => {
                    let matches_track = self.last_music_track.as_ref().is_some_and(|(t, a)| {
                        t == &title && a == &artist
                    });
                    if matches_track {
                        if album.is_some() {
                            self.notch.music_album = album;
                        }
                        if duration_ms > 0 {
                            self.notch.music_duration_ms = duration_ms;
                        }
                        self.notch.update_music_position(position_ms);
                        self.dirty_notch = true;
                    }
                }
                IpcBgResult::LyricsLoaded {
                    title,
                    artist,
                    result,
                } => {
                    let matches_track = self.last_music_track.as_ref().is_some_and(|(t, a)| {
                        t == &title && a == &artist
                    });
                    if matches_track {
                        match result {
                            Ok(res) => {
                                self.core_workspace_seeded = true;
                                self.current_lyrics_cache_key = Some(res.cache_key.clone());
                                self.notch.music_lyric_offset_ms = res.user_offset_ms as i64;
                                if res.duration_sec > 0 && self.notch.music_duration_ms == 0 {
                                    self.notch.music_duration_ms = res.duration_sec as u64 * 1000;
                                }
                                if res.album.is_some() && self.notch.music_album.is_none() {
                                    self.notch.music_album = res.album.clone();
                                }
                                let (synced, plain) = lyrics_result_to_lines(&res);
                                if !synced.is_empty() {
                                    self.notch.set_music_synced_lrc(synced);
                                } else {
                                    self.notch.music_synced_lrc.clear();
                                    self.notch.music_lrc_lines = plain;
                                    self.notch.music_active_lyric_idx = None;
                                }
                                self.dirty_notch = true;
                            }
                            Err(_) => {
                                self.current_lyrics_cache_key = None;
                            }
                        }
                    }
                }
                IpcBgResult::LyricsOffsetSaved(Ok(resp)) => {
                    self.notch.music_lyric_offset_ms = resp.user_offset_ms as i64;
                    let pos = self.notch.music_position_ms;
                    self.notch.update_music_position(pos);
                    self.dirty_notch = true;
                }
                IpcBgResult::LyricsOffsetSaved(Err(_)) => {}
                IpcBgResult::TasksLoaded(res) => {
                    self.notch.tasks_ws.loading = false;
                    match res {
                        Ok(records) => {
                            self.core_workspace_seeded = true;
                            self.notch.tasks_ws.error = None;
                            self.notch.tasks_ws.tasks = records
                                .iter()
                                .map(map_productivity_task_to_card)
                                .collect();
                        }
                        Err(err) => {
                            if self.core_workspace_seeded {
                                self.notch.tasks_ws.error = Some(err.to_string());
                            }
                        }
                    }
                    self.dirty_notch = true;
                }
                IpcBgResult::FocusDataLoaded {
                    routines,
                    screentime,
                } => {
                    if let Ok(list) = routines {
                        self.core_workspace_seeded = true;
                        if let Some(routine) = list.iter().find(|r| r.active).or_else(|| list.first())
                        {
                            self.notch.focus_ws.routine_id = routine.id.clone();
                            self.notch.focus_ws.routine_title = routine.title.clone();
                            self.notch.focus_ws.category = routine.category.clone();
                            self.notch.focus_ws.enforcement =
                                format!("{:?}", routine.enforcement).to_ascii_lowercase();
                            self.notch.focus_ws.streak_days = routine.streak.current_count;
                            if self.notch.focus_ws.status == FocusSessionStatus::Idle {
                                let secs = routine.schedule.duration_min.max(1) * 60;
                                self.notch.focus_ws.total_secs = secs;
                                self.notch.focus_ws.remaining_secs = secs;
                            }
                        }
                        self.dirty_notch = true;
                    }
                    if let Ok(sessions) = screentime {
                        self.core_workspace_seeded = true;
                        let today = current_local_date_str();
                        let (study, neutral, distracting) =
                            summarize_screentime_sessions(&sessions, &today);
                        self.notch.focus_ws.study_secs = study;
                        self.notch.focus_ws.neutral_secs = neutral;
                        self.notch.focus_ws.distracting_secs = distracting;
                        self.dirty_notch = true;
                    }
                }
                IpcBgResult::RoutineStarted(Ok(res)) => {
                    self.active_routine_session_id = Some(res.session.id);
                    let secs = res.session.duration_min.max(1) * 60;
                    self.notch.focus_ws.total_secs = secs;
                    if self.notch.focus_ws.remaining_secs == 0 {
                        self.notch.focus_ws.remaining_secs = secs;
                    }
                    self.dirty_notch = true;
                }
                IpcBgResult::RoutineStarted(Err(err)) => {
                    self.notch
                        .set_ncm_center_alert(Some(format!("Focus start: {err}")));
                    self.dirty_notch = true;
                }
                IpcBgResult::RoutineOutcome(Ok(res)) => {
                    self.notch.focus_ws.streak_days = res.streak.current_count;
                    self.dirty_notch = true;
                }
                IpcBgResult::RoutineOutcome(Err(_)) => {}
                IpcBgResult::EmergencyPaused(Ok(pause)) => {
                    self.notch.set_ncm_center_alert(Some(format!(
                        "Emergency pause ({}m)",
                        pause.duration_min
                    )));
                    self.dirty_notch = true;
                }
                IpcBgResult::EmergencyPaused(Err(err)) => {
                    self.notch
                        .set_ncm_center_alert(Some(format!("Pause: {err}")));
                    self.dirty_notch = true;
                }
                IpcBgResult::ProactiveEvaluated(Ok(items)) => {
                    self.core_workspace_seeded = true;
                    if !self.notch.is_expanded && !self.notch.translate_banner.active {
                        if let Some(sug) = items.into_iter().find(|s| {
                            s.status == "pending" && !self.dismissed_proactive_ids.contains(&s.id)
                        }) {
                            let level = proactive_level_from_suggestion(&sug);
                            self.last_proactive_id = Some(sug.id.clone());
                            self.notch.proactive_banner.show(
                                sug.id,
                                sug.category,
                                level,
                                sug.title,
                                sug.content,
                                HimeExpression::Senang,
                                Some(12.0),
                            );
                            self.notch.set_hidden(false);
                            self.dirty_notch = true;
                        }
                    }
                }
                IpcBgResult::ProactiveEvaluated(Err(_)) => {}
                IpcBgResult::ProactiveResponded(Ok(_)) => {}
                IpcBgResult::ProactiveResponded(Err(err)) => {
                    log_diag(&format!("[proactive] respond failed: {err}"));
                }
                IpcBgResult::TranslateOptInLoaded(Ok(st)) => {
                    self.core_workspace_seeded = true;
                    self.settings.state.translate_opt_in = st.opted_in;
                    self.dirty_settings = true;
                }
                IpcBgResult::TranslateOptInLoaded(Err(_)) => {}
                IpcBgResult::TranslateOptInSaved(Ok(resp)) => {
                    self.settings.state.translate_opt_in = resp.opted_in;
                    let label = if resp.opted_in { "enabled" } else { "disabled" };
                    self.settings
                        .set_status_notice(Some(format!("Translate selection {label}")));
                    self.dirty_settings = true;
                }
                IpcBgResult::TranslateOptInSaved(Err(err)) => {
                    self.settings
                        .set_status_notice(Some(format!("Translate opt-in: {err}")));
                    self.dirty_settings = true;
                }
                IpcBgResult::TranslationDone(Ok(tr)) => {
                    self.notch.translate_banner.show_result(
                        &tr.original,
                        &tr.translated,
                        &tr.target_lang,
                        tr.cached,
                    );
                    if !self.notch.is_expanded {
                        self.notch.set_hidden(false);
                    }
                    self.ai_drawer.set_translation(
                        tr.original,
                        tr.translated,
                        tr.target_lang,
                        tr.cached,
                    );
                    self.dirty_notch = true;
                    self.dirty_drawer = true;
                }
                IpcBgResult::TranslationDone(Err(err)) => {
                    self.notch.translate_banner.show_error(err.to_string());
                    if !self.notch.is_expanded {
                        self.notch.set_hidden(false);
                    }
                    self.dirty_notch = true;
                }
                IpcBgResult::ProactiveSettingsLoaded(Ok(ps)) => {
                    self.core_workspace_seeded = true;
                    self.settings.state.proactive_budget =
                        attention_mode_to_budget(&ps.attention_mode).to_string();
                    self.dirty_settings = true;
                }
                IpcBgResult::ProactiveSettingsLoaded(Err(_)) => {}
                IpcBgResult::ProactiveBudgetSaved(Ok(budget)) => {
                    self.settings.state.proactive_budget = budget.clone();
                    self.settings
                        .set_status_notice(Some(format!("Proactive budget set to {budget}")));
                    self.dirty_settings = true;
                }
                IpcBgResult::ProactiveBudgetSaved(Err(err)) => {
                    self.settings
                        .set_status_notice(Some(format!("Proactive budget: {err}")));
                    self.dirty_settings = true;
                }
                IpcBgResult::MemoryCountLoaded(Ok(count)) => {
                    self.core_workspace_seeded = true;
                    self.settings.state.memory_count = count;
                    if self.settings.visible {
                        self.settings
                            .set_status_notice(Some(format!("Memory refreshed: {count} entries")));
                        self.dirty_settings = true;
                    }
                }
                IpcBgResult::MemoryCountLoaded(Err(err)) => {
                    if self.settings.visible {
                        self.settings
                            .set_status_notice(Some(format!("Memory: {err}")));
                        self.dirty_settings = true;
                    }
                }
                IpcBgResult::SkillsCountLoaded(Ok(count)) => {
                    self.core_workspace_seeded = true;
                    self.settings.state.skills_count = count;
                    if self.settings.visible {
                        self.settings
                            .set_status_notice(Some(format!("Skills reloaded: {count} loaded")));
                        self.dirty_settings = true;
                    }
                }
                IpcBgResult::SkillsCountLoaded(Err(err)) => {
                    if self.settings.visible {
                        self.settings
                            .set_status_notice(Some(format!("Skills: {err}")));
                        self.dirty_settings = true;
                    }
                }
                IpcBgResult::McpCountLoaded(Ok(count)) => {
                    self.core_workspace_seeded = true;
                    self.settings.state.mcp_count = count;
                    if self.settings.visible {
                        self.settings
                            .set_status_notice(Some(format!("MCP servers: {count} connected")));
                        self.dirty_settings = true;
                    }
                }
                IpcBgResult::McpCountLoaded(Err(err)) => {
                    if self.settings.visible {
                        self.settings.set_status_notice(Some(format!("MCP: {err}")));
                        self.dirty_settings = true;
                    }
                }
                IpcBgResult::DevCheckLoaded(Ok(summary)) => {
                    self.ai_drawer.set_developer_summary(summary);
                    self.dirty_drawer = true;
                }
                IpcBgResult::DevCheckLoaded(Err(err)) => {
                    self.ai_drawer.developer.last_check_status = Some(format!("IPC: {err}"));
                    self.dirty_drawer = true;
                }
                IpcBgResult::DocsLoaded(Ok(summary)) => {
                    self.ai_drawer.set_documents_summary(summary);
                    self.dirty_drawer = true;
                }
                IpcBgResult::DocsLoaded(Err(err)) => {
                    self.ai_drawer.documents.latest_job_label = Some(format!("IPC: {err}"));
                    self.dirty_drawer = true;
                }
            }
        }

        let snap = self.telemetry.snapshot();
        self.apply_telemetry(&snap, true);
        self.update_surface_click_through();

        if self.native_settings.suppress_native_notifications && self.tick_count % 2 == 0 {
            self.scan_and_suppress_notifications();
        }
        if self.dock.visible && self.tick_count % 8 == 0 {
            os_shell::ensure_taskbar_hidden();
        }
        if self.tick_count > 0 && self.tick_count % 20 == 0 {
            self.os_worker.send(OsCommand::ScanTrayEntries);
        }
        if self.tick_count % 16 == 0 {
            let sys_light = os_shell::system_uses_light_theme();
            self.dock_surface.set_system_light_theme(sys_light);
            if let Some(ref surf) = self.settings_surface {
                surf.set_system_light_theme(sys_light);
            }
            if let Some(ref surf) = self.drawer_surface {
                surf.set_system_light_theme(sys_light);
            }
        }
        if self.dock.visible && self.tick_count > 0 && self.tick_count % 25 == 0 {
            self.os_worker.send(OsCommand::RefreshVirtualDesktops);
        }

        // Smoothly advance LRC lyrics position while playing & periodically re-sync SMTC timeline
        let is_playing = self.notch.music.as_ref().is_some_and(|m| m.is_playing);
        if is_playing {
            let delta_ms = (dt * 1000.0).round() as u64;
            if delta_ms > 0 {
                let next_pos = self.notch.music_position_ms.saturating_add(delta_ms);
                self.notch.update_music_position(next_pos);
                self.dirty_notch = true;
            }
            if now.duration_since(self.last_media_timeline_sync_at) >= MEDIA_TIMELINE_SYNC_INTERVAL {
                self.last_media_timeline_sync_at = now;
                self.os_worker.send(OsCommand::SyncMediaTimeline);
            }
        }

        // Retry initial workspace bootstrap if Core pipe was still starting up
        if self.native_settings.hime_ai_enabled && !self.core_workspace_seeded {
            let should_retry = self
                .last_core_bootstrap_at
                .is_none_or(|t| now.duration_since(t) >= CORE_BOOTSTRAP_RETRY_INTERVAL);
            if should_retry {
                self.last_core_bootstrap_at = Some(now);
                self.send_ipc(IpcCommand::BootstrapWorkspace);
            }
        }

        // Periodic proactive engine evaluation
        if self.native_settings.hime_ai_enabled
            && now.duration_since(self.last_proactive_eval_at) >= PROACTIVE_EVAL_INTERVAL
        {
            self.last_proactive_eval_at = now;
            self.send_ipc(IpcCommand::EvaluateProactive);
        }

        let prev_focus_status = self.notch.focus_ws.status;
        let notch_anim = self.notch.tick(dt);
        if notch_anim || self.notch.has_active_banner() {
            self.dirty_notch = true;
        }
        if prev_focus_status == FocusSessionStatus::Active
            && self.notch.focus_ws.status == FocusSessionStatus::Completed
            && self.native_settings.hime_ai_enabled
        {
            if let Some(session_id) = self.active_routine_session_id.take() {
                self.send_ipc(IpcCommand::FocusComplete { session_id });
            }
        }
        let dock_anim = self.dock.tick(dt);
        if dock_anim {
            self.dirty_dock = true;
        }
        let vol_anim = self.overlays.volume.tick(dt);
        if vol_anim {
            self.dirty_overlay_left = true;
        }
        let bri_anim = self.overlays.brightness.tick(dt);
        if bri_anim {
            self.dirty_overlay_right = true;
        }
        let set_anim = self.settings.tick(dt);
        if set_anim {
            self.dirty_settings = true;
        }
        let drw_anim = self.ai_drawer.tick(dt);
        if drw_anim {
            self.dirty_drawer = true;
        }

        let watch_outside_click = (self.native_settings.notch_mode != "fixed" && !self.notch.is_hidden)
            || self.notch.is_expanded
            || (self.dock.visible && self.dock.mode != DockMode::Fixed && !self.dock.is_hidden)
            || self.dock.tray_open
            || self.dock.spaces_open
            || self.dock.context_menu.is_some()
            || self.overlays.volume.is_expanded()
            || self.overlays.brightness.is_expanded();

        let animating = notch_anim
            || dock_anim
            || vol_anim
            || bri_anim
            || set_anim
            || drw_anim
            || watch_outside_click
            || self.notch.has_active_banner();

        self.sync_backdrop_worker_targets();
        self.sync_overlay_visibility();
        self.render_all();
        self.release_settings_surface_if_idle();
        self.release_drawer_surface_if_idle();

        if !self.startup_trim_done && now.duration_since(self.startup_at) >= Duration::from_secs(2) {
            self.startup_trim_done = true;
            d2d::trim_working_set();
        } else if !animating && self.tick_count > 0 && self.tick_count % 120 == 0 {
            d2d::trim_working_set();
        }

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
        } else {
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

    fn has_edge_wake_targets(&self) -> bool {
        self.native_settings.notch_mode != "fixed"
            || (self.dock.visible && self.dock.mode != DockMode::Fixed)
            || self.overlays.volume.enabled
            || self.overlays.brightness.enabled
    }

    fn set_idle_timer(&mut self, ctrl_hwnd: HWND) {
        let interval = if self.has_edge_wake_targets() {
            ANIM_INTERVAL_EDGE_POLL_MS
        } else {
            ANIM_INTERVAL_IDLE_MS
        };
        unsafe {
            let _ = SetTimer(Some(ctrl_hwnd), TIMER_ANIM_ID, interval, None);
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

    fn render_all(&mut self) {
        if self.dirty_notch {
            self.render_surface("HimeNotch", &self.notch_surface, |s| self.notch.render(s));
            self.dirty_notch = false;
        }

        if (self.dock.visible || self.dock.is_animating()) && self.dirty_dock {
            self.render_surface("HimeDock", &self.dock_surface, |s| self.dock.render(s));
            self.dirty_dock = false;
            self.sync_dock_tooltip_surface();
        }

        if (self.overlays.volume.enabled || self.overlays.volume.is_visible())
            && self.dirty_overlay_left
        {
            self.render_surface("HimeOverlayLeft", &self.overlay_left_surface, |s| {
                self.overlays.volume.render(s)
            });
            self.dirty_overlay_left = false;
        }

        if (self.overlays.brightness.enabled || self.overlays.brightness.is_visible())
            && self.dirty_overlay_right
        {
            self.render_surface("HimeOverlayRight", &self.overlay_right_surface, |s| {
                self.overlays.brightness.render(s)
            });
            self.dirty_overlay_right = false;
        }

        if (self.settings.visible || self.settings.is_animating()) && self.dirty_settings {
            if let Some(ref surf) = self.settings_surface {
                self.render_surface("HimeSettings", surf, |s| self.settings.render(s));
            }
            self.dirty_settings = false;
        }

        if (self.ai_drawer.visible || self.ai_drawer.is_animating()) && self.dirty_drawer {
            if let Some(ref surf) = self.drawer_surface {
                self.render_surface("HimeAiDrawer", surf, |s| self.ai_drawer.render(s));
            }
            self.dirty_drawer = false;
        }
    }

    fn sync_dock_tooltip_surface(&mut self) {
        if !self.dock_surface.is_gpu_acrylic() {
            return;
        }
        let (dw, dh) = self.dock_surface.dip_size();
        let spec = self.dock.active_tooltip_spec(Rect::new(0.0, 0.0, dw, dh));
        let Some((tip_rect, label, tip_alpha)) = spec else {
            if self.last_dock_tooltip.take().is_some() {
                if let Some(surf) = self.dock_tooltip_surface.take() {
                    drop(surf);
                }
            }
            return;
        };

        let screen = os_shell::primary_screen_bounds();
        let scale = (self.dock_surface.dpi() / 96.0).max(0.5);
        let (dx, dy, _, _) = dock_surface_bounds(screen, scale);
        let pad_dip = 4.0_f32;
        let tx = dx + ((tip_rect.x - pad_dip) * scale).round() as i32;
        let ty = dy + ((tip_rect.y - pad_dip) * scale).round() as i32;
        let tw = (((tip_rect.width + pad_dip * 2.0) * scale).ceil() as u32).max(24);
        let th = (((tip_rect.height + pad_dip * 2.0) * scale).ceil() as u32).max(16);
        let alpha_u8 = (tip_alpha * 255.0).round() as u8;
        let next_key = (tx, ty, tw, th, label.clone(), alpha_u8);
        if self.last_dock_tooltip.as_ref() == Some(&next_key) {
            return;
        }

        if self.dock_tooltip_surface.is_none() {
            if let Ok(surf) = NativeSurface::new("HimeDockTooltip", tx, ty, tw, th) {
                surf.set_click_through(true);
                self.dock_tooltip_surface = Some(surf);
            }
        }
        let Some(ref mut surf) = self.dock_tooltip_surface else {
            return;
        };
        let (cw, ch) = surf.size();
        if cw != tw || ch != th {
            let _ = surf.set_bounds(tx, ty, tw, th);
        } else {
            surf.set_position(tx, ty);
        }
        surf.show();
        let (lw, lh) = surf.dip_size();
        surf.begin_draw();
        if let Err(err) = hime_island_native::dock::draw_cardless_tooltip_text(
            surf,
            Rect::new(0.0, 0.0, lw, lh),
            &label,
            tip_alpha,
        ) {
            log_render_error("HimeDockTooltip", "render", &err);
        }
        if let Err(err) = surf.end_draw_and_present() {
            log_render_error("HimeDockTooltip", "present", &err);
        }
        self.last_dock_tooltip = Some(next_key);
    }

    fn shutdown(&mut self) {
        if let Some(ref mut worker) = self.backdrop_worker {
            worker.shutdown();
        }
        self.ipc_worker = None;
        self.ipc = None;
        if let Some(surf) = self.settings_surface.take() {
            remove_tracked_hwnd(surf.hwnd());
            drop(surf);
        }
        if let Some(surf) = self.drawer_surface.take() {
            remove_tracked_hwnd(surf.hwnd());
            drop(surf);
        }
        if let Some(surf) = self.dock_tooltip_surface.take() {
            drop(surf);
        }
        self.restore_suppressed_notifications();
        let _ = self.shelf.save();
        let _ = self.module_mgr.stop();
        if self.dock.visible || os_shell::has_taskbar_crash_marker() {
            os_shell::restore_taskbar();
        }
        if let Some(evt) = self.quit_event.take() {
            unsafe {
                let _ = CloseHandle(evt);
            }
        }
    }
}

fn collect_live_tray_entries() -> Vec<TrayEntry> {
    telemetry::collect_live_tray_records()
        .into_iter()
        .map(|rec| {
            let mut entry = TrayEntry::new(rec.id, rec.label, rec.exe_path, rec.hwnd);
            entry.pid = rec.pid;
            entry.launch_uri = rec.launch_uri;
            entry.icon_png = rec.icon_png;
            entry
        })
        .collect()
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
    panel.state.color_theme = norm.color_theme.clone();
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
        color_theme: panel.state.color_theme.clone(),
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

fn copy_text_to_clipboard(text: &str) -> bool {
    #[link(name = "user32")]
    extern "system" {
        fn OpenClipboard(hWndNewOwner: isize) -> i32;
        fn EmptyClipboard() -> i32;
        fn SetClipboardData(uFormat: u32, hMem: isize) -> isize;
        fn CloseClipboard() -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> isize;
        fn GlobalLock(hMem: isize) -> *mut c_void;
        fn GlobalUnlock(hMem: isize) -> i32;
        fn GlobalFree(hMem: isize) -> isize;
    }

    const CF_UNICODEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 0x0002;

    let wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let byte_len = wide.len().saturating_mul(std::mem::size_of::<u16>());
    unsafe {
        if OpenClipboard(0) == 0 {
            return false;
        }
        if EmptyClipboard() == 0 {
            let _ = CloseClipboard();
            return false;
        }
        let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_len);
        if hmem == 0 {
            let _ = CloseClipboard();
            return false;
        }
        let ptr = GlobalLock(hmem) as *mut u16;
        if ptr.is_null() {
            let _ = GlobalFree(hmem);
            let _ = CloseClipboard();
            return false;
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        let _ = GlobalUnlock(hmem);
        if SetClipboardData(CF_UNICODEXT, hmem) == 0 {
            let _ = GlobalFree(hmem);
            let _ = CloseClipboard();
            return false;
        }
        let _ = CloseClipboard();
        true
    }
}

fn current_workspace_path() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| ".".to_string())
}

fn current_local_date_str() -> String {
    let st = unsafe { GetLocalTime() };
    format!("{:04}-{:02}-{:02}", st.wYear, st.wMonth, st.wDay)
}

fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn task_priority_to_str(priority: TaskPriority) -> &'static str {
    match priority {
        TaskPriority::Low => "low",
        TaskPriority::Medium => "medium",
        TaskPriority::High => "high",
        TaskPriority::Critical => "critical",
    }
}

fn map_productivity_task_to_card(record: &ProductivityTaskRecord) -> TaskCardItem {
    let priority = match record.priority.trim().to_ascii_lowercase().as_str() {
        "low" => TaskPriority::Low,
        "high" => TaskPriority::High,
        "critical" | "urgent" => TaskPriority::Critical,
        _ => TaskPriority::Medium,
    };
    let status = match record.status.trim().to_ascii_lowercase().as_str() {
        "done" | "completed" => TaskStatus::Done,
        "in_progress" | "active" => TaskStatus::InProgress,
        "review" => TaskStatus::Review,
        _ => TaskStatus::Todo,
    };
    let due_label = if record.column_id.eq_ignore_ascii_case("col-upcoming") {
        Some("Upcoming".to_string())
    } else if let Some(due) = record.due_date {
        if due > now_unix_secs() + 43_200 {
            Some("Upcoming".to_string())
        } else {
            Some("Today".to_string())
        }
    } else {
        Some("Today".to_string())
    };
    TaskCardItem {
        id: record.id.clone(),
        title: record.title.clone(),
        priority,
        status,
        due_label,
    }
}

fn lyrics_result_to_lines(res: &LyricsResult) -> (Vec<LrcLine>, Vec<String>) {
    if let Some(raw) = res.synced_lyrics.as_deref() {
        if !raw.trim().is_empty() {
            let parsed = parse_lrc_lines(raw);
            if !parsed.is_empty() {
                let texts = parsed.iter().map(|l| l.text.clone()).collect();
                return (parsed, texts);
            }
        }
    }
    if !res.lines.is_empty() {
        let mut parsed: Vec<LrcLine> = res
            .lines
            .iter()
            .map(|l| LrcLine {
                ms: l.ms as u64,
                text: l.text.clone(),
            })
            .collect();
        parsed.sort_by_key(|l| l.ms);
        let texts = parsed.iter().map(|l| l.text.clone()).collect();
        return (parsed, texts);
    }
    if let Some(plain) = res.plain_lyrics.as_deref() {
        let lines: Vec<String> = plain
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        return (Vec::new(), lines);
    }
    (Vec::new(), Vec::new())
}

fn summarize_screentime_sessions(
    sessions: &[ScreenTimeSessionRecord],
    today_date_str: &str,
) -> (u64, u64, u64) {
    let mut study = 0u64;
    let mut neutral = 0u64;
    let mut distracting = 0u64;
    for s in sessions {
        if let Some(d) = s.date_str.as_deref() {
            if !d.is_empty() && d != today_date_str {
                continue;
            }
        }
        let secs = s.duration_secs.max(0) as u64;
        match s.category.trim().to_ascii_lowercase().as_str() {
            "study" | "productive" | "work" | "focus" | "coding" | "learning" => {
                study = study.saturating_add(secs);
            }
            "distracting" | "entertainment" | "social" | "gaming" => {
                distracting = distracting.saturating_add(secs);
            }
            _ => {
                neutral = neutral.saturating_add(secs);
            }
        }
    }
    (study, neutral, distracting)
}

fn summarize_document_jobs(jobs: &[DocumentJobRecord]) -> DocumentsWorkspaceSummary {
    let mut active_jobs = 0usize;
    let mut completed_jobs = 0usize;
    let mut failed_jobs = 0usize;
    for job in jobs {
        match job.status.trim().to_ascii_lowercase().as_str() {
            "completed" | "done" | "success" => completed_jobs += 1,
            "failed" | "error" => failed_jobs += 1,
            _ => active_jobs += 1,
        }
    }
    let latest_job_label = jobs
        .iter()
        .max_by_key(|j| j.created_at)
        .map(|job| {
            let name = job
                .target_path
                .as_deref()
                .or(job.source_path.as_deref())
                .and_then(|p| Path::new(p).file_name())
                .and_then(|n| n.to_str())
                .unwrap_or(&job.operation);
            format!("{name} ({})", job.status)
        });
    DocumentsWorkspaceSummary {
        active_jobs,
        completed_jobs,
        failed_jobs,
        latest_job_label,
    }
}

fn proactive_level_from_suggestion(item: &ProactiveSuggestionItem) -> ProactiveLevel {
    if item.score >= 0.85 {
        ProactiveLevel::Penting
    } else if item.score >= 0.6 {
        ProactiveLevel::Tanya
    } else if item.score >= 0.35 {
        ProactiveLevel::Colek
    } else {
        ProactiveLevel::Bisik
    }
}

fn budget_to_attention_mode(budget: &str) -> &'static str {
    match budget.trim().to_ascii_lowercase().as_str() {
        "quiet" | "pendiam" => "pendiam",
        "chatty" | "cerewet" => "cerewet",
        _ => "seimbang",
    }
}

fn attention_mode_to_budget(mode: &str) -> &'static str {
    match mode.trim().to_ascii_lowercase().as_str() {
        "pendiam" | "quiet" => "quiet",
        "cerewet" | "chatty" => "chatty",
        _ => "balanced",
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

fn is_cursor_in_notch_drop_band(
    cursor_x: f32,
    cursor_y: f32,
    notch: &NotchView,
    screen: os_shell::ScreenBounds,
) -> bool {
    let rx = cursor_x - screen.x as f32;
    let ry = cursor_y - screen.y as f32;
    let sw = screen.width.max(1) as f32;
    let sh = screen.height.max(1) as f32;
    let rect = notch.computed_rect(sw, sh);
    match notch.edge {
        NotchEdge::Top => {
            let span = rect.width.max(320.0);
            let left = (sw - span) * 0.5;
            rx >= left && rx <= left + span && ry >= 0.0 && ry <= 48.0
        }
        NotchEdge::Left => {
            let span = rect.height.max(260.0);
            let top = (sh - span) * 0.5;
            rx >= 0.0 && rx <= 48.0 && ry >= top && ry <= top + span
        }
        NotchEdge::Right => {
            let span = rect.height.max(260.0);
            let top = (sh - span) * 0.5;
            rx >= (sw - 48.0).max(0.0) && rx <= sw && ry >= top && ry <= top + span
        }
    }
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
        let raw_wnd = HWND(hwnd_raw as *mut std::ffi::c_void);
        if IsWindow(Some(raw_wnd)).as_bool() {
            let root = GetAncestor(raw_wnd, GA_ROOT);
            let wnd = if !root.0.is_null() && IsWindow(Some(root)).as_bool() {
                root
            } else {
                raw_wnd
            };
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
        let is_first_fg = unsafe {
            let first = HWND(item.hwnds[0] as *mut std::ffi::c_void);
            let root = GetAncestor(first, GA_ROOT);
            item.hwnds[0] == fg || (!root.0.is_null() && root.0 as isize == fg)
        };
        let target_raw = if item.hwnds.len() > 1 && is_first_fg {
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
    let arrow_cursor = unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() };
    let wc = WNDCLASSW {
        lpfnWndProc: Some(controller_wnd_proc),
        hInstance: hinstance.into(),
        hCursor: arrow_cursor,
        lpszClassName: CONTROLLER_CLASS,
        ..Default::default()
    };
    unsafe {
        RegisterClassW(&wc);
        let _ = SetCursor(Some(arrow_cursor));
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

    let mut shell = AppShell::new(ctrl_hwnd).expect("initialize AppShell");
    let start_fast = shell.dock.is_animating() || shell.notch.is_hidden;
    if start_fast {
        shell.ensure_fast_timer(ctrl_hwnd);
    } else {
        shell.set_idle_timer(ctrl_hwnd);
    }
    let quit_raw_handle = shell.quit_event.map(|h| h.0 as isize);
    APP_SHELL.with(|cell| *cell.borrow_mut() = Some(shell));

    let hk_mgr = HotkeyManager::register(ctrl_hwnd, true);
    HOTKEY_MGR.with(|cell| *cell.borrow_mut() = Some(hk_mgr));
    let translate_hk_ok = unsafe {
        RegisterHotKey(
            Some(ctrl_hwnd),
            HOTKEY_TRANSLATE_CST,
            HOT_KEY_MODIFIERS((MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT).0),
            VK_T,
        )
        .is_ok()
    };

    #[link(name = "user32")]
    extern "system" {
        fn MsgWaitForMultipleObjects(
            nCount: u32,
            pHandles: *const isize,
            fWaitAll: i32,
            dwMilliseconds: u32,
            dwWakeMask: u32,
        ) -> u32;
    }
    const QS_ALLINPUT: u32 = 0x04FF;

    unsafe {
        let mut msg = MSG::default();
        if let Some(evt_raw) = quit_raw_handle {
            let mut should_quit = false;
            while !should_quit {
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    if msg.message == WM_QUIT {
                        should_quit = true;
                        break;
                    }
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
                if should_quit
                    || WaitForSingleObject(HANDLE(evt_raw as *mut c_void), 0).0 == 0
                {
                    break;
                }
                let wait_rc = MsgWaitForMultipleObjects(1, &evt_raw, 0, u32::MAX, QS_ALLINPUT);
                if wait_rc == 0 {
                    break;
                }
            }
        } else {
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
        }
        let _ = KillTimer(Some(ctrl_hwnd), TIMER_ANIM_ID);
    }

    if translate_hk_ok {
        unsafe {
            let _ = UnregisterHotKey(Some(ctrl_hwnd), HOTKEY_TRANSLATE_CST);
        }
    }
    HOTKEY_MGR.with(|cell| {
        cell.borrow_mut().take();
    });
    APP_SHELL.with(|cell| {
        if let Some(mut s) = cell.borrow_mut().take() {
            s.shutdown();
        }
    });
    remove_tray_icon(ctrl_hwnd);
}

unsafe extern "system" fn controller_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_SETCURSOR => {
            if ((lparam.0 & 0xFFFF) as u32) == HTCLIENT {
                if let Ok(arrow) = LoadCursorW(None, IDC_ARROW) {
                    let _ = SetCursor(Some(arrow));
                }
                return LRESULT(1);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_HOTKEY => {
            if wparam.0 as i32 == HOTKEY_TRANSLATE_CST {
                APP_SHELL.with(|cell| {
                    if let Some(shell) = cell.borrow_mut().as_mut() {
                        shell.trigger_translate_selection(hwnd);
                    }
                });
                return LRESULT(0);
            }
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
        WM_BACKDROP_READY => {
            APP_SHELL.with(|cell| {
                if let Some(shell) = cell.borrow_mut().as_mut() {
                    shell.drain_dynamic_backdrops();
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
        ns.color_theme = "light".to_string();
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
        assert_eq!(dw, 1080);
        assert_eq!(dh, 304);
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

    #[test]
    fn auto_hide_settings_startup_and_wake_on_hover_regression() {
        let ns = SettingsStore::from_json_str(r#"{"bloom-dock-mode":"auto-hide"}"#).unwrap();
        assert_eq!(ns.dock_mode, "auto-hide");
        assert_eq!(parse_dock_mode(&ns.dock_mode), DockMode::AutoHide);

        let mut dock = DockBar::new();
        dock.set_mode(parse_dock_mode(&ns.dock_mode));
        assert!(dock.is_hidden);
        assert!(dock.is_animating());

        let surf_rect = Rect::new(0.0, 0.0, DOCK_DIP_WIDTH, DOCK_DIP_HEIGHT);
        for _ in 0..45 {
            dock.tick(1.0 / 60.0);
        }
        assert_eq!(dock.show_progress.value, 0.0);
        let strip = dock.handle_strip_rect(surf_rect);
        assert_eq!(strip.width, 80.0);
        assert_eq!(strip.height, 4.0);

        // Hover wake at bottom edge brings Dock back to visible
        let wake = dock.wake_hit_rect(surf_rect);
        assert!(os_shell::is_point_in_rect(
            strip.x + 40.0,
            strip.y + 2.0,
            wake
        ));
        dock.set_edge_hovered(true);
        assert!(!dock.is_hidden);
        for _ in 0..45 {
            dock.tick(1.0 / 60.0);
        }
        assert!((dock.show_progress.value - 1.0).abs() < 0.05);

        // Leaving bottom edge starts grace timer and collapses back to 80x4
        dock.set_edge_hovered(false);
        for _ in 0..240 {
            dock.tick(1.0 / 60.0);
        }
        assert!(dock.is_hidden);
        assert_eq!(dock.show_progress.value, 0.0);
    }

    #[test]
    fn lyrics_result_parses_synced_lrc_and_falls_back_to_lines() {
        let synced = LyricsResult {
            cache_key: "k1".into(),
            title: "Nightcall".into(),
            artist: "Kavinsky".into(),
            album: None,
            duration_sec: 258,
            source: "lrclib".into(),
            synced_lyrics: Some("[00:01.00]First\n[00:04.50]Second".into()),
            plain_lyrics: None,
            lines: Vec::new(),
            is_synced: true,
            is_instrumental: false,
            user_offset_ms: -200,
        };
        let (lrc, texts) = lyrics_result_to_lines(&synced);
        assert_eq!(lrc.len(), 2);
        assert_eq!(lrc[0].ms, 1000);
        assert_eq!(texts, vec!["First", "Second"]);

        let from_vec = LyricsResult {
            synced_lyrics: None,
            lines: vec![hime_island_native::ipc_client::LyricLine {
                ms: 2500,
                text: "From vector".into(),
            }],
            ..synced
        };
        let (lrc2, texts2) = lyrics_result_to_lines(&from_vec);
        assert_eq!(lrc2.len(), 1);
        assert_eq!(lrc2[0].ms, 2500);
        assert_eq!(texts2, vec!["From vector"]);
    }

    #[test]
    fn productivity_and_workspace_mappers_convert_core_records() {
        let rec = ProductivityTaskRecord {
            id: "ptask-1".into(),
            board_id: "board-default".into(),
            column_id: "col-upcoming".into(),
            title: "Ship Phase 5".into(),
            description: None,
            priority: "critical".into(),
            status: "done".into(),
            due_date: None,
            estimated_minutes: None,
            actual_minutes: None,
            labels: None,
            parent_task_id: None,
            created_at: 100,
            updated_at: 100,
            completed_at: Some(200),
        };
        let card = map_productivity_task_to_card(&rec);
        assert_eq!(card.priority, TaskPriority::Critical);
        assert_eq!(card.status, TaskStatus::Done);
        assert_eq!(card.due_label.as_deref(), Some("Upcoming"));

        let sessions = vec![
            ScreenTimeSessionRecord {
                id: "s1".into(),
                app_name: "code.exe".into(),
                window_title: None,
                category: "coding".into(),
                duration_secs: 1800,
                timestamp: 100,
                date_str: Some("2026-11-26".into()),
            },
            ScreenTimeSessionRecord {
                id: "s2".into(),
                app_name: "youtube.exe".into(),
                window_title: None,
                category: "distracting".into(),
                duration_secs: 300,
                timestamp: 120,
                date_str: Some("2026-11-26".into()),
            },
            ScreenTimeSessionRecord {
                id: "s3".into(),
                app_name: "old.exe".into(),
                window_title: None,
                category: "study".into(),
                duration_secs: 9999,
                timestamp: 10,
                date_str: Some("2026-11-25".into()),
            },
        ];
        assert_eq!(
            summarize_screentime_sessions(&sessions, "2026-11-26"),
            (1800, 0, 300)
        );

        assert_eq!(budget_to_attention_mode("quiet"), "pendiam");
        assert_eq!(budget_to_attention_mode("balanced"), "seimbang");
        assert_eq!(budget_to_attention_mode("chatty"), "cerewet");
        assert_eq!(attention_mode_to_budget("pendiam"), "quiet");
        assert_eq!(attention_mode_to_budget("seimbang"), "balanced");
        assert_eq!(attention_mode_to_budget("cerewet"), "chatty");

        let jobs = vec![
            DocumentJobRecord {
                id: "j1".into(),
                operation: "merge_pdfs".into(),
                source_path: None,
                target_path: Some(r"C:\docs\merged.pdf".into()),
                status: "completed".into(),
                format: "pdf".into(),
                metadata: None,
                created_at: 20,
                completed_at: Some(25),
            },
            DocumentJobRecord {
                id: "j2".into(),
                operation: "create_report".into(),
                source_path: None,
                target_path: None,
                status: "running".into(),
                format: "html".into(),
                metadata: None,
                created_at: 10,
                completed_at: None,
            },
        ];
        let docs = summarize_document_jobs(&jobs);
        assert_eq!(docs.active_jobs, 1);
        assert_eq!(docs.completed_jobs, 1);
        assert_eq!(docs.failed_jobs, 0);
        assert_eq!(docs.latest_job_label.as_deref(), Some("merged.pdf (completed)"));
    }

    #[test]
    fn notch_drop_band_targets_anchored_edge_without_hijacking_corners() {
        let screen = os_shell::ScreenBounds::new(0, 0, 1920, 1080);
        let mut notch = NotchView::new(NotchEdge::Top);
        assert!(is_cursor_in_notch_drop_band(960.0, 24.0, &notch, screen));
        // Top-right close button / corner should not be stolen for Notch file drop
        assert!(!is_cursor_in_notch_drop_band(1900.0, 16.0, &notch, screen));

        notch.set_edge(NotchEdge::Left);
        assert!(is_cursor_in_notch_drop_band(20.0, 540.0, &notch, screen));
        assert!(!is_cursor_in_notch_drop_band(20.0, 40.0, &notch, screen));

        notch.set_edge(NotchEdge::Right);
        assert!(is_cursor_in_notch_drop_band(1900.0, 540.0, &notch, screen));
        assert!(!is_cursor_in_notch_drop_band(1820.0, 540.0, &notch, screen));
    }

    #[test]
    fn backdrop_slot_refreshes_dynamically_even_when_layout_unchanged() {
        let now = Instant::now();
        let slot_empty = BackdropSlot::default();
        assert!(!should_skip_settled_backdrop_refresh(&slot_empty, now, 123, false));

        // Wake slot skips only within the ~30 FPS cadence, then refreshes dynamically even when layout is unchanged
        let wake_slot_fresh = BackdropSlot {
            last_at: Some(now - Duration::from_millis(10)),
            last_hash: Some(0xdead_beef),
            layout_hash: Some(42),
            from_wake: true,
        };
        assert!(should_skip_settled_backdrop_refresh(&wake_slot_fresh, now, 42, false));

        let wake_slot_stale = BackdropSlot {
            last_at: Some(now - Duration::from_millis(45)),
            last_hash: Some(0xdead_beef),
            layout_hash: Some(42),
            from_wake: true,
        };
        assert!(!should_skip_settled_backdrop_refresh(&wake_slot_stale, now, 42, false));
        // Window opened/closed/moved behind surface: must refresh immediately
        assert!(!should_skip_settled_backdrop_refresh(&wake_slot_fresh, now, 99, false));
        // Forced refresh always runs
        assert!(!should_skip_settled_backdrop_refresh(&wake_slot_fresh, now, 42, true));

        // Non-wake composite slot also refreshes once STATIC_COMPOSITE_REFRESH_INTERVAL (33ms) elapses
        let composite_slot = BackdropSlot {
            last_at: Some(now - Duration::from_millis(10)),
            last_hash: Some(0xdead_beef),
            layout_hash: Some(42),
            from_wake: false,
        };
        assert!(should_skip_settled_backdrop_refresh(&composite_slot, now, 42, false));
        assert!(!should_skip_settled_backdrop_refresh(
            &composite_slot,
            now + Duration::from_millis(40),
            42,
            false
        ));
    }
}
