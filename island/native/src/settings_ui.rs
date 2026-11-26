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

use crate::animation::{Spring, SpringValue};
use crate::d2d::{LiquidGlassStyle, NativeSurface, Rect, SfSymbol, TextAlign};
use crate::theme::{
    Color, DIVIDER_HAIRLINE, GLASS_CLEAR, GLASS_CONTROL, GLASS_CONTROL_HOVER, GLASS_ELEVATED,
    GLASS_HAIRLINE, STATUS_LIVE_GREEN, STATUS_MUTED, SYSTEM_BLUE, SYSTEM_BLUE_TINT, TEXT_PRIMARY,
    TEXT_SECONDARY, TEXT_TERTIARY,
};

pub const SETTINGS_WIDTH: f32 = 740.0;
pub const SETTINGS_HEIGHT: f32 = 520.0;
pub const SETTINGS_WINDOW_WIDTH: u32 = 740;
pub const SETTINGS_WINDOW_HEIGHT: u32 = 520;

const TITLEBAR_HEIGHT: f32 = 42.0;
const SIDEBAR_WIDTH: f32 = 176.0;
const CONTENT_PAD_X: f32 = 22.0;
const ROW_HEIGHT: f32 = 46.0;
const ROW_GAP: f32 = 8.0;
const TAB_START_Y: f32 = TITLEBAR_HEIGHT + 68.0;
const TAB_HEIGHT: f32 = 34.0;
const TAB_STEP_Y: f32 = 38.0;
const PRESS_MICRO_SCALE: f32 = 0.968;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

    pub volume_overlay_enabled: bool,
    pub brightness_overlay_enabled: bool,

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
            volume_overlay_enabled: true,
            brightness_overlay_enabled: true,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsAction {
    SelectTab(SettingsTab),
    UpdateSettings,
    ToggleDockMode,
    ToggleNotchMode,
    ToggleFreeDrag,
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
    ToggleTimeFormat24h,
    ToggleNotifSuppression,
    ResetToDefaults,
    CycleNotchBehavior,
    CycleNotchEdge,
    ToggleFreeDrag,
    ToggleMusicMode,
    ToggleDockEnabled,
    CycleDockMode,
    ToggleDockWinNumber,
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
}

struct RenderedRow {
    id: RowControlId,
    title: &'static str,
    desc: String,
    kind: RowVisual,
}

enum RowVisual {
    Toggle(bool),
    Picker(String),
    Button {
        label: String,
        status_dot: Option<Color>,
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
    status_notice: Option<String>,
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
            status_notice: None,
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
        if changed {
            self.tab_indicator_y
                .animate_to(Self::tab_y(tab), Spring::SMOOTH);
            self.content_fade.snap_to(0.15);
            self.content_fade.launch_to(1.0, Spring::EMERGE, 1.0);
            self.sync_toggle_targets(true);
        }
    }

    pub fn set_status_notice(&mut self, msg: Option<String>) {
        self.status_notice = msg;
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
            HitTarget::Row(row) => Some(row_to_action(row)),
        }
    }

    /// Hit-tests a click at `(x, y)`, mutates `self.state` / `self.active_tab`, triggers
    /// press micro-scale feedback, and returns the [`SettingsAction`] for the host shell.
    pub fn handle_click(&mut self, x: f32, y: f32) -> Option<SettingsAction> {
        if !self.visible {
            return None;
        }
        let target = self.hit_test_target(x, y)?;
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
            HitTarget::Row(row) => {
                let action = self.apply_row_control(row);
                self.sync_toggle_targets(false);
                Some(action)
            }
        }
    }

    /// Legacy row hit-test preserved for existing callers.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<SettingsControl> {
        if !self.visible {
            return None;
        }
        match self.hit_test_target(x, y)? {
            HitTarget::Close => Some(SettingsControl::Close),
            HitTarget::Tab(_) => None,
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
            RowControlId::ToggleDockWinNumber => {
                self.state.dock_win_number_enabled = !self.state.dock_win_number_enabled;
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
                RowControlId::ToggleDockWinNumber,
                self.state.dock_win_number_enabled,
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
                    id: RowControlId::ToggleTimeFormat24h,
                    title: "24-Hour Clock",
                    desc: "Display 24-hour time in the Notch status bar".to_string(),
                    kind: RowVisual::Toggle(self.state.time_format_24h),
                },
                RenderedRow {
                    id: RowControlId::ToggleNotifSuppression,
                    title: "Suppress Native Notifications",
                    desc: "Hide Windows toast popups and show alerts in Notch".to_string(),
                    kind: RowVisual::Toggle(self.state.suppress_native_notifications),
                },
                RenderedRow {
                    id: RowControlId::ResetToDefaults,
                    title: "Reset to Defaults",
                    desc: "Restore all Notch, Dock, Overlay, and AI settings".to_string(),
                    kind: RowVisual::Button {
                        label: "Reset".to_string(),
                        status_dot: None,
                    },
                },
            ],
            SettingsTab::Notch => vec![
                RenderedRow {
                    id: RowControlId::CycleNotchBehavior,
                    title: "Notch Behavior",
                    desc: "Fixed (always visible), Smart (auto-hide), or Peek".to_string(),
                    kind: RowVisual::Picker(capitalize_mode(&self.state.notch_behavior)),
                },
                RenderedRow {
                    id: RowControlId::CycleNotchEdge,
                    title: "Notch Screen Edge",
                    desc: "Anchor the Dynamic Island to Top, Left, or Right".to_string(),
                    kind: RowVisual::Picker(capitalize_mode(&self.state.notch_edge)),
                },
                RenderedRow {
                    id: RowControlId::ToggleFreeDrag,
                    title: "Free Position / Draggable",
                    desc: "Drag the Notch freely across screen edges".to_string(),
                    kind: RowVisual::Toggle(self.state.free_drag_enabled),
                },
                RenderedRow {
                    id: RowControlId::ToggleMusicMode,
                    title: "Music Mode & Visualizer",
                    desc: "Show live SMTC media visualizer & lyrics in Notch".to_string(),
                    kind: RowVisual::Toggle(self.state.music_mode_enabled),
                },
            ],
            SettingsTab::Dock => vec![
                RenderedRow {
                    id: RowControlId::ToggleDockEnabled,
                    title: "Hime Dock",
                    desc: "Replace Windows taskbar with the floating Dock".to_string(),
                    kind: RowVisual::Toggle(self.state.dock_enabled),
                },
                RenderedRow {
                    id: RowControlId::CycleDockMode,
                    title: "Dock Behavior",
                    desc: "Fixed (always show), Auto-Hide, or Smart overlap".to_string(),
                    kind: RowVisual::Picker(capitalize_mode(&self.state.dock_mode)),
                },
                RenderedRow {
                    id: RowControlId::ToggleDockWinNumber,
                    title: "Win+1..9 Shortcuts",
                    desc: "Focus or launch Dock slots via Win+1 through Win+9".to_string(),
                    kind: RowVisual::Toggle(self.state.dock_win_number_enabled),
                },
                RenderedRow {
                    id: RowControlId::ClearPinnedApps,
                    title: "Pinned Applications",
                    desc: format!(
                        "{} pinned app(s) saved in settings.json",
                        self.state.dock_pinned.len()
                    ),
                    kind: RowVisual::Button {
                        label: if self.state.dock_pinned.is_empty() {
                            "Empty".to_string()
                        } else {
                            "Clear Pinned".to_string()
                        },
                        status_dot: None,
                    },
                },
            ],
            SettingsTab::Overlays => vec![
                RenderedRow {
                    id: RowControlId::ToggleVolumeOverlay,
                    title: "Volume HUD (Left Edge)",
                    desc: "Show interactive volume bar on left screen edge".to_string(),
                    kind: RowVisual::Toggle(self.state.volume_overlay_enabled),
                },
                RenderedRow {
                    id: RowControlId::ToggleBrightnessOverlay,
                    title: "Brightness HUD (Right Edge)",
                    desc: "Show interactive brightness bar on right screen edge".to_string(),
                    kind: RowVisual::Toggle(self.state.brightness_overlay_enabled),
                },
                RenderedRow {
                    id: RowControlId::ToggleFreeDrag,
                    title: "Draggable Overlays",
                    desc: "Allow dragging Volume and Brightness HUD panels".to_string(),
                    kind: RowVisual::Toggle(self.state.free_drag_enabled),
                },
            ],
            SettingsTab::HimeAi => vec![
                RenderedRow {
                    id: RowControlId::ToggleHimeAi,
                    title: "Enable Hime AI",
                    desc: "Activate Hime Core sidecar, AI Drawer, and Notch tab".to_string(),
                    kind: RowVisual::Toggle(self.state.hime_ai_enabled),
                },
                RenderedRow {
                    id: RowControlId::CycleProviderPreset,
                    title: "AI Provider Preset",
                    desc: self.state.hime_router_url.clone(),
                    kind: RowVisual::Picker(self.state.active_profile.preset.label().to_string()),
                },
                RenderedRow {
                    id: RowControlId::CycleRouterModel,
                    title: "Default Model",
                    desc: "Click to cycle model (Custom preset accepts typing)".to_string(),
                    kind: RowVisual::Picker(self.state.hime_router_model.clone()),
                },
                RenderedRow {
                    id: RowControlId::TestAndSaveProfile,
                    title: "9Router Gateway Status",
                    desc: if self.state.router_9_detected {
                        "9Router detected on 127.0.0.1:20128".to_string()
                    } else {
                        "Gateway offline or standby — click to test & save".to_string()
                    },
                    kind: RowVisual::Button {
                        label: if self.state.router_9_detected {
                            "Connected".to_string()
                        } else {
                            "Test & Save".to_string()
                        },
                        status_dot: Some(if self.state.router_9_detected {
                            STATUS_LIVE_GREEN
                        } else {
                            STATUS_MUTED
                        }),
                    },
                },
                RenderedRow {
                    id: RowControlId::ToggleTranslateOptIn,
                    title: "Translate Selection",
                    desc: "Privacy opt-in: send highlighted text for quick translation".to_string(),
                    kind: RowVisual::Toggle(self.state.translate_opt_in),
                },
                RenderedRow {
                    id: RowControlId::CycleProactiveBudget,
                    title: "Proactive Budget",
                    desc: proactive_budget_desc(&self.state.proactive_budget).to_string(),
                    kind: RowVisual::Picker(
                        proactive_budget_label(&self.state.proactive_budget).to_string(),
                    ),
                },
                RenderedRow {
                    id: RowControlId::RefreshMemory,
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
                    title: "Hime Island Native",
                    desc: "Pure Rust Direct2D + DirectWrite + Layered Composition".to_string(),
                    kind: RowVisual::Info("v1.0.0-rc.1".to_string()),
                },
                RenderedRow {
                    id: RowControlId::AboutInfo,
                    title: "Settings File",
                    desc: "%APPDATA%\\ai.hime.desktop\\settings.json".to_string(),
                    kind: RowVisual::Info("JSON Sync".to_string()),
                },
                RenderedRow {
                    id: RowControlId::AboutInfo,
                    title: "License",
                    desc: "GNU General Public License v3.0 (Derivative of Bloom)".to_string(),
                    kind: RowVisual::Info("GPL-3.0".to_string()),
                },
            ],
        }
    }

    fn row_rects(&self) -> Vec<(RowControlId, Rect)> {
        let rows = self.rows_for_active_tab();
        let content_x = SIDEBAR_WIDTH + CONTENT_PAD_X;
        let row_w = self.width - content_x - CONTENT_PAD_X;
        if self.active_tab == SettingsTab::HimeAi {
            let stacked_count = rows
                .iter()
                .filter(|r| !matches!(r.kind, RowVisual::SummaryCard { .. }))
                .count();
            let mut out = Vec::with_capacity(rows.len());
            let mut y = TITLEBAR_HEIGHT + 70.0;
            let compact_h = 41.0;
            let compact_gap = 5.0;
            for row in rows.iter().take(stacked_count) {
                out.push((row.id, Rect::new(content_x, y, row_w, compact_h)));
                y += compact_h + compact_gap;
            }
            let summary_rows: Vec<&RenderedRow> = rows.iter().skip(stacked_count).collect();
            if !summary_rows.is_empty() {
                let card_gap = 8.0;
                let cols = summary_rows.len() as f32;
                let card_w = ((row_w - card_gap * (cols - 1.0)) / cols).max(80.0);
                let card_h = 66.0;
                let strip_y = y + 2.0;
                for (i, row) in summary_rows.into_iter().enumerate() {
                    let cx = content_x + i as f32 * (card_w + card_gap);
                    out.push((row.id, Rect::new(cx, strip_y, card_w, card_h)));
                }
            }
            return out;
        }
        let mut y = TITLEBAR_HEIGHT + 74.0;
        rows.into_iter()
            .map(|r| {
                let rect = Rect::new(content_x, y, row_w, ROW_HEIGHT);
                y += ROW_HEIGHT + ROW_GAP;
                (r.id, rect)
            })
            .collect()
    }

    fn hit_test_target(&self, x: f32, y: f32) -> Option<HitTarget> {
        if point_in_rect(x, y, self.close_rect()) {
            return Some(HitTarget::Close);
        }
        for (tab, rect) in self.tab_rects() {
            if point_in_rect(x, y, rect) {
                return Some(HitTarget::Tab(tab));
            }
        }
        for (row_id, rect) in self.row_rects() {
            if point_in_rect(x, y, rect) {
                return Some(HitTarget::Row(row_id));
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

        let panel_rect = Rect::new(0.0, 0.0, draw_w, draw_h);
        surface.set_rounded_region(panel_rect, 16.0);
        surface.draw_liquid_glass_panel(
            panel_rect,
            LiquidGlassStyle::regular(16.0).without_shadow(),
        )?;

        // Subtle macOS window titlebar reflecting active pane
        surface.draw_text(
            "Settings",
            Rect::new(18.0, 8.0, 120.0, 26.0),
            12.0,
            TEXT_TERTIARY,
            TextAlign::Leading,
        )?;

        // Crisp vector CloseX button inside Liquid Glass circle
        let close_base = Rect::new(draw_w - 38.0, 9.0, 24.0, 24.0);
        let close_r = scale_rect_centered(close_base, self.target_scale(HitTarget::Close));
        let close_hovered = self.hovered_target == Some(HitTarget::Close);
        let close_pressed = self.pressed_target == Some(HitTarget::Close);
        surface.draw_liquid_glass_button(close_r, 12.0, None, close_hovered, close_pressed)?;
        surface.draw_sf_symbol(
            inset_rect(close_r, 6.0),
            SfSymbol::CloseX,
            if close_hovered {
                TEXT_PRIMARY
            } else {
                TEXT_SECONDARY
            },
            1.5,
        )?;

        if draw_h <= TITLEBAR_HEIGHT + 20.0 {
            return Ok(());
        }

        // Sidebar smoked glass pane & hairline dividers
        let sidebar_bg = Rect::new(
            1.0,
            TITLEBAR_HEIGHT,
            SIDEBAR_WIDTH - 1.0,
            draw_h - TITLEBAR_HEIGHT - 1.0,
        );
        surface.fill_rounded_rect(sidebar_bg, 0.0, GLASS_CLEAR.with_alpha(0.42))?;
        surface.fill_rounded_rect(
            Rect::new(SIDEBAR_WIDTH, 1.0, 1.0, draw_h - 2.0),
            0.0,
            DIVIDER_HAIRLINE,
        )?;
        surface.fill_rounded_rect(
            Rect::new(SIDEBAR_WIDTH + 1.0, TITLEBAR_HEIGHT, draw_w - SIDEBAR_WIDTH - 2.0, 1.0),
            0.0,
            DIVIDER_HAIRLINE,
        )?;

        // Sidebar app badge & title
        let badge_rect = Rect::new(18.0, TITLEBAR_HEIGHT + 13.0, 28.0, 28.0);
        surface.draw_liquid_glass_panel(badge_rect, LiquidGlassStyle::elevated(8.0))?;
        surface.draw_sf_symbol(
            inset_rect(badge_rect, 6.0),
            SfSymbol::Sparkle,
            TEXT_PRIMARY,
            1.4,
        )?;
        surface.draw_text_bold(
            "Hime",
            Rect::new(54.0, TITLEBAR_HEIGHT + 10.0, SIDEBAR_WIDTH - 64.0, 20.0),
            14.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            "System Shell",
            Rect::new(54.0, TITLEBAR_HEIGHT + 29.0, SIDEBAR_WIDTH - 64.0, 16.0),
            10.5,
            TEXT_SECONDARY,
            TextAlign::Leading,
        )?;

        // Animated active tab indicator pill sliding smoothly with Spring::SMOOTH
        let ind_y = self.tab_indicator_y.value;
        if ind_y + TAB_HEIGHT <= draw_h - 8.0 {
            let ind_rect = scale_rect_centered(
                Rect::new(10.0, ind_y, SIDEBAR_WIDTH - 20.0, TAB_HEIGHT),
                self.target_scale(HitTarget::Tab(self.active_tab)),
            );
            surface.draw_liquid_glass_panel(
                ind_rect,
                LiquidGlassStyle::tinted(SYSTEM_BLUE_TINT, 8.0).without_shadow(),
            )?;
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
                    8.0,
                    GLASS_CONTROL.with_alpha_mul(self.hover_progress.value.clamp(0.4, 1.0)),
                )?;
            }

            let icon_color = if is_active {
                SYSTEM_BLUE
            } else if is_hovered {
                TEXT_PRIMARY
            } else {
                TEXT_SECONDARY
            };
            let text_color = if is_active || is_hovered {
                TEXT_PRIMARY
            } else {
                TEXT_PRIMARY.with_alpha(0.84)
            };

            let icon_rect = Rect::new(rect.x + 10.0, rect.y + (rect.height - 15.0) * 0.5, 15.0, 15.0);
            surface.draw_sf_symbol(icon_rect, tab.symbol(), icon_color, 1.45)?;
            surface.draw_text(
                tab.label(),
                Rect::new(rect.x + 32.0, rect.y, rect.width - 38.0, rect.height),
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
            Rect::new(content_x, TITLEBAR_HEIGHT + 12.0, content_w, 26.0),
            18.0,
            TEXT_PRIMARY.with_alpha_mul(content_alpha),
            TextAlign::Leading,
        )?;
        surface.draw_text(
            self.active_tab.subtitle(),
            Rect::new(content_x, TITLEBAR_HEIGHT + 39.0, content_w, 18.0),
            11.5,
            TEXT_SECONDARY.with_alpha_mul(content_alpha),
            TextAlign::Leading,
        )?;

        // Content rows
        let rows = self.rows_for_active_tab();
        let rects = self.row_rects();
        let base_content_w = (self.width - content_x - CONTENT_PAD_X).max(1.0);
        let width_scale = content_w / base_content_w;

        for (idx, (row, (_, rect))) in rows.into_iter().zip(rects.into_iter()).enumerate() {
            if rect.y + rect.height > draw_h - 14.0 {
                continue;
            }
            let base_r = if matches!(row.kind, RowVisual::SummaryCard { .. }) {
                Rect::new(
                    content_x + (rect.x - content_x) * width_scale,
                    rect.y,
                    rect.width * width_scale,
                    rect.height,
                )
            } else {
                Rect::new(rect.x, rect.y, content_w, rect.height)
            };
            let r = scale_rect_centered(base_r, self.target_scale(HitTarget::Row(row.id)));
            let hovered = self.hovered_target == Some(HitTarget::Row(row.id));
            let selected = self.selected_index == Some(idx);

            let card_fill = if hovered {
                GLASS_ELEVATED.with_alpha_mul(content_alpha)
            } else {
                GLASS_ELEVATED.with_alpha_mul(0.74 * content_alpha)
            };
            surface.fill_rounded_rect(r, 10.0, card_fill)?;
            let border_color = if selected {
                SYSTEM_BLUE.with_alpha(0.75 * content_alpha)
            } else if hovered {
                GLASS_HAIRLINE.with_alpha_mul(1.25 * content_alpha)
            } else {
                DIVIDER_HAIRLINE.with_alpha_mul(content_alpha)
            };
            surface.stroke_rounded_rect(r, 10.0, border_color, if selected { 1.2 } else { 1.0 })?;

            if let RowVisual::SummaryCard {
                icon,
                count,
                action_label,
            } = row.kind
            {
                let icon_rect = Rect::new(r.x + 10.0, r.y + 9.0, 14.0, 14.0);
                surface.draw_sf_symbol(
                    icon_rect,
                    icon,
                    TEXT_SECONDARY.with_alpha_mul(content_alpha),
                    1.4,
                )?;
                surface.draw_text(
                    row.title,
                    Rect::new(r.x + 28.0, r.y + 7.0, (r.width - 78.0).max(40.0), 18.0),
                    11.5,
                    TEXT_PRIMARY.with_alpha_mul(content_alpha),
                    TextAlign::Leading,
                )?;

                let badge = Rect::new(r.x + r.width - 46.0, r.y + 7.0, 36.0, 18.0);
                surface.fill_rounded_rect(
                    badge,
                    6.0,
                    GLASS_CONTROL.with_alpha_mul(content_alpha),
                )?;
                surface.draw_text(
                    &count.to_string(),
                    badge,
                    11.0,
                    TEXT_PRIMARY.with_alpha_mul(content_alpha),
                    TextAlign::Center,
                )?;

                let btn = Rect::new(r.x + 10.0, r.y + 35.0, (r.width - 20.0).max(40.0), 22.0);
                surface.fill_rounded_rect(
                    btn,
                    7.0,
                    if hovered {
                        GLASS_CONTROL_HOVER.with_alpha_mul(content_alpha)
                    } else {
                        GLASS_CONTROL.with_alpha_mul(content_alpha)
                    },
                )?;
                surface.stroke_rounded_rect(
                    btn,
                    7.0,
                    GLASS_HAIRLINE.with_alpha_mul(content_alpha),
                    1.0,
                )?;
                let refresh_r = Rect::new(btn.x + btn.width * 0.5 - 30.0, btn.y + 5.0, 12.0, 12.0);
                surface.draw_sf_symbol(
                    refresh_r,
                    SfSymbol::Refresh,
                    TEXT_SECONDARY.with_alpha_mul(content_alpha),
                    1.3,
                )?;
                surface.draw_text(
                    action_label,
                    Rect::new(btn.x + 10.0, btn.y, btn.width - 10.0, btn.height),
                    11.0,
                    TEXT_PRIMARY.with_alpha_mul(content_alpha),
                    TextAlign::Center,
                )?;
                continue;
            }

            // Crisp monochrome title & secondary subtitle
            surface.draw_text(
                row.title,
                Rect::new(r.x + 14.0, r.y + 6.0, r.width * 0.62, 18.0),
                12.0,
                TEXT_PRIMARY.with_alpha_mul(content_alpha),
                TextAlign::Leading,
            )?;
            surface.draw_text(
                &row.desc,
                Rect::new(r.x + 14.0, r.y + 23.0, r.width * 0.64, 16.0),
                10.5,
                TEXT_SECONDARY.with_alpha_mul(content_alpha),
                TextAlign::Leading,
            )?;

            // Right-hand interactive control
            match row.kind {
                RowVisual::Toggle(on) => {
                    let sw_w = 40.0;
                    let sw_h = 22.0;
                    let sw_x = r.x + r.width - sw_w - 14.0;
                    let sw_y = r.y + (r.height - sw_h) * 0.5;
                    let prog = self.toggle_progress(row.id, on);
                    let off_track = GLASS_CONTROL_HOVER;
                    let track_color = off_track
                        .lerp(SYSTEM_BLUE, prog)
                        .with_alpha_mul(content_alpha);
                    surface.fill_rounded_rect(
                        Rect::new(sw_x, sw_y, sw_w, sw_h),
                        sw_h * 0.5,
                        track_color,
                    )?;
                    if prog < 0.5 {
                        surface.stroke_rounded_rect(
                            Rect::new(sw_x, sw_y, sw_w, sw_h),
                            sw_h * 0.5,
                            GLASS_HAIRLINE.with_alpha_mul((1.0 - prog) * content_alpha),
                            1.0,
                        )?;
                    }
                    let knob_d = 16.0;
                    let knob_x = sw_x + 3.0 + (sw_w - knob_d - 6.0) * prog;
                    surface.fill_rounded_rect(
                        Rect::new(knob_x, sw_y + 3.0, knob_d, knob_d),
                        knob_d * 0.5,
                        TEXT_PRIMARY.with_alpha_mul(content_alpha),
                    )?;
                }
                RowVisual::Picker(val) => {
                    let pill_w = 144.0_f32.min(r.width * 0.35);
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
                            GLASS_CONTROL_HOVER.with_alpha_mul(content_alpha)
                        } else {
                            GLASS_CONTROL.with_alpha_mul(content_alpha)
                        },
                    )?;
                    surface.stroke_rounded_rect(
                        pill_rect,
                        7.0,
                        GLASS_HAIRLINE.with_alpha_mul(content_alpha),
                        1.0,
                    )?;
                    surface.draw_text(
                        &val,
                        Rect::new(pill_rect.x + 10.0, pill_rect.y, pill_rect.width - 28.0, pill_rect.height),
                        11.5,
                        TEXT_PRIMARY.with_alpha_mul(content_alpha),
                        TextAlign::Leading,
                    )?;
                    surface.draw_sf_symbol(
                        Rect::new(
                            pill_rect.x + pill_rect.width - 18.0,
                            pill_rect.y + 7.0,
                            11.0,
                            11.0,
                        ),
                        SfSymbol::ChevronDown,
                        TEXT_SECONDARY.with_alpha_mul(content_alpha),
                        1.4,
                    )?;
                }
                RowVisual::Button { label, status_dot } => {
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
                            GLASS_CONTROL_HOVER.with_alpha_mul(content_alpha)
                        } else {
                            GLASS_CONTROL.with_alpha_mul(content_alpha)
                        },
                    )?;
                    surface.stroke_rounded_rect(
                        pill_rect,
                        7.0,
                        GLASS_HAIRLINE.with_alpha_mul(content_alpha),
                        1.0,
                    )?;
                    if let Some(dot_color) = status_dot {
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
                            TEXT_PRIMARY.with_alpha_mul(content_alpha),
                            TextAlign::Center,
                        )?;
                    } else {
                        surface.draw_text(
                            &label,
                            pill_rect,
                            11.5,
                            TEXT_PRIMARY.with_alpha_mul(content_alpha),
                            TextAlign::Center,
                        )?;
                    }
                }
                RowVisual::Info(val) => {
                    surface.draw_text(
                        &val,
                        Rect::new(r.x + r.width * 0.65, r.y, r.width * 0.35 - 14.0, r.height),
                        11.5,
                        TEXT_SECONDARY.with_alpha_mul(content_alpha),
                        TextAlign::Trailing,
                    )?;
                }
                RowVisual::SummaryCard { .. } => {}
            }
        }

        if let Some(notice) = &self.status_notice {
            if draw_h > 470.0 {
                surface.draw_sf_symbol(
                    Rect::new(content_x, draw_h - 26.0, 12.0, 12.0),
                    SfSymbol::Check,
                    STATUS_LIVE_GREEN,
                    1.5,
                )?;
                surface.draw_text(
                    notice,
                    Rect::new(content_x + 18.0, draw_h - 30.0, content_w - 18.0, 20.0),
                    11.0,
                    TEXT_SECONDARY,
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

fn row_to_action(row: RowControlId) -> SettingsAction {
    match row {
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

fn capitalize_mode(mode: &str) -> String {
    match mode {
        "fixed" => "Fixed".to_string(),
        "smart" => "Smart".to_string(),
        "peek" => "Peek".to_string(),
        "auto-hide" => "Auto-Hide".to_string(),
        "top" => "Top".to_string(),
        "left" => "Left".to_string(),
        "right" => "Right".to_string(),
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
        assert_eq!(
            panel.handle_click(60.0, 160.0),
            Some(SettingsAction::SelectTab(SettingsTab::Notch))
        );
        assert_eq!(panel.active_tab, SettingsTab::Notch);
        assert!(panel.is_animating());

        // First row on Notch tab: Notch Behavior
        assert_eq!(
            panel.handle_click(300.0, 130.0),
            Some(SettingsAction::ToggleNotchMode)
        );
        assert_eq!(panel.state.notch_behavior, "fixed");

        // Third row on Notch tab: Free Drag
        assert_eq!(
            panel.handle_click(300.0, 240.0),
            Some(SettingsAction::ToggleFreeDrag)
        );
        assert!(panel.state.free_drag_enabled);

        // Switch to Dock tab
        assert_eq!(
            panel.handle_click(60.0, 200.0),
            Some(SettingsAction::SelectTab(SettingsTab::Dock))
        );
        // Second row on Dock tab: Dock Behavior
        assert_eq!(
            panel.handle_click(300.0, 185.0),
            Some(SettingsAction::ToggleDockMode)
        );
        assert_eq!(panel.state.dock_mode, "smart");

        // Switch to Hime AI tab
        assert_eq!(
            panel.handle_click(60.0, 278.0),
            Some(SettingsAction::SelectTab(SettingsTab::HimeAi))
        );
        assert_eq!(
            panel.handle_click(300.0, 130.0),
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
        assert_eq!(panel.handle_key_down(0x28), None); // VK_DOWN -> row 1 (Suppress Notifs)
        assert_eq!(panel.selected_index, Some(1));
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
        panel.handle_click(60.0, 160.0);
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
        for &tab in &SettingsTab::ALL {
            panel.select_tab(tab);
            for _ in 0..45 {
                panel.tick(1.0 / 60.0);
            }
            surface.begin_draw();
            panel.render(&surface).expect("settings render");
            surface.end_draw_and_present().expect("settings present");
        }
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
                    r.x + r.width <= SETTINGS_WIDTH - CONTENT_PAD_X + 0.5,
                    "row right clipped on {:?}: {:?}",
                    tab,
                    r
                );
                assert!(r.y >= TITLEBAR_HEIGHT, "row top clipped on {:?}", tab);
                assert!(
                    r.y + r.height <= SETTINGS_HEIGHT - 14.0,
                    "row bottom clipped on {:?}: {:?}",
                    tab,
                    r
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

        let rects = panel.row_rects();
        let find_center = |target: RowControlId| -> (f32, f32) {
            let r = rects
                .iter()
                .find(|(id, _)| *id == target)
                .map(|(_, r)| *r)
                .expect("expected control rect");
            (r.x + r.width * 0.5, r.y + r.height * 0.5)
        };

        // Translate Selection opt-in toggle
        let (tx, ty) = find_center(RowControlId::ToggleTranslateOptIn);
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
        let (px, py) = find_center(RowControlId::CycleProactiveBudget);
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
        let (mx, my) = find_center(RowControlId::RefreshMemory);
        assert_eq!(
            panel.handle_click(mx, my),
            Some(SettingsAction::RefreshMemory)
        );

        let (sx, sy) = find_center(RowControlId::ReloadSkills);
        assert_eq!(
            panel.handle_click(sx, sy),
            Some(SettingsAction::ReloadSkills)
        );

        let (mcx, mcy) = find_center(RowControlId::ReloadMcp);
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
    }
}
