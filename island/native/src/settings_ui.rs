//! Interactive Native Settings Window rendered with Direct2D (`NativeSurface`).
//!
//! Matches `island/src/Settings.tsx` (`~740x520` px) with crisp white text,
//! left sidebar tabs (`General`, `Notch`, `Dock`, `Overlays`, `Hime AI`, `About`),
//! interactive toggles & mode cyclers, and full mouse click + keyboard hit-testing.

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::SpringValue;
use crate::d2d::{NativeSurface, Rect, TextAlign};
use crate::theme::{
    ACCENT_CYAN, DOCK_SURFACE, GLASS_BORDER, GLASS_SURFACE, NOTCH_BG_SOLID, STATUS_OFFLINE,
    STATUS_SUCCESS, SURFACE_ELEVATED, TEXT_PRIMARY, TEXT_SECONDARY,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Pill(String, crate::theme::Color),
    Button(String, crate::theme::Color),
    SummaryCard {
        count: usize,
        action_label: &'static str,
        accent: crate::theme::Color,
    },
    Info(String),
}

/// Interactive Direct2D Settings window (`740x520` px) matching `Settings.tsx`.
pub struct SettingsPanel {
    pub state: SettingsState,
    pub visible: bool,
    pub active_tab: SettingsTab,
    pub selected_index: Option<usize>,
    hovered_target: Option<HitTarget>,
    status_notice: Option<String>,
    open_spring: SpringValue,
    width: f32,
    height: f32,
}

impl SettingsPanel {
    pub fn new() -> Self {
        Self {
            state: SettingsState::default(),
            visible: false,
            active_tab: SettingsTab::General,
            selected_index: None,
            hovered_target: None,
            status_notice: None,
            open_spring: SpringValue::snappy(0.0),
            width: SETTINGS_WIDTH,
            height: SETTINGS_HEIGHT,
        }
    }

    pub fn open(&mut self) {
        self.visible = true;
        if self.selected_index.is_none() {
            self.selected_index = Some(0);
        }
        self.open_spring.snap_to(1.0);
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.hovered_target = None;
        self.open_spring.snap_to(0.0);
    }

    pub fn toggle(&mut self) {
        if self.visible {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn select_tab(&mut self, tab: SettingsTab) {
        self.active_tab = tab;
        self.selected_index = Some(0);
    }

    pub fn set_status_notice(&mut self, msg: Option<String>) {
        self.status_notice = msg;
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let was_settled = self.open_spring.is_settled();
        self.open_spring.tick(dt);
        !was_settled || !self.open_spring.is_settled()
    }

    pub fn is_animating(&self) -> bool {
        !self.open_spring.is_settled()
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

    /// Hit-tests a click at `(x, y)`, mutates `self.state` / `self.active_tab`, and returns
    /// the [`SettingsAction`] for the host shell to persist and apply live.
    pub fn handle_click(&mut self, x: f32, y: f32) -> Option<SettingsAction> {
        if !self.visible {
            return None;
        }
        let target = self.hit_test_target(x, y)?;
        match target {
            HitTarget::Close => {
                self.close();
                Some(SettingsAction::Close)
            }
            HitTarget::Tab(tab) => {
                self.select_tab(tab);
                Some(SettingsAction::SelectTab(tab))
            }
            HitTarget::Row(row) => Some(self.apply_row_control(row)),
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
                self.state.notch_mode =
                    NotchMode::from_edge_and_behavior(&self.state.notch_edge, &self.state.notch_behavior);
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
                let idx = SettingsTab::ALL
                    .iter()
                    .position(|t| *t == self.active_tab)
                    .unwrap_or(0);
                let prev = if idx == 0 {
                    SettingsTab::ALL.len() - 1
                } else {
                    idx - 1
                };
                self.select_tab(SettingsTab::ALL[prev]);
                None
            }
            VK_RIGHT => {
                let idx = SettingsTab::ALL
                    .iter()
                    .position(|t| *t == self.active_tab)
                    .unwrap_or(0);
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

    fn close_rect(&self) -> Rect {
        Rect::new(self.width - 38.0, 9.0, 24.0, 24.0)
    }

    fn tab_rects(&self) -> Vec<(SettingsTab, Rect)> {
        let mut y = TITLEBAR_HEIGHT + 68.0;
        SettingsTab::ALL
            .iter()
            .map(|&tab| {
                let r = Rect::new(10.0, y, SIDEBAR_WIDTH - 20.0, 34.0);
                y += 38.0;
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
                    kind: RowVisual::Button("Reset".to_string(), ACCENT_CYAN),
                },
            ],
            SettingsTab::Notch => vec![
                RenderedRow {
                    id: RowControlId::CycleNotchBehavior,
                    title: "Notch Behavior",
                    desc: "Fixed (always visible), Smart (auto-hide), or Peek".to_string(),
                    kind: RowVisual::Pill(
                        capitalize_mode(&self.state.notch_behavior),
                        ACCENT_CYAN,
                    ),
                },
                RenderedRow {
                    id: RowControlId::CycleNotchEdge,
                    title: "Notch Screen Edge",
                    desc: "Anchor the Dynamic Island to Top, Left, or Right".to_string(),
                    kind: RowVisual::Pill(capitalize_mode(&self.state.notch_edge), ACCENT_CYAN),
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
                    kind: RowVisual::Pill(capitalize_mode(&self.state.dock_mode), ACCENT_CYAN),
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
                    desc: format!("{} pinned app(s) saved in settings.json", self.state.dock_pinned.len()),
                    kind: RowVisual::Button(
                        if self.state.dock_pinned.is_empty() {
                            "Empty".to_string()
                        } else {
                            "Clear Pinned".to_string()
                        },
                        TEXT_SECONDARY,
                    ),
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
                    kind: RowVisual::Pill(
                        self.state.active_profile.preset.label().to_string(),
                        ACCENT_CYAN,
                    ),
                },
                RenderedRow {
                    id: RowControlId::CycleRouterModel,
                    title: "Default Model",
                    desc: "Click to cycle model (Custom preset accepts typing)".to_string(),
                    kind: RowVisual::Pill(self.state.hime_router_model.clone(), TEXT_PRIMARY),
                },
                RenderedRow {
                    id: RowControlId::TestAndSaveProfile,
                    title: "9Router Gateway Status",
                    desc: if self.state.router_9_detected {
                        "9Router detected on 127.0.0.1:20128".to_string()
                    } else {
                        "Gateway offline or standby — click to test & save".to_string()
                    },
                    kind: RowVisual::Button(
                        if self.state.router_9_detected {
                            "Connected • Test".to_string()
                        } else {
                            "Test & Save".to_string()
                        },
                        if self.state.router_9_detected {
                            STATUS_SUCCESS
                        } else {
                            ACCENT_CYAN
                        },
                    ),
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
                    kind: RowVisual::Pill(
                        proactive_budget_label(&self.state.proactive_budget).to_string(),
                        ACCENT_CYAN,
                    ),
                },
                RenderedRow {
                    id: RowControlId::RefreshMemory,
                    title: "Memory",
                    desc: format!("{} entries", self.state.memory_count),
                    kind: RowVisual::SummaryCard {
                        count: self.state.memory_count,
                        action_label: "Refresh",
                        accent: ACCENT_CYAN,
                    },
                },
                RenderedRow {
                    id: RowControlId::ReloadSkills,
                    title: "Skills",
                    desc: format!("{} loaded", self.state.skills_count),
                    kind: RowVisual::SummaryCard {
                        count: self.state.skills_count,
                        action_label: "Reload",
                        accent: STATUS_SUCCESS,
                    },
                },
                RenderedRow {
                    id: RowControlId::ReloadMcp,
                    title: "MCP Servers",
                    desc: format!("{} connected", self.state.mcp_count),
                    kind: RowVisual::SummaryCard {
                        count: self.state.mcp_count,
                        action_label: "Reload",
                        accent: ACCENT_CYAN,
                    },
                },
            ],
            SettingsTab::About => vec![
                RenderedRow {
                    id: RowControlId::AboutInfo,
                    title: "Hime Island Native",
                    desc: "Pure Rust Direct2D + DirectWrite + DirectComposition".to_string(),
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
            let mut y = TITLEBAR_HEIGHT + 70.0; // 112.0 (contains y=130 for row 0)
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
        let draw_w = if sw > 10.0 { sw.min(self.width) } else { self.width };
        let draw_h = if sh > 10.0 { sh.min(self.height) } else { self.height };
        (draw_w, draw_h)
    }

    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        let (sw, sh) = surface.dip_size();
        let (draw_w, draw_h) = self.render_size_for_surface(sw, sh);

        let panel_rect = Rect::new(0.0, 0.0, draw_w, draw_h);
        surface.set_rounded_region(panel_rect, 14.0);
        surface.fill_rounded_rect(panel_rect, 14.0, NOTCH_BG_SOLID.with_alpha(0.96))?;
        surface.stroke_rounded_rect(panel_rect, 14.0, GLASS_BORDER, 1.0)?;

        // Title bar
        surface.draw_text(
            "SETTINGS",
            Rect::new(18.0, 8.0, 140.0, 26.0),
            11.5,
            TEXT_PRIMARY.with_alpha(0.75),
            TextAlign::Leading,
        )?;
        let close_r = Rect::new(draw_w - 38.0, 9.0, 24.0, 24.0);
        let close_bg = if self.hovered_target == Some(HitTarget::Close) {
            SURFACE_ELEVATED
        } else {
            GLASS_SURFACE.with_alpha(0.45)
        };
        surface.fill_rounded_rect(close_r, 12.0, close_bg)?;
        surface.draw_text("X", close_r, 11.5, TEXT_PRIMARY, TextAlign::Center)?;

        // Sidebar background & divider
        if draw_h > TITLEBAR_HEIGHT + 20.0 {
            let sidebar_bg = Rect::new(1.0, TITLEBAR_HEIGHT, SIDEBAR_WIDTH - 1.0, draw_h - TITLEBAR_HEIGHT - 1.0);
            surface.fill_rounded_rect(sidebar_bg, 0.0, DOCK_SURFACE.with_alpha(0.45))?;
            surface.fill_rounded_rect(
                Rect::new(SIDEBAR_WIDTH, TITLEBAR_HEIGHT, 1.0, draw_h - TITLEBAR_HEIGHT),
                0.0,
                GLASS_BORDER,
            )?;

            // Sidebar header
            surface.draw_text(
                "Hime Island",
                Rect::new(18.0, TITLEBAR_HEIGHT + 10.0, SIDEBAR_WIDTH - 24.0, 22.0),
                15.0,
                TEXT_PRIMARY,
                TextAlign::Leading,
            )?;
            surface.draw_text(
                "Make it yours",
                Rect::new(18.0, TITLEBAR_HEIGHT + 32.0, SIDEBAR_WIDTH - 24.0, 16.0),
                11.0,
                TEXT_SECONDARY,
                TextAlign::Leading,
            )?;

            // Sidebar tabs
            for (tab, rect) in self.tab_rects() {
                if rect.y + rect.height > draw_h - 8.0 {
                    continue;
                }
                let is_active = self.active_tab == tab;
                let is_hovered = self.hovered_target == Some(HitTarget::Tab(tab));
                if is_active {
                    surface.fill_rounded_rect(rect, 8.0, SURFACE_ELEVATED)?;
                    surface.stroke_rounded_rect(rect, 8.0, GLASS_BORDER, 1.0)?;
                    surface.fill_rounded_rect(
                        Rect::new(rect.x + 3.0, rect.y + 8.0, 3.0, rect.height - 16.0),
                        1.5,
                        ACCENT_CYAN,
                    )?;
                } else if is_hovered {
                    surface.fill_rounded_rect(rect, 8.0, SURFACE_ELEVATED.with_alpha(0.55))?;
                }
                let text_color = if is_active || is_hovered {
                    TEXT_PRIMARY
                } else {
                    TEXT_PRIMARY.with_alpha(0.84)
                };
                surface.draw_text(
                    tab.label(),
                    Rect::new(rect.x + 14.0, rect.y, rect.width - 20.0, rect.height),
                    12.5,
                    text_color,
                    TextAlign::Leading,
                )?;
            }

            // Content header
            let content_x = SIDEBAR_WIDTH + CONTENT_PAD_X;
            let content_w = (draw_w - content_x - CONTENT_PAD_X).max(120.0);
            surface.draw_text(
                self.active_tab.label(),
                Rect::new(content_x, TITLEBAR_HEIGHT + 10.0, content_w, 26.0),
                18.0,
                TEXT_PRIMARY,
                TextAlign::Leading,
            )?;
            surface.draw_text(
                self.active_tab.subtitle(),
                Rect::new(content_x, TITLEBAR_HEIGHT + 38.0, content_w, 18.0),
                11.5,
                TEXT_SECONDARY,
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
                let r = if matches!(row.kind, RowVisual::SummaryCard { .. }) {
                    Rect::new(
                        content_x + (rect.x - content_x) * width_scale,
                        rect.y,
                        rect.width * width_scale,
                        rect.height,
                    )
                } else {
                    Rect::new(rect.x, rect.y, content_w, rect.height)
                };
                let hovered = self.hovered_target == Some(HitTarget::Row(row.id));
                let selected = self.selected_index == Some(idx);

                surface.fill_rounded_rect(
                    r,
                    10.0,
                    if hovered {
                        SURFACE_ELEVATED
                    } else {
                        SURFACE_ELEVATED.with_alpha(0.82)
                    },
                )?;
                if selected {
                    surface.stroke_rounded_rect(r, 10.0, ACCENT_CYAN.with_alpha(0.8), 1.2)?;
                } else {
                    surface.stroke_rounded_rect(r, 10.0, GLASS_BORDER, 1.0)?;
                }

                if let RowVisual::SummaryCard {
                    count,
                    action_label,
                    accent,
                } = row.kind
                {
                    surface.draw_text(
                        row.title,
                        Rect::new(r.x + 10.0, r.y + 8.0, (r.width - 66.0).max(40.0), 18.0),
                        11.5,
                        TEXT_PRIMARY,
                        TextAlign::Leading,
                    )?;
                    let badge = Rect::new(r.x + r.width - 50.0, r.y + 7.0, 40.0, 20.0);
                    surface.fill_rounded_rect(badge, 6.0, accent.with_alpha(0.18))?;
                    surface.draw_text(
                        &count.to_string(),
                        badge,
                        11.0,
                        accent,
                        TextAlign::Center,
                    )?;

                    let btn = Rect::new(r.x + 10.0, r.y + 35.0, (r.width - 20.0).max(40.0), 22.0);
                    surface.fill_rounded_rect(btn, 6.0, accent.with_alpha(0.14))?;
                    surface.stroke_rounded_rect(btn, 6.0, accent.with_alpha(0.42), 1.0)?;
                    surface.draw_text(
                        action_label,
                        btn,
                        11.0,
                        TEXT_PRIMARY,
                        TextAlign::Center,
                    )?;
                    continue;
                }

                // Pure white title & readable secondary subtitle
                surface.draw_text(
                    row.title,
                    Rect::new(r.x + 14.0, r.y + 5.0, r.width * 0.62, 18.0),
                    12.0,
                    TEXT_PRIMARY,
                    TextAlign::Leading,
                )?;
                surface.draw_text(
                    &row.desc,
                    Rect::new(r.x + 14.0, r.y + 22.0, r.width * 0.64, 16.0),
                    10.5,
                    TEXT_SECONDARY,
                    TextAlign::Leading,
                )?;

                // Right-hand interactive control
                match row.kind {
                    RowVisual::Toggle(on) => {
                        let sw_w = 40.0;
                        let sw_h = 22.0;
                        let sw_x = r.x + r.width - sw_w - 14.0;
                        let sw_y = r.y + (r.height - sw_h) * 0.5;
                        let track_color = if on { ACCENT_CYAN } else { STATUS_OFFLINE.with_alpha(0.55) };
                        surface.fill_rounded_rect(
                            Rect::new(sw_x, sw_y, sw_w, sw_h),
                            sw_h * 0.5,
                            track_color,
                        )?;
                        let knob_d = 16.0;
                        let knob_x = if on {
                            sw_x + sw_w - knob_d - 3.0
                        } else {
                            sw_x + 3.0
                        };
                        surface.fill_rounded_rect(
                            Rect::new(knob_x, sw_y + 3.0, knob_d, knob_d),
                            knob_d * 0.5,
                            TEXT_PRIMARY,
                        )?;
                    }
                    RowVisual::Pill(val, accent) | RowVisual::Button(val, accent) => {
                        let pill_w = 138.0_f32.min(r.width * 0.34);
                        let pill_h = 26.0;
                        let pill_rect = Rect::new(
                            r.x + r.width - pill_w - 14.0,
                            r.y + (r.height - pill_h) * 0.5,
                            pill_w,
                            pill_h,
                        );
                        surface.fill_rounded_rect(
                            pill_rect,
                            8.0,
                            accent.with_alpha(0.16),
                        )?;
                        surface.stroke_rounded_rect(
                            pill_rect,
                            8.0,
                            accent.with_alpha(0.45),
                            1.0,
                        )?;
                        surface.draw_text(
                            &val,
                            pill_rect,
                            11.5,
                            TEXT_PRIMARY,
                            TextAlign::Center,
                        )?;
                    }
                    RowVisual::Info(val) => {
                        surface.draw_text(
                            &val,
                            Rect::new(r.x + r.width * 0.65, r.y, r.width * 0.35 - 14.0, r.height),
                            11.5,
                            TEXT_PRIMARY,
                            TextAlign::Trailing,
                        )?;
                    }
                    RowVisual::SummaryCard { .. } => {}
                }
            }

            if let Some(notice) = &self.status_notice {
                if draw_h > 470.0 {
                    surface.draw_text(
                        notice,
                        Rect::new(content_x, draw_h - 30.0, content_w, 20.0),
                        11.0,
                        STATUS_SUCCESS,
                        TextAlign::Leading,
                    )?;
                }
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

        // Click Notch tab in left sidebar (index 1 -> y = 42 + 68 + 38 = 148..182)
        assert_eq!(
            panel.handle_click(60.0, 160.0),
            Some(SettingsAction::SelectTab(SettingsTab::Notch))
        );
        assert_eq!(panel.active_tab, SettingsTab::Notch);

        // First row on Notch tab: Notch Behavior (y = 42 + 74 = 116..162)
        assert_eq!(
            panel.handle_click(300.0, 130.0),
            Some(SettingsAction::ToggleNotchMode)
        );
        assert_eq!(panel.state.notch_behavior, "fixed");

        // Third row on Notch tab: Free Drag (y = 116 + 2 * 54 = 224..270)
        assert_eq!(
            panel.handle_click(300.0, 240.0),
            Some(SettingsAction::ToggleFreeDrag)
        );
        assert!(panel.state.free_drag_enabled);

        // Switch to Dock tab (index 2 -> y = 42 + 68 + 2*38 = 186..220)
        assert_eq!(
            panel.handle_click(60.0, 200.0),
            Some(SettingsAction::SelectTab(SettingsTab::Dock))
        );
        // Second row on Dock tab: Dock Behavior (y = 116 + 54 = 170..216)
        assert_eq!(
            panel.handle_click(300.0, 185.0),
            Some(SettingsAction::ToggleDockMode)
        );
        assert_eq!(panel.state.dock_mode, "smart");

        // Switch to Hime AI tab (index 4 -> y = 42 + 68 + 4*38 = 262..296)
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
