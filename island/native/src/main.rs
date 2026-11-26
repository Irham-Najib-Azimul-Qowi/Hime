//! Hime Island Native Desktop Shell (`hime-island-native.exe`).
//! Pure-Rust Direct2D + DirectWrite + DirectComposition shell with live Win32 telemetry,
//! safe taskbar integration, 3-edge Dynamic Island, Dock, HUD Overlay, Settings, and Hime AI IPC.

#![windows_subsystem = "windows"]

use std::cell::RefCell;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows::Win32::UI::Shell::{
    ShellExecuteW, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    FindWindowW, GetCursorPos, GetMessageW, KillTimer, LoadIconW, PostQuitMessage, RegisterClassW,
    SetForegroundWindow, SetTimer, TrackPopupMenu, TranslateMessage, IDI_APPLICATION, MF_STRING,
    MSG, SW_SHOWNORMAL, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD, WINDOW_EX_STYLE, WM_APP,
    WM_CHAR, WM_DESTROY, WM_HOTKEY, WM_KEYDOWN, WM_LBUTTONUP, WM_RBUTTONUP, WM_TIMER, WNDCLASSW,
    WS_OVERLAPPED,
};

use hime_island_native::ai_drawer::{
    AiDrawer, ApprovalCard, ApprovalStatus, DrawerAction, DrawerAttachment,
};
use hime_island_native::d2d::{
    self, HimeExpression, NativeSurface, Rect, SurfaceEvent, SurfaceEventKind, WM_SURFACE_EVENT,
};
use hime_island_native::dock::{DockBar, DockItem};
use hime_island_native::hotkey::{self, HotkeyAction, HotkeyManager};
use hime_island_native::ipc_client::{
    AiProfileUpsertParams, ApprovalResolution, ChatTurnParams, ChatTurnResult, HimeIpcClient,
    IpcError, IPC_CONTRACT_VERSION,
};
use hime_island_native::module_mgr::HimeAiModuleManager;
use hime_island_native::notch::{NotchEdge, NotchView};
use hime_island_native::os_shell;
use hime_island_native::overlay::{HudEdge, OverlayHud};
use hime_island_native::settings_ui::{NotchMode, SettingsControl, SettingsPanel};
use hime_island_native::shelf::FileShelf;
use hime_island_native::telemetry::{TelemetryHandle, TelemetrySnapshot};

const CONTROLLER_CLASS: PCWSTR = w!("HimeIslandNativeController");
const TRAY_ICON_ID: u32 = 1;
const WM_TRAY_CALLBACK: u32 = WM_APP + 1;
const TIMER_ANIM_ID: usize = 1;
const ANIM_INTERVAL_ACTIVE_MS: u32 = 16;
const ANIM_INTERVAL_IDLE_MS: u32 = 250;

const MENU_TOGGLE_NOTCH: usize = 1001;
const MENU_TOGGLE_DOCK: usize = 1002;
const MENU_TOGGLE_AI_DRAWER: usize = 1003;
const MENU_TOGGLE_SETTINGS: usize = 1004;
const MENU_EXIT: usize = 1005;

enum IpcBgResult {
    ChatTurn(Result<ChatTurnResult, IpcError>),
    ProfileTest(Result<String, IpcError>),
    ApprovalResolved(Result<(), IpcError>),
}

struct AppShell {
    notch: NotchView,
    dock: DockBar,
    overlay: OverlayHud,
    settings: SettingsPanel,
    ai_drawer: AiDrawer,
    shelf: FileShelf,
    telemetry: TelemetryHandle,
    module_mgr: HimeAiModuleManager,
    ipc: Arc<HimeIpcClient>,
    ipc_tx: mpsc::Sender<IpcBgResult>,
    ipc_rx: mpsc::Receiver<IpcBgResult>,
    notch_surface: NativeSurface,
    dock_surface: NativeSurface,
    overlay_surface: NativeSurface,
    settings_surface: NativeSurface,
    drawer_surface: NativeSurface,
    last_tick: Instant,
    fast_timer_active: bool,
    last_volume: Option<(u8, bool)>,
    dragging_notch: bool,
    drag_moved: bool,
    drag_start_cursor: (i32, i32),
    quit_event: Option<HANDLE>,
    tick_count: u64,
}

impl AppShell {
    fn new(ctrl_hwnd: HWND) -> windows::core::Result<Self> {
        let screen = os_shell::primary_screen_bounds();

        let (nx, ny, nw, nh) = notch_surface_bounds(NotchEdge::Top, screen);
        let notch_surface = NativeSurface::new("HimeNotch", nx, ny, nw, nh)?;
        notch_surface.set_click_through(true);
        notch_surface.show();
        os_shell::enable_drag_drop(notch_surface.hwnd(), true);

        let (dx, dy, dock_w, dock_h) = dock_surface_bounds(screen);
        let dock_surface = NativeSurface::new("HimeDock", dx, dy, dock_w, dock_h)?;
        dock_surface.show();

        let overlay_surface =
            NativeSurface::new("HimeOverlay", screen.x + 16, screen.y + screen.height / 3, 280, 120)?;
        overlay_surface.set_click_through(true);

        let settings_surface = NativeSurface::new(
            "HimeSettings",
            screen.x + (screen.width - 420) / 2,
            screen.y + (screen.height - 440) / 2,
            420,
            440,
        )?;

        let drawer_surface = NativeSurface::new(
            "HimeAiDrawer",
            screen.x + (screen.width - 440) / 2,
            screen.y + 84,
            440,
            500,
        )?;
        os_shell::enable_drag_drop(drawer_surface.hwnd(), true);

        d2d::set_surface_event_target(ctrl_hwnd);

        let shelf = FileShelf::load().unwrap_or_else(|_| FileShelf::new());
        let module_mgr = HimeAiModuleManager::with_default_dir();
        let router_probe = module_mgr.probe_9router(Duration::from_millis(200));

        let mut settings = SettingsPanel::new();
        settings.state.router_9_detected = router_probe.available;

        let telemetry = TelemetryHandle::start();
        let (ipc_tx, ipc_rx) = mpsc::channel();
        let ipc = Arc::new(HimeIpcClient::with_default_pipe());

        let quit_event = unsafe { CreateEventW(None, true, false, w!("HimeQuit")) }.ok();

        let mut dock = DockBar::new();
        dock.set_visible(settings.state.dock_enabled);
        if dock.visible {
            os_shell::hide_taskbar();
        }

        let mut shell = Self {
            notch: NotchView::new(NotchEdge::Top),
            dock,
            overlay: OverlayHud::new(HudEdge::Left),
            settings,
            ai_drawer: AiDrawer::new(),
            shelf,
            telemetry,
            module_mgr,
            ipc,
            ipc_tx,
            ipc_rx,
            notch_surface,
            dock_surface,
            overlay_surface,
            settings_surface,
            drawer_surface,
            last_tick: Instant::now(),
            fast_timer_active: false,
            last_volume: None,
            dragging_notch: false,
            drag_moved: false,
            drag_start_cursor: (0, 0),
            quit_event,
            tick_count: 0,
        };
        let initial_snap = shell.telemetry.snapshot();
        shell.apply_telemetry(&initial_snap, false);
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
        use windows::Win32::Foundation::RECT;
        use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, IsWindowVisible};
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

    fn apply_telemetry(&mut self, snap: &TelemetrySnapshot, notify_volume: bool) {
        self.notch.set_clock(snap.hours, snap.minutes);
        self.notch.set_net_stats(snap.network.stats);
        self.notch.set_battery(snap.battery);
        self.notch.set_music(snap.music.clone());

        let expr = if snap.network.stats.is_internet_down {
            HimeExpression::Offline
        } else if snap.music.as_ref().map(|m| m.is_playing).unwrap_or(false) {
            HimeExpression::Senang
        } else {
            HimeExpression::Netral
        };
        self.notch.set_expression(expr);

        // Sync running windows into DockBar while preserving pinned items.
        let active_ids: Vec<String> = snap
            .running_windows
            .iter()
            .map(|w| format!("win-{}", w.hwnd))
            .collect();
        let stale_ids: Vec<String> = self
            .dock
            .items()
            .iter()
            .filter(|it| !it.pinned && !active_ids.contains(&it.id))
            .map(|it| it.id.clone())
            .collect();
        for id in stale_ids {
            self.dock.remove_item(&id);
        }
        for win in snap.running_windows.iter().take(12) {
            self.dock.add_item(win.to_dock_item());
        }

        let vol_pair = (snap.volume.percent, snap.volume.muted);
        if notify_volume && self.last_volume.is_some_and(|prev| prev != vol_pair) {
            self.overlay.show_volume(
                snap.volume.percent,
                snap.volume.muted,
                snap.volume.device_name.clone(),
            );
        }
        self.last_volume = Some(vol_pair);
    }

    fn sync_drawer_attachments(&mut self) {
        let attachments = self
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
        self.ai_drawer.attach_items(attachments);
    }

    fn apply_notch_edge(&mut self, edge: NotchEdge) {
        if self.notch.edge == edge {
            return;
        }
        self.notch.set_edge(edge);
        let screen = os_shell::primary_screen_bounds();
        let (x, y, w, h) = notch_surface_bounds(edge, screen);
        let _ = self.notch_surface.set_bounds(x, y, w, h);
    }

    fn update_surface_click_through(&self) {
        if let Some((cx, cy)) = os_shell::cursor_position() {
            let screen = os_shell::screen_bounds_at_point(cx, cy);
            let (nx, ny, _, _) = notch_surface_bounds(self.notch.edge, screen);
            let notch_scale = (self.notch_surface.dpi() / 96.0).max(0.5);
            let (nw_dip, nh_dip) = self.notch_surface.dip_size();
            let local_nx = (cx - nx) as f32 / notch_scale;
            let local_ny = (cy - ny) as f32 / notch_scale;
            let over_notch =
                os_shell::hit_test_notch(local_nx, local_ny, &self.notch, nw_dip, nh_dip);
            let near_wake = os_shell::is_notch_edge_hovered(
                cx as f32,
                cy as f32,
                &self.notch,
                screen,
                os_shell::DEFAULT_EDGE_WAKE_THRESHOLD_PX,
            );
            self.notch_surface
                .set_click_through(!(over_notch || near_wake || self.dragging_notch));

            if self.dock.visible {
                let primary = os_shell::primary_screen_bounds();
                let (dx, dy, _, _) = dock_surface_bounds(primary);
                let dock_scale = (self.dock_surface.dpi() / 96.0).max(0.5);
                let (dw_dip, dh_dip) = self.dock_surface.dip_size();
                let local_dx = (cx - dx) as f32 / dock_scale;
                let local_dy = (cy - dy) as f32 / dock_scale;
                let dock_rect = self.dock.dock_rect(Rect::new(0.0, 0.0, dw_dip, dh_dip));
                let over_dock = os_shell::is_point_in_rect(local_dx, local_dy, dock_rect);
                self.dock_surface.set_click_through(!over_dock);
            } else {
                self.dock_surface.set_click_through(true);
            }
        }
        self.drawer_surface
            .set_click_through(!self.ai_drawer.visible);
        self.settings_surface
            .set_click_through(!self.settings.visible);
    }

    fn handle_surface_event(&mut self, ev: SurfaceEvent, ctrl_hwnd: HWND) {
        if ev.hwnd == self.notch_surface.hwnd().0 as isize {
            match ev.kind {
                SurfaceEventKind::LeftButtonDown { .. } => {
                    self.dragging_notch = true;
                    self.drag_moved = false;
                    self.drag_start_cursor = os_shell::cursor_position().unwrap_or((0, 0));
                }
                SurfaceEventKind::MouseMove { .. } => {
                    if self.notch.is_hidden {
                        self.notch.set_hidden(false);
                    }
                    if self.dragging_notch {
                        if let Some((cx, cy)) = os_shell::cursor_position() {
                            let dx = (cx - self.drag_start_cursor.0).abs();
                            let dy = (cy - self.drag_start_cursor.1).abs();
                            if dx > 8 || dy > 8 {
                                self.drag_moved = true;
                                let next_edge = os_shell::snap_notch_edge_from_cursor();
                                self.apply_notch_edge(next_edge);
                            }
                        }
                    }
                }
                SurfaceEventKind::LeftButtonUp { .. } => {
                    if self.dragging_notch {
                        self.dragging_notch = false;
                        if !self.drag_moved {
                            self.notch.set_expanded(!self.notch.is_expanded);
                        }
                    }
                }
                SurfaceEventKind::DropFiles { wparam } => {
                    let added = os_shell::handle_wm_dropfiles(WPARAM(wparam), &mut self.shelf);
                    if !added.is_empty() {
                        let _ = self.shelf.save();
                        self.sync_drawer_attachments();
                        self.notch
                            .set_ncm_center_alert(Some(format!("+{} file(s)", added.len())));
                    }
                }
            }
            self.ensure_fast_timer(ctrl_hwnd);
            return;
        }

        if ev.hwnd == self.dock_surface.hwnd().0 as isize {
            let scale = (self.dock_surface.dpi() / 96.0).max(0.5);
            let (dw, dh) = self.dock_surface.dip_size();
            let rect = Rect::new(0.0, 0.0, dw, dh);
            match ev.kind {
                SurfaceEventKind::MouseMove { x, y } => {
                    self.dock.set_hovered_at(x / scale, y / scale, rect);
                }
                SurfaceEventKind::LeftButtonUp { x, y } => {
                    if let Some(idx) = self.dock.hit_test(x / scale, y / scale, rect) {
                        if let Some(item) = self.dock.items().get(idx).cloned() {
                            launch_dock_item(&item);
                        }
                    }
                }
                SurfaceEventKind::LeftButtonDown { .. } | SurfaceEventKind::DropFiles { .. } => {}
            }
            self.ensure_fast_timer(ctrl_hwnd);
            return;
        }

        if ev.hwnd == self.settings_surface.hwnd().0 as isize {
            let scale = (self.settings_surface.dpi() / 96.0).max(0.5);
            match ev.kind {
                SurfaceEventKind::LeftButtonDown { .. } => {
                    if self.settings.visible {
                        os_shell::focus_window(self.settings_surface.hwnd());
                    }
                }
                SurfaceEventKind::LeftButtonUp { x, y } => {
                    if let Some(ctrl) = self.settings.hit_test(x / scale, y / scale) {
                        self.apply_settings_control(ctrl, ctrl_hwnd);
                    } else if self.settings.visible {
                        os_shell::focus_window(self.settings_surface.hwnd());
                    }
                }
                SurfaceEventKind::MouseMove { .. } | SurfaceEventKind::DropFiles { .. } => {}
            }
            return;
        }

        if ev.hwnd == self.drawer_surface.hwnd().0 as isize {
            let scale = (self.drawer_surface.dpi() / 96.0).max(0.5);
            match ev.kind {
                SurfaceEventKind::LeftButtonDown { .. } => {
                    if self.ai_drawer.visible {
                        os_shell::focus_window(self.drawer_surface.hwnd());
                    }
                }
                SurfaceEventKind::LeftButtonUp { x, y } => {
                    if let Some(action) = self.ai_drawer.hit_test(x / scale, y / scale) {
                        self.apply_drawer_action(action, ctrl_hwnd);
                    } else if self.ai_drawer.visible {
                        os_shell::focus_window(self.drawer_surface.hwnd());
                    }
                }
                SurfaceEventKind::DropFiles { wparam } => {
                    let added = os_shell::handle_wm_dropfiles(WPARAM(wparam), &mut self.shelf);
                    if !added.is_empty() {
                        let _ = self.shelf.save();
                        self.sync_drawer_attachments();
                        self.ensure_fast_timer(ctrl_hwnd);
                    }
                }
                SurfaceEventKind::MouseMove { .. } => {}
            }
        }
    }

    fn apply_settings_control(&mut self, ctrl: SettingsControl, ctrl_hwnd: HWND) {
        self.settings.apply_control(ctrl);
        match ctrl {
            SettingsControl::CycleNotchMode => match self.settings.state.notch_mode {
                NotchMode::Top => {
                    self.notch.set_hidden(false);
                    self.apply_notch_edge(NotchEdge::Top);
                }
                NotchMode::Left => {
                    self.notch.set_hidden(false);
                    self.apply_notch_edge(NotchEdge::Left);
                }
                NotchMode::Right => {
                    self.notch.set_hidden(false);
                    self.apply_notch_edge(NotchEdge::Right);
                }
                NotchMode::Hidden => {
                    self.notch.set_hidden(true);
                }
            },
            SettingsControl::ToggleDock => {
                let enabled = self.settings.state.dock_enabled;
                self.dock.set_visible(enabled);
                if enabled {
                    self.dock_surface.show();
                    os_shell::hide_taskbar();
                } else {
                    self.dock_surface.hide();
                    os_shell::restore_taskbar();
                }
            }
            SettingsControl::ToggleHimeAi => {
                if self.settings.state.hime_ai_enabled {
                    let _ = self.module_mgr.spawn();
                } else {
                    let _ = self.module_mgr.stop();
                }
            }
            SettingsControl::TestAndSaveProfile => {
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
            SettingsControl::Close => {
                os_shell::set_window_focusable(self.settings_surface.hwnd(), false);
                self.settings_surface.hide();
            }
            SettingsControl::ToggleDockWinNumber
            | SettingsControl::ToggleNotifSuppression
            | SettingsControl::CycleProviderPreset => {}
        }
        self.ensure_fast_timer(ctrl_hwnd);
    }

    fn apply_drawer_action(&mut self, action: DrawerAction, ctrl_hwnd: HWND) {
        match action {
            DrawerAction::Close => {
                self.ai_drawer.close();
                os_shell::set_window_focusable(self.drawer_surface.hwnd(), false);
                self.drawer_surface.hide();
            }
            DrawerAction::ClearAttachments => {
                self.ai_drawer.clear_attachments();
                self.shelf.clear();
                let _ = self.shelf.save();
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
                WM_CHAR => char::from_u32(msg.wParam.0 as u32)
                    .and_then(|ch| self.ai_drawer.handle_char(ch)),
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
            let ctrl = match msg.message {
                WM_KEYDOWN => self.settings.handle_key_down(msg.wParam.0 as u32),
                WM_CHAR => char::from_u32(msg.wParam.0 as u32)
                    .and_then(|ch| self.settings.handle_char(ch)),
                _ => None,
            };
            if let Some(c) = ctrl {
                self.apply_settings_control(c, ctrl_hwnd);
            } else {
                self.ensure_fast_timer(ctrl_hwnd);
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
                if !self.ai_drawer.visible {
                    self.sync_drawer_attachments();
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
                }
            }
            HotkeyAction::ToggleNotch | HotkeyAction::ToggleCalendar => {
                let next = !self.notch.is_expanded;
                self.notch.set_expanded(next);
            }
            HotkeyAction::ToggleMusic => {
                self.notch.set_expanded(!self.notch.is_expanded);
            }
            HotkeyAction::ToggleDock => {
                let next = !self.dock.visible;
                self.dock.set_visible(next);
                self.settings.state.dock_enabled = next;
                if next {
                    self.dock_surface.show();
                    os_shell::hide_taskbar();
                    self.update_surface_click_through();
                    self.render_all();
                } else {
                    self.dock_surface.hide();
                    os_shell::restore_taskbar();
                }
            }
            HotkeyAction::ToggleShortcuts => {
                self.settings.toggle();
                if self.settings.visible {
                    self.settings_surface.set_click_through(false);
                    os_shell::set_window_focusable(self.settings_surface.hwnd(), true);
                    self.settings_surface.show();
                    os_shell::focus_window(self.settings_surface.hwnd());
                    self.render_all();
                } else {
                    os_shell::set_window_focusable(self.settings_surface.hwnd(), false);
                    self.settings_surface.hide();
                }
            }
            HotkeyAction::FocusDockSlot(slot) => {
                self.dock.set_hovered(Some(slot as usize));
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
                    self.notch.set_ncm_center_alert(Some(msg));
                }
                IpcBgResult::ProfileTest(Err(err)) => {
                    self.notch.set_ncm_center_alert(Some(format!("IPC: {err}")));
                }
                IpcBgResult::ApprovalResolved(Ok(())) => {}
                IpcBgResult::ApprovalResolved(Err(err)) => {
                    self.notch.set_ncm_center_alert(Some(format!("Approval: {err}")));
                }
            }
        }

        let snap = self.telemetry.snapshot();
        self.apply_telemetry(&snap, true);
        self.update_surface_click_through();

        let mut animating = false;
        animating |= self.notch.tick(dt);
        animating |= self.dock.tick(dt);
        animating |= self.overlay.tick(dt);
        animating |= self.settings.tick(dt);
        animating |= self.ai_drawer.tick(dt);

        self.render_all();
        self.tick_count = self.tick_count.saturating_add(1);
        if self.tick_count == 12 {
            self.log_tick_diag("tick_3s");
            os_shell::ensure_topmost(self.notch_surface.hwnd());
            if self.dock.visible {
                os_shell::ensure_topmost(self.dock_surface.hwnd());
            }
        }
        if animating {
            self.ensure_fast_timer(ctrl_hwnd);
        } else if self.fast_timer_active {
            self.set_idle_timer(ctrl_hwnd);
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

        if self.overlay.is_visible() {
            self.overlay_surface.show();
            self.render_surface("HimeOverlay", &self.overlay_surface, |s| {
                self.overlay.render(s)
            });
        } else {
            self.overlay_surface.hide();
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

fn dock_surface_bounds(screen: os_shell::ScreenBounds) -> (i32, i32, u32, u32) {
    let w = 720u32;
    let h = 96u32;
    (
        screen.x + (screen.width - w as i32) / 2,
        screen.y + screen.height - h as i32 - 8,
        w,
        h,
    )
}

fn notch_surface_bounds(edge: NotchEdge, screen: os_shell::ScreenBounds) -> (i32, i32, u32, u32) {
    match edge {
        NotchEdge::Top => {
            let w = 540u32;
            let h = 200u32;
            (screen.x + (screen.width - w as i32) / 2, screen.y, w, h)
        }
        NotchEdge::Left => {
            let w = 320u32;
            let h = 380u32;
            (screen.x, screen.y + (screen.height - h as i32) / 2, w, h)
        }
        NotchEdge::Right => {
            let w = 320u32;
            let h = 380u32;
            (
                screen.x + screen.width - w as i32,
                screen.y + (screen.height - h as i32) / 2,
                w,
                h,
            )
        }
    }
}

fn launch_dock_item(item: &DockItem) {
    if item.executable.trim().is_empty() {
        return;
    }
    let wide: Vec<u16> = OsStr::new(&item.executable)
        .encode_wide()
        .chain(Some(0))
        .collect();
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
    // Crash-recovery: restore taskbar if a previous run crashed while hidden.
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

    let action = match cmd.0 as usize {
        MENU_TOGGLE_NOTCH => Some(HotkeyAction::ToggleNotch),
        MENU_TOGGLE_DOCK => Some(HotkeyAction::ToggleDock),
        MENU_TOGGLE_AI_DRAWER => Some(HotkeyAction::ToggleAssistant),
        MENU_TOGGLE_SETTINGS => Some(HotkeyAction::ToggleShortcuts),
        MENU_EXIT => Some(HotkeyAction::StopHime),
        _ => None,
    };
    if let Some(act) = action {
        APP_SHELL.with(|cell| {
            if let Some(shell) = cell.borrow_mut().as_mut() {
                shell.handle_action(act, hwnd);
            }
        });
    }
}
