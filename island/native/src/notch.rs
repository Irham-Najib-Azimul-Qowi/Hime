//! Native 3-edge Dynamic Island / Notch surface (`NotchView`) rendered via Direct2D.
//!
//! Visual and functional parity with the original Tauri Hime Dynamic Island:
//! - Pure `#000000` (~94% opacity) `.edge-surface` pill with concave flares and 1px rim border.
//! - Crisp `#FFFFFF` (`TEXT_PRIMARY`) status, clock, speed, battery, and menu typography.
//! - Interactive multi-panel Notch (`Status`, `CommandCenter`, `Calendar`, `Music`, `Wifi`,
//!   `Shortcuts`, `Shelf`, `HimeAi`) with shared layout/hit-testing via `hit_test_click`.

use serde::{Deserialize, Serialize};
use windows::core::Result;
use windows::Win32::System::SystemInformation::GetLocalTime;

use crate::animation::SpringValue;
use crate::d2d::{EdgeSide, HimeExpression, NativeSurface, Rect, TextAlign};
use crate::notch_workspace::{
    active_lyric_index, parse_lrc_lines, FocusHitAction, FocusSessionStatus, FocusWorkspaceState,
    LrcLine, ProactiveBannerAction, ProactiveBannerState, TaskTabFilter, TasksHitAction,
    TasksWorkspaceState, TranslateBannerAction, TranslateBannerState,
};
use crate::theme::{
    ACCENT_CYAN, CORNER_RADIUS, SAFE_PADDING, STATUS_ALERT, STATUS_SUCCESS, TEXT_PRIMARY,
};

const HIDDEN_BAR_THICKNESS: f32 = 3.0;
const TOP_COLLAPSED_WIDTH: f32 = 276.0;
const TOP_ALERT_WIDTH: f32 = 344.0;
const TOP_BANNER_WIDTH: f32 = 380.0;
const TOP_BANNER_HEIGHT: f32 = 96.0;
const TOP_COLLAPSED_HEIGHT: f32 = 36.0;
const TOP_EXPANDED_WIDTH: f32 = 400.0;
const TOP_EXPANDED_HEIGHT: f32 = 280.0;
const TOP_SHORTCUTS_WIDTH: f32 = 464.0;
const TOP_SHORTCUTS_HEIGHT: f32 = 280.0;
const SIDE_COLLAPSED_WIDTH: f32 = 46.0;
const SIDE_COLLAPSED_HEIGHT: f32 = 200.0;
const SIDE_ALERT_HEIGHT: f32 = 256.0;
const SIDE_BANNER_WIDTH: f32 = 300.0;
const SIDE_BANNER_HEIGHT: f32 = 112.0;
const SIDE_EXPANDED_WIDTH: f32 = 400.0;
const SIDE_EXPANDED_HEIGHT: f32 = 300.0;
const EAR_WIDTH: f32 = 13.0;

const HEADER_HEIGHT: f32 = 36.0;

/// Deep black surface matching `.edge-surface` (`#000000` at ~94% alpha).
const NOTCH_BG_BLACK: crate::theme::Color = crate::theme::Color::from_rgba_u8(0, 0, 0, 240);
/// Thin 1px outer border rim matching `.edge-surface`.
const NOTCH_EDGE_BORDER: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 38);
/// Subtle elevated card background inside the black notch.
const CARD_BG: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 14);
const CARD_BG_ACTIVE: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 28);
const CARD_BORDER: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 26);
/// Muted white for secondary captions (kept white-tinted, never blue/cyan).
const TEXT_MUTED_WHITE: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 175);
const TEXT_SUBTLE_WHITE: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 120);
/// Hime crimson accent for active Pomodoro / Today highlight / subtle dots.
const HIME_CRIMSON: crate::theme::Color = crate::theme::Color::from_rgb_u8(0xE5, 0x09, 0x14);
const BRIGHTNESS_AMBER: crate::theme::Color = crate::theme::Color::from_rgb_u8(0xFF, 0xB7, 0x03);
const BT_BLUE: crate::theme::Color = crate::theme::Color::from_rgb_u8(0x3A, 0x86, 0xFF);
const TRACK_BG: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 38);

/// Screen edge where the Dynamic Island / Notch is anchored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotchEdge {
    #[default]
    Top,
    Left,
    Right,
}

/// Active panel displayed inside the Dynamic Island / Notch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NotchPanel {
    #[default]
    Status,
    CommandCenter,
    Calendar,
    Music,
    Wifi,
    Shortcuts,
    Shelf,
    Tasks,
    Focus,
    HimeAi,
}

impl NotchPanel {
    pub fn short_label(self) -> &'static str {
        match self {
            Self::Status => "Status",
            Self::CommandCenter => "Controls",
            Self::Calendar => "Calendar",
            Self::Music => "Music",
            Self::Wifi => "Wi-Fi",
            Self::Shortcuts => "Keys",
            Self::Shelf => "Shelf",
            Self::Tasks => "Tasks",
            Self::Focus => "Focus",
            Self::HimeAi => "AI",
        }
    }
}

/// Interactive action emitted by [`NotchView::hit_test_click`].
#[derive(Debug, Clone, PartialEq)]
pub enum NotchAction {
    ExpandPanel(NotchPanel),
    Collapse,
    PrevPanel,
    NextPanel,
    ToggleWifi,
    OpenWifiList,
    ToggleBluetooth,
    CycleNotchMode,
    ToggleFreeDrag,
    OpenSettings,
    OpenShortcuts,
    ToggleDockMode,
    ToggleVolumeOverlay,
    ToggleBrightnessOverlay,
    ToggleNotifSuppression,
    ToggleHimeAi,
    SetVolume(f32),
    SetBrightness(u8),
    CalendarPrevMonth,
    CalendarNextMonth,
    CalendarToday,
    PomodoroStartPause,
    PomodoroReset,
    PomodoroPreset(u16),
    MusicPrev,
    MusicPlayPause,
    MusicNext,
    MusicToggleLyrics,
    MusicSeek(f32),
    LyricsAdjustOffset(i64),
    LyricsResetOffset,
    WifiConnect(String),
    WifiRefresh,
    OpenAiDrawer,
    ClearShelf,
    TasksSelectTab(TaskTabFilter),
    TasksToggleComplete(String),
    TasksDelete(String),
    TasksCyclePriority,
    TasksQuickAdd(String),
    FocusStart,
    FocusComplete,
    FocusSkip,
    FocusEmergencyPause,
    ProactiveAccept(String),
    ProactiveDismiss(String),
    ProactiveSnooze(String),
    ProactiveOpenDrawer,
    TranslateClose,
    TranslateOpenDrawer,
}

/// Network throughput & connectivity state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct NetStats {
    pub down_kbps: f32,
    pub up_kbps: f32,
    pub is_internet_down: bool,
}

/// Battery telemetry state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryStatus {
    pub percent: u8,
    pub charging: bool,
    pub present: bool,
}

impl Default for BatteryStatus {
    fn default() -> Self {
        Self {
            percent: 100,
            charging: false,
            present: true,
        }
    }
}

/// Active media playback + synced LRC lyric line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MusicInfo {
    pub title: String,
    pub artist: String,
    pub lyric_line: Option<String>,
    pub is_playing: bool,
}

/// Nearby Wi-Fi SSID entry shown in [`NotchPanel::Wifi`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WifiNetworkItem {
    pub ssid: String,
    pub signal: u8,
    pub secured: bool,
    pub connected: bool,
}

/// Compact summary item shown in [`NotchPanel::Shelf`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShelfSummaryItem {
    pub name: String,
    pub kind: String,
    pub size_label: String,
}

/// Pomodoro focus timer state shown inside [`NotchPanel::Calendar`] and compact pill.
#[derive(Debug, Clone, PartialEq)]
pub struct PomodoroState {
    pub remaining_secs: u32,
    pub total_secs: u32,
    pub running: bool,
    pub finished: bool,
    pub preset_mins: u16,
    accum_dt: f32,
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self {
            remaining_secs: 25 * 60,
            total_secs: 25 * 60,
            running: false,
            finished: false,
            preset_mins: 25,
            accum_dt: 0.0,
        }
    }
}

impl PomodoroState {
    pub fn set_preset(&mut self, mins: u16) {
        let clamped = mins.clamp(1, 180);
        let secs = clamped as u32 * 60;
        self.preset_mins = clamped;
        self.total_secs = secs;
        self.remaining_secs = secs;
        self.running = false;
        self.finished = false;
        self.accum_dt = 0.0;
    }

    pub fn toggle_start_pause(&mut self) {
        if self.finished || self.remaining_secs == 0 {
            self.remaining_secs = self.total_secs.max(60);
            self.finished = false;
        }
        self.running = !self.running;
    }

    pub fn reset(&mut self) {
        self.running = false;
        self.finished = false;
        self.remaining_secs = self.total_secs;
        self.accum_dt = 0.0;
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.running || dt <= 0.0 {
            return false;
        }
        self.accum_dt += dt;
        let mut changed = false;
        while self.accum_dt >= 1.0 {
            self.accum_dt -= 1.0;
            if self.remaining_secs > 1 {
                self.remaining_secs -= 1;
                changed = true;
            } else {
                self.remaining_secs = 0;
                self.running = false;
                self.finished = true;
                changed = true;
                break;
            }
        }
        changed
    }

    pub fn formatted(&self) -> String {
        let mins = self.remaining_secs / 60;
        let secs = self.remaining_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}

/// Native Direct2D Dynamic Island / Notch view supporting Top, Left, and Right edges
/// and all Hime interactive panels.
#[derive(Debug, Clone)]
pub struct NotchView {
    pub edge: NotchEdge,
    pub is_hidden: bool,
    pub is_expanded: bool,
    pub active_panel: NotchPanel,
    pub last_tool_panel: NotchPanel,
    pub clock_hours: u8,
    pub clock_minutes: u8,
    pub time_format_24h: bool,
    pub net: NetStats,
    pub battery: BatteryStatus,
    pub expression: HimeExpression,
    pub blink_progress: f32,
    pub music: Option<MusicInfo>,
    pub music_album: Option<String>,
    pub music_position_ms: u64,
    pub music_duration_ms: u64,
    pub music_lrc_lines: Vec<String>,
    pub music_synced_lrc: Vec<LrcLine>,
    pub music_lyric_offset_ms: i64,
    pub music_active_lyric_idx: Option<usize>,
    pub music_show_lyrics: bool,
    pub ncm_center_alert: Option<String>,
    pub wifi_enabled: bool,
    pub wifi_ssid: Option<String>,
    pub wifi_networks: Vec<WifiNetworkItem>,
    pub bluetooth_enabled: bool,
    pub bluetooth_label: Option<String>,
    pub notch_mode_label: String,
    pub free_drag_enabled: bool,
    pub dock_fixed: bool,
    pub volume_overlay_enabled: bool,
    pub brightness_overlay_enabled: bool,
    pub notif_suppression_enabled: bool,
    pub ai_enabled: bool,
    pub ai_status_text: String,
    pub volume_level: f32,
    pub volume_muted: bool,
    pub brightness_level: u8,
    pub calendar_year: i32,
    pub calendar_month: u8,
    pub calendar_today_year: i32,
    pub calendar_today_month: u8,
    pub calendar_today_day: u8,
    pub pomodoro: PomodoroState,
    pub shelf_items: Vec<ShelfSummaryItem>,
    pub tasks_ws: TasksWorkspaceState,
    pub focus_ws: FocusWorkspaceState,
    pub proactive_banner: ProactiveBannerState,
    pub translate_banner: TranslateBannerState,
    pub visualizer_phase: f32,
    pub width_spring: SpringValue,
    pub height_spring: SpringValue,
    pub expand_spring: SpringValue,
}

impl Default for NotchView {
    fn default() -> Self {
        Self::new(NotchEdge::Top)
    }
}

impl NotchView {
    /// Creates a new collapsed `NotchView` anchored at `edge`.
    pub fn new(edge: NotchEdge) -> Self {
        let (today_year, today_month, today_day) = current_local_date();
        let (w, h) = target_dimensions(edge, false, false, NotchPanel::Status, false, false);
        Self {
            edge,
            is_hidden: false,
            is_expanded: false,
            active_panel: NotchPanel::Status,
            last_tool_panel: NotchPanel::CommandCenter,
            clock_hours: 12,
            clock_minutes: 0,
            time_format_24h: true,
            net: NetStats::default(),
            battery: BatteryStatus::default(),
            expression: HimeExpression::Netral,
            blink_progress: 0.0,
            music: None,
            music_album: None,
            music_position_ms: 0,
            music_duration_ms: 0,
            music_lrc_lines: Vec::new(),
            music_synced_lrc: Vec::new(),
            music_lyric_offset_ms: 0,
            music_active_lyric_idx: None,
            music_show_lyrics: false,
            ncm_center_alert: None,
            wifi_enabled: true,
            wifi_ssid: None,
            wifi_networks: Vec::new(),
            bluetooth_enabled: true,
            bluetooth_label: None,
            notch_mode_label: "Fixed".to_string(),
            free_drag_enabled: true,
            dock_fixed: true,
            volume_overlay_enabled: true,
            brightness_overlay_enabled: true,
            notif_suppression_enabled: false,
            ai_enabled: false,
            ai_status_text: "Ready".to_string(),
            volume_level: 0.6,
            volume_muted: false,
            brightness_level: 80,
            calendar_year: today_year,
            calendar_month: today_month,
            calendar_today_year: today_year,
            calendar_today_month: today_month,
            calendar_today_day: today_day,
            pomodoro: PomodoroState::default(),
            shelf_items: Vec::new(),
            tasks_ws: TasksWorkspaceState::default(),
            focus_ws: FocusWorkspaceState::default(),
            proactive_banner: ProactiveBannerState::default(),
            translate_banner: TranslateBannerState::default(),
            visualizer_phase: 0.0,
            width_spring: SpringValue::snappy(w),
            height_spring: SpringValue::snappy(h),
            expand_spring: SpringValue::gentle(0.0),
        }
    }

    pub fn set_edge(&mut self, edge: NotchEdge) {
        self.edge = edge;
        self.sync_targets();
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.is_hidden = hidden;
        if hidden {
            self.is_expanded = false;
            self.active_panel = NotchPanel::Status;
        }
        self.sync_targets();
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        if self.is_hidden && expanded {
            self.is_hidden = false;
        }
        self.is_expanded = expanded;
        if expanded {
            if self.active_panel == NotchPanel::Status {
                let target = if self.last_tool_panel == NotchPanel::HimeAi && !self.ai_enabled {
                    NotchPanel::CommandCenter
                } else if self.last_tool_panel == NotchPanel::Status {
                    NotchPanel::CommandCenter
                } else {
                    self.last_tool_panel
                };
                self.active_panel = target;
            }
        } else {
            if self.active_panel != NotchPanel::Status {
                self.last_tool_panel = self.active_panel;
            }
            self.active_panel = NotchPanel::Status;
        }
        self.sync_targets();
    }

    pub fn set_panel(&mut self, panel: NotchPanel) {
        if panel == NotchPanel::Status {
            self.set_expanded(false);
            return;
        }
        if panel == NotchPanel::HimeAi && !self.ai_enabled {
            self.active_panel = NotchPanel::CommandCenter;
        } else {
            self.active_panel = panel;
            self.last_tool_panel = panel;
        }
        self.is_hidden = false;
        self.is_expanded = true;
        self.sync_targets();
    }

    pub fn set_ai_enabled(&mut self, enabled: bool) {
        self.ai_enabled = enabled;
        if !enabled && self.active_panel == NotchPanel::HimeAi {
            self.active_panel = NotchPanel::CommandCenter;
            self.last_tool_panel = NotchPanel::CommandCenter;
            self.sync_targets();
        }
    }

    pub fn available_panels(&self) -> Vec<NotchPanel> {
        let mut panels = vec![
            NotchPanel::CommandCenter,
            NotchPanel::Calendar,
            NotchPanel::Music,
            NotchPanel::Wifi,
            NotchPanel::Shortcuts,
            NotchPanel::Shelf,
            NotchPanel::Tasks,
            NotchPanel::Focus,
        ];
        if self.ai_enabled {
            panels.push(NotchPanel::HimeAi);
        }
        panels
    }

    pub fn cycle_panel(&mut self, delta: i32) {
        let panels = self.available_panels();
        if panels.is_empty() {
            return;
        }
        let idx = panels
            .iter()
            .position(|&p| p == self.active_panel)
            .unwrap_or(0) as isize;
        let len = panels.len() as isize;
        let next = ((idx + delta as isize) % len + len) % len;
        self.set_panel(panels[next as usize]);
    }

    pub fn set_clock(&mut self, hours: u8, minutes: u8) {
        self.clock_hours = hours % 24;
        self.clock_minutes = minutes % 60;
    }

    pub fn set_net_stats(&mut self, net: NetStats) {
        self.net = net;
    }

    pub fn set_battery(&mut self, battery: BatteryStatus) {
        self.battery = BatteryStatus {
            percent: battery.percent.min(100),
            ..battery
        };
    }

    pub fn set_expression(&mut self, expression: HimeExpression) {
        self.expression = expression;
    }

    pub fn set_music(&mut self, music: Option<MusicInfo>) {
        self.music = music;
    }

    pub fn set_music_details(
        &mut self,
        album: Option<String>,
        position_ms: u64,
        duration_ms: u64,
        lrc_lines: Vec<String>,
        active_idx: Option<usize>,
    ) {
        self.music_album = album;
        self.music_position_ms = position_ms;
        self.music_duration_ms = duration_ms;
        self.music_lrc_lines = lrc_lines;
        self.music_active_lyric_idx = active_idx;
    }

    pub fn set_music_synced_lrc(&mut self, lines: Vec<LrcLine>) {
        self.music_synced_lrc = lines;
        self.music_lrc_lines = self
            .music_synced_lrc
            .iter()
            .map(|l| l.text.clone())
            .collect();
        self.music_active_lyric_idx = active_lyric_index(
            &self.music_synced_lrc,
            self.music_position_ms,
            self.music_lyric_offset_ms,
        );
    }

    pub fn set_music_raw_lrc(&mut self, raw_lrc: &str) {
        self.set_music_synced_lrc(parse_lrc_lines(raw_lrc));
    }

    pub fn update_music_position(&mut self, position_ms: u64) {
        self.music_position_ms = position_ms;
        if !self.music_synced_lrc.is_empty() {
            self.music_active_lyric_idx = active_lyric_index(
                &self.music_synced_lrc,
                self.music_position_ms,
                self.music_lyric_offset_ms,
            );
        }
    }

    pub fn adjust_lyrics_offset(&mut self, delta_ms: i64) {
        self.music_lyric_offset_ms = (self.music_lyric_offset_ms + delta_ms).clamp(-10_000, 10_000);
        if !self.music_synced_lrc.is_empty() {
            self.music_active_lyric_idx = active_lyric_index(
                &self.music_synced_lrc,
                self.music_position_ms,
                self.music_lyric_offset_ms,
            );
        }
    }

    pub fn reset_lyrics_offset(&mut self) {
        self.music_lyric_offset_ms = 0;
        if !self.music_synced_lrc.is_empty() {
            self.music_active_lyric_idx = active_lyric_index(
                &self.music_synced_lrc,
                self.music_position_ms,
                self.music_lyric_offset_ms,
            );
        }
    }

    pub fn has_active_banner(&self) -> bool {
        self.proactive_banner.active || self.translate_banner.active
    }

    fn banner_area(&self, outer: Rect) -> Rect {
        Rect::new(
            outer.x + 12.0,
            outer.y + 4.0,
            (outer.width - 24.0).max(160.0),
            (outer.height - 8.0).max(64.0),
        )
    }

    pub fn dismiss_proactive_banner(&mut self) {
        self.proactive_banner.dismiss();
        self.sync_targets();
    }

    pub fn close_translate_banner(&mut self) {
        self.translate_banner.close();
        self.sync_targets();
    }

    pub fn set_wifi_status(&mut self, enabled: bool, ssid: Option<String>, networks: Vec<WifiNetworkItem>) {
        self.wifi_enabled = enabled;
        self.wifi_ssid = ssid;
        self.wifi_networks = networks;
    }

    pub fn set_volume(&mut self, level: f32, muted: bool) {
        self.volume_level = level.clamp(0.0, 1.0);
        self.volume_muted = muted;
    }

    pub fn set_brightness(&mut self, percent: u8) {
        self.brightness_level = percent.min(100);
    }

    pub fn set_shelf_items(&mut self, items: Vec<ShelfSummaryItem>) {
        self.shelf_items = items;
    }

    pub fn set_today(&mut self, year: i32, month: u8, day: u8) {
        self.calendar_today_year = year;
        self.calendar_today_month = month.clamp(1, 12);
        self.calendar_today_day = day.clamp(1, days_in_month(year, self.calendar_today_month));
    }

    pub fn set_ncm_center_alert(&mut self, alert: Option<String>) {
        self.ncm_center_alert = alert.filter(|s| !s.trim().is_empty());
        self.sync_targets();
    }

    /// Formats clock text horizontally (`HH:MM`) for Top edge or vertically (`HH\nMM`) for Left/Right.
    pub fn formatted_clock(&self) -> String {
        let display_hour = if self.time_format_24h {
            self.clock_hours
        } else {
            let h = self.clock_hours % 12;
            if h == 0 { 12 } else { h }
        };
        match self.edge {
            NotchEdge::Top => format!("{:02}:{:02}", display_hour, self.clock_minutes),
            NotchEdge::Left | NotchEdge::Right => {
                format!("{:02}\n{:02}", display_hour, self.clock_minutes)
            }
        }
    }

    /// Advances size, expansion, Pomodoro timer, focus session, banners, and visualizer phase by `dt` seconds.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.width_spring.tick(dt);
        self.height_spring.tick(dt);
        self.expand_spring.tick(dt);
        let pomo_changed = self.pomodoro.tick(dt);
        let focus_changed = self.focus_ws.tick(dt);
        let proactive_expired = self.proactive_banner.tick(dt);
        let translate_expired = self.translate_banner.tick(dt);
        if proactive_expired || translate_expired {
            self.sync_targets();
        }
        let is_playing = self.music.as_ref().is_some_and(|m| m.is_playing);
        if is_playing && dt > 0.0 {
            self.visualizer_phase = (self.visualizer_phase + dt * 5.2) % std::f32::consts::TAU;
        }
        !self.width_spring.is_settled()
            || !self.height_spring.is_settled()
            || !self.expand_spring.is_settled()
            || pomo_changed
            || focus_changed
            || self.pomodoro.running
            || (self.focus_ws.status == FocusSessionStatus::Active && !self.focus_ws.emergency_paused)
            || is_playing
    }

    /// Applies a local UI state transition for `action` (used alongside OS side-effects in `main.rs`).
    pub fn apply_action(&mut self, action: &NotchAction) {
        match action {
            NotchAction::ExpandPanel(panel) => self.set_panel(*panel),
            NotchAction::Collapse => self.set_expanded(false),
            NotchAction::PrevPanel => self.cycle_panel(-1),
            NotchAction::NextPanel => self.cycle_panel(1),
            NotchAction::ToggleWifi => self.wifi_enabled = !self.wifi_enabled,
            NotchAction::OpenWifiList => self.set_panel(NotchPanel::Wifi),
            NotchAction::ToggleBluetooth => self.bluetooth_enabled = !self.bluetooth_enabled,
            NotchAction::CycleNotchMode => {
                self.notch_mode_label = match self.notch_mode_label.as_str() {
                    "Fixed" => "Smart".to_string(),
                    "Smart" => "Peek".to_string(),
                    _ => "Fixed".to_string(),
                };
            }
            NotchAction::ToggleFreeDrag => self.free_drag_enabled = !self.free_drag_enabled,
            NotchAction::OpenShortcuts => self.set_panel(NotchPanel::Shortcuts),
            NotchAction::ToggleDockMode => self.dock_fixed = !self.dock_fixed,
            NotchAction::ToggleVolumeOverlay => {
                self.volume_overlay_enabled = !self.volume_overlay_enabled;
            }
            NotchAction::ToggleBrightnessOverlay => {
                self.brightness_overlay_enabled = !self.brightness_overlay_enabled;
            }
            NotchAction::ToggleNotifSuppression => {
                self.notif_suppression_enabled = !self.notif_suppression_enabled;
            }
            NotchAction::ToggleHimeAi => {
                let next = !self.ai_enabled;
                self.set_ai_enabled(next);
            }
            NotchAction::SetVolume(v) => {
                self.volume_level = v.clamp(0.0, 1.0);
                self.volume_muted = self.volume_level <= 0.005;
            }
            NotchAction::SetBrightness(b) => {
                self.brightness_level = (*b).min(100);
            }
            NotchAction::CalendarPrevMonth => self.shift_calendar_month(-1),
            NotchAction::CalendarNextMonth => self.shift_calendar_month(1),
            NotchAction::CalendarToday => {
                self.calendar_year = self.calendar_today_year;
                self.calendar_month = self.calendar_today_month;
            }
            NotchAction::PomodoroStartPause => self.pomodoro.toggle_start_pause(),
            NotchAction::PomodoroReset => self.pomodoro.reset(),
            NotchAction::PomodoroPreset(m) => self.pomodoro.set_preset(*m),
            NotchAction::MusicToggleLyrics => self.music_show_lyrics = !self.music_show_lyrics,
            NotchAction::MusicPlayPause => {
                if let Some(m) = self.music.as_mut() {
                    m.is_playing = !m.is_playing;
                }
            }
            NotchAction::MusicSeek(ratio) => {
                if self.music_duration_ms > 0 {
                    let pos = ((self.music_duration_ms as f32) * ratio.clamp(0.0, 1.0)) as u64;
                    self.update_music_position(pos);
                }
            }
            NotchAction::LyricsAdjustOffset(delta_ms) => self.adjust_lyrics_offset(*delta_ms),
            NotchAction::LyricsResetOffset => self.reset_lyrics_offset(),
            NotchAction::ClearShelf => self.shelf_items.clear(),
            NotchAction::TasksSelectTab(tab) => self.tasks_ws.active_tab = *tab,
            NotchAction::TasksToggleComplete(id) => {
                self.tasks_ws.toggle_task(id);
            }
            NotchAction::TasksDelete(id) => {
                self.tasks_ws.delete_task(id);
            }
            NotchAction::TasksCyclePriority => self.tasks_ws.cycle_quick_add_priority(),
            NotchAction::TasksQuickAdd(title) => {
                if self.tasks_ws.quick_add_draft.trim().is_empty() && !title.trim().is_empty() {
                    self.tasks_ws.quick_add_draft = title.clone();
                }
                self.tasks_ws.commit_quick_add();
            }
            NotchAction::FocusStart => self.focus_ws.start(),
            NotchAction::FocusComplete => self.focus_ws.complete(),
            NotchAction::FocusSkip => self.focus_ws.skip(),
            NotchAction::FocusEmergencyPause => self.focus_ws.toggle_emergency_pause(),
            NotchAction::ProactiveAccept(_)
            | NotchAction::ProactiveDismiss(_)
            | NotchAction::ProactiveSnooze(_)
            | NotchAction::ProactiveOpenDrawer => self.dismiss_proactive_banner(),
            NotchAction::TranslateClose | NotchAction::TranslateOpenDrawer => {
                self.close_translate_banner();
            }
            NotchAction::OpenSettings
            | NotchAction::MusicPrev
            | NotchAction::MusicNext
            | NotchAction::WifiConnect(_)
            | NotchAction::WifiRefresh
            | NotchAction::OpenAiDrawer => {}
        }
    }

    fn shift_calendar_month(&mut self, delta: i32) {
        let total = self.calendar_year * 12 + (self.calendar_month as i32 - 1) + delta;
        let next_year = total.div_euclid(12).clamp(1900, 2999);
        let next_month = (total.rem_euclid(12) + 1) as u8;
        self.calendar_year = next_year;
        self.calendar_month = next_month;
    }

    /// Computes the notch pill rectangle flush to the configured screen edge.
    pub fn computed_rect(&self, surface_width: f32, surface_height: f32) -> Rect {
        let w = self.width_spring.value.max(HIDDEN_BAR_THICKNESS);
        let h = self.height_spring.value.max(HIDDEN_BAR_THICKNESS);
        match self.edge {
            NotchEdge::Top => Rect::new((surface_width - w) * 0.5, 0.0, w, h),
            NotchEdge::Left => Rect::new(0.0, (surface_height - h) * 0.5, w, h),
            NotchEdge::Right => Rect::new(surface_width - w, (surface_height - h) * 0.5, w, h),
        }
    }

    /// Returns the inner safe-padded content rect (respecting `theme::SAFE_PADDING = 24.0` when expanded).
    pub fn content_rect(&self, surface_width: f32, surface_height: f32) -> Rect {
        let outer = self.computed_rect(surface_width, surface_height);
        let pad = if self.is_expanded { SAFE_PADDING } else { 10.0 };
        Rect::new(
            outer.x + pad,
            outer.y + pad * 0.5,
            (outer.width - pad * 2.0).max(0.0),
            (outer.height - pad).max(0.0),
        )
    }

    /// Hit-tests pointer click `(x, y)` inside `surface_rect` and returns the corresponding [`NotchAction`].
    pub fn hit_test_click(&self, x: f32, y: f32, surface_rect: Rect) -> Option<NotchAction> {
        if self.is_hidden {
            return None;
        }
        let local_outer = self.computed_rect(surface_rect.width, surface_rect.height);
        let outer = Rect::new(
            surface_rect.x + local_outer.x,
            surface_rect.y + local_outer.y,
            local_outer.width,
            local_outer.height,
        );
        if !point_in_rect(x, y, outer) {
            return None;
        }

        if !self.is_expanded {
            if self.has_active_banner() {
                let area = self.banner_area(outer);
                if self.translate_banner.active {
                    return self.translate_banner.hit_test(x, y, area).map(|act| match act {
                        TranslateBannerAction::Close => NotchAction::TranslateClose,
                        TranslateBannerAction::OpenDrawer => NotchAction::TranslateOpenDrawer,
                    });
                }
                if self.proactive_banner.active {
                    return self.proactive_banner.hit_test(x, y, area).map(|act| match act {
                        ProactiveBannerAction::Accept(id) => NotchAction::ProactiveAccept(id),
                        ProactiveBannerAction::Dismiss(id) => NotchAction::ProactiveDismiss(id),
                        ProactiveBannerAction::Snooze(id) => NotchAction::ProactiveSnooze(id),
                        ProactiveBannerAction::OpenDrawer => NotchAction::ProactiveOpenDrawer,
                    });
                }
                return None;
            }
            if self.music.is_some() {
                let badge_rect = Rect::new(outer.x + outer.width - 48.0, outer.y + 4.0, 32.0, 28.0);
                if point_in_rect(x, y, badge_rect) {
                    return Some(NotchAction::ExpandPanel(NotchPanel::Music));
                }
            }
            let clock_rect = Rect::new(
                outer.x + outer.width * 0.5 - 40.0,
                outer.y,
                80.0,
                outer.height,
            );
            if self.edge == NotchEdge::Top && point_in_rect(x, y, clock_rect) {
                return Some(NotchAction::ExpandPanel(NotchPanel::Calendar));
            }
            let preferred = if self.last_tool_panel == NotchPanel::HimeAi && !self.ai_enabled {
                NotchPanel::CommandCenter
            } else if self.last_tool_panel == NotchPanel::Status {
                NotchPanel::CommandCenter
            } else {
                self.last_tool_panel
            };
            return Some(NotchAction::ExpandPanel(preferred));
        }

        let layout = self.compute_expanded_layout(outer);

        // Header hit-testing
        if point_in_rect(x, y, layout.prev_btn) {
            return Some(NotchAction::PrevPanel);
        }
        if point_in_rect(x, y, layout.next_btn) {
            return Some(NotchAction::NextPanel);
        }
        if point_in_rect(x, y, layout.close_btn) {
            return Some(NotchAction::Collapse);
        }
        for (panel, tab_rect) in &layout.tabs {
            if point_in_rect(x, y, *tab_rect) {
                return Some(NotchAction::ExpandPanel(*panel));
            }
        }
        if point_in_rect(x, y, layout.header_clock) {
            return Some(NotchAction::ExpandPanel(NotchPanel::Calendar));
        }

        // Panel-specific hit-testing
        match &layout.body {
            BodyLayout::CommandCenter(cc) => {
                if point_in_rect(x, y, cc.wifi_more_btn) {
                    return Some(NotchAction::OpenWifiList);
                }
                if point_in_rect(x, y, cc.wifi_tile) {
                    return Some(NotchAction::ToggleWifi);
                }
                if point_in_rect(x, y, cc.bluetooth_tile) {
                    return Some(NotchAction::ToggleBluetooth);
                }
                if point_in_rect(x, y, cc.drag_badge) {
                    return Some(NotchAction::ToggleFreeDrag);
                }
                if point_in_rect(x, y, cc.notch_mode_tile) {
                    return Some(NotchAction::CycleNotchMode);
                }
                if point_in_rect(x, y, cc.settings_gear_btn) {
                    return Some(NotchAction::OpenSettings);
                }
                if point_in_rect(x, y, cc.shortcuts_tile) {
                    return Some(NotchAction::OpenShortcuts);
                }

                let toggle_actions = [
                    NotchAction::ToggleDockMode,
                    NotchAction::ToggleVolumeOverlay,
                    NotchAction::ToggleBrightnessOverlay,
                    NotchAction::ToggleNotifSuppression,
                    NotchAction::ToggleHimeAi,
                ];
                for (idx, rect) in cc.quick_toggles.iter().enumerate() {
                    if point_in_rect(x, y, *rect) {
                        return Some(toggle_actions[idx].clone());
                    }
                }

                if point_in_rect(x, y, cc.volume_icon_btn) {
                    let target = if self.volume_muted || self.volume_level <= 0.01 {
                        0.5
                    } else {
                        0.0
                    };
                    return Some(NotchAction::SetVolume(target));
                }
                if point_in_rect(x, y, cc.volume_track) {
                    let ratio = ((x - cc.volume_track.x) / cc.volume_track.width.max(1.0))
                        .clamp(0.0, 1.0);
                    return Some(NotchAction::SetVolume(ratio));
                }
                if point_in_rect(x, y, cc.brightness_track) {
                    let ratio = ((x - cc.brightness_track.x)
                        / cc.brightness_track.width.max(1.0))
                    .clamp(0.0, 1.0);
                    return Some(NotchAction::SetBrightness((ratio * 100.0).round() as u8));
                }
            }
            BodyLayout::Calendar(cal) => {
                if point_in_rect(x, y, cal.prev_month_btn) {
                    return Some(NotchAction::CalendarPrevMonth);
                }
                if point_in_rect(x, y, cal.next_month_btn) {
                    return Some(NotchAction::CalendarNextMonth);
                }
                if point_in_rect(x, y, cal.today_btn) {
                    return Some(NotchAction::CalendarToday);
                }
                if point_in_rect(x, y, cal.pomo_play_btn) {
                    return Some(NotchAction::PomodoroStartPause);
                }
                if point_in_rect(x, y, cal.pomo_reset_btn) {
                    return Some(NotchAction::PomodoroReset);
                }
                for (mins, rect) in &cal.preset_btns {
                    if point_in_rect(x, y, *rect) {
                        return Some(NotchAction::PomodoroPreset(*mins));
                    }
                }
            }
            BodyLayout::Music(mus) => {
                if point_in_rect(x, y, mus.lyrics_btn) {
                    return Some(NotchAction::MusicToggleLyrics);
                }
                if point_in_rect(x, y, mus.progress_track) {
                    let ratio = ((x - mus.progress_track.x)
                        / mus.progress_track.width.max(1.0))
                    .clamp(0.0, 1.0);
                    return Some(NotchAction::MusicSeek(ratio));
                }
                if point_in_rect(x, y, mus.prev_btn) {
                    return Some(NotchAction::MusicPrev);
                }
                if point_in_rect(x, y, mus.play_pause_btn) {
                    return Some(NotchAction::MusicPlayPause);
                }
                if point_in_rect(x, y, mus.next_btn) {
                    return Some(NotchAction::MusicNext);
                }
                if self.music_show_lyrics {
                    if point_in_rect(x, y, mus.offset_minus_btn) {
                        return Some(NotchAction::LyricsAdjustOffset(-200));
                    }
                    if point_in_rect(x, y, mus.offset_plus_btn) {
                        return Some(NotchAction::LyricsAdjustOffset(200));
                    }
                    if point_in_rect(x, y, mus.offset_reset_btn) {
                        return Some(NotchAction::LyricsResetOffset);
                    }
                }
            }
            BodyLayout::Wifi(wf) => {
                if point_in_rect(x, y, wf.back_btn) {
                    return Some(NotchAction::ExpandPanel(NotchPanel::CommandCenter));
                }
                if point_in_rect(x, y, wf.toggle_btn) {
                    return Some(NotchAction::ToggleWifi);
                }
                if point_in_rect(x, y, wf.refresh_btn) {
                    return Some(NotchAction::WifiRefresh);
                }
                for (ssid, row_rect) in &wf.network_rows {
                    if point_in_rect(x, y, *row_rect) {
                        return Some(NotchAction::WifiConnect(ssid.clone()));
                    }
                }
            }
            BodyLayout::Shortcuts(sc) => {
                if point_in_rect(x, y, sc.settings_btn) {
                    return Some(NotchAction::OpenSettings);
                }
            }
            BodyLayout::Shelf(sh) => {
                if let Some(clear_btn) = sh.clear_btn {
                    if point_in_rect(x, y, clear_btn) {
                        return Some(NotchAction::ClearShelf);
                    }
                }
            }
            BodyLayout::Tasks(body_rect) => {
                return self.tasks_ws.hit_test(x, y, *body_rect).map(|act| match act {
                    TasksHitAction::SelectTab(tab) => NotchAction::TasksSelectTab(tab),
                    TasksHitAction::ToggleTask(id) => NotchAction::TasksToggleComplete(id),
                    TasksHitAction::DeleteTask(id) => NotchAction::TasksDelete(id),
                    TasksHitAction::CyclePriority => NotchAction::TasksCyclePriority,
                    TasksHitAction::QuickAdd(title) => NotchAction::TasksQuickAdd(title),
                });
            }
            BodyLayout::Focus(body_rect) => {
                return self.focus_ws.hit_test(x, y, *body_rect).map(|act| match act {
                    FocusHitAction::Start => NotchAction::FocusStart,
                    FocusHitAction::Complete => NotchAction::FocusComplete,
                    FocusHitAction::Skip => NotchAction::FocusSkip,
                    FocusHitAction::EmergencyPause => NotchAction::FocusEmergencyPause,
                });
            }
            BodyLayout::HimeAi(ai) => {
                if point_in_rect(x, y, ai.open_drawer_btn) {
                    return Some(NotchAction::OpenAiDrawer);
                }
            }
        }

        None
    }

    /// Renders the notch onto `surface`.
    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let (sw, sh) = surface.dip_size();
        let rect = self.computed_rect(sw, sh);
        let edge_side = match self.edge {
            NotchEdge::Top => EdgeSide::Top,
            NotchEdge::Left => EdgeSide::Left,
            NotchEdge::Right => EdgeSide::Right,
        };

        if self.is_hidden && self.width_spring.is_settled() && self.height_spring.is_settled() {
            surface.set_rounded_region(Rect::new(0.0, 0.0, 0.0, 0.0), 0.0);
            return Ok(());
        }

        let radius = if self.is_hidden { 1.5 } else { CORNER_RADIUS };
        let flares = !self.is_hidden;
        surface.set_edge_surface_region(rect, EAR_WIDTH, radius, edge_side, flares);
        surface.draw_edge_surface(
            rect,
            EAR_WIDTH,
            radius,
            edge_side,
            flares,
            NOTCH_BG_BLACK,
            Some(NOTCH_EDGE_BORDER),
        )?;

        if self.is_hidden {
            return Ok(());
        }

        if self.is_expanded || self.expand_spring.value > 0.25 {
            return self.render_expanded_contents(surface, rect);
        }

        if self.has_active_banner() {
            let area = self.banner_area(rect);
            if self.translate_banner.active {
                return self.translate_banner.render(surface, area);
            }
            if self.proactive_banner.active {
                return self.proactive_banner.render(surface, area);
            }
        }

        match self.edge {
            NotchEdge::Top => self.render_top_collapsed(surface, rect),
            NotchEdge::Left | NotchEdge::Right => self.render_side_collapsed(surface, rect),
        }
    }

    fn render_top_collapsed(&self, surface: &NativeSurface, outer: Rect) -> Result<()> {
        let is_playing = self.music.as_ref().is_some_and(|m| m.is_playing);
        let has_music = self.music.is_some();
        let has_alert = self.ncm_center_alert.is_some()
            || self.net.is_internet_down
            || self.expression != HimeExpression::Netral;

        // Left slot: Music visualizer when active, else Wi-Fi icon + ↓speed (WHITE)
        if is_playing {
            let bars = animated_visualizer_bars(self.visualizer_phase);
            surface.draw_music_visualizer(outer.x + 22.0, outer.y + 10.0, 16.0, &bars, true)?;
        } else {
            let wifi_icon_color = if self.net.is_internet_down {
                STATUS_ALERT
            } else if !self.wifi_enabled {
                TEXT_PRIMARY.with_alpha(0.45)
            } else {
                TEXT_PRIMARY
            };
            surface.draw_wifi_icon(
                outer.x + 26.0,
                outer.y + 18.0,
                13.0,
                self.wifi_enabled && !self.net.is_internet_down,
                self.net.is_internet_down || !self.wifi_enabled,
                wifi_icon_color,
            )?;
            let speed_text = if self.net.is_internet_down {
                "Offline".to_string()
            } else {
                format!("↓{}", format_net_speed(self.net.down_kbps))
            };
            let speed_rect = Rect::new(outer.x + 38.0, outer.y + 6.0, 68.0, 24.0);
            surface.draw_text(&speed_text, speed_rect, 10.5, TEXT_PRIMARY, TextAlign::Leading)?;
        }

        // Center slot: Pomodoro timer (when active), HimeEyes + alert, or bold WHITE clock
        let center_rect = Rect::new(
            outer.x + 104.0,
            outer.y + 5.0,
            (outer.width - 208.0).max(52.0),
            26.0,
        );
        if self.pomodoro.running || self.pomodoro.finished {
            let timer_color = if self.pomodoro.finished {
                STATUS_ALERT
            } else {
                TEXT_PRIMARY
            };
            surface.draw_text_bold(
                &self.pomodoro.formatted(),
                center_rect,
                13.5,
                timer_color,
                TextAlign::Center,
            )?;
        } else if has_alert {
            let eye_cx = if self.ncm_center_alert.is_some() {
                center_rect.x + 16.0
            } else {
                outer.x + outer.width * 0.5
            };
            surface.draw_hime_eyes(eye_cx, outer.y + 18.0, 0.85, self.expression, self.blink_progress)?;
            if let Some(alert) = &self.ncm_center_alert {
                let msg_rect = Rect::new(
                    center_rect.x + 32.0,
                    outer.y + 6.0,
                    (center_rect.width - 34.0).max(40.0),
                    24.0,
                );
                surface.draw_text(alert, msg_rect, 11.0, TEXT_PRIMARY, TextAlign::Leading)?;
            }
        } else {
            surface.draw_text_bold(
                &self.formatted_clock(),
                center_rect,
                14.0,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;
        }

        // Right slot: Album badge if music active, else Battery % (WHITE) + Battery icon
        if has_music && is_playing {
            let badge = Rect::new(outer.x + outer.width - 42.0, outer.y + 7.0, 22.0, 22.0);
            surface.fill_rounded_rect(badge, 6.0, CARD_BG_ACTIVE)?;
            surface.stroke_rounded_rect(badge, 6.0, NOTCH_EDGE_BORDER, 1.0)?;
            surface.draw_music_note_icon(
                badge.x + 11.0,
                badge.y + 11.0,
                12.0,
                TEXT_PRIMARY,
            )?;
        } else if self.battery.present {
            let bat_text_rect = Rect::new(outer.x + outer.width - 88.0, outer.y + 6.0, 42.0, 24.0);
            surface.draw_text(
                &format!("{}%", self.battery.percent),
                bat_text_rect,
                11.0,
                TEXT_PRIMARY,
                TextAlign::Trailing,
            )?;
            surface.draw_battery_icon(
                outer.x + outer.width - 42.0,
                outer.y + 12.5,
                20.0,
                11.0,
                self.battery.percent,
                self.battery.charging,
            )?;
        }

        Ok(())
    }

    fn render_side_collapsed(&self, surface: &NativeSurface, outer: Rect) -> Result<()> {
        let cx = outer.x + outer.width * 0.5;
        let has_alert = self.ncm_center_alert.is_some()
            || self.net.is_internet_down
            || self.expression != HimeExpression::Netral;

        let wifi_color = if self.net.is_internet_down {
            STATUS_ALERT
        } else {
            TEXT_PRIMARY
        };
        surface.draw_wifi_icon(
            cx,
            outer.y + 24.0,
            12.0,
            self.wifi_enabled && !self.net.is_internet_down,
            self.net.is_internet_down || !self.wifi_enabled,
            wifi_color,
        )?;
        if !self.net.is_internet_down {
            let net_rect = Rect::new(outer.x + 2.0, outer.y + 38.0, outer.width - 4.0, 20.0);
            surface.draw_text(
                &format_net_speed(self.net.down_kbps),
                net_rect,
                8.5,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;
        }

        if has_alert {
            surface.draw_hime_eyes(
                cx,
                outer.y + outer.height * 0.5,
                0.8,
                self.expression,
                self.blink_progress,
            )?;
        } else {
            let clock_rect = Rect::new(
                outer.x + 2.0,
                outer.y + outer.height * 0.5 - 24.0,
                outer.width - 4.0,
                48.0,
            );
            surface.draw_text_bold(
                &self.formatted_clock(),
                clock_rect,
                13.0,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;
        }

        if self.battery.present {
            surface.draw_battery_icon(
                cx - 10.0,
                outer.y + outer.height - 46.0,
                20.0,
                10.0,
                self.battery.percent,
                self.battery.charging,
            )?;
            let bat_rect = Rect::new(
                outer.x + 2.0,
                outer.y + outer.height - 32.0,
                outer.width - 4.0,
                18.0,
            );
            surface.draw_text(
                &format!("{}%", self.battery.percent),
                bat_rect,
                10.0,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;
        }
        Ok(())
    }

    fn render_expanded_contents(&self, surface: &NativeSurface, outer: Rect) -> Result<()> {
        let layout = self.compute_expanded_layout(outer);

        // Header bar: [<] [Clock] [>]  ...  [Tabs] [X]
        surface.fill_rounded_rect(layout.prev_btn, 7.0, CARD_BG)?;
        surface.draw_chevron_icon(
            layout.prev_btn.x + layout.prev_btn.width * 0.5,
            layout.prev_btn.y + layout.prev_btn.height * 0.5,
            9.0,
            false,
            TEXT_PRIMARY,
        )?;

        let clock_display = format!("{:02}:{:02}", self.clock_hours, self.clock_minutes);
        surface.draw_text_bold(
            &clock_display,
            layout.header_clock,
            13.0,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        surface.fill_rounded_rect(layout.next_btn, 7.0, CARD_BG)?;
        surface.draw_chevron_icon(
            layout.next_btn.x + layout.next_btn.width * 0.5,
            layout.next_btn.y + layout.next_btn.height * 0.5,
            9.0,
            true,
            TEXT_PRIMARY,
        )?;

        // Tab switcher pills
        for (panel, tab_rect) in &layout.tabs {
            let active = *panel == self.active_panel;
            if active {
                surface.fill_rounded_rect(*tab_rect, 7.0, CARD_BG_ACTIVE)?;
                surface.stroke_rounded_rect(*tab_rect, 7.0, NOTCH_EDGE_BORDER, 1.0)?;
                // Small active accent dot underneath
                surface.fill_circle(
                    tab_rect.x + tab_rect.width * 0.5,
                    tab_rect.y + tab_rect.height - 2.5,
                    1.8,
                    ACCENT_CYAN,
                )?;
            } else {
                surface.fill_rounded_rect(*tab_rect, 7.0, CARD_BG)?;
            }
            let cx = tab_rect.x + tab_rect.width * 0.5;
            let cy = tab_rect.y + tab_rect.height * 0.5 - if active { 1.0 } else { 0.0 };
            let icon_color = TEXT_PRIMARY;
            match panel {
                NotchPanel::CommandCenter => {
                    surface.draw_dock_icon(cx, cy, 11.0, icon_color)?;
                }
                NotchPanel::Calendar => {
                    surface.draw_calendar_icon(cx, cy, 11.0, icon_color)?;
                }
                NotchPanel::Music => {
                    surface.draw_music_note_icon(cx, cy, 11.0, icon_color)?;
                }
                NotchPanel::Wifi => {
                    surface.draw_wifi_icon(cx, cy, 11.0, self.wifi_enabled, !self.wifi_enabled, icon_color)?;
                }
                NotchPanel::Shortcuts => {
                    surface.draw_keyboard_icon(cx, cy, 11.0, icon_color)?;
                }
                NotchPanel::Shelf => {
                    surface.draw_shelf_icon(cx, cy, 11.0, icon_color)?;
                }
                NotchPanel::Tasks => {
                    surface.draw_line(cx - 4.0, cy - 2.0, cx - 2.0, cy, icon_color, 1.3)?;
                    surface.draw_line(cx - 2.0, cy, cx + 1.5, cy - 3.5, icon_color, 1.3)?;
                    surface.draw_line(cx - 4.0, cy + 3.5, cx + 4.0, cy + 3.5, icon_color, 1.3)?;
                }
                NotchPanel::Focus => {
                    surface.stroke_circle(cx, cy, 4.8, icon_color, 1.2)?;
                    surface.fill_circle(cx, cy, 1.8, icon_color)?;
                }
                NotchPanel::HimeAi => {
                    surface.draw_sparkles_icon(cx, cy, 11.0, icon_color)?;
                }
                NotchPanel::Status => {}
            }
        }

        // Collapse (X) button
        surface.fill_rounded_rect(layout.close_btn, 7.0, CARD_BG)?;
        let xcx = layout.close_btn.x + layout.close_btn.width * 0.5;
        let xcy = layout.close_btn.y + layout.close_btn.height * 0.5;
        surface.draw_line(xcx - 3.5, xcy - 3.5, xcx + 3.5, xcy + 3.5, TEXT_PRIMARY, 1.4)?;
        surface.draw_line(xcx + 3.5, xcy - 3.5, xcx - 3.5, xcy + 3.5, TEXT_PRIMARY, 1.4)?;

        // Subtle header divider
        let div_y = outer.y + HEADER_HEIGHT + 1.0;
        surface.draw_line(
            outer.x + SAFE_PADDING,
            div_y,
            outer.x + outer.width - SAFE_PADDING,
            div_y,
            crate::theme::Color::from_rgba_u8(255, 255, 255, 22),
            1.0,
        )?;

        // Body
        match &layout.body {
            BodyLayout::CommandCenter(cc) => self.render_command_center(surface, cc),
            BodyLayout::Calendar(cal) => self.render_calendar(surface, cal),
            BodyLayout::Music(mus) => self.render_music(surface, mus),
            BodyLayout::Wifi(wf) => self.render_wifi(surface, wf),
            BodyLayout::Shortcuts(sc) => self.render_shortcuts(surface, sc),
            BodyLayout::Shelf(sh) => self.render_shelf(surface, sh),
            BodyLayout::Tasks(body_rect) => self.tasks_ws.render(surface, *body_rect),
            BodyLayout::Focus(body_rect) => self.focus_ws.render(surface, *body_rect),
            BodyLayout::HimeAi(ai) => self.render_hime_ai(surface, ai),
        }
    }

    fn render_command_center(&self, surface: &NativeSurface, cc: &CommandCenterLayout) -> Result<()> {
        // 1. Wi-Fi Card Tile
        let wifi_border = if self.wifi_enabled { ACCENT_CYAN.with_alpha(0.75) } else { CARD_BORDER };
        let wifi_bg = if self.wifi_enabled { CARD_BG_ACTIVE } else { CARD_BG };
        surface.fill_rounded_rect(cc.wifi_tile, 12.0, wifi_bg)?;
        surface.stroke_rounded_rect(cc.wifi_tile, 12.0, wifi_border, 1.0)?;
        surface.draw_wifi_icon(
            cc.wifi_tile.x + 17.0,
            cc.wifi_tile.y + cc.wifi_tile.height * 0.5,
            13.0,
            self.wifi_enabled,
            !self.wifi_enabled,
            TEXT_PRIMARY,
        )?;
        let wifi_sub = if !self.wifi_enabled {
            "Off"
        } else if let Some(ssid) = &self.wifi_ssid {
            ssid.as_str()
        } else if self.net.is_internet_down {
            "No Internet"
        } else {
            "Connected"
        };
        surface.draw_text_bold(
            "Wi-Fi",
            Rect::new(cc.wifi_tile.x + 31.0, cc.wifi_tile.y + 4.0, cc.wifi_tile.width - 56.0, 16.0),
            11.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            wifi_sub,
            Rect::new(cc.wifi_tile.x + 31.0, cc.wifi_tile.y + 19.0, cc.wifi_tile.width - 56.0, 14.0),
            9.5,
            TEXT_MUTED_WHITE,
            TextAlign::Leading,
        )?;
        if self.wifi_enabled {
            surface.fill_circle(cc.wifi_tile.x + 10.0, cc.wifi_tile.y + 10.0, 2.5, ACCENT_CYAN)?;
        }
        surface.fill_rounded_rect(cc.wifi_more_btn, 7.0, CARD_BG)?;
        surface.draw_chevron_icon(
            cc.wifi_more_btn.x + cc.wifi_more_btn.width * 0.5,
            cc.wifi_more_btn.y + cc.wifi_more_btn.height * 0.5,
            9.0,
            true,
            TEXT_PRIMARY,
        )?;

        // 2. Bluetooth Card Tile
        let bt_border = if self.bluetooth_enabled { BT_BLUE.with_alpha(0.8) } else { CARD_BORDER };
        let bt_bg = if self.bluetooth_enabled { CARD_BG_ACTIVE } else { CARD_BG };
        surface.fill_rounded_rect(cc.bluetooth_tile, 12.0, bt_bg)?;
        surface.stroke_rounded_rect(cc.bluetooth_tile, 12.0, bt_border, 1.0)?;
        surface.draw_bluetooth_icon(
            cc.bluetooth_tile.x + 17.0,
            cc.bluetooth_tile.y + cc.bluetooth_tile.height * 0.5,
            13.0,
            TEXT_PRIMARY,
        )?;
        let bt_sub = if !self.bluetooth_enabled {
            "Off"
        } else if let Some(dev) = &self.bluetooth_label {
            dev.as_str()
        } else {
            "On"
        };
        surface.draw_text_bold(
            "Bluetooth",
            Rect::new(cc.bluetooth_tile.x + 31.0, cc.bluetooth_tile.y + 4.0, cc.bluetooth_tile.width - 38.0, 16.0),
            11.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            bt_sub,
            Rect::new(cc.bluetooth_tile.x + 31.0, cc.bluetooth_tile.y + 19.0, cc.bluetooth_tile.width - 38.0, 14.0),
            9.5,
            TEXT_MUTED_WHITE,
            TextAlign::Leading,
        )?;
        if self.bluetooth_enabled {
            surface.fill_circle(cc.bluetooth_tile.x + 10.0, cc.bluetooth_tile.y + 10.0, 2.5, BT_BLUE)?;
        }

        // 3. Notch Mode / Free Drag Tile
        let mode_active = self.notch_mode_label == "Fixed";
        surface.fill_rounded_rect(
            cc.notch_mode_tile,
            12.0,
            if mode_active { CARD_BG_ACTIVE } else { CARD_BG },
        )?;
        surface.stroke_rounded_rect(
            cc.notch_mode_tile,
            12.0,
            if mode_active { HIME_CRIMSON.with_alpha(0.75) } else { CARD_BORDER },
            1.0,
        )?;
        surface.draw_dock_icon(
            cc.notch_mode_tile.x + 17.0,
            cc.notch_mode_tile.y + cc.notch_mode_tile.height * 0.5,
            12.0,
            TEXT_PRIMARY,
        )?;
        surface.draw_text_bold(
            "Notch Mode",
            Rect::new(cc.notch_mode_tile.x + 31.0, cc.notch_mode_tile.y + 4.0, 86.0, 16.0),
            11.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            &self.notch_mode_label,
            Rect::new(cc.notch_mode_tile.x + 31.0, cc.notch_mode_tile.y + 19.0, 76.0, 14.0),
            9.5,
            TEXT_MUTED_WHITE,
            TextAlign::Leading,
        )?;
        surface.fill_rounded_rect(
            cc.drag_badge,
            7.0,
            if self.free_drag_enabled { ACCENT_CYAN.with_alpha(0.28) } else { CARD_BG },
        )?;
        surface.stroke_rounded_rect(
            cc.drag_badge,
            7.0,
            if self.free_drag_enabled { ACCENT_CYAN } else { CARD_BORDER },
            1.0,
        )?;
        surface.draw_text(
            if self.free_drag_enabled { "Drag" } else { "Lock" },
            cc.drag_badge,
            8.5,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        // 4. Shortcuts / Settings Tile
        surface.fill_rounded_rect(cc.shortcuts_tile, 12.0, CARD_BG)?;
        surface.stroke_rounded_rect(cc.shortcuts_tile, 12.0, CARD_BORDER, 1.0)?;
        surface.draw_keyboard_icon(
            cc.shortcuts_tile.x + 17.0,
            cc.shortcuts_tile.y + cc.shortcuts_tile.height * 0.5,
            13.0,
            TEXT_PRIMARY,
        )?;
        surface.draw_text_bold(
            "Shortcuts",
            Rect::new(cc.shortcuts_tile.x + 31.0, cc.shortcuts_tile.y + 4.0, cc.shortcuts_tile.width - 60.0, 16.0),
            11.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            "F1 • Settings",
            Rect::new(cc.shortcuts_tile.x + 31.0, cc.shortcuts_tile.y + 19.0, cc.shortcuts_tile.width - 60.0, 14.0),
            9.5,
            TEXT_MUTED_WHITE,
            TextAlign::Leading,
        )?;
        surface.fill_rounded_rect(cc.settings_gear_btn, 7.0, CARD_BG_ACTIVE)?;
        surface.draw_settings_icon(
            cc.settings_gear_btn.x + cc.settings_gear_btn.width * 0.5,
            cc.settings_gear_btn.y + cc.settings_gear_btn.height * 0.5,
            11.0,
            TEXT_PRIMARY,
        )?;

        // 5. Circular Quick Toggles Row (Dock Fixed/Auto-Hide, Vol HUD, Bri HUD, Notif Suppress, Hime AI)
        let toggle_defs = [
            (
                if self.dock_fixed { "Dock Fix" } else { "Auto-Hide" },
                self.dock_fixed,
                ACCENT_CYAN,
            ),
            ("Vol HUD", self.volume_overlay_enabled, ACCENT_CYAN),
            ("Bri HUD", self.brightness_overlay_enabled, BRIGHTNESS_AMBER),
            ("Quiet", self.notif_suppression_enabled, HIME_CRIMSON),
            ("Hime AI", self.ai_enabled, STATUS_SUCCESS),
        ];
        for (idx, rect) in cc.quick_toggles.iter().enumerate() {
            let (label, active, dot_color) = toggle_defs[idx];
            let cx = rect.x + rect.width * 0.5;
            let cy = rect.y + 17.0;
            let r = 16.5;
            surface.fill_circle(cx, cy, r, if active { CARD_BG_ACTIVE } else { CARD_BG })?;
            surface.stroke_circle(cx, cy, r, if active { dot_color } else { CARD_BORDER }, 1.2)?;
            match idx {
                0 => surface.draw_dock_icon(cx, cy, 12.0, TEXT_PRIMARY)?,
                1 => surface.draw_volume_icon(cx, cy, 12.0, !self.volume_overlay_enabled, 0.8, TEXT_PRIMARY)?,
                2 => surface.draw_sun_icon(cx, cy, 12.0, TEXT_PRIMARY)?,
                3 => surface.draw_bell_icon(cx, cy, 12.0, TEXT_PRIMARY)?,
                _ => surface.draw_sparkles_icon(cx, cy, 12.0, TEXT_PRIMARY)?,
            }
            if active {
                surface.fill_circle(cx + 11.0, cy - 11.0, 2.4, dot_color)?;
            }
            surface.draw_text(
                label,
                Rect::new(rect.x - 6.0, rect.y + 35.0, rect.width + 12.0, 14.0),
                8.5,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;
        }

        // 6. Horizontal Volume & Brightness Sliders
        let vol_ratio = if self.volume_muted {
            0.0
        } else {
            self.volume_level.clamp(0.0, 1.0)
        };
        self.render_slider_row(
            surface,
            cc.volume_icon_btn,
            cc.volume_track,
            cc.volume_pct_rect,
            "Volume",
            vol_ratio,
            ACCENT_CYAN,
            true,
        )?;
        self.render_slider_row(
            surface,
            cc.brightness_icon_btn,
            cc.brightness_track,
            cc.brightness_pct_rect,
            "Brightness",
            self.brightness_level as f32 / 100.0,
            BRIGHTNESS_AMBER,
            false,
        )?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn render_slider_row(
        &self,
        surface: &NativeSurface,
        icon_rect: Rect,
        track_rect: Rect,
        pct_rect: Rect,
        label: &str,
        ratio: f32,
        fill_color: crate::theme::Color,
        is_volume: bool,
    ) -> Result<()> {
        let clamped = ratio.clamp(0.0, 1.0);
        let cx = icon_rect.x + 9.0;
        let cy = icon_rect.y + icon_rect.height * 0.5;
        if is_volume {
            surface.draw_volume_icon(cx, cy, 12.0, self.volume_muted || clamped <= 0.005, clamped, TEXT_PRIMARY)?;
        } else {
            surface.draw_sun_icon(cx, cy, 12.0, TEXT_PRIMARY)?;
        }
        surface.draw_text(
            label,
            Rect::new(icon_rect.x + 20.0, icon_rect.y, 60.0, icon_rect.height),
            9.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        // Track pill
        let bar_h = 8.0;
        let bar_y = track_rect.y + (track_rect.height - bar_h) * 0.5;
        let bar_rect = Rect::new(track_rect.x, bar_y, track_rect.width, bar_h);
        surface.fill_rounded_rect(bar_rect, 4.0, TRACK_BG)?;
        surface.stroke_rounded_rect(bar_rect, 4.0, fill_color.with_alpha(0.35), 1.0)?;

        let fill_w = (bar_rect.width * clamped).max(if clamped > 0.0 { 8.0 } else { 0.0 });
        if fill_w > 0.0 {
            surface.fill_rounded_rect(
                Rect::new(bar_rect.x, bar_rect.y, fill_w, bar_h),
                4.0,
                fill_color,
            )?;
            // Small white knob dot at fill tip
            let knob_x = (bar_rect.x + fill_w).min(bar_rect.x + bar_rect.width - 4.0);
            surface.fill_circle(knob_x, bar_rect.y + bar_h * 0.5, 4.5, TEXT_PRIMARY)?;
        }

        let pct = (clamped * 100.0).round() as u8;
        surface.draw_text(
            &format!("{}%", pct),
            pct_rect,
            9.5,
            TEXT_PRIMARY,
            TextAlign::Trailing,
        )?;
        Ok(())
    }

    fn render_calendar(&self, surface: &NativeSurface, cal: &CalendarLayout) -> Result<()> {
        // Left column: Real month calendar grid
        let title = format!("{} {}", month_name(self.calendar_month), self.calendar_year);
        surface.draw_text_bold(&title, cal.month_title, 12.0, TEXT_PRIMARY, TextAlign::Leading)?;

        surface.fill_rounded_rect(cal.prev_month_btn, 6.0, CARD_BG)?;
        surface.draw_chevron_icon(
            cal.prev_month_btn.x + cal.prev_month_btn.width * 0.5,
            cal.prev_month_btn.y + cal.prev_month_btn.height * 0.5,
            8.0,
            false,
            TEXT_PRIMARY,
        )?;
        surface.fill_rounded_rect(cal.next_month_btn, 6.0, CARD_BG)?;
        surface.draw_chevron_icon(
            cal.next_month_btn.x + cal.next_month_btn.width * 0.5,
            cal.next_month_btn.y + cal.next_month_btn.height * 0.5,
            8.0,
            true,
            TEXT_PRIMARY,
        )?;
        surface.fill_rounded_rect(cal.today_btn, 6.0, CARD_BG)?;
        surface.draw_text("Today", cal.today_btn, 9.0, TEXT_PRIMARY, TextAlign::Center)?;

        let weekdays = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        let cell_w = cal.grid_rect.width / 7.0;
        let row_h = (cal.grid_rect.height - 18.0) / 6.0;
        for (i, wd) in weekdays.iter().enumerate() {
            surface.draw_text(
                wd,
                Rect::new(cal.grid_rect.x + i as f32 * cell_w, cal.grid_rect.y, cell_w, 16.0),
                9.0,
                TEXT_MUTED_WHITE,
                TextAlign::Center,
            )?;
        }

        let offset = first_weekday_sun0(self.calendar_year, self.calendar_month) as usize;
        let total_days = days_in_month(self.calendar_year, self.calendar_month);
        let is_current_month = self.calendar_year == self.calendar_today_year
            && self.calendar_month == self.calendar_today_month;

        for day in 1..=total_days {
            let slot = offset + (day as usize - 1);
            let col = slot % 7;
            let row = slot / 7;
            if row >= 6 {
                break;
            }
            let cx = cal.grid_rect.x + col as f32 * cell_w + cell_w * 0.5;
            let cy = cal.grid_rect.y + 18.0 + row as f32 * row_h + row_h * 0.5;
            let cell_rect = Rect::new(
                cal.grid_rect.x + col as f32 * cell_w,
                cal.grid_rect.y + 18.0 + row as f32 * row_h,
                cell_w,
                row_h,
            );
            let is_today = is_current_month && day == self.calendar_today_day;
            if is_today {
                surface.fill_circle(cx, cy, (row_h * 0.45).min(11.0), HIME_CRIMSON)?;
                surface.draw_text_bold(&day.to_string(), cell_rect, 10.0, TEXT_PRIMARY, TextAlign::Center)?;
            } else {
                surface.draw_text(&day.to_string(), cell_rect, 10.0, TEXT_PRIMARY, TextAlign::Center)?;
            }
        }

        // Vertical separator
        let sep_x = cal.grid_rect.x + cal.grid_rect.width + 6.0;
        surface.draw_line(
            sep_x,
            cal.grid_rect.y - 18.0,
            sep_x,
            cal.grid_rect.y + cal.grid_rect.height,
            CARD_BORDER,
            1.0,
        )?;

        // Right column: Pomodoro Timer
        let status_label = if self.pomodoro.finished {
            "Time's up!"
        } else if self.pomodoro.running {
            "Focusing"
        } else if self.pomodoro.remaining_secs < self.pomodoro.total_secs {
            "Paused"
        } else {
            "Pomodoro"
        };
        surface.draw_text(
            status_label,
            Rect::new(cal.pomo_panel.x, cal.pomo_panel.y + 4.0, cal.pomo_panel.width, 16.0),
            10.0,
            TEXT_MUTED_WHITE,
            TextAlign::Center,
        )?;
        surface.draw_text_bold(
            &self.pomodoro.formatted(),
            Rect::new(cal.pomo_panel.x, cal.pomo_panel.y + 26.0, cal.pomo_panel.width, 40.0),
            26.0,
            if self.pomodoro.finished { STATUS_ALERT } else { TEXT_PRIMARY },
            TextAlign::Center,
        )?;

        // Start/Pause pill button + Reset button
        let play_bg = if self.pomodoro.running {
            CARD_BG_ACTIVE
        } else {
            HIME_CRIMSON
        };
        surface.fill_rounded_rect(cal.pomo_play_btn, 14.0, play_bg)?;
        surface.stroke_rounded_rect(cal.pomo_play_btn, 14.0, NOTCH_EDGE_BORDER, 1.0)?;
        surface.draw_text_bold(
            if self.pomodoro.running { "Pause" } else { "Start" },
            cal.pomo_play_btn,
            11.0,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        surface.fill_rounded_rect(cal.pomo_reset_btn, 14.0, CARD_BG)?;
        surface.stroke_rounded_rect(cal.pomo_reset_btn, 14.0, CARD_BORDER, 1.0)?;
        surface.draw_text("Rst", cal.pomo_reset_btn, 9.5, TEXT_PRIMARY, TextAlign::Center)?;

        // Preset buttons (5m, 15m, 25m, 50m)
        for (mins, rect) in &cal.preset_btns {
            let active = self.pomodoro.preset_mins == *mins;
            surface.fill_rounded_rect(*rect, 6.0, if active { CARD_BG_ACTIVE } else { CARD_BG })?;
            if active {
                surface.stroke_rounded_rect(*rect, 6.0, HIME_CRIMSON.with_alpha(0.75), 1.0)?;
            }
            surface.draw_text(
                &format!("{}m", mins),
                *rect,
                9.5,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;
        }

        Ok(())
    }

    fn render_music(&self, surface: &NativeSurface, mus: &MusicLayout) -> Result<()> {
        let title = self
            .music
            .as_ref()
            .map(|m| m.title.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("No Media Playing");
        let artist = self
            .music
            .as_ref()
            .map(|m| m.artist.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("Play music on your PC to sync");
        let is_playing = self.music.as_ref().is_some_and(|m| m.is_playing);

        // Album badge
        surface.fill_rounded_rect(mus.album_badge, 11.0, CARD_BG_ACTIVE)?;
        surface.stroke_rounded_rect(mus.album_badge, 11.0, NOTCH_EDGE_BORDER, 1.0)?;
        surface.draw_music_note_icon(
            mus.album_badge.x + mus.album_badge.width * 0.5,
            mus.album_badge.y + mus.album_badge.height * 0.5,
            20.0,
            TEXT_PRIMARY,
        )?;
        if is_playing {
            surface.fill_circle(
                mus.album_badge.x + mus.album_badge.width - 7.0,
                mus.album_badge.y + 7.0,
                3.0,
                STATUS_SUCCESS,
            )?;
        }

        // Title + Artist + optional album
        surface.draw_text_bold(
            title,
            Rect::new(mus.title_rect.x, mus.title_rect.y, mus.title_rect.width, 18.0),
            13.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        let sub_text = if let Some(alb) = &self.music_album {
            if !alb.is_empty() {
                format!("{} • {}", artist, alb)
            } else {
                artist.to_string()
            }
        } else {
            artist.to_string()
        };
        surface.draw_text(
            &sub_text,
            Rect::new(mus.title_rect.x, mus.title_rect.y + 18.0, mus.title_rect.width, 16.0),
            10.5,
            TEXT_MUTED_WHITE,
            TextAlign::Leading,
        )?;

        // Visualizer + Lirik toggle button
        let bars = animated_visualizer_bars(self.visualizer_phase);
        surface.draw_music_visualizer(
            mus.lyrics_btn.x - 26.0,
            mus.lyrics_btn.y + 4.0,
            16.0,
            &bars,
            is_playing,
        )?;
        surface.fill_rounded_rect(
            mus.lyrics_btn,
            9.0,
            if self.music_show_lyrics { CARD_BG_ACTIVE } else { CARD_BG },
        )?;
        surface.stroke_rounded_rect(
            mus.lyrics_btn,
            9.0,
            if self.music_show_lyrics { ACCENT_CYAN } else { CARD_BORDER },
            1.0,
        )?;
        surface.draw_text_bold("Lirik", mus.lyrics_btn, 10.0, TEXT_PRIMARY, TextAlign::Center)?;

        // Progress bar
        let ratio = if self.music_duration_ms > 0 {
            (self.music_position_ms as f32 / self.music_duration_ms as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        surface.draw_text(
            &format_ms(self.music_position_ms),
            mus.elapsed_rect,
            9.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        let bar_h = 5.0;
        let bar_y = mus.progress_track.y + (mus.progress_track.height - bar_h) * 0.5;
        let bar_rect = Rect::new(mus.progress_track.x, bar_y, mus.progress_track.width, bar_h);
        surface.fill_rounded_rect(bar_rect, 2.5, TRACK_BG)?;
        if ratio > 0.0 {
            surface.fill_rounded_rect(
                Rect::new(bar_rect.x, bar_rect.y, (bar_rect.width * ratio).max(4.0), bar_h),
                2.5,
                TEXT_PRIMARY,
            )?;
        }
        surface.draw_text(
            &format_remaining_ms(self.music_position_ms, self.music_duration_ms),
            mus.remaining_rect,
            9.5,
            TEXT_PRIMARY,
            TextAlign::Trailing,
        )?;

        // Transport controls (Prev / Play-Pause / Next)
        surface.fill_rounded_rect(mus.prev_btn, 14.0, CARD_BG)?;
        surface.draw_skip_icon(
            mus.prev_btn.x + mus.prev_btn.width * 0.5,
            mus.prev_btn.y + mus.prev_btn.height * 0.5,
            13.0,
            false,
            TEXT_PRIMARY,
        )?;
        surface.fill_rounded_rect(mus.play_pause_btn, 18.0, CARD_BG_ACTIVE)?;
        surface.stroke_rounded_rect(mus.play_pause_btn, 18.0, NOTCH_EDGE_BORDER, 1.0)?;
        surface.draw_play_pause_icon(
            mus.play_pause_btn.x + mus.play_pause_btn.width * 0.5,
            mus.play_pause_btn.y + mus.play_pause_btn.height * 0.5,
            15.0,
            is_playing,
            TEXT_PRIMARY,
        )?;
        surface.fill_rounded_rect(mus.next_btn, 14.0, CARD_BG)?;
        surface.draw_skip_icon(
            mus.next_btn.x + mus.next_btn.width * 0.5,
            mus.next_btn.y + mus.next_btn.height * 0.5,
            13.0,
            true,
            TEXT_PRIMARY,
        )?;

        // LRC Lyrics Area
        surface.fill_rounded_rect(mus.lyrics_box, 10.0, CARD_BG)?;
        surface.stroke_rounded_rect(mus.lyrics_box, 10.0, CARD_BORDER, 1.0)?;

        let active_line = self
            .music_active_lyric_idx
            .and_then(|idx| self.music_lrc_lines.get(idx).cloned())
            .or_else(|| self.music.as_ref().and_then(|m| m.lyric_line.clone()))
            .or_else(|| self.music_lrc_lines.first().cloned());

        if self.music_show_lyrics && !self.music_lrc_lines.is_empty() {
            let active_idx = self.music_active_lyric_idx.unwrap_or(0);
            let start = active_idx.saturating_sub(1);
            let line_h = (mus.lyrics_text_box.height - 4.0) / 3.0;
            for row_i in 0..3 {
                let line_idx = start + row_i;
                let Some(line) = self.music_lrc_lines.get(line_idx) else {
                    continue;
                };
                let is_curr = line_idx == active_idx;
                let rect = Rect::new(
                    mus.lyrics_text_box.x + 10.0,
                    mus.lyrics_text_box.y + 2.0 + row_i as f32 * line_h,
                    mus.lyrics_text_box.width - 20.0,
                    line_h,
                );
                if is_curr {
                    surface.draw_text_bold(line, rect, 11.0, ACCENT_CYAN, TextAlign::Center)?;
                } else {
                    surface.draw_text(line, rect, 9.5, TEXT_SUBTLE_WHITE, TextAlign::Center)?;
                }
            }
        } else if let Some(line) = active_line {
            let next_line = self
                .music_active_lyric_idx
                .and_then(|idx| self.music_lrc_lines.get(idx + 1).cloned());
            if let Some(upcoming) = next_line {
                let half_h = (mus.lyrics_text_box.height - 4.0) * 0.5;
                surface.draw_text_bold(
                    &line,
                    Rect::new(mus.lyrics_text_box.x + 10.0, mus.lyrics_text_box.y + 2.0, mus.lyrics_text_box.width - 20.0, half_h),
                    11.0,
                    TEXT_PRIMARY,
                    TextAlign::Center,
                )?;
                surface.draw_text(
                    &upcoming,
                    Rect::new(mus.lyrics_text_box.x + 10.0, mus.lyrics_text_box.y + 2.0 + half_h, mus.lyrics_text_box.width - 20.0, half_h),
                    9.5,
                    TEXT_MUTED_WHITE,
                    TextAlign::Center,
                )?;
            } else {
                surface.draw_text_bold(&line, mus.lyrics_text_box, 11.0, TEXT_PRIMARY, TextAlign::Center)?;
            }
        } else {
            let fallback = if self.music_show_lyrics {
                "♪ Synced lyrics unavailable for this track"
            } else {
                "♪ Click Lirik for synced LRC lyrics"
            };
            surface.draw_text(fallback, mus.lyrics_text_box, 10.0, TEXT_MUTED_WHITE, TextAlign::Center)?;
        }

        if self.music_show_lyrics {
            let offset_label = if self.music_lyric_offset_ms > 0 {
                format!("Offset: +{}ms", self.music_lyric_offset_ms)
            } else {
                format!("Offset: {}ms", self.music_lyric_offset_ms)
            };
            surface.draw_text(
                &offset_label,
                mus.offset_label_rect,
                8.5,
                if self.music_lyric_offset_ms != 0 { ACCENT_CYAN } else { TEXT_MUTED_WHITE },
                TextAlign::Leading,
            )?;
            surface.fill_rounded_rect(mus.offset_minus_btn, 5.0, CARD_BG_ACTIVE)?;
            surface.draw_text("-0.2s", mus.offset_minus_btn, 8.5, TEXT_PRIMARY, TextAlign::Center)?;
            surface.fill_rounded_rect(mus.offset_plus_btn, 5.0, CARD_BG_ACTIVE)?;
            surface.draw_text("+0.2s", mus.offset_plus_btn, 8.5, TEXT_PRIMARY, TextAlign::Center)?;
            surface.fill_rounded_rect(
                mus.offset_reset_btn,
                5.0,
                if self.music_lyric_offset_ms != 0 {
                    ACCENT_CYAN.with_alpha(0.22)
                } else {
                    CARD_BG
                },
            )?;
            surface.draw_text("Rst", mus.offset_reset_btn, 8.5, TEXT_PRIMARY, TextAlign::Center)?;
        }

        Ok(())
    }

    fn render_wifi(&self, surface: &NativeSurface, wf: &WifiLayout) -> Result<()> {
        surface.fill_rounded_rect(wf.back_btn, 8.0, CARD_BG)?;
        surface.draw_chevron_icon(
            wf.back_btn.x + 12.0,
            wf.back_btn.y + wf.back_btn.height * 0.5,
            8.0,
            false,
            TEXT_PRIMARY,
        )?;
        surface.draw_text("Back", Rect::new(wf.back_btn.x + 16.0, wf.back_btn.y, wf.back_btn.width - 18.0, wf.back_btn.height), 9.5, TEXT_PRIMARY, TextAlign::Center)?;

        surface.draw_text_bold(
            "Wi-Fi Networks",
            wf.title_rect,
            12.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        surface.fill_rounded_rect(
            wf.toggle_btn,
            8.0,
            if self.wifi_enabled { CARD_BG_ACTIVE } else { CARD_BG },
        )?;
        surface.stroke_rounded_rect(
            wf.toggle_btn,
            8.0,
            if self.wifi_enabled { ACCENT_CYAN } else { CARD_BORDER },
            1.0,
        )?;
        surface.draw_text(
            if self.wifi_enabled { "On" } else { "Off" },
            wf.toggle_btn,
            9.5,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        surface.fill_rounded_rect(wf.refresh_btn, 8.0, CARD_BG)?;
        surface.draw_text("↻", wf.refresh_btn, 12.0, TEXT_PRIMARY, TextAlign::Center)?;

        if !self.wifi_enabled {
            surface.fill_rounded_rect(wf.list_box, 10.0, CARD_BG)?;
            surface.draw_text("Wi-Fi is turned off", wf.list_box, 11.0, TEXT_MUTED_WHITE, TextAlign::Center)?;
            return Ok(());
        }

        if self.wifi_networks.is_empty() {
            surface.fill_rounded_rect(wf.list_box, 10.0, CARD_BG)?;
            let msg = if let Some(ssid) = &self.wifi_ssid {
                format!("Connected to {}", ssid)
            } else {
                "No nearby Wi-Fi networks found".to_string()
            };
            surface.draw_text(&msg, wf.list_box, 11.0, TEXT_PRIMARY, TextAlign::Center)?;
            return Ok(());
        }

        for (idx, (ssid, row_rect)) in wf.network_rows.iter().enumerate() {
            let item = &self.wifi_networks[idx];
            surface.fill_rounded_rect(
                *row_rect,
                9.0,
                if item.connected { CARD_BG_ACTIVE } else { CARD_BG },
            )?;
            surface.stroke_rounded_rect(
                *row_rect,
                9.0,
                if item.connected { ACCENT_CYAN.with_alpha(0.7) } else { CARD_BORDER },
                1.0,
            )?;
            surface.draw_wifi_icon(
                row_rect.x + 16.0,
                row_rect.y + row_rect.height * 0.5,
                11.0,
                item.signal > 0,
                false,
                TEXT_PRIMARY,
            )?;
            surface.draw_text_bold(
                ssid,
                Rect::new(row_rect.x + 30.0, row_rect.y, row_rect.width - 116.0, row_rect.height),
                10.5,
                TEXT_PRIMARY,
                TextAlign::Leading,
            )?;
            let action_rect = Rect::new(
                row_rect.x + row_rect.width - 78.0,
                row_rect.y + 4.0,
                70.0,
                row_rect.height - 8.0,
            );
            surface.fill_rounded_rect(
                action_rect,
                6.0,
                if item.connected { STATUS_SUCCESS.with_alpha(0.25) } else { CARD_BG_ACTIVE },
            )?;
            surface.draw_text(
                if item.connected { "Connected" } else { "Connect" },
                action_rect,
                9.0,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;
        }

        Ok(())
    }

    fn render_shortcuts(&self, surface: &NativeSurface, sc: &ShortcutsLayout) -> Result<()> {
        surface.draw_keyboard_icon(
            sc.header_rect.x + 10.0,
            sc.header_rect.y + sc.header_rect.height * 0.5,
            13.0,
            TEXT_PRIMARY,
        )?;
        surface.draw_text_bold(
            "Daftar Pintasan Keyboard",
            Rect::new(sc.header_rect.x + 24.0, sc.header_rect.y, sc.header_rect.width - 96.0, sc.header_rect.height),
            12.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.fill_rounded_rect(sc.settings_btn, 7.0, CARD_BG)?;
        surface.stroke_rounded_rect(sc.settings_btn, 7.0, CARD_BORDER, 1.0)?;
        surface.draw_text("Settings", sc.settings_btn, 9.0, TEXT_PRIMARY, TextAlign::Center)?;

        let left_items = [
            ("Notch", "F12 / Pause"),
            ("Asisten", "Ctrl+Shift+H"),
            ("Musik & Lirik", "Ctrl+Shift+M"),
            ("Dock", "Ctrl+Shift+D"),
            ("Pintasan", "F1"),
            ("Tutup Modal", "Esc"),
        ];
        let right_items = [
            ("Chat Panjang", "/chat"),
            ("Kirim Pesan", "Enter"),
            ("Baris Baru", "Shift+Enter"),
            ("Ganti Tab", "Shift+Scroll"),
            ("Keluar", "Ctrl+Shift+Q"),
            ("Jeda Darurat", "Ctrl+Alt+P"),
        ];

        self.render_shortcut_column(surface, sc.left_col, "Aktivasi & Tampilan", &left_items)?;
        self.render_shortcut_column(surface, sc.right_col, "Navigasi & Sistem", &right_items)?;
        Ok(())
    }

    fn render_shortcut_column(
        &self,
        surface: &NativeSurface,
        col: Rect,
        heading: &str,
        items: &[(&str, &str)],
    ) -> Result<()> {
        surface.draw_text_bold(
            heading,
            Rect::new(col.x + 2.0, col.y, col.width - 4.0, 14.0),
            9.0,
            ACCENT_CYAN,
            TextAlign::Leading,
        )?;
        let box_rect = Rect::new(col.x, col.y + 16.0, col.width, col.height - 16.0);
        surface.fill_rounded_rect(box_rect, 10.0, CARD_BG)?;
        surface.stroke_rounded_rect(box_rect, 10.0, CARD_BORDER, 1.0)?;

        let row_h = (box_rect.height - 8.0) / items.len() as f32;
        for (i, (label, keys)) in items.iter().enumerate() {
            let ry = box_rect.y + 4.0 + i as f32 * row_h;
            surface.draw_text(
                label,
                Rect::new(box_rect.x + 8.0, ry, box_rect.width * 0.48, row_h),
                9.5,
                TEXT_PRIMARY,
                TextAlign::Leading,
            )?;
            let key_rect = Rect::new(
                box_rect.x + box_rect.width * 0.50,
                ry + 2.0,
                box_rect.width * 0.50 - 8.0,
                (row_h - 4.0).max(14.0),
            );
            surface.fill_rounded_rect(key_rect, 5.0, CARD_BG_ACTIVE)?;
            surface.stroke_rounded_rect(key_rect, 5.0, CARD_BORDER, 0.8)?;
            surface.draw_text(keys, key_rect, 8.5, TEXT_PRIMARY, TextAlign::Center)?;
        }
        Ok(())
    }

    fn render_shelf(&self, surface: &NativeSurface, sh: &ShelfLayout) -> Result<()> {
        surface.draw_text_bold(
            &format!("File Shelf ({})", self.shelf_items.len()),
            sh.header_rect,
            12.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        if let Some(clear_btn) = sh.clear_btn {
            surface.fill_rounded_rect(clear_btn, 7.0, CARD_BG)?;
            surface.stroke_rounded_rect(clear_btn, 7.0, CARD_BORDER, 1.0)?;
            surface.draw_text("Clear", clear_btn, 9.5, TEXT_PRIMARY, TextAlign::Center)?;
        }

        if self.shelf_items.is_empty() {
            surface.fill_rounded_rect(sh.body_rect, 12.0, CARD_BG)?;
            surface.stroke_rounded_rect(sh.body_rect, 12.0, CARD_BORDER, 1.0)?;
            surface.draw_text(
                "Drop files onto the Notch to stash them here",
                sh.body_rect,
                11.0,
                TEXT_MUTED_WHITE,
                TextAlign::Center,
            )?;
            return Ok(());
        }

        let max_rows = 4.min(self.shelf_items.len());
        let row_h = 38.0;
        let gap = 6.0;
        for i in 0..max_rows {
            let item = &self.shelf_items[i];
            let ry = sh.body_rect.y + i as f32 * (row_h + gap);
            let row = Rect::new(sh.body_rect.x, ry, sh.body_rect.width, row_h);
            surface.fill_rounded_rect(row, 9.0, CARD_BG)?;
            surface.stroke_rounded_rect(row, 9.0, CARD_BORDER, 1.0)?;

            let tag_rect = Rect::new(row.x + 8.0, row.y + 8.0, 44.0, 22.0);
            surface.fill_rounded_rect(tag_rect, 5.0, CARD_BG_ACTIVE)?;
            surface.draw_text(&item.kind, tag_rect, 8.5, TEXT_PRIMARY, TextAlign::Center)?;

            surface.draw_text_bold(
                &item.name,
                Rect::new(row.x + 60.0, row.y + 4.0, row.width - 134.0, row.height - 8.0),
                10.5,
                TEXT_PRIMARY,
                TextAlign::Leading,
            )?;
            surface.draw_text(
                &item.size_label,
                Rect::new(row.x + row.width - 70.0, row.y + 4.0, 62.0, row.height - 8.0),
                9.5,
                TEXT_MUTED_WHITE,
                TextAlign::Trailing,
            )?;
        }

        Ok(())
    }

    fn render_hime_ai(&self, surface: &NativeSurface, ai: &HimeAiLayout) -> Result<()> {
        surface.fill_rounded_rect(ai.card_rect, 12.0, CARD_BG)?;
        surface.stroke_rounded_rect(ai.card_rect, 12.0, CARD_BORDER, 1.0)?;
        surface.draw_hime_eyes(
            ai.card_rect.x + ai.card_rect.width * 0.5,
            ai.card_rect.y + 42.0,
            1.3,
            self.expression,
            self.blink_progress,
        )?;
        surface.draw_text_bold(
            "Hime AI Assistant",
            Rect::new(ai.card_rect.x + 16.0, ai.card_rect.y + 72.0, ai.card_rect.width - 32.0, 22.0),
            13.0,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;
        surface.draw_text(
            &self.ai_status_text,
            Rect::new(ai.card_rect.x + 16.0, ai.card_rect.y + 98.0, ai.card_rect.width - 32.0, 20.0),
            10.5,
            TEXT_MUTED_WHITE,
            TextAlign::Center,
        )?;
        surface.fill_rounded_rect(ai.open_drawer_btn, 10.0, HIME_CRIMSON)?;
        surface.stroke_rounded_rect(ai.open_drawer_btn, 10.0, NOTCH_EDGE_BORDER, 1.0)?;
        surface.draw_text_bold(
            "Open Hime AI Drawer",
            ai.open_drawer_btn,
            11.0,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;
        Ok(())
    }

    fn compute_expanded_layout(&self, outer: Rect) -> ExpandedLayout {
        let inner_x = outer.x + SAFE_PADDING;
        let inner_w = (outer.width - SAFE_PADDING * 2.0).max(180.0);
        let top_y = outer.y + 6.0;

        let prev_btn = Rect::new(inner_x, top_y + 2.0, 22.0, 22.0);
        let header_clock = Rect::new(inner_x + 24.0, top_y + 1.0, 48.0, 24.0);
        let next_btn = Rect::new(inner_x + 74.0, top_y + 2.0, 22.0, 22.0);
        let close_btn = Rect::new(inner_x + inner_w - 22.0, top_y + 2.0, 22.0, 22.0);

        let panels = self.available_panels();
        let avail_tabs_w = (close_btn.x - 8.0) - (next_btn.x + next_btn.width + 6.0);
        let tab_gap = 3.0;
        let tab_w = ((avail_tabs_w - (panels.len().saturating_sub(1)) as f32 * tab_gap)
            / panels.len().max(1) as f32)
            .clamp(16.0, 24.0);
        let total_tabs_w = panels.len() as f32 * tab_w + (panels.len().saturating_sub(1)) as f32 * tab_gap;
        let tabs_start_x = (close_btn.x - 8.0 - total_tabs_w).max(next_btn.x + next_btn.width + 6.0);
        let tabs = panels
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    p,
                    Rect::new(tabs_start_x + i as f32 * (tab_w + tab_gap), top_y + 2.0, tab_w, 22.0),
                )
            })
            .collect();

        let body_y = outer.y + HEADER_HEIGHT + 8.0;
        let body_h = (outer.y + outer.height - 14.0 - body_y).max(120.0);

        let body = match self.active_panel {
            NotchPanel::Status | NotchPanel::CommandCenter => {
                let gap = 10.0;
                let tile_w = (inner_w - gap) * 0.5;
                let tile_h = 38.0;
                let wifi_tile = Rect::new(inner_x, body_y, tile_w, tile_h);
                let wifi_more_btn = Rect::new(wifi_tile.x + tile_w - 24.0, wifi_tile.y + 8.0, 18.0, 22.0);
                let bluetooth_tile = Rect::new(inner_x + tile_w + gap, body_y, tile_w, tile_h);
                let row2_y = body_y + tile_h + 8.0;
                let notch_mode_tile = Rect::new(inner_x, row2_y, tile_w, tile_h);
                let drag_badge = Rect::new(notch_mode_tile.x + tile_w - 38.0, row2_y + 9.0, 32.0, 20.0);
                let shortcuts_tile = Rect::new(inner_x + tile_w + gap, row2_y, tile_w, tile_h);
                let settings_gear_btn = Rect::new(shortcuts_tile.x + tile_w - 26.0, row2_y + 7.0, 20.0, 24.0);

                let toggles_y = row2_y + tile_h + 12.0;
                let col_step = inner_w / 5.0;
                let mut quick_toggles = [Rect::new(0.0, 0.0, 0.0, 0.0); 5];
                for (i, slot) in quick_toggles.iter_mut().enumerate() {
                    let cx = inner_x + col_step * (i as f32 + 0.5);
                    *slot = Rect::new(cx - 18.0, toggles_y, 36.0, 48.0);
                }

                let slider1_y = toggles_y + 56.0;
                let slider_h = 22.0;
                let slider2_y = slider1_y + slider_h + 8.0;
                let label_w = 82.0;
                let pct_w = 34.0;
                let track_w = (inner_w - label_w - pct_w - 8.0).max(80.0);

                BodyLayout::CommandCenter(CommandCenterLayout {
                    wifi_tile,
                    wifi_more_btn,
                    bluetooth_tile,
                    notch_mode_tile,
                    drag_badge,
                    shortcuts_tile,
                    settings_gear_btn,
                    quick_toggles,
                    volume_icon_btn: Rect::new(inner_x, slider1_y, label_w, slider_h),
                    volume_track: Rect::new(inner_x + label_w + 4.0, slider1_y, track_w, slider_h),
                    volume_pct_rect: Rect::new(inner_x + inner_w - pct_w, slider1_y, pct_w, slider_h),
                    brightness_icon_btn: Rect::new(inner_x, slider2_y, label_w, slider_h),
                    brightness_track: Rect::new(inner_x + label_w + 4.0, slider2_y, track_w, slider_h),
                    brightness_pct_rect: Rect::new(inner_x + inner_w - pct_w, slider2_y, pct_w, slider_h),
                })
            }
            NotchPanel::Calendar => {
                let left_w = (inner_w * 0.57).round();
                let right_x = inner_x + left_w + 14.0;
                let right_w = (inner_x + inner_w - right_x).max(110.0);

                let month_title = Rect::new(inner_x, body_y, left_w - 78.0, 20.0);
                let prev_month_btn = Rect::new(inner_x + left_w - 76.0, body_y, 18.0, 18.0);
                let next_month_btn = Rect::new(inner_x + left_w - 54.0, body_y, 18.0, 18.0);
                let today_btn = Rect::new(inner_x + left_w - 34.0, body_y, 34.0, 18.0);
                let grid_rect = Rect::new(inner_x, body_y + 24.0, left_w, (body_h - 26.0).max(120.0));

                let pomo_panel = Rect::new(right_x, body_y, right_w, body_h);
                let pomo_play_btn = Rect::new(right_x, body_y + 82.0, right_w - 36.0, 30.0);
                let pomo_reset_btn = Rect::new(right_x + right_w - 30.0, body_y + 82.0, 30.0, 30.0);

                let presets = [5u16, 15, 25, 50];
                let p_gap = 4.0;
                let p_w = (right_w - p_gap * 3.0) / 4.0;
                let preset_btns = presets
                    .iter()
                    .enumerate()
                    .map(|(i, &m)| {
                        (
                            m,
                            Rect::new(right_x + i as f32 * (p_w + p_gap), body_y + 124.0, p_w, 24.0),
                        )
                    })
                    .collect();

                BodyLayout::Calendar(CalendarLayout {
                    month_title,
                    prev_month_btn,
                    next_month_btn,
                    today_btn,
                    grid_rect,
                    pomo_panel,
                    pomo_play_btn,
                    pomo_reset_btn,
                    preset_btns,
                })
            }
            NotchPanel::Music => {
                let album_badge = Rect::new(inner_x, body_y, 46.0, 46.0);
                let lyrics_btn = Rect::new(inner_x + inner_w - 46.0, body_y + 10.0, 46.0, 24.0);
                let title_rect = Rect::new(inner_x + 56.0, body_y + 4.0, (inner_w - 138.0).max(90.0), 38.0);

                let prog_y = body_y + 54.0;
                let elapsed_rect = Rect::new(inner_x, prog_y, 34.0, 18.0);
                let remaining_rect = Rect::new(inner_x + inner_w - 38.0, prog_y, 38.0, 18.0);
                let progress_track = Rect::new(inner_x + 38.0, prog_y, (inner_w - 80.0).max(80.0), 18.0);

                let ctrl_y = prog_y + 24.0;
                let mid_x = inner_x + inner_w * 0.5;
                let play_pause_btn = Rect::new(mid_x - 18.0, ctrl_y, 36.0, 36.0);
                let prev_btn = Rect::new(mid_x - 62.0, ctrl_y + 4.0, 28.0, 28.0);
                let next_btn = Rect::new(mid_x + 34.0, ctrl_y + 4.0, 28.0, 28.0);

                let lyrics_y = ctrl_y + 44.0;
                let lyrics_h = (body_y + body_h - lyrics_y).max(52.0);
                let lyrics_box = Rect::new(inner_x, lyrics_y, inner_w, lyrics_h);
                let lyrics_text_box = if self.music_show_lyrics && lyrics_h > 44.0 {
                    Rect::new(lyrics_box.x, lyrics_box.y, lyrics_box.width, lyrics_box.height - 22.0)
                } else {
                    lyrics_box
                };
                let bar_y = lyrics_box.y + lyrics_box.height - 20.0;
                let offset_reset_btn = Rect::new(lyrics_box.x + lyrics_box.width - 34.0, bar_y, 28.0, 16.0);
                let offset_plus_btn = Rect::new(offset_reset_btn.x - 40.0, bar_y, 36.0, 16.0);
                let offset_minus_btn = Rect::new(offset_plus_btn.x - 40.0, bar_y, 36.0, 16.0);
                let offset_label_rect = Rect::new(
                    lyrics_box.x + 8.0,
                    bar_y,
                    (offset_minus_btn.x - lyrics_box.x - 12.0).max(60.0),
                    16.0,
                );

                BodyLayout::Music(MusicLayout {
                    album_badge,
                    title_rect,
                    lyrics_btn,
                    elapsed_rect,
                    progress_track,
                    remaining_rect,
                    prev_btn,
                    play_pause_btn,
                    next_btn,
                    lyrics_box,
                    lyrics_text_box,
                    offset_label_rect,
                    offset_minus_btn,
                    offset_plus_btn,
                    offset_reset_btn,
                })
            }
            NotchPanel::Wifi => {
                let back_btn = Rect::new(inner_x, body_y, 52.0, 22.0);
                let title_rect = Rect::new(inner_x + 60.0, body_y, inner_w - 136.0, 22.0);
                let toggle_btn = Rect::new(inner_x + inner_w - 68.0, body_y, 38.0, 22.0);
                let refresh_btn = Rect::new(inner_x + inner_w - 26.0, body_y, 26.0, 22.0);
                let list_box = Rect::new(inner_x, body_y + 30.0, inner_w, (body_h - 32.0).max(120.0));

                let mut network_rows = Vec::new();
                let row_h = 32.0;
                let gap = 6.0;
                for (i, item) in self.wifi_networks.iter().take(5).enumerate() {
                    let ry = list_box.y + i as f32 * (row_h + gap);
                    if ry + row_h <= list_box.y + list_box.height + 2.0 {
                        network_rows.push((
                            item.ssid.clone(),
                            Rect::new(list_box.x, ry, list_box.width, row_h),
                        ));
                    }
                }

                BodyLayout::Wifi(WifiLayout {
                    back_btn,
                    title_rect,
                    toggle_btn,
                    refresh_btn,
                    list_box,
                    network_rows,
                })
            }
            NotchPanel::Shortcuts => {
                let header_rect = Rect::new(inner_x, body_y, inner_w, 22.0);
                let settings_btn = Rect::new(inner_x + inner_w - 64.0, body_y, 64.0, 20.0);
                let col_y = body_y + 28.0;
                let col_h = (body_h - 28.0).max(140.0);
                let col_gap = 12.0;
                let col_w = (inner_w - col_gap) * 0.5;
                BodyLayout::Shortcuts(ShortcutsLayout {
                    header_rect,
                    settings_btn,
                    left_col: Rect::new(inner_x, col_y, col_w, col_h),
                    right_col: Rect::new(inner_x + col_w + col_gap, col_y, col_w, col_h),
                })
            }
            NotchPanel::Shelf => {
                let header_rect = Rect::new(inner_x, body_y, inner_w - 62.0, 22.0);
                let clear_btn = if self.shelf_items.is_empty() {
                    None
                } else {
                    Some(Rect::new(inner_x + inner_w - 56.0, body_y, 56.0, 22.0))
                };
                let body_rect = Rect::new(inner_x, body_y + 30.0, inner_w, (body_h - 30.0).max(120.0));
                BodyLayout::Shelf(ShelfLayout {
                    header_rect,
                    clear_btn,
                    body_rect,
                })
            }
            NotchPanel::Tasks => BodyLayout::Tasks(Rect::new(inner_x, body_y, inner_w, body_h)),
            NotchPanel::Focus => BodyLayout::Focus(Rect::new(inner_x, body_y, inner_w, body_h)),
            NotchPanel::HimeAi => {
                let card_rect = Rect::new(inner_x, body_y, inner_w, body_h);
                let open_drawer_btn = Rect::new(
                    inner_x + (inner_w - 170.0) * 0.5,
                    body_y + body_h - 48.0,
                    170.0,
                    32.0,
                );
                BodyLayout::HimeAi(HimeAiLayout {
                    card_rect,
                    open_drawer_btn,
                })
            }
        };

        ExpandedLayout {
            prev_btn,
            header_clock,
            next_btn,
            tabs,
            close_btn,
            body,
        }
    }

    fn sync_targets(&mut self) {
        let has_alert = self.ncm_center_alert.is_some();
        let has_banner = self.has_active_banner();
        let (w, h) = target_dimensions(
            self.edge,
            self.is_hidden,
            self.is_expanded,
            self.active_panel,
            has_alert,
            has_banner,
        );
        self.width_spring.set_target(w);
        self.height_spring.set_target(h);
        self.expand_spring
            .set_target(if self.is_expanded && !self.is_hidden { 1.0 } else { 0.0 });
    }
}

struct ExpandedLayout {
    prev_btn: Rect,
    header_clock: Rect,
    next_btn: Rect,
    tabs: Vec<(NotchPanel, Rect)>,
    close_btn: Rect,
    body: BodyLayout,
}

enum BodyLayout {
    CommandCenter(CommandCenterLayout),
    Calendar(CalendarLayout),
    Music(MusicLayout),
    Wifi(WifiLayout),
    Shortcuts(ShortcutsLayout),
    Shelf(ShelfLayout),
    Tasks(Rect),
    Focus(Rect),
    HimeAi(HimeAiLayout),
}

struct CommandCenterLayout {
    wifi_tile: Rect,
    wifi_more_btn: Rect,
    bluetooth_tile: Rect,
    notch_mode_tile: Rect,
    drag_badge: Rect,
    shortcuts_tile: Rect,
    settings_gear_btn: Rect,
    quick_toggles: [Rect; 5],
    volume_icon_btn: Rect,
    volume_track: Rect,
    volume_pct_rect: Rect,
    brightness_icon_btn: Rect,
    brightness_track: Rect,
    brightness_pct_rect: Rect,
}

struct CalendarLayout {
    month_title: Rect,
    prev_month_btn: Rect,
    next_month_btn: Rect,
    today_btn: Rect,
    grid_rect: Rect,
    pomo_panel: Rect,
    pomo_play_btn: Rect,
    pomo_reset_btn: Rect,
    preset_btns: Vec<(u16, Rect)>,
}

struct MusicLayout {
    album_badge: Rect,
    title_rect: Rect,
    lyrics_btn: Rect,
    elapsed_rect: Rect,
    progress_track: Rect,
    remaining_rect: Rect,
    prev_btn: Rect,
    play_pause_btn: Rect,
    next_btn: Rect,
    lyrics_box: Rect,
    lyrics_text_box: Rect,
    offset_label_rect: Rect,
    offset_minus_btn: Rect,
    offset_plus_btn: Rect,
    offset_reset_btn: Rect,
}

struct WifiLayout {
    back_btn: Rect,
    title_rect: Rect,
    toggle_btn: Rect,
    refresh_btn: Rect,
    list_box: Rect,
    network_rows: Vec<(String, Rect)>,
}

struct ShortcutsLayout {
    header_rect: Rect,
    settings_btn: Rect,
    left_col: Rect,
    right_col: Rect,
}

struct ShelfLayout {
    header_rect: Rect,
    clear_btn: Option<Rect>,
    body_rect: Rect,
}

struct HimeAiLayout {
    card_rect: Rect,
    open_drawer_btn: Rect,
}

fn target_dimensions(
    edge: NotchEdge,
    is_hidden: bool,
    is_expanded: bool,
    panel: NotchPanel,
    has_alert: bool,
    has_banner: bool,
) -> (f32, f32) {
    match (edge, is_hidden, is_expanded) {
        (NotchEdge::Top, true, _) => (48.0, HIDDEN_BAR_THICKNESS),
        (NotchEdge::Left | NotchEdge::Right, true, _) => (HIDDEN_BAR_THICKNESS, 48.0),
        (NotchEdge::Top, false, false) => {
            if has_banner {
                (TOP_BANNER_WIDTH, TOP_BANNER_HEIGHT)
            } else {
                (
                    if has_alert {
                        TOP_ALERT_WIDTH
                    } else {
                        TOP_COLLAPSED_WIDTH
                    },
                    TOP_COLLAPSED_HEIGHT,
                )
            }
        }
        (NotchEdge::Top, false, true) => match panel {
            NotchPanel::Shortcuts => (TOP_SHORTCUTS_WIDTH, TOP_SHORTCUTS_HEIGHT),
            _ => (TOP_EXPANDED_WIDTH, TOP_EXPANDED_HEIGHT),
        },
        (NotchEdge::Left | NotchEdge::Right, false, false) => {
            if has_banner {
                (SIDE_BANNER_WIDTH, SIDE_BANNER_HEIGHT)
            } else {
                (
                    SIDE_COLLAPSED_WIDTH,
                    if has_alert {
                        SIDE_ALERT_HEIGHT
                    } else {
                        SIDE_COLLAPSED_HEIGHT
                    },
                )
            }
        }
        (NotchEdge::Left | NotchEdge::Right, false, true) => match panel {
            NotchPanel::Shortcuts => (TOP_SHORTCUTS_WIDTH, SIDE_EXPANDED_HEIGHT),
            _ => (SIDE_EXPANDED_WIDTH, SIDE_EXPANDED_HEIGHT),
        },
    }
}

fn point_in_rect(x: f32, y: f32, rect: Rect) -> bool {
    x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height
}

#[cfg(test)]
fn rect_center(rect: Rect) -> (f32, f32) {
    (rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
}

fn animated_visualizer_bars(phase: f32) -> [f32; 4] {
    [
        0.45 + 0.45 * (phase).sin().abs(),
        0.40 + 0.55 * (phase + 1.1).sin().abs(),
        0.35 + 0.55 * (phase + 2.2).sin().abs(),
        0.45 + 0.45 * (phase + 3.1).sin().abs(),
    ]
}

fn format_ms(ms: u64) -> String {
    let total_secs = ms / 1000;
    format!("{}:{:02}", total_secs / 60, total_secs % 60)
}

fn format_remaining_ms(pos_ms: u64, dur_ms: u64) -> String {
    let rem_secs = dur_ms.saturating_sub(pos_ms) / 1000;
    format!("-{}:{:02}", rem_secs / 60, rem_secs % 60)
}

fn current_local_date() -> (i32, u8, u8) {
    let st = unsafe { GetLocalTime() };
    let year = (st.wYear as i32).clamp(1970, 2999);
    let month = (st.wMonth as u8).clamp(1, 12);
    let day = (st.wDay as u8).clamp(1, days_in_month(year, month));
    (year, month, day)
}

pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30,
    }
}

/// Sakamoto's algorithm: returns `0 = Sunday .. 6 = Saturday` for day 1 of `(year, month)`.
pub fn first_weekday_sun0(year: i32, month: u8) -> u8 {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year;
    let m = month.clamp(1, 12) as usize;
    if m < 3 {
        y -= 1;
    }
    ((y + y / 4 - y / 100 + y / 400 + T[m - 1] + 1).rem_euclid(7)) as u8
}

fn month_name(month: u8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Month",
    }
}

/// Formats throughput in KB/s into a compact human-readable string.
pub fn format_net_speed(kbps: f32) -> String {
    let v = kbps.max(0.0);
    if v >= 1024.0 {
        format!("{:.1} MB/s", v / 1024.0)
    } else {
        format!("{:.0} KB/s", v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(notch: &mut NotchView) {
        for _ in 0..160 {
            notch.tick(1.0 / 60.0);
        }
    }

    #[test]
    fn hidden_mode_collapses_to_3px_flush_bar() {
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_hidden(true);
        settle(&mut notch);
        let top_rect = notch.computed_rect(1920.0, 1080.0);
        assert_eq!(top_rect.y, 0.0);
        assert!((top_rect.height - 3.0).abs() < 0.1);

        notch.set_edge(NotchEdge::Right);
        settle(&mut notch);
        let right_rect = notch.computed_rect(1920.0, 1080.0);
        assert!((right_rect.width - 3.0).abs() < 0.1);
        assert!((right_rect.x + right_rect.width - 1920.0).abs() < 0.1);
    }

    #[test]
    fn clock_formats_horizontal_for_top_and_vertical_for_sides() {
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_clock(9, 5);
        assert_eq!(notch.formatted_clock(), "09:05");

        notch.set_edge(NotchEdge::Left);
        assert_eq!(notch.formatted_clock(), "09\n05");
    }

    #[test]
    fn expanded_content_respects_safe_padding_24() {
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_expanded(true);
        settle(&mut notch);
        let outer = notch.computed_rect(800.0, 400.0);
        let inner = notch.content_rect(800.0, 400.0);
        assert_eq!(inner.x - outer.x, 24.0);
        assert_eq!(outer.width - inner.width, 48.0);
        assert!((outer.width - 400.0).abs() < 0.2);
        assert!((outer.height - 280.0).abs() < 0.2);
    }

    #[test]
    fn hime_ai_tab_is_hidden_when_ai_disabled_and_visible_when_enabled() {
        let mut notch = NotchView::new(NotchEdge::Top);
        assert!(!notch.available_panels().contains(&NotchPanel::HimeAi));

        notch.set_panel(NotchPanel::HimeAi);
        assert_eq!(notch.active_panel, NotchPanel::CommandCenter);

        notch.set_ai_enabled(true);
        assert!(notch.available_panels().contains(&NotchPanel::HimeAi));
        notch.set_panel(NotchPanel::HimeAi);
        assert_eq!(notch.active_panel, NotchPanel::HimeAi);

        notch.set_ai_enabled(false);
        assert_eq!(notch.active_panel, NotchPanel::CommandCenter);
    }

    #[test]
    fn command_center_hit_test_covers_tiles_toggles_and_sliders() {
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_panel(NotchPanel::CommandCenter);
        settle(&mut notch);

        let surf = Rect::new(0.0, 0.0, 560.0, 320.0);
        let outer = notch.computed_rect(surf.width, surf.height);
        let layout = notch.compute_expanded_layout(outer);
        let BodyLayout::CommandCenter(cc) = layout.body else {
            panic!("expected CommandCenter layout");
        };

        let (wx, wy) = rect_center(cc.wifi_tile);
        assert_eq!(notch.hit_test_click(wx, wy, surf), Some(NotchAction::ToggleWifi));

        let (wmx, wmy) = rect_center(cc.wifi_more_btn);
        assert_eq!(notch.hit_test_click(wmx, wmy, surf), Some(NotchAction::OpenWifiList));

        let (bx, by) = rect_center(cc.bluetooth_tile);
        assert_eq!(notch.hit_test_click(bx, by, surf), Some(NotchAction::ToggleBluetooth));

        let (dx, dy) = rect_center(cc.drag_badge);
        assert_eq!(notch.hit_test_click(dx, dy, surf), Some(NotchAction::ToggleFreeDrag));

        let (nx, ny) = rect_center(cc.notch_mode_tile);
        assert_eq!(notch.hit_test_click(nx, ny, surf), Some(NotchAction::CycleNotchMode));

        let (sx, sy) = rect_center(cc.settings_gear_btn);
        assert_eq!(notch.hit_test_click(sx, sy, surf), Some(NotchAction::OpenSettings));

        let (kx, ky) = rect_center(cc.shortcuts_tile);
        assert_eq!(notch.hit_test_click(kx, ky, surf), Some(NotchAction::OpenShortcuts));

        // 5 quick toggles
        let (qx, qy) = rect_center(cc.quick_toggles[0]);
        assert_eq!(notch.hit_test_click(qx, qy, surf), Some(NotchAction::ToggleDockMode));
        let (qx4, qy4) = rect_center(cc.quick_toggles[4]);
        assert_eq!(notch.hit_test_click(qx4, qy4, surf), Some(NotchAction::ToggleHimeAi));

        // Volume slider midpoint -> ~0.5
        let vx = cc.volume_track.x + cc.volume_track.width * 0.5;
        let vy = cc.volume_track.y + cc.volume_track.height * 0.5;
        match notch.hit_test_click(vx, vy, surf) {
            Some(NotchAction::SetVolume(v)) => assert!((v - 0.5).abs() < 0.02),
            other => panic!("unexpected volume hit: {other:?}"),
        }

        // Brightness slider 75%
        let brx = cc.brightness_track.x + cc.brightness_track.width * 0.75;
        let bry = cc.brightness_track.y + cc.brightness_track.height * 0.5;
        assert_eq!(
            notch.hit_test_click(brx, bry, surf),
            Some(NotchAction::SetBrightness(75))
        );
    }

    #[test]
    fn calendar_grid_and_pomodoro_timer_work_accurately() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2025, 2), 28);
        assert_eq!(first_weekday_sun0(2026, 11), 0); // Nov 1, 2026 is Sunday

        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_panel(NotchPanel::Calendar);
        settle(&mut notch);

        let surf = Rect::new(0.0, 0.0, 560.0, 320.0);
        let outer = notch.computed_rect(surf.width, surf.height);
        let layout = notch.compute_expanded_layout(outer);
        let BodyLayout::Calendar(cal) = layout.body else {
            panic!("expected Calendar layout");
        };

        let (px, py) = rect_center(cal.pomo_play_btn);
        let act = notch.hit_test_click(px, py, surf).unwrap();
        assert_eq!(act, NotchAction::PomodoroStartPause);
        notch.apply_action(&act);
        assert!(notch.pomodoro.running);

        notch.tick(2.0);
        assert_eq!(notch.pomodoro.remaining_secs, 25 * 60 - 2);

        let (rx, ry) = rect_center(cal.pomo_reset_btn);
        let rst = notch.hit_test_click(rx, ry, surf).unwrap();
        notch.apply_action(&rst);
        assert!(!notch.pomodoro.running);
        assert_eq!(notch.pomodoro.remaining_secs, 25 * 60);
    }

    #[test]
    fn music_and_wifi_panels_hit_test_controls() {
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_music(Some(MusicInfo {
            title: "Nightcall".into(),
            artist: "Kavinsky".into(),
            lyric_line: Some("I'm giving you a night call".into()),
            is_playing: true,
        }));
        notch.set_music_details(
            Some("OutRun".into()),
            30_000,
            120_000,
            vec!["Line 1".into(), "Line 2".into()],
            Some(0),
        );
        notch.set_panel(NotchPanel::Music);
        settle(&mut notch);

        let surf = Rect::new(0.0, 0.0, 560.0, 320.0);
        let outer = notch.computed_rect(surf.width, surf.height);
        let layout = notch.compute_expanded_layout(outer);
        let BodyLayout::Music(mus) = layout.body else {
            panic!("expected Music layout");
        };

        let (lx, ly) = rect_center(mus.lyrics_btn);
        assert_eq!(notch.hit_test_click(lx, ly, surf), Some(NotchAction::MusicToggleLyrics));
        let (ppx, ppy) = rect_center(mus.play_pause_btn);
        assert_eq!(notch.hit_test_click(ppx, ppy, surf), Some(NotchAction::MusicPlayPause));

        notch.set_wifi_status(
            true,
            Some("Hime-5G".into()),
            vec![WifiNetworkItem {
                ssid: "Hime-5G".into(),
                signal: 90,
                secured: true,
                connected: true,
            }],
        );
        notch.set_panel(NotchPanel::Wifi);
        settle(&mut notch);
        let outer_w = notch.computed_rect(surf.width, surf.height);
        let layout_w = notch.compute_expanded_layout(outer_w);
        let BodyLayout::Wifi(wf) = layout_w.body else {
            panic!("expected Wifi layout");
        };
        let (rx, ry) = rect_center(wf.network_rows[0].1);
        assert_eq!(
            notch.hit_test_click(rx, ry, surf),
            Some(NotchAction::WifiConnect("Hime-5G".into()))
        );
    }

    #[test]
    fn notch_view_renders_all_panels_on_native_surface() {
        let surface = match NativeSurface::new("NotchMenuTest", -2200, -2200, 560, 320) {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_ai_enabled(true);
        notch.set_music(Some(MusicInfo {
            title: "Starlight".into(),
            artist: "Hime".into(),
            lyric_line: Some("Shining across the night".into()),
            is_playing: true,
        }));
        notch.set_shelf_items(vec![ShelfSummaryItem {
            name: "notes.md".into(),
            kind: "DOC".into(),
            size_label: "12 KB".into(),
        }]);
        notch.set_wifi_status(
            true,
            Some("HimeNet".into()),
            vec![WifiNetworkItem {
                ssid: "HimeNet".into(),
                signal: 88,
                secured: true,
                connected: true,
            }],
        );

        let panels = [
            NotchPanel::Status,
            NotchPanel::CommandCenter,
            NotchPanel::Calendar,
            NotchPanel::Music,
            NotchPanel::Wifi,
            NotchPanel::Shortcuts,
            NotchPanel::Shelf,
            NotchPanel::Tasks,
            NotchPanel::Focus,
            NotchPanel::HimeAi,
        ];
        for panel in panels {
            notch.set_panel(panel);
            settle(&mut notch);
            surface.begin_draw();
            notch.render(&surface).expect("render panel should succeed");
            surface.end_draw_and_present().expect("present should succeed");
        }
    }

    #[test]
    fn tasks_focus_lrc_and_banners_integrate_with_notch_view() {
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_music_raw_lrc("[00:01.00]Line A\n[00:03.00]Line B\n[00:05.00]Line C");
        notch.update_music_position(3_100);
        assert_eq!(notch.music_active_lyric_idx, Some(1));

        notch.music_show_lyrics = true;
        notch.set_panel(NotchPanel::Music);
        settle(&mut notch);
        let surf = Rect::new(0.0, 0.0, 560.0, 320.0);
        let outer = notch.computed_rect(surf.width, surf.height);
        let layout = notch.compute_expanded_layout(outer);
        let BodyLayout::Music(mus) = layout.body else {
            panic!("expected Music layout");
        };
        let (mx, my) = rect_center(mus.offset_minus_btn);
        assert_eq!(
            notch.hit_test_click(mx, my, surf),
            Some(NotchAction::LyricsAdjustOffset(-200))
        );
        notch.apply_action(&NotchAction::LyricsAdjustOffset(-200));
        assert_eq!(notch.music_lyric_offset_ms, -200);
        assert_eq!(notch.music_active_lyric_idx, Some(0));

        // Tasks panel
        notch.tasks_ws.quick_add_draft = "Ship Rust shell".into();
        notch.set_panel(NotchPanel::Tasks);
        settle(&mut notch);
        let outer_t = notch.computed_rect(surf.width, surf.height);
        let layout_t = notch.compute_expanded_layout(outer_t);
        let BodyLayout::Tasks(tbody) = layout_t.body else {
            panic!("expected Tasks layout");
        };
        let tlayout = notch.tasks_ws.layout(tbody);
        let (ax, ay) = rect_center(tlayout.quick_add_submit_btn);
        let add_act = notch.hit_test_click(ax, ay, surf).expect("quick add hit");
        assert_eq!(add_act, NotchAction::TasksQuickAdd("Ship Rust shell".into()));
        notch.apply_action(&add_act);
        assert_eq!(notch.tasks_ws.tasks.len(), 1);

        // Focus panel
        notch.set_panel(NotchPanel::Focus);
        settle(&mut notch);
        let outer_f = notch.computed_rect(surf.width, surf.height);
        let layout_f = notch.compute_expanded_layout(outer_f);
        let BodyLayout::Focus(fbody) = layout_f.body else {
            panic!("expected Focus layout");
        };
        let flayout = notch.focus_ws.layout(fbody);
        let (sx, sy) = rect_center(flayout.start_btn.expect("start btn"));
        assert_eq!(notch.hit_test_click(sx, sy, surf), Some(NotchAction::FocusStart));
        notch.apply_action(&NotchAction::FocusStart);
        assert_eq!(notch.focus_ws.status, FocusSessionStatus::Active);

        // Collapsed HUD banner hit-test
        notch.set_expanded(false);
        notch.proactive_banner.show(
            "p-1",
            "focus_check",
            crate::notch_workspace::ProactiveLevel::Tanya,
            "Stay focused",
            "Ready to resume?",
            HimeExpression::Fokus,
            None,
        );
        notch.sync_targets();
        settle(&mut notch);
        let outer_b = notch.computed_rect(surf.width, surf.height);
        let b_area = notch.banner_area(outer_b);
        let p_layout = notch.proactive_banner.layout(b_area);
        let (pax, pay) = rect_center(p_layout.accept_btn);
        let p_act = notch.hit_test_click(pax, pay, surf).expect("accept hit");
        assert_eq!(p_act, NotchAction::ProactiveAccept("p-1".into()));
        notch.apply_action(&p_act);
        assert!(!notch.proactive_banner.active);
    }
}
