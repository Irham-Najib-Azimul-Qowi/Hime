//! Native 3-edge Dynamic Island / Notch surface (`NotchView`) rendered via Direct2D.
//!
//! Styled cleanly for lightweight performance:
//! - Plain pure black (`#000000`) contour with concave ears and zero border strokes,
//!   zero backdrop blur, and zero gradients.
//! - Monochrome label hierarchy (`TEXT_PRIMARY`..`TEXT_TERTIARY`) with restrained `SYSTEM_BLUE`
//!   (`#0A84FF`) reserved for active selections, toggles, and Shelf drop highlights.
//! - Crisp `SfSymbol` vector strokes across every control and Apple `Spring` (`EMERGE`/`RECEDE`/`SMOOTH`)
//!   transitions.

use serde::{Deserialize, Serialize};
use windows::core::Result;
use windows::Win32::System::SystemInformation::GetLocalTime;

use crate::animation::{Spring, SpringValue};
use crate::d2d::{
    EdgeSide, HimeExpression, NativeSurface, Rect, SfSymbol, TextAlign,
};
use crate::notch_workspace::{
    active_lyric_index, parse_lrc_lines, FocusHitAction, FocusSessionStatus, FocusWorkspaceState,
    LrcLine, ProactiveBannerAction, ProactiveBannerState, TaskTabFilter, TasksHitAction,
    TasksWorkspaceState, TranslateBannerAction, TranslateBannerState,
};
use crate::theme::{
    AdaptivePalette, Color, CORNER_RADIUS, SAFE_PADDING, TEXT_PRIMARY,
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
    pub drag_hover: bool,
    pub pointer_pos: Option<(f32, f32)>,
    pub pointer_pressed: bool,
    pub width_spring: SpringValue,
    pub height_spring: SpringValue,
    pub expand_spring: SpringValue,
    pub tab_spring: SpringValue,
    pub drag_hover_spring: SpringValue,
    pub hover_spring: SpringValue,
    pub press_spring: SpringValue,
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
        let (w, h) = target_dimensions(edge, false, false, NotchPanel::Status, false, false, false);
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
            drag_hover: false,
            pointer_pos: None,
            pointer_pressed: false,
            width_spring: SpringValue::from_spring(w, Spring::EMERGE),
            height_spring: SpringValue::from_spring(h, Spring::EMERGE),
            expand_spring: SpringValue::from_spring(0.0, Spring::EMERGE),
            tab_spring: SpringValue::from_spring(0.0, Spring::SMOOTH),
            drag_hover_spring: SpringValue::from_spring(0.0, Spring::INTERACTIVE),
            hover_spring: SpringValue::from_spring(0.0, Spring::INTERACTIVE),
            press_spring: SpringValue::from_spring(1.0, Spring::KNOB_SETTLE),
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
            self.drag_hover = false;
            self.drag_hover_spring.animate_to(0.0, Spring::RECEDE);
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
            self.sync_tab_spring();
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
        self.sync_tab_spring();
        self.sync_targets();
    }

    /// Updates external file drag-hover feedback on the Notch / Shelf drop zone.
    pub fn set_drag_hover(&mut self, hovering: bool) {
        if self.drag_hover == hovering {
            return;
        }
        self.drag_hover = hovering;
        if hovering {
            if self.is_hidden {
                self.is_hidden = false;
            }
            self.drag_hover_spring.launch_to(1.0, Spring::EMERGE, 1.0);
        } else {
            self.drag_hover_spring.animate_to(0.0, Spring::RECEDE);
        }
        self.sync_targets();
    }

    /// Handles dropped files cleanly in both AI and non-AI modes by revealing and focusing Shelf.
    pub fn handle_files_dropped(&mut self, added_count: usize) {
        self.drag_hover = false;
        self.drag_hover_spring.animate_to(0.0, Spring::RECEDE);
        if added_count > 0 {
            self.is_hidden = false;
            self.set_ncm_center_alert(Some(format!("+{added_count} file(s)")));
            self.set_panel(NotchPanel::Shelf);
        } else {
            self.sync_targets();
        }
    }

    /// Updates local pointer hover coordinates (in surface DIPs) for subtle glass hover states.
    pub fn set_pointer_hover(&mut self, pos: Option<(f32, f32)>) {
        self.pointer_pos = pos;
        let hovered = pos.is_some();
        self.hover_spring
            .animate_to(if hovered { 1.0 } else { 0.0 }, Spring::INTERACTIVE);
    }

    /// Updates pointer press state for tactile Apple button press micro-scale (`0.97`).
    pub fn set_pointer_pressed(&mut self, pressed: bool) {
        self.pointer_pressed = pressed;
        self.press_spring
            .animate_to(if pressed { 0.97 } else { 1.0 }, Spring::KNOB_SETTLE);
    }

    fn sync_tab_spring(&mut self) {
        let panels = self.available_panels();
        if let Some(idx) = panels.iter().position(|&p| p == self.active_panel) {
            self.tab_spring.animate_to(idx as f32, Spring::SMOOTH);
        }
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

    /// Advances size, expansion, tab glide, hover/drag-hover, Pomodoro timer, focus session,
    /// banners, and visualizer phase by `dt` seconds.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.width_spring.tick(dt);
        self.height_spring.tick(dt);
        self.expand_spring.tick(dt);
        self.tab_spring.tick(dt);
        self.drag_hover_spring.tick(dt);
        self.hover_spring.tick(dt);
        self.press_spring.tick(dt);
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
            || !self.tab_spring.is_settled()
            || !self.drag_hover_spring.is_settled()
            || !self.hover_spring.is_settled()
            || !self.press_spring.is_settled()
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

    fn is_hovered(&self, rect: Rect) -> bool {
        self.pointer_pos
            .is_some_and(|(x, y)| point_in_rect(x, y, rect))
    }

    fn is_pressed(&self, rect: Rect) -> bool {
        self.pointer_pressed && self.is_hovered(rect)
    }

    fn press_rect(&self, rect: Rect) -> Rect {
        if self.is_pressed(rect) {
            let scale = self.press_spring.value.clamp(0.95, 1.0);
            let dx = rect.width * (1.0 - scale) * 0.5;
            let dy = rect.height * (1.0 - scale) * 0.5;
            Rect::new(
                rect.x + dx,
                rect.y + dy,
                (rect.width - dx * 2.0).max(4.0),
                (rect.height - dy * 2.0).max(4.0),
            )
        } else {
            rect
        }
    }

    /// Renders the notch onto `surface` using a plain solid black `#000000` edge contour
    /// without dynamic color flipping, glass refraction, strokes, or gradients.
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

        let p = AdaptivePalette::dark_glass();
        let radius = if self.is_hidden { 1.5 } else { CORNER_RADIUS };
        let flares = !self.is_hidden;
        surface.set_edge_surface_region(rect, EAR_WIDTH, radius, edge_side, flares);
        surface.draw_edge_surface(
            rect,
            EAR_WIDTH,
            radius,
            edge_side,
            flares,
            Color::rgba(0.0, 0.0, 0.0, 1.0),
            None,
        )?;

        if self.is_hidden {
            return Ok(());
        }

        if self.is_expanded || self.expand_spring.value > 0.25 {
            return self.render_expanded_contents(surface, rect, &p);
        }

        if self.has_active_banner() && !self.drag_hover {
            let area = self.banner_area(rect);
            if self.translate_banner.active {
                return self.translate_banner.render(surface, area);
            }
            if self.proactive_banner.active {
                return self.proactive_banner.render(surface, area);
            }
        }

        match self.edge {
            NotchEdge::Top => self.render_top_collapsed(surface, rect, &p),
            NotchEdge::Left | NotchEdge::Right => self.render_side_collapsed(surface, rect, &p),
        }
    }

    fn render_adaptive_card(
        &self,
        surface: &NativeSurface,
        rect: Rect,
        radius: f32,
        p: &AdaptivePalette,
    ) -> Result<()> {
        surface.fill_rounded_rect(rect, radius, p.glass_elevated)
    }

    fn render_plain_button(
        &self,
        surface: &NativeSurface,
        rect: Rect,
        radius: f32,
        accent: Option<Color>,
        hovered: bool,
        pressed: bool,
        p: &AdaptivePalette,
    ) -> Result<()> {
        let r = radius.max(0.0).min(rect.width * 0.5).min(rect.height * 0.5);
        let fill = if let Some(acc) = accent {
            if pressed {
                acc.with_alpha(0.75)
            } else if hovered {
                acc
            } else {
                acc.with_alpha(0.88)
            }
        } else if pressed {
            p.control_active
        } else if hovered {
            p.control_hover
        } else {
            p.control_fill
        };
        surface.fill_rounded_rect(rect, r, fill)
    }

    fn render_plain_icon_hover(
        &self,
        surface: &NativeSurface,
        rect: Rect,
        radius: f32,
        hovered: bool,
        pressed: bool,
        active: bool,
        p: &AdaptivePalette,
    ) -> Result<()> {
        if (!hovered && !pressed && !active) || rect.width <= 0.0 || rect.height <= 0.0 {
            return Ok(());
        }
        let r = radius.max(0.0).min(rect.width * 0.5).min(rect.height * 0.5);
        let fill = if pressed || active {
            p.control_active
        } else {
            p.control_hover
        };
        surface.fill_rounded_rect(rect, r, fill)
    }

    fn render_adaptive_visualizer(
        &self,
        surface: &NativeSurface,
        x: f32,
        y: f32,
        h: f32,
        bars: &[f32],
        is_playing: bool,
        p: &AdaptivePalette,
    ) -> Result<()> {
        let bar_w = 2.5_f32;
        let gap = 2.0_f32;
        let color = if is_playing { p.accent } else { p.status_muted };
        for (i, &level) in bars.iter().enumerate() {
            let norm = if is_playing {
                level.clamp(0.18, 1.0)
            } else {
                0.22
            };
            let bh = (h * norm).max(3.0);
            let bx = x + (i as f32) * (bar_w + gap);
            let by = y + (h - bh) * 0.5;
            surface.fill_rounded_rect(Rect::new(bx, by, bar_w, bh), bar_w * 0.5, color)?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn render_adaptive_battery_icon(
        &self,
        surface: &NativeSurface,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        percent: u8,
        charging: bool,
        p: &AdaptivePalette,
    ) -> Result<()> {
        let bw = (w - 2.5).max(8.0);
        let bh = h.max(6.0);
        let pct = (percent.min(100) as f32) / 100.0;
        let fill_color = if charging {
            p.status_green
        } else if percent <= 20 {
            p.status_amber
        } else {
            p.text_primary
        };
        surface.stroke_rounded_rect(Rect::new(x, y, bw, bh), 2.5, p.text_secondary, 1.2)?;
        let nub_h = (bh * 0.45).max(3.0);
        surface.fill_rounded_rect(
            Rect::new(x + bw + 0.5, y + (bh - nub_h) * 0.5, 2.0, nub_h),
            0.8,
            p.text_secondary,
        )?;
        let pad = 2.0;
        let max_fill_w = (bw - pad * 2.0).max(1.0);
        let fill_w = (max_fill_w * pct).max(if percent > 0 { 2.0 } else { 0.0 });
        if fill_w > 0.0 {
            surface.fill_rounded_rect(
                Rect::new(x + pad, y + pad, fill_w, (bh - pad * 2.0).max(2.0)),
                1.2,
                fill_color,
            )?;
        }
        Ok(())
    }

    fn render_top_collapsed(
        &self,
        surface: &NativeSurface,
        outer: Rect,
        p: &AdaptivePalette,
    ) -> Result<()> {
        if self.drag_hover {
            let pill = Rect::new(
                outer.x + 22.0,
                outer.y + 5.0,
                (outer.width - 44.0).max(80.0),
                outer.height - 10.0,
            );
            surface.fill_rounded_rect(pill, 10.0, p.accent_tint)?;
            surface.draw_sf_symbol(
                Rect::new(pill.x + 14.0, pill.y + 5.0, 16.0, 16.0),
                SfSymbol::FolderDrop,
                p.text_primary,
                1.5,
            )?;
            surface.draw_text_bold(
                "Drop files to Shelf",
                Rect::new(pill.x + 34.0, pill.y, (pill.width - 44.0).max(40.0), pill.height),
                11.5,
                p.text_primary,
                TextAlign::Center,
            )?;
            return Ok(());
        }

        let is_playing = self.music.as_ref().is_some_and(|m| m.is_playing);
        let has_music = self.music.is_some();
        let has_alert = self.ncm_center_alert.is_some()
            || self.net.is_internet_down
            || self.expression != HimeExpression::Netral;

        // Left slot: Music visualizer when active, else SfSymbol::Wifi + monochrome speed
        if is_playing {
            let bars = animated_visualizer_bars(self.visualizer_phase);
            self.render_adaptive_visualizer(surface, outer.x + 22.0, outer.y + 10.0, 16.0, &bars, true, p)?;
        } else {
            let (wifi_sym, wifi_color) = if self.net.is_internet_down {
                (SfSymbol::WifiOff, p.status_red)
            } else if !self.wifi_enabled {
                (SfSymbol::WifiOff, p.text_tertiary)
            } else {
                (SfSymbol::Wifi, p.text_primary)
            };
            surface.draw_sf_symbol(
                Rect::new(outer.x + 20.0, outer.y + 11.5, 13.0, 13.0),
                wifi_sym,
                wifi_color,
                1.4,
            )?;
            if self.net.is_internet_down {
                surface.draw_text(
                    "Offline",
                    Rect::new(outer.x + 37.0, outer.y + 6.0, 68.0, 24.0),
                    10.5,
                    p.status_red,
                    TextAlign::Leading,
                )?;
            } else {
                surface.draw_sf_symbol(
                    Rect::new(outer.x + 36.0, outer.y + 13.5, 9.0, 9.0),
                    SfSymbol::ChevronDown,
                    p.text_secondary,
                    1.2,
                )?;
                let speed_rect = Rect::new(outer.x + 47.0, outer.y + 6.0, 62.0, 24.0);
                surface.draw_text(
                    &format_net_speed(self.net.down_kbps),
                    speed_rect,
                    10.5,
                    p.text_secondary,
                    TextAlign::Leading,
                )?;
            }
        }

        // Center slot: Pomodoro timer (when active), HimeEyes + alert, or crisp monochrome clock
        let center_rect = Rect::new(
            outer.x + 104.0,
            outer.y + 5.0,
            (outer.width - 208.0).max(52.0),
            26.0,
        );
        if self.pomodoro.running || self.pomodoro.finished {
            let timer_color = if self.pomodoro.finished {
                p.status_red
            } else {
                p.text_primary
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
                surface.draw_text(alert, msg_rect, 11.0, p.text_primary, TextAlign::Leading)?;
            }
        } else {
            surface.draw_text_bold(
                &self.formatted_clock(),
                center_rect,
                14.0,
                p.text_primary,
                TextAlign::Center,
            )?;
        }

        // Right slot: Album badge if music active, else Battery % + Battery icon
        if has_music && is_playing {
            let badge = Rect::new(outer.x + outer.width - 42.0, outer.y + 7.0, 22.0, 22.0);
            surface.fill_rounded_rect(badge, 7.0, p.control_active)?;
            surface.draw_sf_symbol(
                Rect::new(badge.x + 4.5, badge.y + 4.5, 13.0, 13.0),
                SfSymbol::MusicNote,
                p.text_primary,
                1.3,
            )?;
        } else if self.battery.present {
            let bat_text_rect = Rect::new(outer.x + outer.width - 88.0, outer.y + 6.0, 42.0, 24.0);
            surface.draw_text(
                &format!("{}%", self.battery.percent),
                bat_text_rect,
                11.0,
                p.text_secondary,
                TextAlign::Trailing,
            )?;
            self.render_adaptive_battery_icon(
                surface,
                outer.x + outer.width - 42.0,
                outer.y + 12.5,
                20.0,
                11.0,
                self.battery.percent,
                self.battery.charging,
                p,
            )?;
        }

        Ok(())
    }

    fn render_side_collapsed(
        &self,
        surface: &NativeSurface,
        outer: Rect,
        p: &AdaptivePalette,
    ) -> Result<()> {
        let cx = outer.x + outer.width * 0.5;
        if self.drag_hover {
            surface.fill_rounded_rect(
                Rect::new(cx - 16.0, outer.y + outer.height * 0.5 - 24.0, 32.0, 48.0),
                10.0,
                p.accent_tint,
            )?;
            surface.draw_sf_symbol(
                Rect::new(cx - 10.0, outer.y + outer.height * 0.5 - 10.0, 20.0, 20.0),
                SfSymbol::FolderDrop,
                p.text_primary,
                1.5,
            )?;
            return Ok(());
        }

        let has_alert = self.ncm_center_alert.is_some()
            || self.net.is_internet_down
            || self.expression != HimeExpression::Netral;

        let (wifi_sym, wifi_color) = if self.net.is_internet_down {
            (SfSymbol::WifiOff, p.status_red)
        } else if !self.wifi_enabled {
            (SfSymbol::WifiOff, p.text_tertiary)
        } else {
            (SfSymbol::Wifi, p.text_primary)
        };
        surface.draw_sf_symbol(
            Rect::new(cx - 6.0, outer.y + 18.0, 12.0, 12.0),
            wifi_sym,
            wifi_color,
            1.3,
        )?;
        if !self.net.is_internet_down {
            let net_rect = Rect::new(outer.x + 2.0, outer.y + 38.0, outer.width - 4.0, 20.0);
            surface.draw_text(
                &format_net_speed(self.net.down_kbps),
                net_rect,
                8.5,
                p.text_secondary,
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
                p.text_primary,
                TextAlign::Center,
            )?;
        }

        if self.battery.present {
            self.render_adaptive_battery_icon(
                surface,
                cx - 10.0,
                outer.y + outer.height - 46.0,
                20.0,
                10.0,
                self.battery.percent,
                self.battery.charging,
                p,
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
                p.text_secondary,
                TextAlign::Center,
            )?;
        }
        Ok(())
    }

    fn render_expanded_contents(
        &self,
        surface: &NativeSurface,
        outer: Rect,
        p: &AdaptivePalette,
    ) -> Result<()> {
        let layout = self.compute_expanded_layout(outer);

        // Header bar: [Prev] [Clock] [Next]  [Segmented Tabs] [Close]
        let prev_r = self.press_rect(layout.prev_btn);
        self.render_plain_icon_hover(
            surface,
            prev_r,
            7.0,
            self.is_hovered(layout.prev_btn),
            self.is_pressed(layout.prev_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(prev_r.x + 5.0, prev_r.y + 5.0, 12.0, 12.0),
            SfSymbol::ChevronLeft,
            p.text_primary,
            1.4,
        )?;

        let clock_display = format!("{:02}:{:02}", self.clock_hours, self.clock_minutes);
        surface.draw_text_bold(
            &clock_display,
            layout.header_clock,
            13.0,
            p.text_primary,
            TextAlign::Center,
        )?;

        let next_r = self.press_rect(layout.next_btn);
        self.render_plain_icon_hover(
            surface,
            next_r,
            7.0,
            self.is_hovered(layout.next_btn),
            self.is_pressed(layout.next_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(next_r.x + 5.0, next_r.y + 5.0, 12.0, 12.0),
            SfSymbol::ChevronRight,
            p.text_primary,
            1.4,
        )?;

        // Segmented tab strip + smooth spring-gliding active pill (single accent)
        if let (Some(first), Some(last)) = (layout.tabs.first(), layout.tabs.last()) {
            let strip = Rect::new(
                first.1.x - 2.0,
                first.1.y - 1.0,
                (last.1.x + last.1.width) - first.1.x + 4.0,
                first.1.height + 2.0,
            );
            surface.fill_rounded_rect(strip, 8.0, p.control_fill)?;

            let max_idx = layout.tabs.len().saturating_sub(1);
            let slide = self.tab_spring.value.clamp(0.0, max_idx as f32);
            let lo = (slide.floor() as usize).min(max_idx);
            let hi = (slide.ceil() as usize).min(max_idx);
            let frac = slide - lo as f32;
            let r_lo = layout.tabs[lo].1;
            let r_hi = layout.tabs[hi].1;
            let pill = Rect::new(
                r_lo.x + (r_hi.x - r_lo.x) * frac,
                r_lo.y,
                r_lo.width,
                r_lo.height,
            );
            surface.fill_rounded_rect(pill, 7.0, p.accent)?;
        }

        for (panel, tab_rect) in &layout.tabs {
            let active = *panel == self.active_panel;
            let hovered = !active && self.is_hovered(*tab_rect);
            if hovered {
                surface.fill_rounded_rect(*tab_rect, 7.0, p.control_hover)?;
            }
            let icon_color = if active {
                TEXT_PRIMARY
            } else if hovered {
                p.text_primary
            } else {
                p.text_secondary
            };
            let symbol = match panel {
                NotchPanel::CommandCenter => SfSymbol::SettingsSliders,
                NotchPanel::Calendar => SfSymbol::Calendar,
                NotchPanel::Music => SfSymbol::MusicNote,
                NotchPanel::Wifi => {
                    if self.wifi_enabled {
                        SfSymbol::Wifi
                    } else {
                        SfSymbol::WifiOff
                    }
                }
                NotchPanel::Shortcuts => SfSymbol::ShortcutKeys,
                NotchPanel::Shelf => SfSymbol::FolderDrop,
                NotchPanel::Tasks => SfSymbol::ListChecklist,
                NotchPanel::Focus => SfSymbol::FocusTimer,
                NotchPanel::HimeAi => SfSymbol::Sparkle,
                NotchPanel::Status => SfSymbol::SettingsSliders,
            };
            let icon_size = 12.0;
            let icon_rect = Rect::new(
                tab_rect.x + (tab_rect.width - icon_size) * 0.5,
                tab_rect.y + (tab_rect.height - icon_size) * 0.5,
                icon_size,
                icon_size,
            );
            surface.draw_sf_symbol(icon_rect, symbol, icon_color, 1.3)?;
        }

        // Collapse (X) button via SfSymbol::CloseX
        let close_r = self.press_rect(layout.close_btn);
        self.render_plain_icon_hover(
            surface,
            close_r,
            7.0,
            self.is_hovered(layout.close_btn),
            self.is_pressed(layout.close_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(close_r.x + 5.0, close_r.y + 5.0, 12.0, 12.0),
            SfSymbol::CloseX,
            p.text_secondary,
            1.4,
        )?;

        // Subtle hairline header divider
        let div_y = outer.y + HEADER_HEIGHT + 1.0;
        surface.draw_line(
            outer.x + SAFE_PADDING,
            div_y,
            outer.x + outer.width - SAFE_PADDING,
            div_y,
            p.divider,
            1.0,
        )?;

        // Body
        match &layout.body {
            BodyLayout::CommandCenter(cc) => self.render_command_center(surface, cc, p),
            BodyLayout::Calendar(cal) => self.render_calendar(surface, cal, p),
            BodyLayout::Music(mus) => self.render_music(surface, mus, p),
            BodyLayout::Wifi(wf) => self.render_wifi(surface, wf, p),
            BodyLayout::Shortcuts(sc) => self.render_shortcuts(surface, sc, p),
            BodyLayout::Shelf(sh) => self.render_shelf(surface, sh, p),
            BodyLayout::Tasks(body_rect) => self.tasks_ws.render(surface, *body_rect),
            BodyLayout::Focus(body_rect) => self.focus_ws.render(surface, *body_rect),
            BodyLayout::HimeAi(ai) => self.render_hime_ai(surface, ai, p),
        }
    }

    fn render_command_center(
        &self,
        surface: &NativeSurface,
        cc: &CommandCenterLayout,
        p: &AdaptivePalette,
    ) -> Result<()> {
        // 1. Wi-Fi Card Tile
        self.render_adaptive_card(surface, cc.wifi_tile, 12.0, p)?;
        let wifi_badge_cx = cc.wifi_tile.x + 17.0;
        let wifi_badge_cy = cc.wifi_tile.y + cc.wifi_tile.height * 0.5;
        surface.fill_circle(
            wifi_badge_cx,
            wifi_badge_cy,
            11.0,
            if self.wifi_enabled { p.accent } else { p.control_fill },
        )?;
        surface.draw_sf_symbol(
            Rect::new(wifi_badge_cx - 6.5, wifi_badge_cy - 6.5, 13.0, 13.0),
            if self.wifi_enabled { SfSymbol::Wifi } else { SfSymbol::WifiOff },
            if self.wifi_enabled { TEXT_PRIMARY } else { p.text_secondary },
            1.35,
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
            Rect::new(cc.wifi_tile.x + 34.0, cc.wifi_tile.y + 4.0, cc.wifi_tile.width - 60.0, 16.0),
            11.0,
            p.text_primary,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            wifi_sub,
            Rect::new(cc.wifi_tile.x + 34.0, cc.wifi_tile.y + 19.0, cc.wifi_tile.width - 60.0, 14.0),
            9.5,
            p.text_secondary,
            TextAlign::Leading,
        )?;
        let wifi_more = self.press_rect(cc.wifi_more_btn);
        self.render_plain_icon_hover(
            surface,
            wifi_more,
            6.0,
            self.is_hovered(cc.wifi_more_btn),
            self.is_pressed(cc.wifi_more_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(wifi_more.x + 3.5, wifi_more.y + 5.5, 11.0, 11.0),
            SfSymbol::ChevronRight,
            p.text_secondary,
            1.3,
        )?;

        // 2. Bluetooth Card Tile
        self.render_adaptive_card(surface, cc.bluetooth_tile, 12.0, p)?;
        let bt_cx = cc.bluetooth_tile.x + 17.0;
        let bt_cy = cc.bluetooth_tile.y + cc.bluetooth_tile.height * 0.5;
        surface.fill_circle(
            bt_cx,
            bt_cy,
            11.0,
            if self.bluetooth_enabled { p.accent } else { p.control_fill },
        )?;
        surface.draw_sf_symbol(
            Rect::new(bt_cx - 6.5, bt_cy - 6.5, 13.0, 13.0),
            SfSymbol::Bluetooth,
            if self.bluetooth_enabled { TEXT_PRIMARY } else { p.text_secondary },
            1.35,
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
            Rect::new(cc.bluetooth_tile.x + 34.0, cc.bluetooth_tile.y + 4.0, cc.bluetooth_tile.width - 42.0, 16.0),
            11.0,
            p.text_primary,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            bt_sub,
            Rect::new(cc.bluetooth_tile.x + 34.0, cc.bluetooth_tile.y + 19.0, cc.bluetooth_tile.width - 42.0, 14.0),
            9.5,
            p.text_secondary,
            TextAlign::Leading,
        )?;

        // 3. Notch Mode / Free Drag Tile
        self.render_adaptive_card(surface, cc.notch_mode_tile, 12.0, p)?;
        let mode_active = self.notch_mode_label == "Fixed";
        let nm_cx = cc.notch_mode_tile.x + 17.0;
        let nm_cy = cc.notch_mode_tile.y + cc.notch_mode_tile.height * 0.5;
        surface.fill_circle(
            nm_cx,
            nm_cy,
            11.0,
            if mode_active { p.control_active } else { p.control_fill },
        )?;
        surface.draw_sf_symbol(
            Rect::new(nm_cx - 6.5, nm_cy - 6.5, 13.0, 13.0),
            SfSymbol::Desktops,
            p.text_primary,
            1.3,
        )?;
        surface.draw_text_bold(
            "Notch Mode",
            Rect::new(cc.notch_mode_tile.x + 34.0, cc.notch_mode_tile.y + 4.0, 84.0, 16.0),
            11.0,
            p.text_primary,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            &self.notch_mode_label,
            Rect::new(cc.notch_mode_tile.x + 34.0, cc.notch_mode_tile.y + 19.0, 74.0, 14.0),
            9.5,
            p.text_secondary,
            TextAlign::Leading,
        )?;
        let drag_r = self.press_rect(cc.drag_badge);
        self.render_plain_button(
            surface,
            drag_r,
            7.0,
            if self.free_drag_enabled { Some(p.accent) } else { None },
            self.is_hovered(cc.drag_badge),
            self.is_pressed(cc.drag_badge),
            p,
        )?;
        surface.draw_text(
            if self.free_drag_enabled { "Drag" } else { "Lock" },
            drag_r,
            8.5,
            if self.free_drag_enabled { TEXT_PRIMARY } else { p.text_secondary },
            TextAlign::Center,
        )?;

        // 4. Shortcuts / Settings Tile
        self.render_adaptive_card(surface, cc.shortcuts_tile, 12.0, p)?;
        let sc_cx = cc.shortcuts_tile.x + 17.0;
        let sc_cy = cc.shortcuts_tile.y + cc.shortcuts_tile.height * 0.5;
        surface.fill_circle(sc_cx, sc_cy, 11.0, p.control_fill)?;
        surface.draw_sf_symbol(
            Rect::new(sc_cx - 6.5, sc_cy - 6.5, 13.0, 13.0),
            SfSymbol::ShortcutKeys,
            p.text_primary,
            1.3,
        )?;
        surface.draw_text_bold(
            "Shortcuts",
            Rect::new(cc.shortcuts_tile.x + 34.0, cc.shortcuts_tile.y + 4.0, cc.shortcuts_tile.width - 64.0, 16.0),
            11.0,
            p.text_primary,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            "F1 • Settings",
            Rect::new(cc.shortcuts_tile.x + 34.0, cc.shortcuts_tile.y + 19.0, cc.shortcuts_tile.width - 64.0, 14.0),
            9.5,
            p.text_secondary,
            TextAlign::Leading,
        )?;
        let gear_r = self.press_rect(cc.settings_gear_btn);
        self.render_plain_icon_hover(
            surface,
            gear_r,
            7.0,
            self.is_hovered(cc.settings_gear_btn),
            self.is_pressed(cc.settings_gear_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(gear_r.x + 4.0, gear_r.y + 6.0, 12.0, 12.0),
            SfSymbol::Gear,
            p.text_primary,
            1.3,
        )?;

        // 5. Circular Quick Toggles Row (Monochrome when off, single p.accent when active)
        let toggle_defs = [
            (
                if self.dock_fixed { "Dock Fix" } else { "Auto-Hide" },
                self.dock_fixed,
                SfSymbol::Pin,
            ),
            (
                "Vol HUD",
                self.volume_overlay_enabled,
                if self.volume_overlay_enabled {
                    SfSymbol::VolumeHigh
                } else {
                    SfSymbol::VolumeMuted
                },
            ),
            ("Bri HUD", self.brightness_overlay_enabled, SfSymbol::BrightnessSun),
            ("Quiet", self.notif_suppression_enabled, SfSymbol::Bell),
            ("Hime AI", self.ai_enabled, SfSymbol::Sparkle),
        ];
        for (idx, rect) in cc.quick_toggles.iter().enumerate() {
            let (label, active, symbol) = toggle_defs[idx];
            let cx = rect.x + rect.width * 0.5;
            let cy = rect.y + 17.0;
            let r = if self.is_pressed(*rect) { 15.8 } else { 16.5 };
            let hovered = self.is_hovered(*rect);
            let bg = if active {
                p.accent
            } else if hovered {
                p.control_hover
            } else {
                p.control_fill
            };
            surface.fill_circle(cx, cy, r, bg)?;
            surface.stroke_circle(
                cx,
                cy,
                r,
                if active { p.accent } else { p.hairline_border },
                1.0,
            )?;
            surface.draw_sf_symbol(
                Rect::new(cx - 7.0, cy - 7.0, 14.0, 14.0),
                symbol,
                if active { TEXT_PRIMARY } else { p.text_secondary },
                1.35,
            )?;
            surface.draw_text(
                label,
                Rect::new(rect.x - 6.0, rect.y + 35.0, rect.width + 12.0, 14.0),
                8.5,
                if active { p.text_primary } else { p.text_secondary },
                TextAlign::Center,
            )?;
        }

        // 6. Horizontal Monochrome Liquid Glass Volume & Brightness Sliders
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
            true,
            p,
        )?;
        self.render_slider_row(
            surface,
            cc.brightness_icon_btn,
            cc.brightness_track,
            cc.brightness_pct_rect,
            "Brightness",
            self.brightness_level as f32 / 100.0,
            false,
            p,
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
        is_volume: bool,
        p: &AdaptivePalette,
    ) -> Result<()> {
        let clamped = ratio.clamp(0.0, 1.0);
        let sym = if is_volume {
            if self.volume_muted || clamped <= 0.005 {
                SfSymbol::VolumeMuted
            } else {
                SfSymbol::VolumeHigh
            }
        } else {
            SfSymbol::BrightnessSun
        };
        let sym_rect = Rect::new(
            icon_rect.x + 2.0,
            icon_rect.y + (icon_rect.height - 13.0) * 0.5,
            13.0,
            13.0,
        );
        surface.draw_sf_symbol(sym_rect, sym, p.text_primary, 1.35)?;
        surface.draw_text(
            label,
            Rect::new(icon_rect.x + 20.0, icon_rect.y, 60.0, icon_rect.height),
            9.5,
            p.text_primary,
            TextAlign::Leading,
        )?;

        // Flat borderless track pill
        let bar_h = 8.0;
        let bar_y = track_rect.y + (track_rect.height - bar_h) * 0.5;
        let bar_rect = Rect::new(track_rect.x, bar_y, track_rect.width, bar_h);
        surface.fill_rounded_rect(bar_rect, 4.0, p.slider_track)?;

        let fill_w = (bar_rect.width * clamped).max(if clamped > 0.0 { 8.0 } else { 0.0 });
        if fill_w > 0.0 {
            surface.fill_rounded_rect(
                Rect::new(bar_rect.x, bar_rect.y, fill_w, bar_h),
                4.0,
                p.slider_fill,
            )?;
            let knob_x = (bar_rect.x + fill_w).min(bar_rect.x + bar_rect.width - 4.5);
            surface.fill_circle(knob_x, bar_rect.y + bar_h * 0.5, 4.5, p.text_primary)?;
        }

        let pct = (clamped * 100.0).round() as u8;
        surface.draw_text(
            &format!("{}%", pct),
            pct_rect,
            9.5,
            p.text_secondary,
            TextAlign::Trailing,
        )?;
        Ok(())
    }

    fn render_calendar(
        &self,
        surface: &NativeSurface,
        cal: &CalendarLayout,
        p: &AdaptivePalette,
    ) -> Result<()> {
        // Left column: Real month calendar grid
        let title = format!("{} {}", month_name(self.calendar_month), self.calendar_year);
        surface.draw_text_bold(&title, cal.month_title, 12.0, p.text_primary, TextAlign::Leading)?;

        let prev_m = self.press_rect(cal.prev_month_btn);
        self.render_plain_icon_hover(
            surface,
            prev_m,
            6.0,
            self.is_hovered(cal.prev_month_btn),
            self.is_pressed(cal.prev_month_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(prev_m.x + 4.0, prev_m.y + 4.0, 10.0, 10.0),
            SfSymbol::ChevronLeft,
            p.text_primary,
            1.3,
        )?;
        let next_m = self.press_rect(cal.next_month_btn);
        self.render_plain_icon_hover(
            surface,
            next_m,
            6.0,
            self.is_hovered(cal.next_month_btn),
            self.is_pressed(cal.next_month_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(next_m.x + 4.0, next_m.y + 4.0, 10.0, 10.0),
            SfSymbol::ChevronRight,
            p.text_primary,
            1.3,
        )?;
        let today_r = self.press_rect(cal.today_btn);
        self.render_plain_button(
            surface,
            today_r,
            6.0,
            None,
            self.is_hovered(cal.today_btn),
            self.is_pressed(cal.today_btn),
            p,
        )?;
        surface.draw_text("Today", today_r, 9.0, p.text_primary, TextAlign::Center)?;

        let weekdays = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        let cell_w = cal.grid_rect.width / 7.0;
        let row_h = (cal.grid_rect.height - 18.0) / 6.0;
        for (i, wd) in weekdays.iter().enumerate() {
            surface.draw_text(
                wd,
                Rect::new(cal.grid_rect.x + i as f32 * cell_w, cal.grid_rect.y, cell_w, 16.0),
                9.0,
                p.text_tertiary,
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
                surface.fill_circle(cx, cy, (row_h * 0.45).min(11.0), p.accent)?;
                surface.draw_text_bold(&day.to_string(), cell_rect, 10.0, TEXT_PRIMARY, TextAlign::Center)?;
            } else {
                surface.draw_text(&day.to_string(), cell_rect, 10.0, p.text_primary, TextAlign::Center)?;
            }
        }

        // Hairline vertical separator
        let sep_x = cal.grid_rect.x + cal.grid_rect.width + 6.0;
        surface.draw_line(
            sep_x,
            cal.grid_rect.y - 18.0,
            sep_x,
            cal.grid_rect.y + cal.grid_rect.height,
            p.divider,
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
            p.text_secondary,
            TextAlign::Center,
        )?;
        surface.draw_text_bold(
            &self.pomodoro.formatted(),
            Rect::new(cal.pomo_panel.x, cal.pomo_panel.y + 26.0, cal.pomo_panel.width, 40.0),
            26.0,
            if self.pomodoro.finished { p.status_red } else { p.text_primary },
            TextAlign::Center,
        )?;

        // Primary Start/Pause button + Reset (`SfSymbol::Refresh`)
        let play_r = self.press_rect(cal.pomo_play_btn);
        let play_primary = !self.pomodoro.running;
        self.render_plain_button(
            surface,
            play_r,
            14.0,
            if play_primary { Some(p.accent) } else { None },
            self.is_hovered(cal.pomo_play_btn),
            self.is_pressed(cal.pomo_play_btn),
            p,
        )?;
        let play_fg = if play_primary { TEXT_PRIMARY } else { p.text_primary };
        surface.draw_sf_symbol(
            Rect::new(play_r.x + 12.0, play_r.y + 9.0, 12.0, 12.0),
            if self.pomodoro.running { SfSymbol::Pause } else { SfSymbol::Play },
            play_fg,
            1.3,
        )?;
        surface.draw_text_bold(
            if self.pomodoro.running { "Pause" } else { "Start" },
            Rect::new(play_r.x + 22.0, play_r.y, (play_r.width - 26.0).max(24.0), play_r.height),
            11.0,
            play_fg,
            TextAlign::Center,
        )?;

        let reset_r = self.press_rect(cal.pomo_reset_btn);
        self.render_plain_icon_hover(
            surface,
            reset_r,
            14.0,
            self.is_hovered(cal.pomo_reset_btn),
            self.is_pressed(cal.pomo_reset_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(reset_r.x + 8.0, reset_r.y + 8.0, 14.0, 14.0),
            SfSymbol::Refresh,
            p.text_secondary,
            1.4,
        )?;

        // Preset buttons (5m, 15m, 25m, 50m)
        for (mins, rect) in &cal.preset_btns {
            let active = self.pomodoro.preset_mins == *mins;
            let btn_r = self.press_rect(*rect);
            self.render_plain_button(
                surface,
                btn_r,
                6.0,
                if active { Some(p.accent) } else { None },
                self.is_hovered(*rect),
                self.is_pressed(*rect),
                p,
            )?;
            surface.draw_text(
                &format!("{}m", mins),
                btn_r,
                9.5,
                if active { TEXT_PRIMARY } else { p.text_secondary },
                TextAlign::Center,
            )?;
        }

        Ok(())
    }

    fn render_music(
        &self,
        surface: &NativeSurface,
        mus: &MusicLayout,
        p: &AdaptivePalette,
    ) -> Result<()> {
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
        self.render_adaptive_card(surface, mus.album_badge, 11.0, p)?;
        surface.draw_sf_symbol(
            Rect::new(mus.album_badge.x + 13.0, mus.album_badge.y + 13.0, 20.0, 20.0),
            SfSymbol::MusicNote,
            p.text_primary,
            1.5,
        )?;
        if is_playing {
            surface.fill_circle(
                mus.album_badge.x + mus.album_badge.width - 7.0,
                mus.album_badge.y + 7.0,
                3.0,
                p.status_green,
            )?;
        }

        // Title + Artist + optional album
        surface.draw_text_bold(
            title,
            Rect::new(mus.title_rect.x, mus.title_rect.y, mus.title_rect.width, 18.0),
            13.0,
            p.text_primary,
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
            p.text_secondary,
            TextAlign::Leading,
        )?;

        // Visualizer + Lyrics toggle button
        let bars = animated_visualizer_bars(self.visualizer_phase);
        self.render_adaptive_visualizer(
            surface,
            mus.lyrics_btn.x - 26.0,
            mus.lyrics_btn.y + 4.0,
            16.0,
            &bars,
            is_playing,
            p,
        )?;
        let lyr_r = self.press_rect(mus.lyrics_btn);
        self.render_plain_button(
            surface,
            lyr_r,
            9.0,
            if self.music_show_lyrics { Some(p.accent) } else { None },
            self.is_hovered(mus.lyrics_btn),
            self.is_pressed(mus.lyrics_btn),
            p,
        )?;
        surface.draw_text_bold(
            "Lyrics",
            lyr_r,
            10.0,
            if self.music_show_lyrics { TEXT_PRIMARY } else { p.text_primary },
            TextAlign::Center,
        )?;

        // Monochrome Liquid Glass Progress bar
        let ratio = if self.music_duration_ms > 0 {
            (self.music_position_ms as f32 / self.music_duration_ms as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        surface.draw_text(
            &format_ms(self.music_position_ms),
            mus.elapsed_rect,
            9.5,
            p.text_secondary,
            TextAlign::Leading,
        )?;
        let bar_h = 5.0;
        let bar_y = mus.progress_track.y + (mus.progress_track.height - bar_h) * 0.5;
        let bar_rect = Rect::new(mus.progress_track.x, bar_y, mus.progress_track.width, bar_h);
        surface.fill_rounded_rect(bar_rect, 2.5, p.slider_track)?;
        if ratio > 0.0 {
            surface.fill_rounded_rect(
                Rect::new(bar_rect.x, bar_rect.y, (bar_rect.width * ratio).max(4.0), bar_h),
                2.5,
                p.slider_fill,
            )?;
        }
        surface.draw_text(
            &format_remaining_ms(self.music_position_ms, self.music_duration_ms),
            mus.remaining_rect,
            9.5,
            p.text_secondary,
            TextAlign::Trailing,
        )?;

        // Transport controls (`SfSymbol::SkipBack` / `Play` / `Pause` / `SkipForward`)
        let prev_r = self.press_rect(mus.prev_btn);
        self.render_plain_icon_hover(
            surface,
            prev_r,
            14.0,
            self.is_hovered(mus.prev_btn),
            self.is_pressed(mus.prev_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(prev_r.x + 7.5, prev_r.y + 7.5, 13.0, 13.0),
            SfSymbol::SkipBack,
            p.text_primary,
            1.4,
        )?;

        let pp_r = self.press_rect(mus.play_pause_btn);
        self.render_plain_icon_hover(
            surface,
            pp_r,
            18.0,
            self.is_hovered(mus.play_pause_btn),
            self.is_pressed(mus.play_pause_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(pp_r.x + 10.5, pp_r.y + 10.5, 15.0, 15.0),
            if is_playing { SfSymbol::Pause } else { SfSymbol::Play },
            p.text_primary,
            1.5,
        )?;

        let next_r = self.press_rect(mus.next_btn);
        self.render_plain_icon_hover(
            surface,
            next_r,
            14.0,
            self.is_hovered(mus.next_btn),
            self.is_pressed(mus.next_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(next_r.x + 7.5, next_r.y + 7.5, 13.0, 13.0),
            SfSymbol::SkipForward,
            p.text_primary,
            1.4,
        )?;

        // LRC Lyrics Area
        self.render_adaptive_card(surface, mus.lyrics_box, 10.0, p)?;

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
                    surface.draw_text_bold(line, rect, 11.0, p.text_primary, TextAlign::Center)?;
                } else {
                    surface.draw_text(line, rect, 9.5, p.text_tertiary, TextAlign::Center)?;
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
                    p.text_primary,
                    TextAlign::Center,
                )?;
                surface.draw_text(
                    &upcoming,
                    Rect::new(mus.lyrics_text_box.x + 10.0, mus.lyrics_text_box.y + 2.0 + half_h, mus.lyrics_text_box.width - 20.0, half_h),
                    9.5,
                    p.text_secondary,
                    TextAlign::Center,
                )?;
            } else {
                surface.draw_text_bold(&line, mus.lyrics_text_box, 11.0, p.text_primary, TextAlign::Center)?;
            }
        } else {
            let fallback = if self.music_show_lyrics {
                "Synced lyrics unavailable for this track"
            } else {
                "Tap Lyrics for synced LRC lyrics"
            };
            surface.draw_text(fallback, mus.lyrics_text_box, 10.0, p.text_secondary, TextAlign::Center)?;
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
                if self.music_lyric_offset_ms != 0 { p.accent } else { p.text_secondary },
                TextAlign::Leading,
            )?;

            let minus_r = self.press_rect(mus.offset_minus_btn);
            self.render_plain_icon_hover(
                surface,
                minus_r,
                5.0,
                self.is_hovered(mus.offset_minus_btn),
                self.is_pressed(mus.offset_minus_btn),
                false,
                p,
            )?;
            surface.draw_sf_symbol(
                Rect::new(minus_r.x + 4.0, minus_r.y + 4.0, 8.0, 8.0),
                SfSymbol::Minus,
                p.text_primary,
                1.2,
            )?;
            surface.draw_text(
                "0.2s",
                Rect::new(minus_r.x + 12.0, minus_r.y, (minus_r.width - 13.0).max(12.0), minus_r.height),
                8.0,
                p.text_primary,
                TextAlign::Center,
            )?;

            let plus_r = self.press_rect(mus.offset_plus_btn);
            self.render_plain_icon_hover(
                surface,
                plus_r,
                5.0,
                self.is_hovered(mus.offset_plus_btn),
                self.is_pressed(mus.offset_plus_btn),
                false,
                p,
            )?;
            surface.draw_sf_symbol(
                Rect::new(plus_r.x + 4.0, plus_r.y + 4.0, 8.0, 8.0),
                SfSymbol::Plus,
                p.text_primary,
                1.2,
            )?;
            surface.draw_text(
                "0.2s",
                Rect::new(plus_r.x + 12.0, plus_r.y, (plus_r.width - 13.0).max(12.0), plus_r.height),
                8.0,
                p.text_primary,
                TextAlign::Center,
            )?;

            let rst_r = self.press_rect(mus.offset_reset_btn);
            let rst_active = self.music_lyric_offset_ms != 0;
            self.render_plain_icon_hover(
                surface,
                rst_r,
                5.0,
                self.is_hovered(mus.offset_reset_btn),
                self.is_pressed(mus.offset_reset_btn),
                rst_active,
                p,
            )?;
            surface.draw_sf_symbol(
                Rect::new(rst_r.x + (rst_r.width - 10.0) * 0.5, rst_r.y + 3.0, 10.0, 10.0),
                SfSymbol::Refresh,
                if rst_active { p.accent } else { p.text_primary },
                1.2,
            )?;
        }

        Ok(())
    }

    fn render_wifi(
        &self,
        surface: &NativeSurface,
        wf: &WifiLayout,
        p: &AdaptivePalette,
    ) -> Result<()> {
        let back_r = self.press_rect(wf.back_btn);
        self.render_plain_button(
            surface,
            back_r,
            8.0,
            None,
            self.is_hovered(wf.back_btn),
            self.is_pressed(wf.back_btn),
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(back_r.x + 6.0, back_r.y + 6.0, 10.0, 10.0),
            SfSymbol::ChevronLeft,
            p.text_primary,
            1.3,
        )?;
        surface.draw_text(
            "Back",
            Rect::new(back_r.x + 16.0, back_r.y, back_r.width - 18.0, back_r.height),
            9.5,
            p.text_primary,
            TextAlign::Center,
        )?;

        surface.draw_text_bold(
            "Wi-Fi Networks",
            wf.title_rect,
            12.0,
            p.text_primary,
            TextAlign::Leading,
        )?;

        let toggle_r = self.press_rect(wf.toggle_btn);
        self.render_plain_button(
            surface,
            toggle_r,
            8.0,
            if self.wifi_enabled { Some(p.accent) } else { None },
            self.is_hovered(wf.toggle_btn),
            self.is_pressed(wf.toggle_btn),
            p,
        )?;
        surface.draw_text(
            if self.wifi_enabled { "On" } else { "Off" },
            toggle_r,
            9.5,
            if self.wifi_enabled { TEXT_PRIMARY } else { p.text_secondary },
            TextAlign::Center,
        )?;

        let ref_r = self.press_rect(wf.refresh_btn);
        self.render_plain_icon_hover(
            surface,
            ref_r,
            8.0,
            self.is_hovered(wf.refresh_btn),
            self.is_pressed(wf.refresh_btn),
            false,
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(ref_r.x + (ref_r.width - 12.0) * 0.5, ref_r.y + 5.0, 12.0, 12.0),
            SfSymbol::Refresh,
            p.text_primary,
            1.35,
        )?;

        if !self.wifi_enabled {
            self.render_adaptive_card(surface, wf.list_box, 10.0, p)?;
            surface.draw_text("Wi-Fi is turned off", wf.list_box, 11.0, p.text_secondary, TextAlign::Center)?;
            return Ok(());
        }

        if self.wifi_networks.is_empty() {
            self.render_adaptive_card(surface, wf.list_box, 10.0, p)?;
            let msg = if let Some(ssid) = &self.wifi_ssid {
                format!("Connected to {}", ssid)
            } else {
                "No nearby Wi-Fi networks found".to_string()
            };
            surface.draw_text(&msg, wf.list_box, 11.0, p.text_primary, TextAlign::Center)?;
            return Ok(());
        }

        for (idx, (ssid, row_rect)) in wf.network_rows.iter().enumerate() {
            let item = &self.wifi_networks[idx];
            if item.connected {
                surface.fill_rounded_rect(*row_rect, 9.0, p.accent_tint)?;
            } else {
                self.render_adaptive_card(surface, *row_rect, 9.0, p)?;
            }
            surface.draw_sf_symbol(
                Rect::new(row_rect.x + 10.0, row_rect.y + (row_rect.height - 12.0) * 0.5, 12.0, 12.0),
                if item.signal > 0 { SfSymbol::Wifi } else { SfSymbol::WifiOff },
                if item.connected { p.accent } else { p.text_primary },
                1.3,
            )?;
            surface.draw_text_bold(
                ssid,
                Rect::new(row_rect.x + 30.0, row_rect.y, row_rect.width - 116.0, row_rect.height),
                10.5,
                p.text_primary,
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
                if item.connected { p.accent_tint } else { p.control_fill },
            )?;
            if item.connected {
                surface.draw_sf_symbol(
                    Rect::new(action_rect.x + 6.0, action_rect.y + 7.0, 10.0, 10.0),
                    SfSymbol::Check,
                    p.accent,
                    1.3,
                )?;
                surface.draw_text(
                    "Joined",
                    Rect::new(action_rect.x + 16.0, action_rect.y, action_rect.width - 18.0, action_rect.height),
                    9.0,
                    p.text_primary,
                    TextAlign::Center,
                )?;
            } else {
                surface.draw_text(
                    "Connect",
                    action_rect,
                    9.0,
                    p.text_secondary,
                    TextAlign::Center,
                )?;
            }
        }

        Ok(())
    }

    fn render_shortcuts(
        &self,
        surface: &NativeSurface,
        sc: &ShortcutsLayout,
        p: &AdaptivePalette,
    ) -> Result<()> {
        surface.draw_sf_symbol(
            Rect::new(sc.header_rect.x + 2.0, sc.header_rect.y + 4.0, 14.0, 14.0),
            SfSymbol::ShortcutKeys,
            p.text_primary,
            1.3,
        )?;
        surface.draw_text_bold(
            "Keyboard Shortcuts",
            Rect::new(sc.header_rect.x + 22.0, sc.header_rect.y, sc.header_rect.width - 96.0, sc.header_rect.height),
            12.0,
            p.text_primary,
            TextAlign::Leading,
        )?;
        let set_r = self.press_rect(sc.settings_btn);
        self.render_plain_button(
            surface,
            set_r,
            7.0,
            None,
            self.is_hovered(sc.settings_btn),
            self.is_pressed(sc.settings_btn),
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(set_r.x + 6.0, set_r.y + 5.0, 10.0, 10.0),
            SfSymbol::Gear,
            p.text_secondary,
            1.2,
        )?;
        surface.draw_text(
            "Settings",
            Rect::new(set_r.x + 17.0, set_r.y, set_r.width - 19.0, set_r.height),
            9.0,
            p.text_primary,
            TextAlign::Center,
        )?;

        let left_items = [
            ("Notch", "F12 / Pause"),
            ("Assistant", "Ctrl+Shift+H"),
            ("Music & Lyrics", "Ctrl+Shift+M"),
            ("Dock", "Ctrl+Shift+D"),
            ("Shortcuts", "F1"),
            ("Close Modal", "Esc"),
        ];
        let right_items = [
            ("Long Chat", "/chat"),
            ("Send Prompt", "Enter"),
            ("New Line", "Shift+Enter"),
            ("Switch Tab", "Shift+Scroll"),
            ("Quit", "Ctrl+Shift+Q"),
            ("Emergency Pause", "Ctrl+Alt+P"),
        ];

        self.render_shortcut_column(surface, sc.left_col, "Display & Activation", &left_items, p)?;
        self.render_shortcut_column(surface, sc.right_col, "Navigation & System", &right_items, p)?;
        Ok(())
    }

    fn render_shortcut_column(
        &self,
        surface: &NativeSurface,
        col: Rect,
        heading: &str,
        items: &[(&str, &str)],
        p: &AdaptivePalette,
    ) -> Result<()> {
        surface.draw_text_bold(
            heading,
            Rect::new(col.x + 2.0, col.y, col.width - 4.0, 14.0),
            9.0,
            p.text_secondary,
            TextAlign::Leading,
        )?;
        let box_rect = Rect::new(col.x, col.y + 16.0, col.width, col.height - 16.0);
        self.render_adaptive_card(surface, box_rect, 10.0, p)?;

        let row_h = (box_rect.height - 8.0) / items.len() as f32;
        for (i, (label, keys)) in items.iter().enumerate() {
            let ry = box_rect.y + 4.0 + i as f32 * row_h;
            surface.draw_text(
                label,
                Rect::new(box_rect.x + 8.0, ry, box_rect.width * 0.48, row_h),
                9.5,
                p.text_primary,
                TextAlign::Leading,
            )?;
            let key_rect = Rect::new(
                box_rect.x + box_rect.width * 0.50,
                ry + 2.0,
                box_rect.width * 0.50 - 8.0,
                (row_h - 4.0).max(14.0),
            );
            surface.fill_rounded_rect(key_rect, 5.0, p.control_fill)?;
            surface.draw_text(keys, key_rect, 8.5, p.text_primary, TextAlign::Center)?;
        }
        Ok(())
    }

    fn render_shelf(
        &self,
        surface: &NativeSurface,
        sh: &ShelfLayout,
        p: &AdaptivePalette,
    ) -> Result<()> {
        surface.draw_sf_symbol(
            Rect::new(sh.header_rect.x, sh.header_rect.y + 4.0, 14.0, 14.0),
            SfSymbol::FolderDrop,
            if self.drag_hover { p.accent } else { p.text_primary },
            1.4,
        )?;
        surface.draw_text_bold(
            &format!("File Shelf ({})", self.shelf_items.len()),
            Rect::new(sh.header_rect.x + 20.0, sh.header_rect.y, (sh.header_rect.width - 20.0).max(60.0), sh.header_rect.height),
            12.0,
            p.text_primary,
            TextAlign::Leading,
        )?;
        if let Some(clear_btn) = sh.clear_btn {
            let clr_r = self.press_rect(clear_btn);
            self.render_plain_icon_hover(
                surface,
                clr_r,
                7.0,
                self.is_hovered(clear_btn),
                self.is_pressed(clear_btn),
                false,
                p,
            )?;
            surface.draw_sf_symbol(
                Rect::new(clr_r.x + 6.0, clr_r.y + 5.0, 11.0, 11.0),
                SfSymbol::Trash,
                p.text_secondary,
                1.3,
            )?;
            surface.draw_text(
                "Clear",
                Rect::new(clr_r.x + 17.0, clr_r.y, clr_r.width - 19.0, clr_r.height),
                9.5,
                p.text_secondary,
                TextAlign::Center,
            )?;
        }

        if self.shelf_items.is_empty() || self.drag_hover {
            if self.drag_hover {
                surface.fill_rounded_rect(sh.body_rect, 12.0, p.accent_tint)?;
            } else {
                self.render_adaptive_card(surface, sh.body_rect, 12.0, p)?;
            }
            let icon_size = 24.0;
            surface.draw_sf_symbol(
                Rect::new(
                    sh.body_rect.x + (sh.body_rect.width - icon_size) * 0.5,
                    sh.body_rect.y + sh.body_rect.height * 0.5 - 24.0,
                    icon_size,
                    icon_size,
                ),
                SfSymbol::FolderDrop,
                if self.drag_hover { p.accent } else { p.text_secondary },
                1.6,
            )?;
            let prompt = if self.drag_hover {
                "Release to add files to Shelf"
            } else {
                "Drop files onto the Notch to stash them here"
            };
            surface.draw_text(
                prompt,
                Rect::new(
                    sh.body_rect.x + 16.0,
                    sh.body_rect.y + sh.body_rect.height * 0.5 + 8.0,
                    sh.body_rect.width - 32.0,
                    24.0,
                ),
                11.0,
                if self.drag_hover { p.text_primary } else { p.text_secondary },
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
            self.render_adaptive_card(surface, row, 9.0, p)?;

            let kind_sym = match item.kind.as_str() {
                "DIR" => SfSymbol::FolderDrop,
                "AUD" => SfSymbol::MusicNote,
                _ => SfSymbol::Document,
            };
            let badge_rect = Rect::new(row.x + 8.0, row.y + 8.0, 22.0, 22.0);
            surface.fill_rounded_rect(badge_rect, 6.0, p.control_fill)?;
            surface.draw_sf_symbol(
                Rect::new(badge_rect.x + 5.0, badge_rect.y + 5.0, 12.0, 12.0),
                kind_sym,
                p.text_primary,
                1.3,
            )?;

            surface.draw_text_bold(
                &item.name,
                Rect::new(row.x + 38.0, row.y + 4.0, row.width - 116.0, row.height - 8.0),
                10.5,
                p.text_primary,
                TextAlign::Leading,
            )?;
            surface.draw_text(
                &item.size_label,
                Rect::new(row.x + row.width - 74.0, row.y + 4.0, 66.0, row.height - 8.0),
                9.5,
                p.text_secondary,
                TextAlign::Trailing,
            )?;
        }

        Ok(())
    }

    fn render_hime_ai(
        &self,
        surface: &NativeSurface,
        ai: &HimeAiLayout,
        p: &AdaptivePalette,
    ) -> Result<()> {
        self.render_adaptive_card(surface, ai.card_rect, 12.0, p)?;
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
            p.text_primary,
            TextAlign::Center,
        )?;
        surface.draw_text(
            &self.ai_status_text,
            Rect::new(ai.card_rect.x + 16.0, ai.card_rect.y + 98.0, ai.card_rect.width - 32.0, 20.0),
            10.5,
            p.text_secondary,
            TextAlign::Center,
        )?;
        let btn_r = self.press_rect(ai.open_drawer_btn);
        self.render_plain_button(
            surface,
            btn_r,
            10.0,
            Some(p.accent),
            self.is_hovered(ai.open_drawer_btn),
            self.is_pressed(ai.open_drawer_btn),
            p,
        )?;
        surface.draw_sf_symbol(
            Rect::new(btn_r.x + 12.0, btn_r.y + 9.0, 14.0, 14.0),
            SfSymbol::Sparkle,
            TEXT_PRIMARY,
            1.4,
        )?;
        surface.draw_text_bold(
            "Open Hime AI Drawer",
            Rect::new(btn_r.x + 26.0, btn_r.y, btn_r.width - 32.0, btn_r.height),
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
            self.drag_hover,
        );
        if self.is_hidden {
            self.width_spring.launch_to(w, Spring::RECEDE, 1.0);
            self.height_spring.launch_to(h, Spring::RECEDE, 1.0);
            self.expand_spring.launch_to(0.0, Spring::RECEDE, 1.0);
        } else if self.is_expanded {
            let was_collapsed = self.expand_spring.value < 0.2;
            if was_collapsed {
                self.width_spring.launch_to(w, Spring::EMERGE, 1.0);
                self.height_spring.launch_to(h, Spring::EMERGE, 1.0);
                self.expand_spring.launch_to(1.0, Spring::EMERGE, 1.0);
            } else {
                self.width_spring.animate_to(w, Spring::SMOOTH);
                self.height_spring.animate_to(h, Spring::SMOOTH);
                self.expand_spring.animate_to(1.0, Spring::SMOOTH);
            }
        } else {
            let was_expanded = self.expand_spring.value > 0.2;
            if was_expanded {
                self.width_spring.launch_to(w, Spring::RECEDE, 1.0);
                self.height_spring.launch_to(h, Spring::RECEDE, 1.0);
                self.expand_spring.launch_to(0.0, Spring::RECEDE, 1.0);
            } else {
                self.width_spring.animate_to(w, Spring::EMERGE);
                self.height_spring.animate_to(h, Spring::EMERGE);
                self.expand_spring.animate_to(0.0, Spring::RECEDE);
            }
        }
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
    drag_hover: bool,
) -> (f32, f32) {
    match (edge, is_hidden, is_expanded) {
        (NotchEdge::Top, true, _) => (48.0, HIDDEN_BAR_THICKNESS),
        (NotchEdge::Left | NotchEdge::Right, true, _) => (HIDDEN_BAR_THICKNESS, 48.0),
        (NotchEdge::Top, false, false) => {
            if has_banner && !drag_hover {
                (TOP_BANNER_WIDTH, TOP_BANNER_HEIGHT)
            } else {
                (
                    if has_alert || drag_hover {
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
            if has_banner && !drag_hover {
                (SIDE_BANNER_WIDTH, SIDE_BANNER_HEIGHT)
            } else {
                (
                    SIDE_COLLAPSED_WIDTH,
                    if has_alert || drag_hover {
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

    #[test]
    fn non_ai_shelf_drag_hover_and_drop_expands_shelf() {
        let mut notch = NotchView::new(NotchEdge::Top);
        assert!(!notch.ai_enabled);
        assert!(notch.available_panels().contains(&NotchPanel::Shelf));

        // Start hidden in Peek mode: dragging files wakes the Notch and shows Shelf drop highlight
        notch.set_hidden(true);
        settle(&mut notch);
        notch.set_drag_hover(true);
        assert!(!notch.is_hidden);
        assert!(notch.drag_hover);
        settle(&mut notch);
        let drag_rect = notch.computed_rect(1920.0, 1080.0);
        assert!((drag_rect.width - TOP_ALERT_WIDTH).abs() < 0.2);

        // Dropping files clears drag_hover and expands directly into Shelf even with ai_enabled == false
        notch.handle_files_dropped(2);
        assert!(!notch.drag_hover);
        assert!(notch.is_expanded);
        assert_eq!(notch.active_panel, NotchPanel::Shelf);
        settle(&mut notch);

        let surface = match NativeSurface::new("NotchShelfDndTest", -2200, -2200, 560, 320) {
            Ok(s) => s,
            Err(_) => return,
        };
        surface.begin_draw();
        notch.render(&surface).expect("Shelf panel should render in non-AI mode");
        surface.end_draw_and_present().expect("present should succeed");

        notch.set_drag_hover(true);
        settle(&mut notch);
        surface.begin_draw();
        notch.render(&surface).expect("Shelf drag highlight should render");
        surface.end_draw_and_present().expect("present should succeed");
    }

    #[test]
    fn hidden_peek_mode_renders_zero_pixels_and_empty_region() {
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_hidden(true);
        settle(&mut notch);
        assert!(notch.is_hidden);
        assert!(notch.width_spring.is_settled());
        assert!(notch.height_spring.is_settled());

        let surface = match NativeSurface::new("NotchHiddenPeekTest", -2200, -2200, 560, 320) {
            Ok(s) => s,
            Err(_) => return,
        };
        surface.begin_draw();
        notch.render(&surface).expect("hidden render should succeed with 0x0 region");
        surface.end_draw_and_present().expect("present should succeed");
    }

    #[test]
    fn apple_spring_presets_and_pointer_feedback_settle_cleanly() {
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_expanded(true);
        assert!(!notch.expand_spring.is_settled());
        settle(&mut notch);
        assert!((notch.expand_spring.value - 1.0).abs() < 0.05);

        notch.set_panel(NotchPanel::Shelf);
        notch.set_pointer_hover(Some((200.0, 20.0)));
        notch.set_pointer_pressed(true);
        settle(&mut notch);
        assert!((notch.press_spring.value - 0.97).abs() < 0.03);

        notch.set_pointer_pressed(false);
        notch.set_expanded(false);
        settle(&mut notch);
        assert!(notch.expand_spring.value.abs() < 0.05);
        assert!((notch.press_spring.value - 1.0).abs() < 0.03);
    }

    #[test]
    fn notch_renders_plain_black_consistently_regardless_of_backdrop() {
        use crate::theme::ColorThemeMode;

        let surface = match NativeSurface::new("NotchPlainBlackTest", -2200, -2200, 560, 320) {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_panel(NotchPanel::CommandCenter);
        settle(&mut notch);

        let mut dark_bg_opaque = vec![12_u8; 64 * 64 * 4];
        for px in dark_bg_opaque.chunks_exact_mut(4) {
            px[3] = 255;
        }
        surface.set_color_theme_mode(ColorThemeMode::Auto);
        surface.set_backdrop_pixels(64, 64, &dark_bg_opaque);
        surface.begin_draw();
        notch.render(&surface).expect("plain black render on dark bg should succeed");
        surface.end_draw_and_present().expect("plain black present should succeed");

        let bright_bg = vec![240_u8; 64 * 64 * 4];
        surface.set_backdrop_pixels(64, 64, &bright_bg);
        surface.begin_draw();
        notch.render(&surface).expect("plain black render on bright bg should succeed");
        surface.end_draw_and_present().expect("plain black present should succeed");
    }
}
