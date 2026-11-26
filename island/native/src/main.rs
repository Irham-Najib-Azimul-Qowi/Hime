//! Hime Island Native Desktop Shell (`hime-island-native.exe`).
//! Pure-Rust Direct2D + DirectWrite + DirectComposition shell with global hotkeys,
//! system tray lifecycle, idle-friendly spring tick, and optional Hime AI integration.

#![windows_subsystem = "windows"]

use std::cell::RefCell;
use std::time::{Duration, Instant};

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    FindWindowW, GetCursorPos, GetMessageW, GetSystemMetrics, KillTimer, LoadIconW,
    PostQuitMessage, RegisterClassW, SetForegroundWindow, SetTimer, TrackPopupMenu,
    TranslateMessage, IDI_APPLICATION, MF_STRING, MSG, SM_CXSCREEN, SM_CYSCREEN, TPM_BOTTOMALIGN,
    TPM_LEFTALIGN, TPM_RETURNCMD, WINDOW_EX_STYLE, WM_APP, WM_DESTROY, WM_HOTKEY, WM_LBUTTONUP,
    WM_RBUTTONUP, WM_TIMER, WNDCLASSW, WS_OVERLAPPED,
};

use hime_island_native::ai_drawer::{AiDrawer, DrawerAttachment};
use hime_island_native::d2d::NativeSurface;
use hime_island_native::dock::DockBar;
use hime_island_native::hotkey::{self, HotkeyAction, HotkeyManager};
use hime_island_native::ipc_client::HimeIpcClient;
use hime_island_native::module_mgr::HimeAiModuleManager;
use hime_island_native::notch::{NotchEdge, NotchView};
use hime_island_native::overlay::{HudEdge, OverlayHud};
use hime_island_native::settings_ui::SettingsPanel;
use hime_island_native::shelf::FileShelf;

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

struct AppShell {
    notch: NotchView,
    dock: DockBar,
    overlay: OverlayHud,
    settings: SettingsPanel,
    ai_drawer: AiDrawer,
    shelf: FileShelf,
    module_mgr: HimeAiModuleManager,
    _ipc: HimeIpcClient,
    notch_surface: NativeSurface,
    dock_surface: NativeSurface,
    overlay_surface: NativeSurface,
    settings_surface: NativeSurface,
    drawer_surface: NativeSurface,
    last_tick: Instant,
    fast_timer_active: bool,
}

impl AppShell {
    fn new() -> windows::core::Result<Self> {
        let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) }.max(800);
        let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) }.max(600);

        let notch_w = 480u32;
        let notch_h = 280u32;
        let notch_surface = NativeSurface::new(
            "HimeNotch",
            (screen_w - notch_w as i32) / 2,
            0,
            notch_w,
            notch_h,
        )?;
        notch_surface.set_click_through(true);
        notch_surface.show();

        let dock_w = 640u32;
        let dock_h = 96u32;
        let dock_surface = NativeSurface::new(
            "HimeDock",
            (screen_w - dock_w as i32) / 2,
            screen_h - dock_h as i32 - 8,
            dock_w,
            dock_h,
        )?;
        dock_surface.show();

        let overlay_surface = NativeSurface::new("HimeOverlay", 16, screen_h / 3, 240, 120)?;
        overlay_surface.set_click_through(true);

        let settings_surface = NativeSurface::new(
            "HimeSettings",
            (screen_w - 420) / 2,
            (screen_h - 440) / 2,
            420,
            440,
        )?;

        let drawer_surface = NativeSurface::new(
            "HimeAiDrawer",
            (screen_w - 440) / 2,
            84,
            440,
            500,
        )?;

        let shelf = FileShelf::load().unwrap_or_else(|_| FileShelf::new());

        let module_mgr = HimeAiModuleManager::with_default_dir();
        let router_probe = module_mgr.probe_9router(Duration::from_millis(200));

        let mut settings = SettingsPanel::new();
        settings.state.router_9_detected = router_probe.available;

        let shell = Self {
            notch: NotchView::new(NotchEdge::Top),
            dock: DockBar::new(),
            overlay: OverlayHud::new(HudEdge::Left),
            settings,
            ai_drawer: AiDrawer::new(),
            shelf,
            module_mgr,
            _ipc: HimeIpcClient::with_default_pipe(),
            notch_surface,
            dock_surface,
            overlay_surface,
            settings_surface,
            drawer_surface,
            last_tick: Instant::now(),
            fast_timer_active: false,
        };
        shell.render_all();
        Ok(shell)
    }

    fn handle_action(&mut self, action: HotkeyAction, ctrl_hwnd: HWND) {
        match action {
            HotkeyAction::ToggleAssistant => {
                if !self.ai_drawer.visible {
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
                self.ai_drawer.toggle();
                if self.ai_drawer.visible {
                    self.drawer_surface.show();
                } else {
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
                if next {
                    self.dock_surface.show();
                } else {
                    self.dock_surface.hide();
                }
            }
            HotkeyAction::ToggleShortcuts => {
                self.settings.toggle();
                if self.settings.visible {
                    self.settings_surface.show();
                } else {
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
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32().min(0.05);
        self.last_tick = now;

        for action in hotkey::drain_ll_actions() {
            self.handle_action(action, ctrl_hwnd);
        }

        let mut animating = false;
        animating |= self.notch.tick(dt);
        animating |= self.dock.tick(dt);
        animating |= self.overlay.tick(dt);
        animating |= self.settings.tick(dt);
        animating |= self.ai_drawer.tick(dt);

        if animating {
            self.render_all();
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

    fn render_all(&self) {
        self.notch_surface.begin_draw();
        let _ = self.notch.render(&self.notch_surface);
        let _ = self.notch_surface.end_draw_and_present();

        if self.dock.visible {
            self.dock_surface.begin_draw();
            let _ = self.dock.render(&self.dock_surface);
            let _ = self.dock_surface.end_draw_and_present();
        }

        if self.overlay.is_visible() {
            self.overlay_surface.show();
            self.overlay_surface.begin_draw();
            let _ = self.overlay.render(&self.overlay_surface);
            let _ = self.overlay_surface.end_draw_and_present();
        } else {
            self.overlay_surface.hide();
        }

        if self.settings.visible || self.settings.is_animating() {
            self.settings_surface.begin_draw();
            let _ = self.settings.render(&self.settings_surface);
            let _ = self.settings_surface.end_draw_and_present();
        }

        if self.ai_drawer.visible || self.ai_drawer.is_animating() {
            self.drawer_surface.begin_draw();
            let _ = self.ai_drawer.render(&self.drawer_surface);
            let _ = self.drawer_surface.end_draw_and_present();
        }
    }

    fn shutdown(&mut self) {
        let _ = self.shelf.save();
        let _ = self.module_mgr.stop();
    }
}

thread_local! {
    static APP_SHELL: RefCell<Option<AppShell>> = const { RefCell::new(None) };
    static HOTKEY_MGR: RefCell<Option<HotkeyManager>> = const { RefCell::new(None) };
}

fn main() {
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

    let shell = AppShell::new().expect("initialize AppShell");
    APP_SHELL.with(|cell| *cell.borrow_mut() = Some(shell));

    let hk_mgr = HotkeyManager::register(ctrl_hwnd, true);
    HOTKEY_MGR.with(|cell| *cell.borrow_mut() = Some(hk_mgr));

    unsafe {
        let _ = SetTimer(Some(ctrl_hwnd), TIMER_ANIM_ID, ANIM_INTERVAL_IDLE_MS, None);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        let _ = KillTimer(Some(ctrl_hwnd), TIMER_ANIM_ID);
    }

    // Explicit cleanup on exit
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
