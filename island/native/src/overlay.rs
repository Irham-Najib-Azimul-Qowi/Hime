//! Native Volume & Brightness HUD overlay (`OverlayHud`) rendered via Direct2D.

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::SpringValue;
use crate::d2d::{NativeSurface, Rect, TextAlign};
use crate::theme::{
    ACCENT_CYAN, CORNER_RADIUS, GLASS_BORDER, GLASS_SURFACE, STATUS_OFFLINE, SURFACE_ELEVATED,
    TEXT_PRIMARY, TEXT_SECONDARY,
};

const DEFAULT_AUTO_HIDE_SECS: f32 = 2.2;
const HUD_WIDTH: f32 = 260.0;
const HUD_HEIGHT: f32 = 72.0;
const HUD_EDGE_MARGIN: f32 = 20.0;

/// Screen edge where the HUD appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HudEdge {
    Left,
    #[default]
    Right,
}

/// Kind of system metric displayed by the HUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HudKind {
    #[default]
    Volume,
    Brightness,
}

/// Native Direct2D HUD for Volume and Brightness feedback with spring auto-hide.
#[derive(Debug, Clone)]
pub struct OverlayHud {
    pub edge: HudEdge,
    pub kind: HudKind,
    pub percent: u8,
    pub muted: bool,
    pub device_name: Option<String>,
    pub visibility: SpringValue,
    pub bar_fill: SpringValue,
    auto_hide_timer: f32,
    auto_hide_duration: f32,
}

impl Default for OverlayHud {
    fn default() -> Self {
        Self::new(HudEdge::Right)
    }
}

impl OverlayHud {
    /// Creates a hidden `OverlayHud` anchored to `edge`.
    pub fn new(edge: HudEdge) -> Self {
        Self {
            edge,
            kind: HudKind::Volume,
            percent: 0,
            muted: false,
            device_name: None,
            visibility: SpringValue::gentle(0.0),
            bar_fill: SpringValue::snappy(0.0),
            auto_hide_timer: 0.0,
            auto_hide_duration: DEFAULT_AUTO_HIDE_SECS,
        }
    }

    /// Shows or updates the HUD in Volume mode and resets the auto-hide timer.
    pub fn show_volume(&mut self, percent: u8, muted: bool, device_name: Option<String>) {
        let clamped = percent.min(100);
        self.kind = HudKind::Volume;
        self.percent = clamped;
        self.muted = muted;
        self.device_name = device_name.filter(|s| !s.trim().is_empty());
        self.visibility.set_target(1.0);
        self.bar_fill
            .set_target(if muted { 0.0 } else { clamped as f32 / 100.0 });
        self.auto_hide_timer = self.auto_hide_duration;
    }

    /// Shows or updates the HUD in Brightness mode and resets the auto-hide timer.
    pub fn show_brightness(&mut self, percent: u8) {
        let clamped = percent.min(100);
        self.kind = HudKind::Brightness;
        self.percent = clamped;
        self.muted = false;
        self.device_name = None;
        self.visibility.set_target(1.0);
        self.bar_fill.set_target(clamped as f32 / 100.0);
        self.auto_hide_timer = self.auto_hide_duration;
    }

    /// Triggers an immediate animated hide.
    pub fn hide(&mut self) {
        self.auto_hide_timer = 0.0;
        self.visibility.set_target(0.0);
    }

    /// Advances auto-hide timer and spring animations by `dt` seconds.
    /// Returns `true` while still visible or animating.
    pub fn tick(&mut self, dt: f32) -> bool {
        if dt > 0.0 && dt.is_finite() && self.auto_hide_timer > 0.0 {
            self.auto_hide_timer = (self.auto_hide_timer - dt).max(0.0);
            if self.auto_hide_timer == 0.0 {
                self.visibility.set_target(0.0);
            }
        }
        self.visibility.tick(dt);
        self.bar_fill.tick(dt);
        self.is_visible() || !self.visibility.is_settled() || !self.bar_fill.is_settled()
    }

    /// Returns `true` if the HUD has non-negligible opacity or is opening.
    pub fn is_visible(&self) -> bool {
        self.visibility.value > 0.01 || self.visibility.target > 0.01
    }

    /// Computes the HUD bounding rectangle inside a surface of `(width, height)` DIPs.
    pub fn computed_rect(&self, surface_width: f32, surface_height: f32) -> Rect {
        let vis = self.visibility.value.clamp(0.0, 1.0);
        let slide_offset = (1.0 - vis) * 24.0;
        let x = match self.edge {
            HudEdge::Left => HUD_EDGE_MARGIN - slide_offset,
            HudEdge::Right => (surface_width - HUD_WIDTH - HUD_EDGE_MARGIN) + slide_offset,
        };
        let y = (surface_height - HUD_HEIGHT) * 0.5;
        Rect::new(x, y.max(8.0), HUD_WIDTH, HUD_HEIGHT)
    }

    /// Renders the HUD onto `surface` using Direct2D/DirectWrite.
    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let alpha = self.visibility.value.clamp(0.0, 1.0);
        if alpha <= 0.01 {
            return Ok(());
        }

        let (sw, sh) = surface.size();
        let rect = self.computed_rect(sw as f32, sh as f32);

        surface.fill_rounded_rect(rect, CORNER_RADIUS, GLASS_SURFACE.with_alpha(0.92 * alpha))?;
        surface.stroke_rounded_rect(
            rect,
            CORNER_RADIUS,
            GLASS_BORDER.with_alpha(0.18 * alpha),
            1.0,
        )?;

        let title = match self.kind {
            HudKind::Volume if self.muted => "Muted",
            HudKind::Volume => "Volume",
            HudKind::Brightness => "Brightness",
        };

        let subtitle = match self.kind {
            HudKind::Volume => self
                .device_name
                .as_deref()
                .unwrap_or("Default Audio Device"),
            HudKind::Brightness => "Display",
        };

        let header_rect = Rect::new(rect.x + 16.0, rect.y + 10.0, rect.width - 76.0, 18.0);
        surface.draw_text(
            title,
            header_rect,
            13.0,
            TEXT_PRIMARY.with_alpha(alpha),
            TextAlign::Leading,
        )?;

        let pct_text = if self.kind == HudKind::Volume && self.muted {
            "0%".to_string()
        } else {
            format!("{}%", self.percent)
        };
        let pct_rect = Rect::new(rect.x + rect.width - 64.0, rect.y + 10.0, 48.0, 18.0);
        surface.draw_text(
            &pct_text,
            pct_rect,
            13.0,
            TEXT_PRIMARY.with_alpha(alpha),
            TextAlign::Trailing,
        )?;

        let sub_rect = Rect::new(rect.x + 16.0, rect.y + 28.0, rect.width - 32.0, 16.0);
        surface.draw_text(
            subtitle,
            sub_rect,
            11.0,
            TEXT_SECONDARY.with_alpha(alpha),
            TextAlign::Leading,
        )?;

        // Progress track & fill bar
        let track_rect = Rect::new(rect.x + 16.0, rect.y + 50.0, rect.width - 32.0, 8.0);
        surface.fill_rounded_rect(track_rect, 4.0, SURFACE_ELEVATED.with_alpha(alpha))?;

        let fill_ratio = self.bar_fill.value.clamp(0.0, 1.0);
        if fill_ratio > 0.01 {
            let fill_color = if self.kind == HudKind::Volume && self.muted {
                STATUS_OFFLINE
            } else {
                ACCENT_CYAN
            };
            let fill_rect = Rect::new(
                track_rect.x,
                track_rect.y,
                (track_rect.width * fill_ratio).max(6.0),
                track_rect.height,
            );
            surface.fill_rounded_rect(fill_rect, 4.0, fill_color.with_alpha(alpha))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_and_brightness_clamp_and_set_targets() {
        let mut hud = OverlayHud::new(HudEdge::Left);
        assert!(!hud.is_visible());

        hud.show_volume(140, false, Some("Headphones (USB)".into()));
        assert!(hud.is_visible());
        assert_eq!(hud.kind, HudKind::Volume);
        assert_eq!(hud.percent, 100);
        assert!(!hud.muted);
        assert_eq!(hud.device_name.as_deref(), Some("Headphones (USB)"));
        assert_eq!(hud.bar_fill.target, 1.0);

        hud.show_volume(55, true, Some("   ".into()));
        assert!(hud.muted);
        assert_eq!(hud.device_name, None);
        assert_eq!(hud.bar_fill.target, 0.0);

        hud.show_brightness(80);
        assert_eq!(hud.kind, HudKind::Brightness);
        assert_eq!(hud.percent, 80);
        assert!(!hud.muted);
        assert_eq!(hud.device_name, None);
        assert!((hud.bar_fill.target - 0.8).abs() < 1e-5);
    }

    #[test]
    fn auto_hide_timer_collapses_hud_after_timeout() {
        let mut hud = OverlayHud::new(HudEdge::Right);
        hud.show_brightness(60);

        // Advance partway; HUD should remain visible.
        for _ in 0..30 {
            hud.tick(1.0 / 60.0);
        }
        assert!(hud.is_visible());
        assert!(hud.visibility.value > 0.5);

        // Advance past auto-hide duration + settle time.
        for _ in 0..240 {
            hud.tick(1.0 / 60.0);
        }
        assert!(!hud.is_visible());
        assert_eq!(hud.visibility.value, 0.0);
    }

    #[test]
    fn computed_rect_respects_left_and_right_edges() {
        let mut left_hud = OverlayHud::new(HudEdge::Left);
        left_hud.visibility.snap_to(1.0);
        let lr = left_hud.computed_rect(1920.0, 1080.0);
        assert_eq!(lr.x, HUD_EDGE_MARGIN);

        let mut right_hud = OverlayHud::new(HudEdge::Right);
        right_hud.visibility.snap_to(1.0);
        let rr = right_hud.computed_rect(1920.0, 1080.0);
        assert_eq!(rr.x, 1920.0 - HUD_WIDTH - HUD_EDGE_MARGIN);
    }
}
