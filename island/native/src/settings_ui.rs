//! Native Settings panel rendered with Direct2D (`NativeSurface`).
//! Covers Notch mode, Dock behavior, Notification suppression,
//! "Enable Hime AI" toggle, and API Profile setup (`9Router`, `Gemini API`, `OpenAI`, `Custom`).

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::SpringValue;
use crate::d2d::{NativeSurface, Rect, TextAlign};
use crate::theme::{
    ACCENT_CYAN, GLASS_BORDER, GLASS_SURFACE, STATUS_OFFLINE, STATUS_SUCCESS, SURFACE_ELEVATED,
    TEXT_PRIMARY, TEXT_SECONDARY,
};

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
            Self::NineRouter => "9Router (127.0.0.1:20128)",
            Self::GeminiApi => "Gemini API",
            Self::OpenAi => "OpenAI",
            Self::Custom => "Custom OpenAI-Compat",
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
            Self::NineRouter => "gpt-4o-mini",
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettingsState {
    pub notch_mode: NotchMode,
    pub dock_enabled: bool,
    pub dock_win_number_enabled: bool,
    pub suppress_native_notifications: bool,
    pub hime_ai_enabled: bool,
    pub router_9_detected: bool,
    pub active_profile: ApiProfileConfig,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            notch_mode: NotchMode::Top,
            dock_enabled: true,
            dock_win_number_enabled: true,
            suppress_native_notifications: false,
            hime_ai_enabled: false,
            router_9_detected: false,
            active_profile: ApiProfileConfig::from_preset(ProviderPreset::NineRouter),
        }
    }
}

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

/// Interactive native Settings panel.
pub struct SettingsPanel {
    pub state: SettingsState,
    pub visible: bool,
    open_spring: SpringValue,
    width: f32,
    height: f32,
}

impl SettingsPanel {
    pub fn new() -> Self {
        Self {
            state: SettingsState::default(),
            visible: false,
            open_spring: SpringValue::snappy(0.0),
            width: 420.0,
            height: 440.0,
        }
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.open_spring.set_target(1.0);
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.open_spring.set_target(0.0);
    }

    pub fn toggle(&mut self) {
        if self.visible {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let was_settled = self.open_spring.is_settled();
        self.open_spring.tick(dt);
        !was_settled || !self.open_spring.is_settled()
    }

    pub fn is_animating(&self) -> bool {
        !self.open_spring.is_settled()
    }

    /// Applies a control action to the local `SettingsState`.
    pub fn apply_control(&mut self, control: SettingsControl) {
        match control {
            SettingsControl::CycleNotchMode => {
                self.state.notch_mode = self.state.notch_mode.next();
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
            }
            SettingsControl::TestAndSaveProfile => {}
            SettingsControl::Close => self.close(),
        }
    }

    fn row_rects(&self) -> Vec<(SettingsControl, Rect)> {
        let pad = 20.0;
        let row_w = self.width - pad * 2.0;
        let row_h = 38.0;
        let mut y = 58.0;
        let controls = [
            SettingsControl::CycleNotchMode,
            SettingsControl::ToggleDock,
            SettingsControl::ToggleDockWinNumber,
            SettingsControl::ToggleNotifSuppression,
            SettingsControl::ToggleHimeAi,
            SettingsControl::CycleProviderPreset,
            SettingsControl::TestAndSaveProfile,
        ];
        let mut out = Vec::with_capacity(controls.len() + 1);
        out.push((
            SettingsControl::Close,
            Rect::new(self.width - 42.0, 14.0, 28.0, 28.0),
        ));
        for ctrl in controls {
            out.push((ctrl, Rect::new(pad, y, row_w, row_h)));
            y += row_h + 8.0;
        }
        out
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<SettingsControl> {
        if !self.visible {
            return None;
        }
        for (ctrl, r) in self.row_rects() {
            if x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height {
                return Some(ctrl);
            }
        }
        None
    }

    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let progress = self.open_spring.value.clamp(0.0, 1.0);
        if progress <= 0.01 && !self.visible {
            return Ok(());
        }

        let panel_rect = Rect::new(0.0, 0.0, self.width, self.height * progress.max(0.2));
        surface.fill_rounded_rect(panel_rect, 18.0, GLASS_SURFACE)?;
        surface.stroke_rounded_rect(panel_rect, 18.0, GLASS_BORDER, 1.0)?;

        surface.draw_text(
            "Hime Settings",
            Rect::new(20.0, 14.0, 220.0, 28.0),
            15.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            "X",
            Rect::new(self.width - 42.0, 14.0, 28.0, 28.0),
            13.0,
            TEXT_SECONDARY,
            TextAlign::Center,
        )?;

        for (ctrl, rect) in self.row_rects() {
            if ctrl == SettingsControl::Close || rect.y + rect.height > panel_rect.height {
                continue;
            }
            surface.fill_rounded_rect(rect, 10.0, SURFACE_ELEVATED)?;
            let (label, value, value_color) = match ctrl {
                SettingsControl::CycleNotchMode => (
                    "Notch Mode",
                    self.state.notch_mode.label().to_string(),
                    ACCENT_CYAN,
                ),
                SettingsControl::ToggleDock => (
                    "Dock Bar",
                    on_off(self.state.dock_enabled).to_string(),
                    toggle_color(self.state.dock_enabled),
                ),
                SettingsControl::ToggleDockWinNumber => (
                    "Win+1..9 Dock Switch",
                    on_off(self.state.dock_win_number_enabled).to_string(),
                    toggle_color(self.state.dock_win_number_enabled),
                ),
                SettingsControl::ToggleNotifSuppression => (
                    "Suppress Native Notifs",
                    on_off(self.state.suppress_native_notifications).to_string(),
                    toggle_color(self.state.suppress_native_notifications),
                ),
                SettingsControl::ToggleHimeAi => (
                    "Enable Hime AI",
                    on_off(self.state.hime_ai_enabled).to_string(),
                    toggle_color(self.state.hime_ai_enabled),
                ),
                SettingsControl::CycleProviderPreset => (
                    "AI Provider Preset",
                    self.state.active_profile.preset.label().to_string(),
                    ACCENT_CYAN,
                ),
                SettingsControl::TestAndSaveProfile => {
                    let status = if self.state.router_9_detected {
                        "Save & Test (9Router Ready)".to_string()
                    } else {
                        format!("Save & Test ({})", self.state.active_profile.model)
                    };
                    ("API Profile", status, ACCENT_CYAN)
                }
                SettingsControl::Close => unreachable!(),
            };

            surface.draw_text(
                label,
                Rect::new(rect.x + 12.0, rect.y, rect.width * 0.52, rect.height),
                12.5,
                TEXT_PRIMARY,
                TextAlign::Leading,
            )?;
            surface.draw_text(
                &value,
                Rect::new(
                    rect.x + rect.width * 0.48,
                    rect.y,
                    rect.width * 0.52 - 12.0,
                    rect.height,
                ),
                12.0,
                value_color,
                TextAlign::Trailing,
            )?;
        }

        Ok(())
    }
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

fn on_off(b: bool) -> &'static str {
    if b {
        "ON"
    } else {
        "OFF"
    }
}

fn toggle_color(b: bool) -> crate::theme::Color {
    if b {
        STATUS_SUCCESS
    } else {
        STATUS_OFFLINE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_state_serde_roundtrip() {
        let mut s = SettingsState::default();
        s.hime_ai_enabled = true;
        s.notch_mode = NotchMode::Left;
        s.active_profile = ApiProfileConfig::from_preset(ProviderPreset::GeminiApi);

        let json = serde_json::to_string(&s).unwrap();
        let parsed: SettingsState = serde_json::from_str(&json).unwrap();
        assert_eq!(s, parsed);
    }

    #[test]
    fn provider_presets_and_controls_update_state() {
        let mut panel = SettingsPanel::new();
        panel.open();
        assert!(!panel.state.hime_ai_enabled);
        panel.apply_control(SettingsControl::ToggleHimeAi);
        assert!(panel.state.hime_ai_enabled);

        panel.apply_control(SettingsControl::CycleProviderPreset);
        assert_eq!(panel.state.active_profile.preset, ProviderPreset::GeminiApi);
        assert_eq!(
            panel.state.active_profile.adapter_kind_str(),
            "GeminiNative"
        );
    }

    #[test]
    fn hit_test_detects_rows_when_visible() {
        let mut panel = SettingsPanel::new();
        assert_eq!(panel.hit_test(100.0, 70.0), None);
        panel.open();
        assert_eq!(
            panel.hit_test(100.0, 70.0),
            Some(SettingsControl::CycleNotchMode)
        );
        assert_eq!(panel.hit_test(390.0, 24.0), Some(SettingsControl::Close));
    }
}

impl ApiProfileConfig {
    pub fn adapter_kind_str(&self) -> &'static str {
        self.preset.adapter_kind()
    }
}
