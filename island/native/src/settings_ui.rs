//! Interactive Native Settings Window rendered with Direct2D (`NativeSurface`).
//!
//! Styled after Apple macOS System Settings HIG & Liquid Glass:
//! - Translucent smoked obsidian Liquid Glass window (`740x520` px)
//! - Monochrome label hierarchy with single understated Apple tint (`#0A84FF`) for active toggles & tabs
//! - Crisp `SfSymbol` vector icons throughout (zero ASCII `"x"` / `"•"` / ad-hoc glyphs)
//! - Smooth analytic Apple Spring transitions for tab switching, toggles, hover states, and press micro-scale

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use windows::core::Result;
use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;
use windows::Win32::Graphics::Direct2D::D2D1_ANTIALIAS_MODE_PER_PRIMITIVE;

use crate::animation::{Spring, SpringValue};
use crate::d2d::{LiquidGlassStyle, NativeSurface, Rect, SfSymbol, TextAlign};
use crate::theme::Color;

pub const SETTINGS_WIDTH: f32 = 740.0;
pub const SETTINGS_HEIGHT: f32 = 520.0;
pub const SETTINGS_WINDOW_WIDTH: u32 = 740;
pub const SETTINGS_WINDOW_HEIGHT: u32 = 520;

const TITLEBAR_HEIGHT: f32 = 42.0;
const CONTENT_TOP: f32 = TITLEBAR_HEIGHT + 66.0;
const SIDEBAR_WIDTH: f32 = 180.0;
const CONTENT_PAD_X: f32 = 26.0;
const ROW_HEIGHT: f32 = 48.0;
const TAB_START_Y: f32 = TITLEBAR_HEIGHT + 68.0;
const TAB_HEIGHT: f32 = 36.0;
const TAB_STEP_Y: f32 = 42.0;
const PRESS_MICRO_SCALE: f32 = 0.968;

fn default_glass_opacity() -> f32 {
    0.78
}

fn clamp_glass_opacity(v: f32) -> f32 {
    if v.is_finite() {
        ((v.clamp(0.20, 1.0) * 100.0).round() / 100.0).clamp(0.20, 1.0)
    } else {
        default_glass_opacity()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsTab {
    General,
    Notch,
    Dock,
    Overlays,
    HimeAi,
    About,
}

impl Default for SettingsTab {
    fn default() -> Self {
        Self::General
    }
}

impl SettingsTab {
    pub const ALL: [Self; 6] = [
        Self::General,
        Self::Notch,
        Self::Dock,
        Self::Overlays,
        Self::HimeAi,
        Self::About,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Notch => "Notch",
            Self::Dock => "Dock",
            Self::Overlays => "Overlays",
            Self::HimeAi => "Hime AI",
            Self::About => "About",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Self::General => "The essentials for how Hime starts and runs.",
            Self::Notch => "Choose how the Dynamic Island appears and behaves.",
            Self::Dock => "Shape the floating taskbar replacement around your workflow.",
            Self::Overlays => "Keep quick volume and brightness controls at screen edges.",
            Self::HimeAi => "Hime AI assistant, 9Router gateway, and model configuration.",
            Self::About => "Version, config storage path, and open source notices.",
        }
    }

    pub fn symbol(self) -> SfSymbol {
        match self {
            Self::General => SfSymbol::Gear,
            Self::Notch => SfSymbol::SettingsSliders,
            Self::Dock => SfSymbol::TrayGrid,
            Self::Overlays => SfSymbol::VolumeHigh,
            Self::HimeAi => SfSymbol::Sparkle,
            Self::About => SfSymbol::ShieldCheck,
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotchMode {
    Top,
    Left,
    Right,
    Hidden,
}

impl Default for NotchMode {
    fn default() -> Self {
        Self::Top
    }
}

impl NotchMode {
    pub fn next(self) -> Self {
        match self {
            Self::Top => Self::Left,
            Self::Left => Self::Right,
            Self::Right => Self::Hidden,
            Self::Hidden => Self::Top,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Top => "Top",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Hidden => "Hidden (3px)",
        }
    }

    pub fn from_edge_and_behavior(edge: &str, behavior: &str) -> Self {
        if behavior == "peek" {
            return Self::Top;
        }
        match edge {
            "left" => Self::Left,
            "right" => Self::Right,
            _ => Self::Top,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderPreset {
    NineRouter,
    GeminiApi,
    OpenAi,
    Custom,
}

impl ProviderPreset {
    pub fn next(self) -> Self {
        match self {
            Self::NineRouter => Self::GeminiApi,
            Self::GeminiApi => Self::OpenAi,
            Self::OpenAi => Self::Custom,
            Self::Custom => Self::NineRouter,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NineRouter => "9Router (Local)",
            Self::GeminiApi => "Gemini API",
            Self::OpenAi => "OpenAI",
            Self::Custom => "Custom Compat",
        }
    }

    pub fn default_base_url(self) -> &'static str {
        match self {
            Self::NineRouter => "http://127.0.0.1:20128/v1",
            Self::GeminiApi => "https://generativelanguage.googleapis.com/v1beta",
            Self::OpenAi => "https://api.openai.com/v1",
            Self::Custom => "",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Self::NineRouter => "hime-combo",
            Self::GeminiApi => "gemini-2.5-flash",
            Self::OpenAi => "gpt-4o-mini",
            Self::Custom => "default",
        }
    }

    pub fn adapter_kind(self) -> &'static str {
        match self {
            Self::GeminiApi => "GeminiNative",
            Self::NineRouter | Self::OpenAi | Self::Custom => "OpenAiCompat",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiProfileConfig {
    pub id: String,
    pub name: String,
    pub preset: ProviderPreset,
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
}

impl ApiProfileConfig {
    pub fn from_preset(preset: ProviderPreset) -> Self {
        Self {
            id: match preset {
                ProviderPreset::NineRouter => "9router-local",
                ProviderPreset::GeminiApi => "gemini-native",
                ProviderPreset::OpenAi => "openai-direct",
                ProviderPreset::Custom => "custom-compat",
            }
            .to_string(),
            name: preset.label().to_string(),
            preset,
            base_url: preset.default_base_url().to_string(),
            model: preset.default_model().to_string(),
            has_api_key: false,
        }
    }

    pub fn adapter_kind_str(&self) -> &'static str {
        self.preset.adapter_kind()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SettingsState {
    pub notch_mode: NotchMode,
    /// `"fixed"` | `"smart"` | `"peek"`
    pub notch_behavior: String,
    /// `"top"` | `"left"` | `"right"`
    pub notch_edge: String,
    pub free_drag_enabled: bool,
    pub time_format_24h: bool,
    pub music_mode_enabled: bool,
    pub suppress_native_notifications: bool,

    pub dock_enabled: bool,
    /// `"fixed"` | `"auto-hide"` | `"smart"`
    pub dock_mode: String,
    pub dock_pinned: Vec<String>,
    pub dock_win_number_enabled: bool,
    pub dock_show_dividers: bool,
    pub dock_show_hime_ai: bool,
    pub dock_show_desktop: bool,
    pub dock_show_tray: bool,
    pub dock_show_notifications: bool,
    /// `"small"` | `"medium"` | `"large"`
    pub dock_size: String,
    pub dock_group_windows: bool,

    pub volume_overlay_enabled: bool,
    pub brightness_overlay_enabled: bool,
    /// `"auto"` | `"dark"` | `"light"`
    pub color_theme: String,
    /// `"clear-glass"` | `"dark-frosted"` | `"light-frosted"` | `"acrylic"` | `"auto"`
    pub glass_theme: String,
    /// Liquid Glass surface opacity (`0.20..=1.0`, default `0.78`).
    #[serde(default = "default_glass_opacity")]
    pub glass_opacity: f32,

    pub hime_ai_enabled: bool,
    pub hime_router_url: String,
    pub hime_router_model: String,
    pub translate_opt_in: bool,
    /// `"quiet"` | `"balanced"` | `"chatty"`
    pub proactive_budget: String,
    pub memory_count: usize,
    pub skills_count: usize,
    pub mcp_count: usize,

    pub router_9_detected: bool,
    pub active_profile: ApiProfileConfig,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            notch_mode: NotchMode::Top,
            notch_behavior: "peek".to_string(),
            notch_edge: "top".to_string(),
            free_drag_enabled: false,
            time_format_24h: true,
            music_mode_enabled: true,
            suppress_native_notifications: true,
            dock_enabled: true,
            dock_mode: "auto-hide".to_string(),
            dock_pinned: Vec::new(),
            dock_win_number_enabled: true,
            dock_show_dividers: true,
            dock_show_hime_ai: true,
            dock_show_desktop: true,
            dock_show_tray: true,
            dock_show_notifications: true,
            dock_size: "medium".to_string(),
            dock_group_windows: true,
            volume_overlay_enabled: true,
            brightness_overlay_enabled: true,
            color_theme: "auto".to_string(),
            glass_theme: "clear-glass".to_string(),
            glass_opacity: default_glass_opacity(),
            hime_ai_enabled: true,
            hime_router_url: "http://127.0.0.1:20128/v1".to_string(),
            hime_router_model: "hime-combo".to_string(),
            translate_opt_in: false,
            proactive_budget: "balanced".to_string(),
            memory_count: 0,
            skills_count: 0,
            mcp_count: 0,
            router_9_detected: false,
            active_profile: ApiProfileConfig::from_preset(ProviderPreset::NineRouter),
        }
    }
}

/// High-level actions emitted when the user interacts with the native Settings window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsAction {
    SelectTab(SettingsTab),
    UpdateSettings,
    ToggleDockMode,
    ToggleNotchMode,
    ToggleFreeDrag,
    CycleColorTheme,
    CycleGlassTheme,
    SetGlassOpacity(f32),
    ToggleHimeAi,
    ToggleTranslateOptIn,
    CycleProactiveBudget,
    ReloadSkills,
    ReloadMcp,
    RefreshMemory,
    TestAndSaveProfile,
    ResetToDefaults,
    Close,
}

/// Legacy single-panel controls kept for backwards compatibility with earlier wiring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsControl {
    CycleNotchMode,
    ToggleDock,
    ToggleDockWinNumber,
    ToggleNotifSuppression,
    ToggleHimeAi,
    CycleProviderPreset,
    TestAndSaveProfile,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RowControlId {
    CycleColorTheme,
    CycleGlassTheme,
    GlassOpacity,
    ToggleTimeFormat24h,
    ToggleNotifSuppression,
    ResetToDefaults,
    CycleNotchBehavior,
    CycleNotchEdge,
    ToggleFreeDrag,
    ToggleMusicMode,
    ToggleDockEnabled,
    CycleDockMode,
    CycleDockSize,
    ToggleDockGroupWindows,
    ToggleDockWinNumber,
    ToggleDockShowDividers,
    ToggleDockShowHimeAi,
    ToggleDockShowDesktop,
    ToggleDockShowTray,
    ToggleDockShowNotifications,
    ClearPinnedApps,
    ToggleVolumeOverlay,
    ToggleBrightnessOverlay,
    ToggleHimeAi,
    CycleProviderPreset,
    CycleRouterModel,
    TestAndSaveProfile,
    ToggleTranslateOptIn,
    CycleProactiveBudget,
    RefreshMemory,
    ReloadSkills,
    ReloadMcp,
    AboutInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HitTarget {
    Close,
    Tab(SettingsTab),
    Row(RowControlId),
    DropdownItem(RowControlId, usize),
}

struct RenderedRow {
    id: RowControlId,
    section: &'static str,
    title: &'static str,
    desc: String,
    kind: RowVisual,
}

enum RowVisual {
    Toggle(bool),
    Picker(String),
    Slider {
        value: f32,
        min: f32,
        max: f32,
    },
    Button {
        label: String,
        status_connected: Option<bool>,
    },
    SummaryCard {
        icon: SfSymbol,
        count: usize,
        action_label: &'static str,
    },
    Info(String),
}

/// Interactive Direct2D Settings window (`740x520` px) following Apple macOS Settings HIG.
pub struct SettingsPanel {
    pub state: SettingsState,
    pub visible: bool,
    pub active_tab: SettingsTab,
    pub selected_index: Option<usize>,
    hovered_target: Option<HitTarget>,
    pressed_target: Option<HitTarget>,
    open_dropdown: Option<RowControlId>,
    dragging_opacity: bool,
    status_notice: Option<String>,
    status_notice_user_triggered: bool,
    pub scroll_offset: f32,
    scroll_target: f32,
    open_spring: SpringValue,
    tab_indicator_y: SpringValue,
    content_fade: SpringValue,
    hover_progress: SpringValue,
    press_scale: SpringValue,
    toggle_springs: HashMap<RowControlId, SpringValue>,
    width: f32,
    height: f32,
}

impl SettingsPanel {
    pub fn new() -> Self {
        let mut panel = Self {
            state: SettingsState::default(),
            visible: false,
            active_tab: SettingsTab::General,
            selected_index: None,
            hovered_target: None,
            pressed_target: None,
            open_dropdown: None,
            dragging_opacity: false,
            status_notice: None,
            status_notice_user_triggered: false,
            scroll_offset: 0.0,
            scroll_target: 0.0,
            open_spring: SpringValue::emerge(0.0),
            tab_indicator_y: SpringValue::smooth(TAB_START_Y),
            content_fade: SpringValue::emerge(1.0),
            hover_progress: SpringValue::interactive(0.0),
            press_scale: SpringValue::knob_settle(1.0),
            toggle_springs: HashMap::new(),
            width: SETTINGS_WIDTH,
            height: SETTINGS_HEIGHT,
        };
        panel.snap_internal_springs();
        panel
    }

    pub fn open(&mut self) {
        self.visible = true;
        if self.selected_index.is_none() {
            self.selected_index = Some(0);
        }
        self.open_spring.snap_to(1.0);
        self.snap_internal_springs();
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.hovered_target = None;
        self.pressed_target = None;
        self.open_dropdown = None;
        self.dragging_opacity = false;
        self.open_spring.snap_to(0.0);
        self.snap_internal_springs();
    }

    pub fn toggle(&mut self) {
        if self.visible {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn select_tab(&mut self, tab: SettingsTab) {
        let changed = self.active_tab != tab;
        self.active_tab = tab;
        self.selected_index = Some(0);
        self.open_dropdown = None;
        self.dragging_opacity = false;
        if changed {
            self.scroll_offset = 0.0;
            self.scroll_target = 0.0;
            self.tab_indicator_y
                .animate_to(Self::tab_y(tab), Spring::SMOOTH);
            self.content_fade.snap_to(0.15);
            self.content_fade.launch_to(1.0, Spring::EMERGE, 1.0);
            self.sync_toggle_targets(true);
        }
    }

    pub fn set_status_notice(&mut self, msg: Option<String>) {
        self.set_status_notice_explicit(msg, false);
    }

    /// Sets a status notice, suppressing raw background IPC transport errors unless `user_triggered`.
    pub fn set_status_notice_explicit(&mut self, msg: Option<String>, user_triggered: bool) {
        if !user_triggered && msg.as_deref().is_some_and(is_background_ipc_error) {
            self.status_notice = None;
            self.status_notice_user_triggered = false;
            return;
        }
        self.status_notice = msg;
        self.status_notice_user_triggered = user_triggered;
    }

    fn visible_status_notice(&self) -> Option<&str> {
        self.status_notice
            .as_deref()
            .filter(|m| self.status_notice_user_triggered || !is_background_ipc_error(m))
    }

    /// Scrolls the active tab's content vertically, returning `true` if the tab is scrollable.
    pub fn handle_scroll(&mut self, dy: f32) -> bool {
        if !self.visible {
            return false;
        }
        self.open_dropdown = None;
        let max_s = self.max_scroll();
        if max_s <= 0.5 {
            self.scroll_offset = 0.0;
            self.scroll_target = 0.0;
            return false;
        }
        let step = if dy.abs() > 10.0 { dy * 0.35 } else { dy * 38.0 };
        self.scroll_target = (self.scroll_target - step).clamp(0.0, max_s);
        true
    }

    pub fn max_scroll(&self) -> f32 {
        let content_top = CONTENT_TOP;
        let content_bottom = self.content_bottom_y(self.height);
        let viewport_h = (content_bottom - content_top).max(40.0);
        let (_, total_h) = self.layout_rows_internal(0.0);
        (total_h - viewport_h).max(0.0)
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let was_settled = !self.is_animating();
        self.open_spring.tick(dt);
        self.tab_indicator_y.tick(dt);
        self.content_fade.tick(dt);
        self.hover_progress.tick(dt);
        self.press_scale.tick(dt);
        for spring in self.toggle_springs.values_mut() {
            spring.tick(dt);
        }
        let scroll_diff = self.scroll_target - self.scroll_offset;
        if scroll_diff.abs() > 0.25 {
            let blend = (1.0 - (-18.0 * dt.clamp(0.0, 0.1)).exp()).clamp(0.0, 1.0);
            self.scroll_offset += scroll_diff * blend;
        } else {
            self.scroll_offset = self.scroll_target;
        }
        if self.press_scale.is_settled() && self.press_scale.target >= 0.999 {
            self.pressed_target = None;
        }
        !was_settled || self.is_animating()
    }

    pub fn is_animating(&self) -> bool {
        if !self.open_spring.is_settled() {
            return true;
        }
        if !self.visible {
            return false;
        }
        !self.tab_indicator_y.is_settled()
            || !self.content_fade.is_settled()
            || !self.hover_progress.is_settled()
            || !self.press_scale.is_settled()
            || (self.scroll_target - self.scroll_offset).abs() > 0.25
            || self.toggle_springs.values().any(|s| !s.is_settled())
    }

    /// Updates hover highlight for mouse movement at `(x, y)` in DIP coordinates.
    pub fn set_hovered_at(&mut self, x: f32, y: f32) -> bool {
        let next = if self.visible {
            self.hit_test_target(x, y)
        } else {
            None
        };
        if next != self.hovered_target {
            self.hovered_target = next;
            let target_alpha = if next.is_some() { 1.0 } else { 0.0 };
            self.hover_progress.animate_to(target_alpha, Spring::INTERACTIVE);
            true
        } else {
            false
        }
    }

    /// Hit-tests `(x, y)` without mutating state and returns the corresponding [`SettingsAction`].
    pub fn hit_test_action(&self, x: f32, y: f32) -> Option<SettingsAction> {
        if !self.visible {
            return None;
        }
        match self.hit_test_target(x, y)? {
            HitTarget::Close => Some(SettingsAction::Close),
            HitTarget::Tab(tab) => Some(SettingsAction::SelectTab(tab)),
            HitTarget::Row(RowControlId::GlassOpacity) => {
                if let Some(row_rect) = self.row_rect_for(RowControlId::GlassOpacity) {
                    Some(SettingsAction::SetGlassOpacity(self.opacity_from_x(x, row_rect)))
                } else {
                    Some(SettingsAction::SetGlassOpacity(self.state.glass_opacity))
                }
            }
            HitTarget::Row(row) => Some(row_to_action(row)),
            HitTarget::DropdownItem(row, _) => Some(row_to_action(row)),
        }
    }

    /// Hit-tests a click at `(x, y)`, mutates `self.state` / `self.active_tab`, triggers
    /// press micro-scale feedback, and returns the [`SettingsAction`] for the host shell.
    pub fn handle_click(&mut self, x: f32, y: f32) -> Option<SettingsAction> {
        if !self.visible {
            return None;
        }
        let target = match self.hit_test_target(x, y) {
            Some(t) => t,
            None => {
                if self.open_dropdown.take().is_some() {
                    return Some(SettingsAction::UpdateSettings);
                }
                return None;
            }
        };
        self.trigger_press_feedback(target);
        match target {
            HitTarget::Close => {
                self.close();
                Some(SettingsAction::Close)
            }
            HitTarget::Tab(tab) => {
                self.select_tab(tab);
                Some(SettingsAction::SelectTab(tab))
            }
            HitTarget::DropdownItem(dd_id, option_idx) => {
                let action = self.apply_dropdown_option(dd_id, option_idx);
                self.open_dropdown = None;
                self.sync_toggle_targets(false);
                Some(action)
            }
            HitTarget::Row(RowControlId::GlassOpacity) => {
                self.open_dropdown = None;
                let row_rect = self.row_rect_for(RowControlId::GlassOpacity)?;
                let opacity = self.opacity_from_x(x, row_rect);
                self.state.glass_opacity = opacity;
                self.dragging_opacity = true;
                Some(SettingsAction::SetGlassOpacity(opacity))
            }
            HitTarget::Row(row) => {
                if !dropdown_options(row).is_empty() {
                    self.open_dropdown = if self.open_dropdown == Some(row) {
                        None
                    } else {
                        Some(row)
                    };
                    return Some(SettingsAction::UpdateSettings);
                }
                self.open_dropdown = None;
                let action = self.apply_row_control(row);
                self.sync_toggle_targets(false);
                Some(action)
            }
        }
    }

    /// Updates an active slider drag at `(x, y)`, returning [`SettingsAction::SetGlassOpacity`]
    /// when dragging the Glass Opacity slider.
    pub fn handle_drag(&mut self, x: f32, _y: f32) -> Option<SettingsAction> {
        if !self.visible || !self.dragging_opacity {
            return None;
        }
        let row_rect = self.row_rect_for(RowControlId::GlassOpacity)?;
        let opacity = self.opacity_from_x(x, row_rect);
        if (self.state.glass_opacity - opacity).abs() > 1e-4 {
            self.state.glass_opacity = opacity;
        }
        Some(SettingsAction::SetGlassOpacity(opacity))
    }

    /// Ends any active pointer drag (e.g. on `WM_LBUTTONUP`).
    pub fn handle_mouse_up(&mut self) {
        self.dragging_opacity = false;
    }

    /// Returns `true` while the user is actively dragging a slider control.
    pub fn is_dragging(&self) -> bool {
        self.dragging_opacity
    }

    /// Legacy row hit-test preserved for existing callers.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<SettingsControl> {
        if !self.visible {
            return None;
        }
        match self.hit_test_target(x, y)? {
            HitTarget::Close => Some(SettingsControl::Close),
            HitTarget::Tab(_) | HitTarget::DropdownItem(_, _) => None,
            HitTarget::Row(row) => match row {
                RowControlId::CycleNotchBehavior | RowControlId::CycleNotchEdge => {
                    Some(SettingsControl::CycleNotchMode)
                }
                RowControlId::ToggleDockEnabled | RowControlId::CycleDockMode => {
                    Some(SettingsControl::ToggleDock)
                }
                RowControlId::ToggleDockWinNumber => Some(SettingsControl::ToggleDockWinNumber),
                RowControlId::ToggleNotifSuppression => {
                    Some(SettingsControl::ToggleNotifSuppression)
                }
                RowControlId::ToggleHimeAi => Some(SettingsControl::ToggleHimeAi),
                RowControlId::CycleProviderPreset | RowControlId::CycleRouterModel => {
                    Some(SettingsControl::CycleProviderPreset)
                }
                RowControlId::TestAndSaveProfile => Some(SettingsControl::TestAndSaveProfile),
                _ => None,
            },
        }
    }

    /// Applies a legacy [`SettingsControl`] action to the local `SettingsState`.
    pub fn apply_control(&mut self, control: SettingsControl) {
        match control {
            SettingsControl::CycleNotchMode => {
                self.state.notch_mode = self.state.notch_mode.next();
                self.state.notch_edge = match self.state.notch_mode {
                    NotchMode::Left => "left".to_string(),
                    NotchMode::Right => "right".to_string(),
                    NotchMode::Top | NotchMode::Hidden => "top".to_string(),
                };
            }
            SettingsControl::ToggleDock => {
                self.state.dock_enabled = !self.state.dock_enabled;
            }
            SettingsControl::ToggleDockWinNumber => {
                self.state.dock_win_number_enabled = !self.state.dock_win_number_enabled;
            }
            SettingsControl::ToggleNotifSuppression => {
                self.state.suppress_native_notifications =
                    !self.state.suppress_native_notifications;
            }
            SettingsControl::ToggleHimeAi => {
                self.state.hime_ai_enabled = !self.state.hime_ai_enabled;
            }
            SettingsControl::CycleProviderPreset => {
                let next = self.state.active_profile.preset.next();
                self.state.active_profile = ApiProfileConfig::from_preset(next);
                self.state.hime_router_url = self.state.active_profile.base_url.clone();
                self.state.hime_router_model = self.state.active_profile.model.clone();
            }
            SettingsControl::TestAndSaveProfile => {}
            SettingsControl::Close => self.close(),
        }
        self.sync_toggle_targets(false);
    }

    fn apply_row_control(&mut self, row: RowControlId) -> SettingsAction {
        match row {
            RowControlId::CycleColorTheme => {
                self.state.color_theme = match self.state.color_theme.as_str() {
                    "auto" => "dark",
                    "dark" => "light",
                    _ => "auto",
                }
                .to_string();
                SettingsAction::CycleColorTheme
            }
            RowControlId::CycleGlassTheme => {
                self.state.glass_theme = match self.state.glass_theme.as_str() {
                    "clear-glass" => "dark-frosted",
                    "dark-frosted" => "light-frosted",
                    "light-frosted" => "acrylic",
                    "acrylic" => "auto",
                    _ => "clear-glass",
                }
                .to_string();
                SettingsAction::CycleGlassTheme
            }
            RowControlId::GlassOpacity => SettingsAction::SetGlassOpacity(self.state.glass_opacity),
            RowControlId::ToggleTimeFormat24h => {
                self.state.time_format_24h = !self.state.time_format_24h;
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleNotifSuppression => {
                self.state.suppress_native_notifications =
                    !self.state.suppress_native_notifications;
                SettingsAction::UpdateSettings
            }
            RowControlId::ResetToDefaults => {
                let router_detected = self.state.router_9_detected;
                self.state = SettingsState::default();
                self.state.router_9_detected = router_detected;
                self.status_notice = Some("Defaults restored".to_string());
                SettingsAction::ResetToDefaults
            }
            RowControlId::CycleNotchBehavior => {
                self.state.notch_behavior = match self.state.notch_behavior.as_str() {
                    "fixed" => "smart",
                    "smart" => "peek",
                    _ => "fixed",
                }
                .to_string();
                SettingsAction::ToggleNotchMode
            }
            RowControlId::CycleNotchEdge => {
                self.state.notch_edge = match self.state.notch_edge.as_str() {
                    "top" => "left",
                    "left" => "right",
                    _ => "top",
                }
                .to_string();
                self.state.notch_mode = NotchMode::from_edge_and_behavior(
                    &self.state.notch_edge,
                    &self.state.notch_behavior,
                );
                SettingsAction::ToggleNotchMode
            }
            RowControlId::ToggleFreeDrag => {
                self.state.free_drag_enabled = !self.state.free_drag_enabled;
                SettingsAction::ToggleFreeDrag
            }
            RowControlId::ToggleMusicMode => {
                self.state.music_mode_enabled = !self.state.music_mode_enabled;
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleDockEnabled => {
                self.state.dock_enabled = !self.state.dock_enabled;
                SettingsAction::ToggleDockMode
            }
            RowControlId::CycleDockMode => {
                self.state.dock_mode = match self.state.dock_mode.as_str() {
                    "fixed" => "auto-hide",
                    "auto-hide" => "smart",
                    _ => "fixed",
                }
                .to_string();
                SettingsAction::ToggleDockMode
            }
            RowControlId::CycleDockSize => {
                self.state.dock_size = match self.state.dock_size.as_str() {
                    "small" => "medium",
                    "medium" => "large",
                    _ => "small",
                }
                .to_string();
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleDockGroupWindows => {
                self.state.dock_group_windows = !self.state.dock_group_windows;
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleDockWinNumber => {
                self.state.dock_win_number_enabled = !self.state.dock_win_number_enabled;
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleDockShowDividers => {
                self.state.dock_show_dividers = !self.state.dock_show_dividers;
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleDockShowHimeAi => {
                self.state.dock_show_hime_ai = !self.state.dock_show_hime_ai;
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleDockShowDesktop => {
                self.state.dock_show_desktop = !self.state.dock_show_desktop;
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleDockShowTray => {
                self.state.dock_show_tray = !self.state.dock_show_tray;
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleDockShowNotifications => {
                self.state.dock_show_notifications = !self.state.dock_show_notifications;
                SettingsAction::UpdateSettings
            }
            RowControlId::ClearPinnedApps => {
                self.state.dock_pinned.clear();
                self.status_notice = Some("Pinned list cleared".to_string());
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleVolumeOverlay => {
                self.state.volume_overlay_enabled = !self.state.volume_overlay_enabled;
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleBrightnessOverlay => {
                self.state.brightness_overlay_enabled = !self.state.brightness_overlay_enabled;
                SettingsAction::UpdateSettings
            }
            RowControlId::ToggleHimeAi => {
                self.state.hime_ai_enabled = !self.state.hime_ai_enabled;
                SettingsAction::ToggleHimeAi
            }
            RowControlId::CycleProviderPreset => {
                let next = self.state.active_profile.preset.next();
                self.state.active_profile = ApiProfileConfig::from_preset(next);
                if !self.state.active_profile.base_url.is_empty() {
                    self.state.hime_router_url = self.state.active_profile.base_url.clone();
                }
                self.state.hime_router_model = self.state.active_profile.model.clone();
                SettingsAction::UpdateSettings
            }
            RowControlId::CycleRouterModel => {
                const MODELS: &[&str] = &[
                    "hime-combo",
                    "gpt-4o-mini",
                    "gemini-2.5-flash",
                    "claude-3-5-sonnet",
                ];
                let idx = MODELS
                    .iter()
                    .position(|m| *m == self.state.hime_router_model)
                    .map(|i| (i + 1) % MODELS.len())
                    .unwrap_or(0);
                self.state.hime_router_model = MODELS[idx].to_string();
                self.state.active_profile.model = self.state.hime_router_model.clone();
                SettingsAction::UpdateSettings
            }
            RowControlId::TestAndSaveProfile => {
                self.status_notice = Some("Saving & testing gateway...".to_string());
                SettingsAction::TestAndSaveProfile
            }
            RowControlId::ToggleTranslateOptIn => {
                self.state.translate_opt_in = !self.state.translate_opt_in;
                SettingsAction::ToggleTranslateOptIn
            }
            RowControlId::CycleProactiveBudget => {
                self.state.proactive_budget =
                    cycle_proactive_budget(&self.state.proactive_budget).to_string();
                SettingsAction::CycleProactiveBudget
            }
            RowControlId::RefreshMemory => {
                self.status_notice = Some("Refreshing memory summary...".to_string());
                SettingsAction::RefreshMemory
            }
            RowControlId::ReloadSkills => {
                self.status_notice = Some("Reloading skills...".to_string());
                SettingsAction::ReloadSkills
            }
            RowControlId::ReloadMcp => {
                self.status_notice = Some("Reloading MCP servers...".to_string());
                SettingsAction::ReloadMcp
            }
            RowControlId::AboutInfo => SettingsAction::UpdateSettings,
        }
    }

    pub fn handle_key_down(&mut self, vk: u32) -> Option<SettingsControl> {
        if !self.visible {
            return None;
        }
        const VK_RETURN: u32 = 0x0D;
        const VK_ESCAPE: u32 = 0x1B;
        const VK_SPACE: u32 = 0x20;
        const VK_LEFT: u32 = 0x25;
        const VK_UP: u32 = 0x26;
        const VK_RIGHT: u32 = 0x27;
        const VK_DOWN: u32 = 0x28;

        let rows = self.rows_for_active_tab();
        match vk {
            VK_ESCAPE => Some(SettingsControl::Close),
            VK_LEFT => {
                let idx = self.active_tab.index();
                let prev = if idx == 0 {
                    SettingsTab::ALL.len() - 1
                } else {
                    idx - 1
                };
                self.select_tab(SettingsTab::ALL[prev]);
                None
            }
            VK_RIGHT => {
                let idx = self.active_tab.index();
                self.select_tab(SettingsTab::ALL[(idx + 1) % SettingsTab::ALL.len()]);
                None
            }
            VK_UP if !rows.is_empty() => {
                let next = match self.selected_index {
                    Some(0) | None => rows.len() - 1,
                    Some(i) => i - 1,
                };
                self.selected_index = Some(next);
                None
            }
            VK_DOWN if !rows.is_empty() => {
                let next = match self.selected_index {
                    Some(i) => (i + 1) % rows.len(),
                    None => 0,
                };
                self.selected_index = Some(next);
                None
            }
            VK_RETURN | VK_SPACE => {
                let row = self.selected_index.and_then(|i| rows.get(i))?.id;
                self.apply_row_control(row);
                self.sync_toggle_targets(false);
                match row {
                    RowControlId::CycleNotchBehavior | RowControlId::CycleNotchEdge => {
                        Some(SettingsControl::CycleNotchMode)
                    }
                    RowControlId::ToggleDockEnabled | RowControlId::CycleDockMode => {
                        Some(SettingsControl::ToggleDock)
                    }
                    RowControlId::ToggleDockWinNumber => {
                        Some(SettingsControl::ToggleDockWinNumber)
                    }
                    RowControlId::ToggleNotifSuppression => {
                        Some(SettingsControl::ToggleNotifSuppression)
                    }
                    RowControlId::ToggleHimeAi => Some(SettingsControl::ToggleHimeAi),
                    RowControlId::CycleProviderPreset | RowControlId::CycleRouterModel => {
                        Some(SettingsControl::CycleProviderPreset)
                    }
                    RowControlId::TestAndSaveProfile => {
                        Some(SettingsControl::TestAndSaveProfile)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn handle_char(&mut self, ch: char) -> Option<SettingsControl> {
        if !self.visible {
            return None;
        }
        if self.active_tab == SettingsTab::HimeAi
            && self.state.active_profile.preset == ProviderPreset::Custom
        {
            match ch {
                '\x08' => {
                    self.state.hime_router_model.pop();
                    self.state.active_profile.model = self.state.hime_router_model.clone();
                }
                c if !c.is_control() => {
                    self.state.hime_router_model.push(c);
                    self.state.active_profile.model = self.state.hime_router_model.clone();
                }
                _ => {}
            }
        }
        None
    }

    fn trigger_press_feedback(&mut self, target: HitTarget) {
        self.pressed_target = Some(target);
        self.press_scale.snap_to(PRESS_MICRO_SCALE);
        self.press_scale.animate_to(1.0, Spring::KNOB_SETTLE);
    }

    fn snap_internal_springs(&mut self) {
        self.tab_indicator_y.snap_to(Self::tab_y(self.active_tab));
        self.content_fade.snap_to(1.0);
        self.hover_progress
            .snap_to(if self.hovered_target.is_some() { 1.0 } else { 0.0 });
        self.press_scale.snap_to(1.0);
        self.pressed_target = None;
        self.sync_toggle_targets(true);
    }

    fn sync_toggle_targets(&mut self, snap: bool) {
        let pairs = [
            (RowControlId::ToggleTimeFormat24h, self.state.time_format_24h),
            (
                RowControlId::ToggleNotifSuppression,
                self.state.suppress_native_notifications,
            ),
            (RowControlId::ToggleFreeDrag, self.state.free_drag_enabled),
            (RowControlId::ToggleMusicMode, self.state.music_mode_enabled),
            (RowControlId::ToggleDockEnabled, self.state.dock_enabled),
            (
                RowControlId::ToggleDockGroupWindows,
                self.state.dock_group_windows,
            ),
            (
                RowControlId::ToggleDockWinNumber,
                self.state.dock_win_number_enabled,
            ),
            (
                RowControlId::ToggleDockShowDividers,
                self.state.dock_show_dividers,
            ),
            (
                RowControlId::ToggleDockShowHimeAi,
                self.state.dock_show_hime_ai,
            ),
            (
                RowControlId::ToggleDockShowDesktop,
                self.state.dock_show_desktop,
            ),
            (
                RowControlId::ToggleDockShowTray,
                self.state.dock_show_tray,
            ),
            (
                RowControlId::ToggleDockShowNotifications,
                self.state.dock_show_notifications,
            ),
            (
                RowControlId::ToggleVolumeOverlay,
                self.state.volume_overlay_enabled,
            ),
            (
                RowControlId::ToggleBrightnessOverlay,
                self.state.brightness_overlay_enabled,
            ),
            (RowControlId::ToggleHimeAi, self.state.hime_ai_enabled),
            (
                RowControlId::ToggleTranslateOptIn,
                self.state.translate_opt_in,
            ),
        ];
        for (id, enabled) in pairs {
            let target = if enabled { 1.0 } else { 0.0 };
            let entry = self
                .toggle_springs
                .entry(id)
                .or_insert_with(|| SpringValue::knob_settle(target));
            if snap {
                entry.snap_to(target);
            } else {
                entry.animate_to(target, Spring::KNOB_SETTLE);
            }
        }
    }

    fn toggle_progress(&self, id: RowControlId, fallback: bool) -> f32 {
        self.toggle_springs
            .get(&id)
            .map(|s| s.value.clamp(0.0, 1.0))
            .unwrap_or(if fallback { 1.0 } else { 0.0 })
    }

    fn tab_y(tab: SettingsTab) -> f32 {
        TAB_START_Y + tab.index() as f32 * TAB_STEP_Y
    }

    fn close_rect(&self) -> Rect {
        Rect::new(self.width - 38.0, 9.0, 24.0, 24.0)
    }

    fn tab_rects(&self) -> Vec<(SettingsTab, Rect)> {
        let mut y = TAB_START_Y;
        SettingsTab::ALL
            .iter()
            .map(|&tab| {
                let r = Rect::new(10.0, y, SIDEBAR_WIDTH - 20.0, TAB_HEIGHT);
                y += TAB_STEP_Y;
                (tab, r)
            })
            .collect()
    }

    fn rows_for_active_tab(&self) -> Vec<RenderedRow> {
        match self.active_tab {
            SettingsTab::General => vec![
                RenderedRow {
                    id: RowControlId::CycleColorTheme,
                    section: "APPEARANCE",
                    title: "Color Theme",
                    desc: "Auto, Dark, or Light glass contrast".to_string(),
                    kind: RowVisual::Picker(capitalize_mode(&self.state.color_theme)),
                },
                RenderedRow {
                    id: RowControlId::CycleGlassTheme,
                    section: "APPEARANCE",
                    title: "Glass Blur Theme",
                    desc: "Clear Glass, Frosted, Acrylic, or Auto across shell".to_string(),
                    kind: RowVisual::Picker(glass_theme_label(&self.state.glass_theme).to_string()),
                },
                RenderedRow {
                    id: RowControlId::GlassOpacity,
                    section: "APPEARANCE",
                    title: "Glass Opacity",
                    desc: "Adjust Liquid Glass translucency (20% – 100%)".to_string(),
                    kind: RowVisual::Slider {
                        value: self.state.glass_opacity,
                        min: 0.20,
                        max: 1.0,
                    },
                },
                RenderedRow {
                    id: RowControlId::ToggleTimeFormat24h,
                    section: "SYSTEM",
                    title: "24-Hour Clock",
                    desc: "Use 24-hour format in the Notch bar".to_string(),
                    kind: RowVisual::Toggle(self.state.time_format_24h),
                },
                RenderedRow {
                    id: RowControlId::ToggleNotifSuppression,
                    section: "SYSTEM",
                    title: "Silence Native Toasts",
                    desc: "Route Windows alerts into the Notch".to_string(),
                    kind: RowVisual::Toggle(self.state.suppress_native_notifications),
                },
                RenderedRow {
                    id: RowControlId::ResetToDefaults,
                    section: "SYSTEM",
                    title: "Reset Settings",
                    desc: "Restore all preferences to defaults".to_string(),
                    kind: RowVisual::Button {
                        label: "Reset".to_string(),
                        status_connected: None,
                    },
                },
            ],
            SettingsTab::Notch => vec![
                RenderedRow {
                    id: RowControlId::CycleNotchBehavior,
                    section: "PLACEMENT & BEHAVIOR",
                    title: "Notch Behavior",
                    desc: "Fixed, Smart auto-hide, or Peek".to_string(),
                    kind: RowVisual::Picker(capitalize_mode(&self.state.notch_behavior)),
                },
                RenderedRow {
                    id: RowControlId::CycleNotchEdge,
                    section: "PLACEMENT & BEHAVIOR",
                    title: "Screen Edge",
                    desc: "Anchor Dynamic Island to Top, Left, or Right".to_string(),
                    kind: RowVisual::Picker(capitalize_mode(&self.state.notch_edge)),
                },
                RenderedRow {
                    id: RowControlId::ToggleFreeDrag,
                    section: "PLACEMENT & BEHAVIOR",
                    title: "Free Drag",
                    desc: "Drag the Notch freely across screen edges".to_string(),
                    kind: RowVisual::Toggle(self.state.free_drag_enabled),
                },
                RenderedRow {
                    id: RowControlId::ToggleMusicMode,
                    section: "MEDIA",
                    title: "Music Visualizer",
                    desc: "Show live media controls and spectrum".to_string(),
                    kind: RowVisual::Toggle(self.state.music_mode_enabled),
                },
            ],
            SettingsTab::Dock => vec![
                RenderedRow {
                    id: RowControlId::ToggleDockEnabled,
                    section: "BEHAVIOR & SIZE",
                    title: "Enable Dock",
                    desc: "Show the floating macOS-style Dock".to_string(),
                    kind: RowVisual::Toggle(self.state.dock_enabled),
                },
                RenderedRow {
                    id: RowControlId::CycleDockMode,
                    section: "BEHAVIOR & SIZE",
                    title: "Dock Behavior",
                    desc: "Fixed, Auto-Hide, or Smart overlap".to_string(),
                    kind: RowVisual::Picker(capitalize_mode(&self.state.dock_mode)),
                },
                RenderedRow {
                    id: RowControlId::CycleDockSize,
                    section: "BEHAVIOR & SIZE",
                    title: "Dock Size",
                    desc: "Small, Medium, or Large icon scale".to_string(),
                    kind: RowVisual::Picker(capitalize_mode(&self.state.dock_size)),
                },
                RenderedRow {
                    id: RowControlId::ToggleDockGroupWindows,
                    section: "WINDOWS & SHORTCUTS",
                    title: "Group Windows",
                    desc: "Combine multiple windows per app".to_string(),
                    kind: RowVisual::Toggle(self.state.dock_group_windows),
                },
                RenderedRow {
                    id: RowControlId::ToggleDockWinNumber,
                    section: "WINDOWS & SHORTCUTS",
                    title: "Win+1..9 Shortcuts",
                    desc: "Focus Dock apps with Win+number keys".to_string(),
                    kind: RowVisual::Toggle(self.state.dock_win_number_enabled),
                },
                RenderedRow {
                    id: RowControlId::ToggleDockShowDividers,
                    section: "UTILITY CLUSTER",
                    title: "Section Dividers",
                    desc: "Show hairline dividers between groups".to_string(),
                    kind: RowVisual::Toggle(self.state.dock_show_dividers),
                },
                RenderedRow {
                    id: RowControlId::ToggleDockShowHimeAi,
                    section: "UTILITY CLUSTER",
                    title: "Hime AI Button",
                    desc: "Show AI drawer launcher in Dock".to_string(),
                    kind: RowVisual::Toggle(self.state.dock_show_hime_ai),
                },
                RenderedRow {
                    id: RowControlId::ToggleDockShowDesktop,
                    section: "UTILITY CLUSTER",
                    title: "Spaces Switcher",
                    desc: "Show virtual desktop button in Dock".to_string(),
                    kind: RowVisual::Toggle(self.state.dock_show_desktop),
                },
                RenderedRow {
                    id: RowControlId::ToggleDockShowTray,
                    section: "UTILITY CLUSTER",
                    title: "System Tray",
                    desc: "Show hidden tray icons flyout".to_string(),
                    kind: RowVisual::Toggle(self.state.dock_show_tray),
                },
                RenderedRow {
                    id: RowControlId::ToggleDockShowNotifications,
                    section: "UTILITY CLUSTER",
                    title: "Notifications",
                    desc: "Show notification center button".to_string(),
                    kind: RowVisual::Toggle(self.state.dock_show_notifications),
                },
                RenderedRow {
                    id: RowControlId::ClearPinnedApps,
                    section: "PINNED APPS",
                    title: "Pinned Apps",
                    desc: format!("{} pinned app(s) saved", self.state.dock_pinned.len()),
                    kind: RowVisual::Button {
                        label: if self.state.dock_pinned.is_empty() {
                            "Empty".to_string()
                        } else {
                            "Clear Pinned".to_string()
                        },
                        status_connected: None,
                    },
                },
            ],
            SettingsTab::Overlays => vec![
                RenderedRow {
                    id: RowControlId::ToggleVolumeOverlay,
                    section: "EDGE HUDS",
                    title: "Volume HUD",
                    desc: "Interactive volume bar on left edge".to_string(),
                    kind: RowVisual::Toggle(self.state.volume_overlay_enabled),
                },
                RenderedRow {
                    id: RowControlId::ToggleBrightnessOverlay,
                    section: "EDGE HUDS",
                    title: "Brightness HUD",
                    desc: "Interactive brightness bar on right edge".to_string(),
                    kind: RowVisual::Toggle(self.state.brightness_overlay_enabled),
                },
                RenderedRow {
                    id: RowControlId::ToggleFreeDrag,
                    section: "EDGE HUDS",
                    title: "Draggable HUDs",
                    desc: "Allow dragging edge HUD panels".to_string(),
                    kind: RowVisual::Toggle(self.state.free_drag_enabled),
                },
            ],
            SettingsTab::HimeAi => vec![
                RenderedRow {
                    id: RowControlId::ToggleHimeAi,
                    section: "ASSISTANT & GATEWAY",
                    title: "Enable Hime AI",
                    desc: "Activate AI assistant and Notch tab".to_string(),
                    kind: RowVisual::Toggle(self.state.hime_ai_enabled),
                },
                RenderedRow {
                    id: RowControlId::CycleProviderPreset,
                    section: "ASSISTANT & GATEWAY",
                    title: "AI Provider",
                    desc: self.state.hime_router_url.clone(),
                    kind: RowVisual::Picker(self.state.active_profile.preset.label().to_string()),
                },
                RenderedRow {
                    id: RowControlId::CycleRouterModel,
                    section: "ASSISTANT & GATEWAY",
                    title: "Default Model",
                    desc: "Cycle active model for requests".to_string(),
                    kind: RowVisual::Picker(self.state.hime_router_model.clone()),
                },
                RenderedRow {
                    id: RowControlId::TestAndSaveProfile,
                    section: "ASSISTANT & GATEWAY",
                    title: "Gateway Status",
                    desc: if self.state.router_9_detected {
                        "Connected to 9Router on 127.0.0.1:20128".to_string()
                    } else {
                        "Test and save 9Router connection".to_string()
                    },
                    kind: RowVisual::Button {
                        label: if self.state.router_9_detected {
                            "Connected".to_string()
                        } else {
                            "Test & Save".to_string()
                        },
                        status_connected: Some(self.state.router_9_detected),
                    },
                },
                RenderedRow {
                    id: RowControlId::ToggleTranslateOptIn,
                    section: "INTELLIGENCE",
                    title: "Translate Selection",
                    desc: "Send highlighted text for quick translation".to_string(),
                    kind: RowVisual::Toggle(self.state.translate_opt_in),
                },
                RenderedRow {
                    id: RowControlId::CycleProactiveBudget,
                    section: "INTELLIGENCE",
                    title: "Proactive Budget",
                    desc: proactive_budget_desc(&self.state.proactive_budget).to_string(),
                    kind: RowVisual::Picker(
                        proactive_budget_label(&self.state.proactive_budget).to_string(),
                    ),
                },
                RenderedRow {
                    id: RowControlId::RefreshMemory,
                    section: "RUNTIME",
                    title: "Memory",
                    desc: format!("{} entries", self.state.memory_count),
                    kind: RowVisual::SummaryCard {
                        icon: SfSymbol::ListChecklist,
                        count: self.state.memory_count,
                        action_label: "Refresh",
                    },
                },
                RenderedRow {
                    id: RowControlId::ReloadSkills,
                    section: "RUNTIME",
                    title: "Skills",
                    desc: format!("{} loaded", self.state.skills_count),
                    kind: RowVisual::SummaryCard {
                        icon: SfSymbol::Sparkle,
                        count: self.state.skills_count,
                        action_label: "Reload",
                    },
                },
                RenderedRow {
                    id: RowControlId::ReloadMcp,
                    section: "RUNTIME",
                    title: "MCP Servers",
                    desc: format!("{} connected", self.state.mcp_count),
                    kind: RowVisual::SummaryCard {
                        icon: SfSymbol::Terminal,
                        count: self.state.mcp_count,
                        action_label: "Reload",
                    },
                },
            ],
            SettingsTab::About => vec![
                RenderedRow {
                    id: RowControlId::AboutInfo,
                    section: "APPLICATION",
                    title: "Hime Island Native",
                    desc: "Direct2D + DirectWrite + Liquid Glass shell".to_string(),
                    kind: RowVisual::Info("v1.0.0-rc.1".to_string()),
                },
                RenderedRow {
                    id: RowControlId::AboutInfo,
                    section: "APPLICATION",
                    title: "Settings File",
                    desc: "%APPDATA%\\ai.hime.desktop\\settings.json".to_string(),
                    kind: RowVisual::Info("JSON Sync".to_string()),
                },
                RenderedRow {
                    id: RowControlId::AboutInfo,
                    section: "APPLICATION",
                    title: "License",
                    desc: "GNU General Public License v3.0".to_string(),
                    kind: RowVisual::Info("GPL-3.0".to_string()),
                },
            ],
        }
    }

    fn content_bottom_y(&self, draw_h: f32) -> f32 {
        draw_h - if self.visible_status_notice().is_some() {
            34.0
        } else {
            14.0
        }
    }

    fn layout_rows_internal(&self, scroll: f32) -> (Vec<(&'static str, f32)>, f32) {
        let (_, sections, total_h) = self.layout_sections_and_rows(scroll);
        (sections, total_h)
    }

    fn layout_sections_and_rows(
        &self,
        scroll: f32,
    ) -> (Vec<(RowControlId, Rect)>, Vec<(&'static str, f32)>, f32) {
        let rows = self.rows_for_active_tab();
        let content_x = SIDEBAR_WIDTH + CONTENT_PAD_X;
        let row_w = self.width - content_x - CONTENT_PAD_X;
        let start_y = CONTENT_TOP;
        let row_gap = 8.0;
        let section_header_h = 20.0;
        let section_gap = 10.0;

        let mut y = start_y;
        let mut prev_section: Option<&'static str> = None;
        let mut sections = Vec::new();
        let mut rects = Vec::with_capacity(rows.len());

        for row in rows {
            if prev_section != Some(row.section) {
                if prev_section.is_some() {
                    y += section_gap;
                }
                sections.push((row.section, y - scroll));
                y += section_header_h;
                prev_section = Some(row.section);
            }
            rects.push((row.id, Rect::new(content_x, y - scroll, row_w, ROW_HEIGHT)));
            y += ROW_HEIGHT + row_gap;
        }

        let total_h = (y - start_y - row_gap).max(0.0);
        (rects, sections, total_h)
    }

    fn row_rects(&self) -> Vec<(RowControlId, Rect)> {
        self.layout_sections_and_rows(self.scroll_offset).0
    }

    fn row_rect_for(&self, id: RowControlId) -> Option<Rect> {
        self.row_rects()
            .into_iter()
            .find(|(rid, _)| *rid == id)
            .map(|(_, r)| r)
    }

    fn opacity_slider_track_rect(&self, row_rect: Rect) -> Rect {
        let track_w = 136.0_f32.min(row_rect.width * 0.32);
        let track_h = 6.0;
        let track_x = row_rect.x + row_rect.width - track_w - 16.0;
        let track_y = row_rect.y + (row_rect.height - track_h) * 0.5;
        Rect::new(track_x, track_y, track_w, track_h)
    }

    fn opacity_from_x(&self, x: f32, row_rect: Rect) -> f32 {
        let track = self.opacity_slider_track_rect(row_rect);
        let t = ((x - track.x) / track.width.max(1.0)).clamp(0.0, 1.0);
        clamp_glass_opacity(0.20 + t * 0.80)
    }

    fn dropdown_menu_rect(&self, id: RowControlId, row_rect: Rect) -> Option<Rect> {
        let opts = dropdown_options(id);
        if opts.is_empty() {
            return None;
        }
        let menu_w = 156.0_f32.min(row_rect.width * 0.38);
        let item_h = 28.0;
        let pad_y = 6.0;
        let menu_h = opts.len() as f32 * item_h + pad_y * 2.0;
        let menu_x = row_rect.x + row_rect.width - menu_w - 14.0;
        let below_y = row_rect.y + row_rect.height - 4.0;
        let menu_y = if below_y + menu_h <= self.height - 10.0 {
            below_y
        } else {
            (row_rect.y - menu_h + 4.0).max(CONTENT_TOP)
        };
        Some(Rect::new(menu_x, menu_y, menu_w, menu_h))
    }

    fn apply_dropdown_option(&mut self, id: RowControlId, option_idx: usize) -> SettingsAction {
        let opts = dropdown_options(id);
        let Some(&(val, _)) = opts.get(option_idx) else {
            return SettingsAction::UpdateSettings;
        };
        match id {
            RowControlId::CycleGlassTheme => {
                self.state.glass_theme = val.to_string();
                SettingsAction::CycleGlassTheme
            }
            RowControlId::CycleNotchBehavior => {
                self.state.notch_behavior = val.to_string();
                self.state.notch_mode = NotchMode::from_edge_and_behavior(
                    &self.state.notch_edge,
                    &self.state.notch_behavior,
                );
                SettingsAction::ToggleNotchMode
            }
            RowControlId::CycleNotchEdge => {
                self.state.notch_edge = val.to_string();
                self.state.notch_mode = NotchMode::from_edge_and_behavior(
                    &self.state.notch_edge,
                    &self.state.notch_behavior,
                );
                SettingsAction::ToggleNotchMode
            }
            RowControlId::CycleDockMode => {
                self.state.dock_mode = val.to_string();
                SettingsAction::ToggleDockMode
            }
            RowControlId::CycleDockSize => {
                self.state.dock_size = val.to_string();
                SettingsAction::UpdateSettings
            }
            _ => SettingsAction::UpdateSettings,
        }
    }

    fn dropdown_current_value(&self, id: RowControlId) -> &str {
        match id {
            RowControlId::CycleGlassTheme => &self.state.glass_theme,
            RowControlId::CycleNotchBehavior => &self.state.notch_behavior,
            RowControlId::CycleNotchEdge => &self.state.notch_edge,
            RowControlId::CycleDockMode => &self.state.dock_mode,
            RowControlId::CycleDockSize => &self.state.dock_size,
            _ => "",
        }
    }

    fn hit_test_target(&self, x: f32, y: f32) -> Option<HitTarget> {
        if let Some(dd_id) = self.open_dropdown {
            if let Some(row_rect) = self.row_rect_for(dd_id) {
                if let Some(menu_rect) = self.dropdown_menu_rect(dd_id, row_rect) {
                    if point_in_rect(x, y, menu_rect) {
                        let pad_y = 6.0;
                        let item_h = 28.0;
                        let opts = dropdown_options(dd_id);
                        let rel_y = y - (menu_rect.y + pad_y);
                        if rel_y >= 0.0 {
                            let idx = (rel_y / item_h) as usize;
                            if idx < opts.len() {
                                return Some(HitTarget::DropdownItem(dd_id, idx));
                            }
                        }
                    }
                }
            }
        }
        if point_in_rect(x, y, self.close_rect()) {
            return Some(HitTarget::Close);
        }
        for (tab, rect) in self.tab_rects() {
            if point_in_rect(x, y, rect) {
                return Some(HitTarget::Tab(tab));
            }
        }
        let content_top = CONTENT_TOP;
        let content_bottom = self.content_bottom_y(self.height);
        if y >= content_top && y < content_bottom {
            for (row_id, rect) in self.row_rects() {
                if rect.y + rect.height > content_top
                    && rect.y < content_bottom
                    && point_in_rect(x, y, rect)
                {
                    return Some(HitTarget::Row(row_id));
                }
            }
        }
        None
    }

    fn render_size_for_surface(&self, sw: f32, sh: f32) -> (f32, f32) {
        let draw_w = if sw > 10.0 {
            sw.min(self.width)
        } else {
            self.width
        };
        let draw_h = if sh > 10.0 {
            sh.min(self.height)
        } else {
            self.height
        };
        (draw_w, draw_h)
    }

    fn target_scale(&self, target: HitTarget) -> f32 {
        if self.pressed_target == Some(target) {
            self.press_scale.value.clamp(0.94, 1.0)
        } else {
            1.0
        }
    }

    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        let (sw, sh) = surface.dip_size();
        let (draw_w, draw_h) = self.render_size_for_surface(sw, sh);
        surface.set_glass_opacity(self.state.glass_opacity);
        let palette = surface.palette();

        let panel_rect = Rect::new(0.0, 0.0, draw_w, draw_h);
        surface.set_rounded_region(panel_rect, 16.0);
        surface.draw_liquid_glass_panel(
            panel_rect,
            glass_style_for_theme(&self.state.glass_theme, 16.0).without_shadow(),
        )?;

        // Subtle macOS window titlebar reflecting active pane
        surface.draw_text(
            "Settings",
            Rect::new(18.0, 8.0, 120.0, 26.0),
            12.0,
            palette.text_tertiary,
            TextAlign::Leading,
        )?;

        // Crisp vector CloseX button (frameless at rest, borderless pill on hover/press)
        let close_base = Rect::new(draw_w - 38.0, 9.0, 24.0, 24.0);
        let close_r = scale_rect_centered(close_base, self.target_scale(HitTarget::Close));
        let close_hovered = self.hovered_target == Some(HitTarget::Close);
        let close_pressed = self.pressed_target == Some(HitTarget::Close);
        surface.draw_icon_button(close_r, 12.0, close_hovered, close_pressed, false)?;
        surface.draw_sf_symbol(
            inset_rect(close_r, 6.0),
            SfSymbol::CloseX,
            if close_hovered {
                palette.text_primary
            } else {
                palette.text_secondary
            },
            1.5,
        )?;

        if draw_h <= TITLEBAR_HEIGHT + 20.0 {
            return Ok(());
        }

        // Sidebar frosted/smoked glass pane & hairline dividers
        let sidebar_bg = Rect::new(
            1.0,
            TITLEBAR_HEIGHT,
            SIDEBAR_WIDTH - 1.0,
            draw_h - TITLEBAR_HEIGHT - 1.0,
        );
        surface.fill_rounded_rect(sidebar_bg, 0.0, palette.glass_clear.with_alpha_mul(0.45))?;
        surface.fill_rounded_rect(
            Rect::new(SIDEBAR_WIDTH, 1.0, 1.0, draw_h - 2.0),
            0.0,
            palette.divider,
        )?;
        surface.fill_rounded_rect(
            Rect::new(SIDEBAR_WIDTH + 1.0, TITLEBAR_HEIGHT, draw_w - SIDEBAR_WIDTH - 2.0, 1.0),
            0.0,
            palette.divider,
        )?;

        // Sidebar app badge & title
        let badge_rect = Rect::new(18.0, TITLEBAR_HEIGHT + 13.0, 28.0, 28.0);
        surface.draw_liquid_glass_panel(badge_rect, LiquidGlassStyle::elevated(8.0))?;
        surface.draw_sf_symbol(
            inset_rect(badge_rect, 6.0),
            SfSymbol::Sparkle,
            palette.text_primary,
            1.4,
        )?;
        surface.draw_text_bold(
            "Hime",
            Rect::new(54.0, TITLEBAR_HEIGHT + 10.0, SIDEBAR_WIDTH - 64.0, 20.0),
            14.0,
            palette.text_primary,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            "System Shell",
            Rect::new(54.0, TITLEBAR_HEIGHT + 29.0, SIDEBAR_WIDTH - 64.0, 16.0),
            10.5,
            palette.text_secondary,
            TextAlign::Leading,
        )?;

        // Animated active tab indicator pill sliding smoothly with Spring::SMOOTH (neutral frosted glass)
        let ind_y = self.tab_indicator_y.value;
        if ind_y + TAB_HEIGHT <= draw_h - 8.0 {
            let ind_rect = scale_rect_centered(
                Rect::new(10.0, ind_y, SIDEBAR_WIDTH - 20.0, TAB_HEIGHT),
                self.target_scale(HitTarget::Tab(self.active_tab)),
            );
            surface.fill_rounded_rect(ind_rect, 9.0, palette.highlight_soft())?;
        }

        // Sidebar tabs with crisp vector SfSymbol icons
        for (tab, base_rect) in self.tab_rects() {
            if base_rect.y + base_rect.height > draw_h - 8.0 {
                continue;
            }
            let is_active = self.active_tab == tab;
            let is_hovered = self.hovered_target == Some(HitTarget::Tab(tab));
            let rect = scale_rect_centered(base_rect, self.target_scale(HitTarget::Tab(tab)));

            if !is_active && is_hovered {
                surface.fill_rounded_rect(
                    rect,
                    9.0,
                    palette.control_fill.with_alpha_mul(self.hover_progress.value.clamp(0.4, 1.0)),
                )?;
            }

            let icon_color = if is_active || is_hovered {
                palette.text_primary
            } else {
                palette.text_secondary
            };
            let text_color = if is_active || is_hovered {
                palette.text_primary
            } else {
                palette.text_primary.with_alpha_mul(0.86)
            };

            let icon_rect = Rect::new(rect.x + 12.0, rect.y + (rect.height - 15.0) * 0.5, 15.0, 15.0);
            surface.draw_sf_symbol(icon_rect, tab.symbol(), icon_color, 1.45)?;
            surface.draw_text(
                tab.label(),
                Rect::new(rect.x + 34.0, rect.y, rect.width - 40.0, rect.height),
                12.5,
                text_color,
                TextAlign::Leading,
            )?;
        }

        // Content header with subtle tab-change fade
        let content_alpha = self.content_fade.value.clamp(0.0, 1.0);
        let content_x = SIDEBAR_WIDTH + CONTENT_PAD_X;
        let content_w = (draw_w - content_x - CONTENT_PAD_X).max(120.0);
        surface.draw_text_bold(
            self.active_tab.label(),
            Rect::new(content_x, TITLEBAR_HEIGHT + 14.0, content_w, 26.0),
            18.5,
            palette.text_primary.with_alpha_mul(content_alpha),
            TextAlign::Leading,
        )?;
        surface.draw_text(
            self.active_tab.subtitle(),
            Rect::new(content_x, TITLEBAR_HEIGHT + 42.0, content_w, 18.0),
            11.5,
            palette.text_secondary.with_alpha_mul(content_alpha),
            TextAlign::Leading,
        )?;

        // Content rows (1 full-width row per setting, grouped under section headers with vertical scroll)
        let content_top = CONTENT_TOP;
        let content_bottom = self.content_bottom_y(draw_h);
        let rows = self.rows_for_active_tab();
        let (rects, sections, total_h) = self.layout_sections_and_rows(self.scroll_offset);
        let base_content_w = (self.width - content_x - CONTENT_PAD_X).max(1.0);
        let width_scale = content_w / base_content_w;

        // Clip scrollable section headers & rows strictly inside the content viewport so
        // partially scrolled items never overlap the fixed tab header or bottom notice/edge.
        let clip_rect = Rect::new(
            content_x - 4.0,
            content_top,
            content_w + 8.0,
            (content_bottom - content_top).max(0.0),
        );
        let d2d_clip = D2D_RECT_F {
            left: clip_rect.x,
            top: clip_rect.y,
            right: clip_rect.x + clip_rect.width,
            bottom: clip_rect.y + clip_rect.height,
        };
        let rt = surface.render_target();
        unsafe {
            rt.PushAxisAlignedClip(&d2d_clip, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
        }

        let scroll_draw_res: Result<()> = (|| {
            for (sec_label, sec_y) in sections {
                if sec_y + 20.0 <= content_top || sec_y >= content_bottom {
                    continue;
                }
                surface.draw_text_bold(
                    sec_label,
                    Rect::new(content_x + 4.0, sec_y + 3.0, content_w - 8.0, 16.0),
                    10.5,
                    palette.text_tertiary.with_alpha_mul(content_alpha),
                    TextAlign::Leading,
                )?;
            }

            for (idx, (row, (_, rect))) in rows.into_iter().zip(rects.into_iter()).enumerate() {
                if rect.y + rect.height <= content_top || rect.y >= content_bottom {
                    continue;
                }
            let base_r = Rect::new(
                content_x + (rect.x - content_x) * width_scale,
                rect.y,
                rect.width * width_scale,
                rect.height,
            );
            let r = scale_rect_centered(base_r, self.target_scale(HitTarget::Row(row.id)));
            let hovered = self.hovered_target == Some(HitTarget::Row(row.id));
            let selected = self.selected_index == Some(idx);

            let card_fill = if hovered {
                palette.glass_elevated.with_alpha_mul(0.95 * content_alpha)
            } else {
                palette.glass_elevated.with_alpha_mul(0.68 * content_alpha)
            };
            surface.fill_rounded_rect(r, 11.0, card_fill)?;
            if selected {
                surface.stroke_rounded_rect(
                    r,
                    11.0,
                    palette.highlight_soft().with_alpha_mul(1.35 * content_alpha),
                    1.0,
                )?;
            } else if hovered {
                surface.stroke_rounded_rect(
                    r,
                    11.0,
                    palette.hairline_border.with_alpha_mul(0.45 * content_alpha),
                    1.0,
                )?;
            }

            if let RowVisual::SummaryCard {
                icon,
                count,
                action_label,
            } = row.kind
            {
                let icon_rect = Rect::new(r.x + 12.0, r.y + (r.height - 15.0) * 0.5, 15.0, 15.0);
                surface.draw_sf_symbol(
                    icon_rect,
                    icon,
                    palette.text_secondary.with_alpha_mul(content_alpha),
                    1.4,
                )?;
                surface.draw_text(
                    row.title,
                    Rect::new(r.x + 34.0, r.y + 6.0, r.width * 0.48, 18.0),
                    12.0,
                    palette.text_primary.with_alpha_mul(content_alpha),
                    TextAlign::Leading,
                )?;
                surface.draw_text(
                    &row.desc,
                    Rect::new(r.x + 34.0, r.y + 23.0, r.width * 0.48, 16.0),
                    10.5,
                    palette.text_secondary.with_alpha_mul(content_alpha),
                    TextAlign::Leading,
                )?;

                let pill_w = 112.0_f32.min(r.width * 0.28);
                let pill_h = 26.0;
                let btn = Rect::new(
                    r.x + r.width - pill_w - 14.0,
                    r.y + (r.height - pill_h) * 0.5,
                    pill_w,
                    pill_h,
                );
                surface.fill_rounded_rect(
                    btn,
                    7.0,
                    if hovered {
                        palette.control_hover.with_alpha_mul(content_alpha)
                    } else {
                        palette.control_fill.with_alpha_mul(content_alpha)
                    },
                )?;
                surface.stroke_rounded_rect(
                    btn,
                    7.0,
                    palette.hairline_border.with_alpha_mul(content_alpha),
                    1.0,
                )?;
                surface.draw_text(
                    &format!("{action_label} ({count})"),
                    btn,
                    11.0,
                    palette.text_primary.with_alpha_mul(content_alpha),
                    TextAlign::Center,
                )?;
                continue;
            }

            // Crisp monochrome title & secondary subtitle
            surface.draw_text(
                row.title,
                Rect::new(r.x + 16.0, r.y + 8.0, r.width * 0.56, 18.0),
                12.5,
                palette.text_primary.with_alpha_mul(content_alpha),
                TextAlign::Leading,
            )?;
            surface.draw_text(
                &row.desc,
                Rect::new(r.x + 16.0, r.y + 27.0, r.width * 0.58, 16.0),
                11.0,
                palette.text_secondary.with_alpha_mul(content_alpha),
                TextAlign::Leading,
            )?;

            // Right-hand interactive control
            match row.kind {
                RowVisual::Toggle(on) => {
                    let sw_w = 40.0;
                    let sw_h = 22.0;
                    let sw_x = r.x + r.width - sw_w - 16.0;
                    let sw_y = r.y + (r.height - sw_h) * 0.5;
                    let prog = self.toggle_progress(row.id, on);
                    let track_color = palette
                        .control_hover
                        .lerp(palette.highlight_pill(), prog)
                        .with_alpha_mul(content_alpha);
                    surface.fill_rounded_rect(
                        Rect::new(sw_x, sw_y, sw_w, sw_h),
                        sw_h * 0.5,
                        track_color,
                    )?;
                    let knob_d = 16.0;
                    let knob_x = sw_x + 3.0 + (sw_w - knob_d - 6.0) * prog;
                    surface.fill_rounded_rect(
                        Rect::new(knob_x, sw_y + 3.0, knob_d, knob_d),
                        knob_d * 0.5,
                        Color::from_rgb_u8(0xFF, 0xFF, 0xFF).with_alpha_mul(content_alpha),
                    )?;
                }
                RowVisual::Slider { value, min, max } => {
                    let track = self.opacity_slider_track_rect(r);
                    let t = ((value - min) / (max - min).max(1e-4)).clamp(0.0, 1.0);
                    let pct_label = format!("{}%", (value * 100.0).round() as i32);
                    surface.draw_text(
                        &pct_label,
                        Rect::new(track.x - 46.0, r.y, 38.0, r.height),
                        11.5,
                        palette.text_secondary.with_alpha_mul(content_alpha),
                        TextAlign::Trailing,
                    )?;
                    surface.fill_rounded_rect(
                        track,
                        track.height * 0.5,
                        palette.slider_track.with_alpha_mul(content_alpha),
                    )?;
                    let fill_w = (track.width * t).max(track.height);
                    surface.fill_rounded_rect(
                        Rect::new(track.x, track.y, fill_w, track.height),
                        track.height * 0.5,
                        palette.slider_fill.with_alpha_mul(content_alpha),
                    )?;
                    let knob_r = 7.0;
                    let knob_cx = track.x + (track.width * t).clamp(knob_r * 0.5, track.width - knob_r * 0.5);
                    let knob_cy = track.y + track.height * 0.5;
                    surface.fill_circle(
                        knob_cx,
                        knob_cy,
                        knob_r,
                        Color::from_rgb_u8(0xFF, 0xFF, 0xFF).with_alpha_mul(content_alpha),
                    )?;
                }
                RowVisual::Picker(val) => {
                    let pill_w = 148.0_f32.min(r.width * 0.36);
                    let pill_h = 28.0;
                    let pill_rect = Rect::new(
                        r.x + r.width - pill_w - 16.0,
                        r.y + (r.height - pill_h) * 0.5,
                        pill_w,
                        pill_h,
                    );
                    let is_open = self.open_dropdown == Some(row.id);
                    surface.fill_rounded_rect(
                        pill_rect,
                        8.0,
                        if is_open {
                            palette.highlight_soft().with_alpha_mul(content_alpha)
                        } else if hovered {
                            palette.control_hover.with_alpha_mul(content_alpha)
                        } else {
                            palette.control_fill.with_alpha_mul(content_alpha)
                        },
                    )?;
                    surface.draw_text(
                        &val,
                        Rect::new(
                            pill_rect.x + 11.0,
                            pill_rect.y,
                            pill_rect.width - 30.0,
                            pill_rect.height,
                        ),
                        11.5,
                        palette.text_primary.with_alpha_mul(content_alpha),
                        TextAlign::Leading,
                    )?;
                    surface.draw_sf_symbol(
                        Rect::new(
                            pill_rect.x + pill_rect.width - 19.0,
                            pill_rect.y + 8.5,
                            11.0,
                            11.0,
                        ),
                        SfSymbol::ChevronDown,
                        palette.text_secondary.with_alpha_mul(content_alpha),
                        1.4,
                    )?;
                }
                RowVisual::Button {
                    label,
                    status_connected,
                } => {
                    let pill_w = 132.0_f32.min(r.width * 0.33);
                    let pill_h = 26.0;
                    let pill_rect = Rect::new(
                        r.x + r.width - pill_w - 14.0,
                        r.y + (r.height - pill_h) * 0.5,
                        pill_w,
                        pill_h,
                    );
                    surface.fill_rounded_rect(
                        pill_rect,
                        7.0,
                        if hovered {
                            palette.control_hover.with_alpha_mul(content_alpha)
                        } else {
                            palette.control_fill.with_alpha_mul(content_alpha)
                        },
                    )?;
                    surface.stroke_rounded_rect(
                        pill_rect,
                        7.0,
                        palette.hairline_border.with_alpha_mul(content_alpha),
                        1.0,
                    )?;
                    if let Some(connected) = status_connected {
                        let dot_color = if connected {
                            palette.status_green
                        } else {
                            palette.status_muted
                        };
                        surface.fill_circle(
                            pill_rect.x + 14.0,
                            pill_rect.y + pill_rect.height * 0.5,
                            3.5,
                            dot_color.with_alpha_mul(content_alpha),
                        )?;
                        surface.draw_text(
                            &label,
                            Rect::new(
                                pill_rect.x + 22.0,
                                pill_rect.y,
                                pill_rect.width - 28.0,
                                pill_rect.height,
                            ),
                            11.5,
                            palette.text_primary.with_alpha_mul(content_alpha),
                            TextAlign::Center,
                        )?;
                    } else {
                        surface.draw_text(
                            &label,
                            pill_rect,
                            11.5,
                            palette.text_primary.with_alpha_mul(content_alpha),
                            TextAlign::Center,
                        )?;
                    }
                }
                RowVisual::Info(val) => {
                    surface.draw_text(
                        &val,
                        Rect::new(r.x + r.width * 0.65, r.y, r.width * 0.35 - 14.0, r.height),
                        11.5,
                        palette.text_secondary.with_alpha_mul(content_alpha),
                        TextAlign::Trailing,
                    )?;
                }
                RowVisual::SummaryCard { .. } => {}
            }
        }
        Ok(())
        })();

        unsafe {
            rt.PopAxisAlignedClip();
        }
        scroll_draw_res?;

        // Subtle vertical scrollbar indicator when active tab overflows viewport
        let viewport_h = (content_bottom - content_top).max(40.0);
        if total_h > viewport_h + 1.0 {
            let max_s = (total_h - viewport_h).max(1.0);
            let thumb_h = (viewport_h * (viewport_h / total_h)).clamp(24.0, viewport_h);
            let frac = (self.scroll_offset / max_s).clamp(0.0, 1.0);
            let thumb_y = content_top + (viewport_h - thumb_h) * frac;
            let track_x = draw_w - 8.0;
            surface.fill_rounded_rect(
                Rect::new(track_x, thumb_y, 3.0, thumb_h),
                1.5,
                palette.text_tertiary.with_alpha_mul(0.55 * content_alpha),
            )?;
        }

        // Floating Liquid Glass dropdown list (if a dropdown is open)
        if let Some(dd_id) = self.open_dropdown {
            if let Some(row_rect) = self.row_rect_for(dd_id) {
                if let Some(menu_rect) = self.dropdown_menu_rect(dd_id, row_rect) {
                    surface.draw_liquid_glass_panel(menu_rect, LiquidGlassStyle::elevated(10.0))?;
                    let opts = dropdown_options(dd_id);
                    let current_val = self.dropdown_current_value(dd_id);
                    let pad_y = 6.0;
                    let item_h = 28.0;
                    for (opt_idx, &(opt_val, opt_label)) in opts.iter().enumerate() {
                        let item_r = Rect::new(
                            menu_rect.x + 5.0,
                            menu_rect.y + pad_y + opt_idx as f32 * item_h,
                            menu_rect.width - 10.0,
                            item_h - 2.0,
                        );
                        let is_selected = opt_val == current_val;
                        let is_hovered =
                            self.hovered_target == Some(HitTarget::DropdownItem(dd_id, opt_idx));
                        if is_hovered || is_selected {
                            surface.fill_rounded_rect(
                                item_r,
                                6.0,
                                if is_hovered {
                                    palette.highlight_soft()
                                } else {
                                    palette.control_fill
                                },
                            )?;
                        }
                        surface.draw_text(
                            opt_label,
                            Rect::new(item_r.x + 10.0, item_r.y, item_r.width - 28.0, item_r.height),
                            11.5,
                            palette.text_primary,
                            TextAlign::Leading,
                        )?;
                        if is_selected {
                            surface.draw_sf_symbol(
                                Rect::new(
                                    item_r.x + item_r.width - 18.0,
                                    item_r.y + (item_r.height - 11.0) * 0.5,
                                    11.0,
                                    11.0,
                                ),
                                SfSymbol::Check,
                                palette.text_primary,
                                1.5,
                            )?;
                        }
                    }
                }
            }
        }

        if let Some(notice) = self.visible_status_notice() {
            if draw_h > 470.0 {
                surface.draw_sf_symbol(
                    Rect::new(content_x, draw_h - 26.0, 12.0, 12.0),
                    SfSymbol::Check,
                    palette.status_green,
                    1.5,
                )?;
                surface.draw_text(
                    notice,
                    Rect::new(content_x + 18.0, draw_h - 30.0, content_w - 18.0, 20.0),
                    11.0,
                    palette.text_secondary,
                    TextAlign::Leading,
                )?;
            }
        }

        Ok(())
    }
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

fn dropdown_options(id: RowControlId) -> &'static [(&'static str, &'static str)] {
    match id {
        RowControlId::CycleGlassTheme => &[
            ("clear-glass", "Clear Glass"),
            ("dark-frosted", "Dark Frosted"),
            ("light-frosted", "Light Frosted"),
            ("acrylic", "Acrylic"),
            ("auto", "Auto"),
        ],
        RowControlId::CycleNotchBehavior => &[
            ("fixed", "Fixed"),
            ("smart", "Smart"),
            ("peek", "Peek"),
        ],
        RowControlId::CycleNotchEdge => &[
            ("top", "Top"),
            ("left", "Left"),
            ("right", "Right"),
        ],
        RowControlId::CycleDockMode => &[
            ("fixed", "Fixed"),
            ("auto-hide", "Auto-Hide"),
            ("smart", "Smart"),
        ],
        RowControlId::CycleDockSize => &[
            ("small", "Small"),
            ("medium", "Medium"),
            ("large", "Large"),
        ],
        _ => &[],
    }
}

fn row_to_action(row: RowControlId) -> SettingsAction {
    match row {
        RowControlId::CycleColorTheme => SettingsAction::CycleColorTheme,
        RowControlId::CycleGlassTheme => SettingsAction::CycleGlassTheme,
        RowControlId::GlassOpacity => SettingsAction::SetGlassOpacity(default_glass_opacity()),
        RowControlId::CycleNotchBehavior | RowControlId::CycleNotchEdge => {
            SettingsAction::ToggleNotchMode
        }
        RowControlId::ToggleFreeDrag => SettingsAction::ToggleFreeDrag,
        RowControlId::ToggleDockEnabled | RowControlId::CycleDockMode => {
            SettingsAction::ToggleDockMode
        }
        RowControlId::ToggleHimeAi => SettingsAction::ToggleHimeAi,
        RowControlId::ToggleTranslateOptIn => SettingsAction::ToggleTranslateOptIn,
        RowControlId::CycleProactiveBudget => SettingsAction::CycleProactiveBudget,
        RowControlId::RefreshMemory => SettingsAction::RefreshMemory,
        RowControlId::ReloadSkills => SettingsAction::ReloadSkills,
        RowControlId::ReloadMcp => SettingsAction::ReloadMcp,
        RowControlId::TestAndSaveProfile => SettingsAction::TestAndSaveProfile,
        RowControlId::ResetToDefaults => SettingsAction::ResetToDefaults,
        _ => SettingsAction::UpdateSettings,
    }
}

fn cycle_proactive_budget(current: &str) -> &'static str {
    match current {
        "quiet" | "pendiam" => "balanced",
        "balanced" | "seimbang" => "chatty",
        _ => "quiet",
    }
}

fn proactive_budget_label(budget: &str) -> &'static str {
    match budget {
        "quiet" | "pendiam" => "Quiet",
        "chatty" | "cerewet" => "Chatty",
        _ => "Balanced",
    }
}

fn proactive_budget_desc(budget: &str) -> &'static str {
    match budget {
        "quiet" | "pendiam" => "Minimal proactive nudges (1 nudge/hr)",
        "chatty" | "cerewet" => "Frequent proactive assistance (6 nudges/hr)",
        _ => "Balanced proactive attention budget (3 nudges/hr)",
    }
}

fn is_background_ipc_error(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("ipc transport error") || lower.contains("hime-ipc")
}

fn glass_theme_label(theme: &str) -> &'static str {
    match theme {
        "dark-frosted" => "Dark Frosted",
        "light-frosted" => "Light Frosted",
        "acrylic" => "Acrylic",
        "auto" => "Auto",
        _ => "Clear Glass",
    }
}

fn glass_style_for_theme(theme: &str, corner_radius: f32) -> LiquidGlassStyle {
    match theme {
        "dark-frosted" => LiquidGlassStyle::tinted(Color::from_rgba_u8(16, 18, 24, 210), corner_radius),
        "light-frosted" => LiquidGlassStyle::tinted(Color::from_rgba_u8(242, 244, 248, 205), corner_radius),
        "acrylic" => LiquidGlassStyle::elevated(corner_radius),
        "auto" => LiquidGlassStyle::regular(corner_radius),
        _ => LiquidGlassStyle::clear(corner_radius),
    }
}

fn capitalize_mode(mode: &str) -> String {
    match mode {
        "auto" => "Auto".to_string(),
        "dark" => "Dark".to_string(),
        "light" => "Light".to_string(),
        "fixed" => "Fixed".to_string(),
        "smart" => "Smart".to_string(),
        "peek" => "Peek".to_string(),
        "auto-hide" => "Auto-Hide".to_string(),
        "top" => "Top".to_string(),
        "left" => "Left".to_string(),
        "right" => "Right".to_string(),
        "small" => "Small".to_string(),
        "medium" => "Medium".to_string(),
        "large" => "Large".to_string(),
        other => other.to_string(),
    }
}

fn point_in_rect(x: f32, y: f32, r: Rect) -> bool {
    x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height
}

fn inset_rect(r: Rect, pad: f32) -> Rect {
    Rect::new(
        r.x + pad,
        r.y + pad,
        (r.width - pad * 2.0).max(2.0),
        (r.height - pad * 2.0).max(2.0),
    )
}

fn scale_rect_centered(r: Rect, scale: f32) -> Rect {
    if (scale - 1.0).abs() < 1e-4 {
        return r;
    }
    let w = r.width * scale;
    let h = r.height * scale;
    Rect::new(
        r.x + (r.width - w) * 0.5,
        r.y + (r.height - h) * 0.5,
        w,
        h,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_state_serde_roundtrip() {
        let mut s = SettingsState::default();
        s.hime_ai_enabled = true;
        s.notch_mode = NotchMode::Left;
        s.notch_behavior = "smart".to_string();
        s.dock_mode = "auto-hide".to_string();
        s.active_profile = ApiProfileConfig::from_preset(ProviderPreset::GeminiApi);

        let json = serde_json::to_string(&s).unwrap();
        let parsed: SettingsState = serde_json::from_str(&json).unwrap();
        assert_eq!(s, parsed);
    }

    #[test]
    fn provider_presets_and_controls_update_state() {
        let mut panel = SettingsPanel::new();
        panel.open();
        assert!(panel.state.hime_ai_enabled);
        panel.apply_control(SettingsControl::ToggleHimeAi);
        assert!(!panel.state.hime_ai_enabled);

        panel.apply_control(SettingsControl::CycleProviderPreset);
        assert_eq!(panel.state.active_profile.preset, ProviderPreset::GeminiApi);
        assert_eq!(panel.state.active_profile.adapter_kind_str(), "GeminiNative");
    }

    #[test]
    fn sidebar_tabs_and_rows_hit_test_and_mutate_state() {
        let mut panel = SettingsPanel::new();
        assert_eq!(panel.hit_test_action(40.0, 120.0), None);
        panel.open();

        // Close button top-right
        assert_eq!(panel.hit_test_action(714.0, 20.0), Some(SettingsAction::Close));

        // Click Notch tab in left sidebar
        let notch_tab_rect = panel.tab_rects()[1].1;
        assert_eq!(
            panel.handle_click(
                notch_tab_rect.x + notch_tab_rect.width * 0.5,
                notch_tab_rect.y + notch_tab_rect.height * 0.5
            ),
            Some(SettingsAction::SelectTab(SettingsTab::Notch))
        );
        assert_eq!(panel.active_tab, SettingsTab::Notch);
        assert!(panel.is_animating());

        // First row on Notch tab: Notch Behavior (Dropdown opens, then option 0 selects "fixed")
        let (nb_x, nb_y) = find_row_center(&mut panel, RowControlId::CycleNotchBehavior);
        assert_eq!(
            panel.handle_click(nb_x, nb_y),
            Some(SettingsAction::UpdateSettings)
        );
        let nb_rect = panel.row_rect_for(RowControlId::CycleNotchBehavior).unwrap();
        let nb_menu = panel
            .dropdown_menu_rect(RowControlId::CycleNotchBehavior, nb_rect)
            .unwrap();
        assert_eq!(
            panel.handle_click(nb_menu.x + 20.0, nb_menu.y + 16.0),
            Some(SettingsAction::ToggleNotchMode)
        );
        assert_eq!(panel.state.notch_behavior, "fixed");

        // Third row on Notch tab: Free Drag
        let (fd_x, fd_y) = find_row_center(&mut panel, RowControlId::ToggleFreeDrag);
        assert_eq!(
            panel.handle_click(fd_x, fd_y),
            Some(SettingsAction::ToggleFreeDrag)
        );
        assert!(panel.state.free_drag_enabled);

        // Switch to Dock tab
        let dock_tab_rect = panel.tab_rects()[2].1;
        assert_eq!(
            panel.handle_click(
                dock_tab_rect.x + dock_tab_rect.width * 0.5,
                dock_tab_rect.y + dock_tab_rect.height * 0.5
            ),
            Some(SettingsAction::SelectTab(SettingsTab::Dock))
        );
        // Second row on Dock tab: Dock Behavior (Dropdown opens, option 2 selects "smart")
        let (dm_x, dm_y) = find_row_center(&mut panel, RowControlId::CycleDockMode);
        assert_eq!(
            panel.handle_click(dm_x, dm_y),
            Some(SettingsAction::UpdateSettings)
        );
        let dm_rect = panel.row_rect_for(RowControlId::CycleDockMode).unwrap();
        let dm_menu = panel
            .dropdown_menu_rect(RowControlId::CycleDockMode, dm_rect)
            .unwrap();
        assert_eq!(
            panel.handle_click(dm_menu.x + 20.0, dm_menu.y + 6.0 + 2.0 * 28.0 + 10.0),
            Some(SettingsAction::ToggleDockMode)
        );
        assert_eq!(panel.state.dock_mode, "smart");

        // Switch to Hime AI tab
        let ai_tab_rect = panel.tab_rects()[4].1;
        assert_eq!(
            panel.handle_click(
                ai_tab_rect.x + ai_tab_rect.width * 0.5,
                ai_tab_rect.y + ai_tab_rect.height * 0.5
            ),
            Some(SettingsAction::SelectTab(SettingsTab::HimeAi))
        );
        let (ai_x, ai_y) = find_row_center(&mut panel, RowControlId::ToggleHimeAi);
        assert_eq!(
            panel.handle_click(ai_x, ai_y),
            Some(SettingsAction::ToggleHimeAi)
        );
        assert!(!panel.state.hime_ai_enabled);
    }

    #[test]
    fn keyboard_navigation_selects_and_activates_controls() {
        let mut panel = SettingsPanel::new();
        assert_eq!(panel.handle_key_down(0x1B), None);

        panel.open();
        assert_eq!(panel.selected_index, Some(0));
        assert_eq!(panel.handle_key_down(0x28), None); // VK_DOWN -> row 1 (Glass Blur Theme)
        assert_eq!(panel.handle_key_down(0x28), None); // VK_DOWN -> row 2 (Glass Opacity)
        assert_eq!(panel.handle_key_down(0x28), None); // VK_DOWN -> row 3 (24-Hour Clock)
        assert_eq!(panel.handle_key_down(0x28), None); // VK_DOWN -> row 4 (Silence Native Toasts)
        assert_eq!(panel.selected_index, Some(4));
        assert_eq!(
            panel.handle_key_down(0x0D),
            Some(SettingsControl::ToggleNotifSuppression)
        );
        assert_eq!(panel.handle_key_down(0x1B), Some(SettingsControl::Close));
    }

    #[test]
    fn open_renders_full_size_on_first_frame_without_clipping() {
        let mut panel = SettingsPanel::new();
        panel.open();

        assert!(panel.visible);
        assert_eq!(panel.open_spring.value, 1.0);
        assert!(!panel.is_animating());
        assert_eq!(
            panel.render_size_for_surface(SETTINGS_WIDTH, SETTINGS_HEIGHT),
            (SETTINGS_WIDTH, SETTINGS_HEIGHT)
        );
        assert_eq!(
            panel.render_size_for_surface(0.0, 0.0),
            (SETTINGS_WIDTH, SETTINGS_HEIGHT)
        );

        // All sidebar tabs and content rows fit inside the full 520px height.
        for (_, rect) in panel.tab_rects() {
            assert!(rect.y + rect.height <= SETTINGS_HEIGHT - 8.0);
        }
        for (_, rect) in panel.row_rects() {
            assert!(rect.y + rect.height <= SETTINGS_HEIGHT - 14.0);
        }

        panel.close();
        assert!(!panel.visible);
        assert_eq!(panel.open_spring.value, 0.0);
        assert!(!panel.is_animating());
    }

    #[test]
    fn spring_animations_settle_on_tab_switch_toggle_and_press() {
        let mut panel = SettingsPanel::new();
        panel.open();
        assert!(!panel.is_animating());

        // Switching tab triggers indicator slide + content fade
        let notch_tab = panel.tab_rects()[1].1;
        panel.handle_click(notch_tab.x + notch_tab.width * 0.5, notch_tab.y + notch_tab.height * 0.5);
        assert_eq!(panel.active_tab, SettingsTab::Notch);
        assert!(panel.is_animating());
        assert!(panel.press_scale.value < 1.0);

        for _ in 0..90 {
            panel.tick(1.0 / 60.0);
        }
        assert!(!panel.is_animating());
        assert!((panel.tab_indicator_y.value - SettingsPanel::tab_y(SettingsTab::Notch)).abs() < 0.05);
        assert!((panel.press_scale.value - 1.0).abs() < 0.02);

        // Toggling a switch animates its knob spring and settles cleanly
        let (rx, ry) = {
            let r = panel
                .row_rects()
                .into_iter()
                .find(|(id, _)| *id == RowControlId::ToggleFreeDrag)
                .unwrap()
                .1;
            (r.x + r.width * 0.5, r.y + r.height * 0.5)
        };
        panel.handle_click(rx, ry);
        assert!(panel.state.free_drag_enabled);
        assert!(panel.is_animating());
        for _ in 0..60 {
            panel.tick(1.0 / 60.0);
        }
        assert!(!panel.is_animating());
        assert!((panel.toggle_progress(RowControlId::ToggleFreeDrag, false) - 1.0).abs() < 0.02);
    }

    #[test]
    fn settings_window_renders_liquid_glass_and_sf_symbols_without_error() {
        use crate::d2d::BackdropBuffer;
        use crate::theme::{ColorThemeMode, ThemePolarity, LIGHT_TEXT_PRIMARY, TEXT_PRIMARY};

        let surface = match NativeSurface::new(
            "HimeSettingsTest",
            -2000,
            -2000,
            SETTINGS_WINDOW_WIDTH,
            SETTINGS_WINDOW_HEIGHT,
        ) {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut panel = SettingsPanel::new();
        panel.open();
        panel.set_status_notice(Some("Connected to 9Router".to_string()));

        let dark_bg = BackdropBuffer::from_bgra_premul(4, 4, vec![0x14, 0x14, 0x18, 0xFF].repeat(16))
            .expect("dark backdrop");
        let bright_bg =
            BackdropBuffer::from_bgra_premul(4, 4, vec![0xF0, 0xF2, 0xF6, 0xFF].repeat(16))
                .expect("bright backdrop");

        surface.set_color_theme_mode(ColorThemeMode::Auto);
        surface.set_backdrop(Some(dark_bg));
        assert_eq!(surface.theme_polarity(), ThemePolarity::DarkGlass);
        assert_eq!(surface.palette().text_primary, TEXT_PRIMARY);
        for &tab in &SettingsTab::ALL {
            panel.select_tab(tab);
            for _ in 0..45 {
                panel.tick(1.0 / 60.0);
            }
            surface.begin_draw();
            panel.render(&surface).expect("dark auto settings render");
            surface.end_draw_and_present().expect("dark auto settings present");
        }

        surface.set_backdrop(Some(bright_bg));
        assert_eq!(surface.theme_polarity(), ThemePolarity::LightGlass);
        assert_eq!(surface.palette().text_primary, LIGHT_TEXT_PRIMARY);
        for &tab in &SettingsTab::ALL {
            panel.select_tab(tab);
            for _ in 0..45 {
                panel.tick(1.0 / 60.0);
            }
            surface.begin_draw();
            panel.render(&surface).expect("light auto settings render");
            surface.end_draw_and_present().expect("light auto settings present");
        }
    }

    #[test]
    fn cycle_color_theme_row_cycles_auto_dark_light() {
        let mut panel = SettingsPanel::new();
        panel.open();
        assert_eq!(panel.state.color_theme, "auto");

        let theme_rect = panel
            .row_rects()
            .into_iter()
            .find(|(id, _)| *id == RowControlId::CycleColorTheme)
            .expect("CycleColorTheme row")
            .1;
        let cx = theme_rect.x + theme_rect.width * 0.5;
        let cy = theme_rect.y + theme_rect.height * 0.5;

        assert_eq!(
            panel.handle_click(cx, cy),
            Some(SettingsAction::CycleColorTheme)
        );
        assert_eq!(panel.state.color_theme, "dark");
        assert_eq!(
            panel.handle_click(cx, cy),
            Some(SettingsAction::CycleColorTheme)
        );
        assert_eq!(panel.state.color_theme, "light");
        assert_eq!(
            panel.handle_click(cx, cy),
            Some(SettingsAction::CycleColorTheme)
        );
        assert_eq!(panel.state.color_theme, "auto");
    }

    fn rects_overlap(a: Rect, b: Rect) -> bool {
        a.x < b.x + b.width
            && a.x + a.width > b.x
            && a.y < b.y + b.height
            && a.y + a.height > b.y
    }

    #[test]
    fn all_tabs_fit_inside_740x520_without_clipping_or_overlap() {
        let mut panel = SettingsPanel::new();
        panel.open();
        let expected_full_w = SETTINGS_WIDTH - SIDEBAR_WIDTH - CONTENT_PAD_X * 2.0;

        for (idx, &tab) in SettingsTab::ALL.iter().enumerate() {
            let tab_rect = panel.tab_rects()[idx].1;
            let cx = tab_rect.x + tab_rect.width * 0.5;
            let cy = tab_rect.y + tab_rect.height * 0.5;
            assert_eq!(
                panel.handle_click(cx, cy),
                Some(SettingsAction::SelectTab(tab))
            );
            assert_eq!(panel.active_tab, tab);

            let row_rects = panel.row_rects();
            assert!(!row_rects.is_empty(), "tab {:?} has no rows", tab);
            for (_, r) in &row_rects {
                assert!(r.x >= SIDEBAR_WIDTH, "row left clipped on {:?}", tab);
                assert!(
                    (r.width - expected_full_w).abs() < 0.5,
                    "row must be 1 full-width column on {:?}, got width {}",
                    tab,
                    r.width
                );
                assert!(
                    r.x + r.width <= SETTINGS_WIDTH - CONTENT_PAD_X + 0.5,
                    "row right clipped on {:?}: {:?}",
                    tab,
                    r
                );
                assert!(r.y >= TITLEBAR_HEIGHT, "row top clipped on {:?}", tab);
            }
            if panel.max_scroll() <= 0.5 {
                for (_, r) in &row_rects {
                    assert!(
                        r.y + r.height <= SETTINGS_HEIGHT - 14.0,
                        "row bottom clipped on {:?}: {:?}",
                        tab,
                        r
                    );
                }
            } else {
                // Scroll to bottom and verify last row fits cleanly inside viewport
                panel.scroll_offset = panel.max_scroll();
                panel.scroll_target = panel.max_scroll();
                let scrolled = panel.row_rects();
                let last = scrolled.last().unwrap().1;
                assert!(
                    last.y + last.height <= SETTINGS_HEIGHT - 13.5,
                    "scrolled last row clipped on {:?}: {:?}",
                    tab,
                    last
                );
            }
            for i in 0..row_rects.len() {
                for j in (i + 1)..row_rects.len() {
                    assert!(
                        !rects_overlap(row_rects[i].1, row_rects[j].1),
                        "overlapping rows on {:?}: {:?} vs {:?}",
                        tab,
                        row_rects[i].0,
                        row_rects[j].0
                    );
                }
            }
        }
    }

    fn find_row_center(panel: &mut SettingsPanel, target: RowControlId) -> (f32, f32) {
        panel.scroll_offset = 0.0;
        panel.scroll_target = 0.0;
        let mut r = panel
            .row_rects()
            .into_iter()
            .find(|(id, _)| *id == target)
            .map(|(_, r)| r)
            .expect("expected control rect");
        let bottom = panel.content_bottom_y(SETTINGS_HEIGHT);
        if r.y + r.height > bottom {
            let delta = (r.y + r.height - bottom + 4.0).clamp(0.0, panel.max_scroll());
            panel.scroll_offset = delta;
            panel.scroll_target = delta;
            r = panel
                .row_rects()
                .into_iter()
                .find(|(id, _)| *id == target)
                .map(|(_, r)| r)
                .expect("expected scrolled control rect");
        }
        (r.x + r.width * 0.5, r.y + r.height * 0.5)
    }

    #[test]
    fn hime_ai_new_controls_hit_test_and_mutate_state() {
        let mut panel = SettingsPanel::new();
        panel.open();
        panel.state.memory_count = 14;
        panel.state.skills_count = 6;
        panel.state.mcp_count = 3;
        panel.select_tab(SettingsTab::HimeAi);

        assert!(!panel.state.translate_opt_in);
        assert_eq!(panel.state.proactive_budget, "balanced");

        // Translate Selection opt-in toggle
        let (tx, ty) = find_row_center(&mut panel, RowControlId::ToggleTranslateOptIn);
        assert_eq!(
            panel.hit_test_action(tx, ty),
            Some(SettingsAction::ToggleTranslateOptIn)
        );
        assert_eq!(
            panel.handle_click(tx, ty),
            Some(SettingsAction::ToggleTranslateOptIn)
        );
        assert!(panel.state.translate_opt_in);
        assert_eq!(
            panel.handle_click(tx, ty),
            Some(SettingsAction::ToggleTranslateOptIn)
        );
        assert!(!panel.state.translate_opt_in);

        // Proactive Budget cycler: balanced -> chatty -> quiet -> balanced
        let (px, py) = find_row_center(&mut panel, RowControlId::CycleProactiveBudget);
        assert_eq!(
            panel.hit_test_action(px, py),
            Some(SettingsAction::CycleProactiveBudget)
        );
        assert_eq!(
            panel.handle_click(px, py),
            Some(SettingsAction::CycleProactiveBudget)
        );
        assert_eq!(panel.state.proactive_budget, "chatty");
        assert_eq!(
            panel.handle_click(px, py),
            Some(SettingsAction::CycleProactiveBudget)
        );
        assert_eq!(panel.state.proactive_budget, "quiet");
        assert_eq!(
            panel.handle_click(px, py),
            Some(SettingsAction::CycleProactiveBudget)
        );
        assert_eq!(panel.state.proactive_budget, "balanced");

        // Memory / Skills / MCP Servers summary triggers
        let (mx, my) = find_row_center(&mut panel, RowControlId::RefreshMemory);
        assert_eq!(
            panel.handle_click(mx, my),
            Some(SettingsAction::RefreshMemory)
        );

        let (sx, sy) = find_row_center(&mut panel, RowControlId::ReloadSkills);
        assert_eq!(
            panel.handle_click(sx, sy),
            Some(SettingsAction::ReloadSkills)
        );

        let (mcx, mcy) = find_row_center(&mut panel, RowControlId::ReloadMcp);
        assert_eq!(
            panel.handle_click(mcx, mcy),
            Some(SettingsAction::ReloadMcp)
        );

        // Summary rows include live counts
        let rows = panel.rows_for_active_tab();
        let mem_row = rows.iter().find(|r| r.id == RowControlId::RefreshMemory).unwrap();
        assert_eq!(mem_row.desc, "14 entries");
        let skills_row = rows.iter().find(|r| r.id == RowControlId::ReloadSkills).unwrap();
        assert_eq!(skills_row.desc, "6 loaded");
        let mcp_row = rows.iter().find(|r| r.id == RowControlId::ReloadMcp).unwrap();
        assert_eq!(mcp_row.desc, "3 connected");
    }

    #[test]
    fn settings_state_backwards_compat_serde_defaults() {
        let legacy_json = r#"{"hime_ai_enabled":true,"dock_enabled":false}"#;
        let parsed: SettingsState = serde_json::from_str(legacy_json).unwrap();
        assert!(parsed.hime_ai_enabled);
        assert!(!parsed.dock_enabled);
        assert!(!parsed.translate_opt_in);
        assert_eq!(parsed.proactive_budget, "balanced");
        assert_eq!(parsed.memory_count, 0);
        assert_eq!(parsed.skills_count, 0);
        assert_eq!(parsed.mcp_count, 0);
        assert_eq!(parsed.dock_size, "medium");
        assert!(parsed.dock_group_windows);
        assert!(parsed.dock_show_dividers);
        assert!(parsed.dock_show_hime_ai);
        assert!(parsed.dock_show_desktop);
        assert!(parsed.dock_show_tray);
        assert!(parsed.dock_show_notifications);
    }

    #[test]
    fn dock_new_controls_hit_test_and_mutate_state() {
        let mut panel = SettingsPanel::new();
        panel.open();
        panel.select_tab(SettingsTab::Dock);

        // Dock Size dropdown: medium -> large -> small -> medium
        let (sx, sy) = find_row_center(&mut panel, RowControlId::CycleDockSize);
        assert_eq!(panel.state.dock_size, "medium");
        for (opt_idx, expected) in [(2, "large"), (0, "small"), (1, "medium")] {
            assert_eq!(panel.handle_click(sx, sy), Some(SettingsAction::UpdateSettings));
            let row_r = panel.row_rect_for(RowControlId::CycleDockSize).unwrap();
            let menu_r = panel.dropdown_menu_rect(RowControlId::CycleDockSize, row_r).unwrap();
            assert_eq!(
                panel.handle_click(menu_r.x + 20.0, menu_r.y + 6.0 + opt_idx as f32 * 28.0 + 10.0),
                Some(SettingsAction::UpdateSettings)
            );
            assert_eq!(panel.state.dock_size, expected);
        }

        // Group Windows toggle
        let (gx, gy) = find_row_center(&mut panel, RowControlId::ToggleDockGroupWindows);
        assert!(panel.state.dock_group_windows);
        assert_eq!(panel.handle_click(gx, gy), Some(SettingsAction::UpdateSettings));
        assert!(!panel.state.dock_group_windows);

        // Dividers, Hime AI, Desktop, Tray, Notifications toggles
        for (id, getter) in [
            (
                RowControlId::ToggleDockShowDividers,
                (|s: &SettingsState| s.dock_show_dividers) as fn(&SettingsState) -> bool,
            ),
            (RowControlId::ToggleDockShowHimeAi, |s| s.dock_show_hime_ai),
            (RowControlId::ToggleDockShowDesktop, |s| s.dock_show_desktop),
            (RowControlId::ToggleDockShowTray, |s| s.dock_show_tray),
            (RowControlId::ToggleDockShowNotifications, |s| {
                s.dock_show_notifications
            }),
        ] {
            let (cx, cy) = find_row_center(&mut panel, id);
            assert!(getter(&panel.state));
            assert_eq!(panel.handle_click(cx, cy), Some(SettingsAction::UpdateSettings));
            assert!(!getter(&panel.state));
        }
    }

    #[test]
    fn glass_theme_dropdown_selects_all_presets() {
        let mut panel = SettingsPanel::new();
        panel.open();
        assert_eq!(panel.state.glass_theme, "clear-glass");

        let (cx, cy) = find_row_center(&mut panel, RowControlId::CycleGlassTheme);
        for (opt_idx, expected) in [
            (1, "dark-frosted"),
            (2, "light-frosted"),
            (3, "acrylic"),
            (4, "auto"),
            (0, "clear-glass"),
        ] {
            assert_eq!(
                panel.handle_click(cx, cy),
                Some(SettingsAction::UpdateSettings)
            );
            let row_r = panel.row_rect_for(RowControlId::CycleGlassTheme).unwrap();
            let menu_r = panel
                .dropdown_menu_rect(RowControlId::CycleGlassTheme, row_r)
                .unwrap();
            assert_eq!(
                panel.handle_click(menu_r.x + 20.0, menu_r.y + 6.0 + opt_idx as f32 * 28.0 + 10.0),
                Some(SettingsAction::CycleGlassTheme)
            );
            assert_eq!(panel.state.glass_theme, expected);
        }
    }

    #[test]
    fn glass_opacity_slider_supports_click_and_drag() {
        let mut panel = SettingsPanel::new();
        panel.open();
        assert!((panel.state.glass_opacity - 0.78).abs() < 1e-4);

        let row_r = panel.row_rect_for(RowControlId::GlassOpacity).unwrap();
        let track = panel.opacity_slider_track_rect(row_r);

        // Click left edge -> 0.20
        let left_act = panel.handle_click(track.x, track.y + track.height * 0.5);
        assert_eq!(left_act, Some(SettingsAction::SetGlassOpacity(0.20)));
        assert!((panel.state.glass_opacity - 0.20).abs() < 1e-4);
        assert!(panel.is_dragging());

        // Drag to midpoint -> 0.60
        let mid_x = track.x + track.width * 0.5;
        let drag_act = panel.handle_drag(mid_x, track.y);
        assert_eq!(drag_act, Some(SettingsAction::SetGlassOpacity(0.60)));
        assert!((panel.state.glass_opacity - 0.60).abs() < 1e-4);

        // Mouse up ends drag
        panel.handle_mouse_up();
        assert!(!panel.is_dragging());
        assert_eq!(panel.handle_drag(track.x + track.width, track.y), None);
    }

    #[test]
    fn background_ipc_error_notice_is_suppressed_unless_user_triggered() {
        let mut panel = SettingsPanel::new();
        panel.set_status_notice(Some(
            "MCP: IPC transport error: open \\\\.\\pipe\\hime-ipc failed".to_string(),
        ));
        assert_eq!(panel.visible_status_notice(), None);

        panel.set_status_notice_explicit(
            Some("MCP: IPC transport error: open \\\\.\\pipe\\hime-ipc failed".to_string()),
            true,
        );
        assert!(panel.visible_status_notice().is_some());

        panel.set_status_notice(Some("Settings saved".to_string()));
        assert_eq!(panel.visible_status_notice(), Some("Settings saved"));
    }

    #[test]
    fn vertical_scroll_clamps_and_resets_on_tab_switch() {
        let mut panel = SettingsPanel::new();
        panel.open();
        panel.select_tab(SettingsTab::Dock);
        assert!(panel.max_scroll() > 0.0);
        assert!(panel.handle_scroll(-3.0));
        for _ in 0..45 {
            panel.tick(1.0 / 60.0);
        }
        assert!(panel.scroll_offset > 0.0);
        assert!(panel.scroll_offset <= panel.max_scroll());

        panel.select_tab(SettingsTab::General);
        assert_eq!(panel.scroll_offset, 0.0);
        assert!(!panel.handle_scroll(-1.0));
    }

    #[test]
    fn scrolled_clipped_rows_do_not_hit_test_outside_content_bounds() {
        let mut panel = SettingsPanel::new();
        panel.open();
        panel.select_tab(SettingsTab::HimeAi);
        // Scroll partially so the first row straddles CONTENT_TOP
        panel.scroll_offset = 44.0;
        panel.scroll_target = 44.0;

        let rects = panel.row_rects();
        let (first_id, first_rect) = rects[0];
        assert!(first_rect.y < CONTENT_TOP);
        assert!(first_rect.y + first_rect.height > CONTENT_TOP);

        let mid_x = first_rect.x + first_rect.width * 0.5;
        // Clicking the clipped upper portion above CONTENT_TOP must NOT hit the row
        assert_eq!(panel.hit_test_target(mid_x, CONTENT_TOP - 4.0), None);
        // Clicking the visible lower portion inside content_top..content_bottom DOES hit it
        assert_eq!(
            panel.hit_test_target(mid_x, CONTENT_TOP + 4.0),
            Some(HitTarget::Row(first_id))
        );

        // Clicking at or below content_bottom must NOT hit any row
        let bottom = panel.content_bottom_y(SETTINGS_HEIGHT);
        assert_eq!(panel.hit_test_target(mid_x, bottom), None);
        assert_eq!(panel.hit_test_target(mid_x, bottom + 6.0), None);
    }
}
