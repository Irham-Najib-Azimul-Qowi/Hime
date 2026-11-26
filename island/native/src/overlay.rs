//! Native Dual Edge Overlays (`OverlayHud`, `DualEdgeOverlays`) rendered via Direct2D.
//!
//! Follows Apple Liquid Glass, AntiSlop contrast, and Motion HIG:
//! - **Minimized / Idle State:** 100% invisible (0 alpha, empty 0x0 region, click-through).
//! - **Left Edge (`HudEdge::Left`, `HudKind::Volume`):** Smoothly emerges (`Spring::EMERGE`)
//!   flush against the left screen edge with curved flare ears (`ear_w = 13`, `flares = true`)
//!   on edge hover, scroll, or volume OSD change, displaying only percentage, monochrome
//!   Liquid Glass slider bar, and `SfSymbol` speaker icon.
//! - **Right Edge (`HudEdge::Right`, `HudKind::Brightness`):** Smoothly emerges flush against
//!   the right screen edge with curved flare ears, displaying percentage, monochrome Liquid
//!   Glass slider bar, and rotating `draw_brightness_sun` icon.

use std::f32::consts::TAU;

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::{Spring, SpringValue};
use crate::d2d::{EdgeSide, LiquidGlassStyle, NativeSurface, Rect, SfSymbol, TextAlign};

const DEFAULT_AUTO_HIDE_SECS: f32 = 2.5;
const EXPANDED_PANEL_WIDTH: f32 = 64.0;
const EXPANDED_PANEL_HEIGHT: f32 = 216.0;
const EDGE_EAR_WIDTH: f32 = 13.0;
const PANEL_CORNER_RADIUS: f32 = 18.0;
const SLIDER_TRACK_WIDTH: f32 = 12.0;
const SLIDER_HIT_MARGIN_X: f32 = 12.0;
const SLIDER_HIT_MARGIN_Y: f32 = 6.0;
const EDGE_WAKE_WIDTH: f32 = 6.0;
const EDGE_WAKE_HEIGHT: f32 = 140.0;
const EDGE_HOVER_BRIDGE: f32 = 8.0;
const MIN_EMERGE_WIDTH_FRAC: f32 = 0.45;
const MIN_EMERGE_HEIGHT_FRAC: f32 = 0.86;

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

/// Native Direct2D edge-anchored HUD for Volume (left) and Brightness (right).
///
/// Completely invisible (`0` alpha / empty region / click-through) when idle; emerges
/// with Apple `Spring::EMERGE` into a screen-edge flared `64x216` Liquid Glass slider
/// contour on edge hover or OSD updates, and collapses cleanly with `Spring::RECEDE`
/// on timeout or outside click.
#[derive(Debug, Clone)]
pub struct OverlayHud {
    pub edge: HudEdge,
    pub kind: HudKind,
    pub percent: u8,
    pub muted: bool,
    pub device_name: Option<String>,
    pub enabled: bool,
    pub is_hovered: bool,
    pub is_dragging: bool,
    pub visibility: SpringValue,
    pub expand: SpringValue,
    pub bar_fill: SpringValue,
    pub sun_angle: SpringValue,
    auto_hide_timer: f32,
    auto_hide_duration: f32,
}

impl Default for OverlayHud {
    fn default() -> Self {
        Self::new(HudEdge::Right)
    }
}

impl OverlayHud {
    /// Creates an enabled edge HUD anchored to `edge` (`Left` -> Volume, `Right` -> Brightness),
    /// resting completely invisible until woken by hover or OSD change.
    pub fn new(edge: HudEdge) -> Self {
        let kind = match edge {
            HudEdge::Left => HudKind::Volume,
            HudEdge::Right => HudKind::Brightness,
        };
        Self::new_with_kind(edge, kind)
    }

    /// Creates a left-edge Volume HUD.
    pub fn volume_left() -> Self {
        Self::new_with_kind(HudEdge::Left, HudKind::Volume)
    }

    /// Creates a right-edge Brightness HUD.
    pub fn brightness_right() -> Self {
        Self::new_with_kind(HudEdge::Right, HudKind::Brightness)
    }

    /// Creates an edge HUD with an explicit `edge` and `kind`.
    pub fn new_with_kind(edge: HudEdge, kind: HudKind) -> Self {
        Self {
            edge,
            kind,
            percent: 50,
            muted: false,
            device_name: None,
            enabled: true,
            is_hovered: false,
            is_dragging: false,
            visibility: SpringValue::emerge(0.0),
            expand: SpringValue::emerge(0.0),
            bar_fill: SpringValue::knob_settle(0.5),
            sun_angle: SpringValue::from_spring(0.5 * TAU, Spring::INTERACTIVE),
            auto_hide_timer: 0.0,
            auto_hide_duration: DEFAULT_AUTO_HIDE_SECS,
        }
    }

    /// Enables or disables this edge overlay. When disabled, it immediately collapses and hides.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.hide();
        }
    }

    /// Shows or updates the HUD in Volume mode and expands it with an auto-hide timer.
    pub fn show_volume(&mut self, percent: u8, muted: bool, device_name: Option<String>) {
        let clamped = percent.min(100);
        self.kind = HudKind::Volume;
        self.percent = clamped;
        self.muted = muted;
        self.device_name = sanitize_device_name(device_name.as_deref());
        let fill = if muted { 0.0 } else { clamped as f32 / 100.0 };
        self.bar_fill.animate_to(fill, Spring::KNOB_SETTLE);
        if self.enabled {
            self.launch_open();
            self.auto_hide_timer = self.auto_hide_duration;
        }
    }

    /// Shows or updates the HUD in Brightness mode and expands it with an auto-hide timer.
    pub fn show_brightness(&mut self, percent: u8) {
        let clamped = percent.min(100);
        self.kind = HudKind::Brightness;
        self.percent = clamped;
        self.muted = false;
        self.device_name = None;
        let ratio = clamped as f32 / 100.0;
        self.bar_fill.animate_to(ratio, Spring::KNOB_SETTLE);
        self.sun_angle.animate_to(ratio * TAU, Spring::KNOB_SETTLE);
        if self.enabled {
            self.launch_open();
            self.auto_hide_timer = self.auto_hide_duration;
        }
    }

    /// Updates hover state. Hovering the edge wakes and emerges the edge-flared Liquid Glass HUD.
    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
        if hovered {
            if self.enabled {
                self.launch_open();
            }
        } else if !self.is_dragging && self.auto_hide_timer <= 0.0 {
            self.launch_close();
        }
    }

    /// Updates active slider drag state, keeping the panel open while dragging.
    pub fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
        if dragging {
            if self.enabled {
                self.launch_open();
            }
        } else {
            self.auto_hide_timer = self.auto_hide_duration;
            let fill = if self.muted {
                0.0
            } else {
                self.percent as f32 / 100.0
            };
            self.bar_fill.animate_to(fill, Spring::KNOB_SETTLE);
            if self.kind == HudKind::Brightness {
                self.sun_angle
                    .animate_to((self.percent as f32 / 100.0) * TAU, Spring::KNOB_SETTLE);
            }
        }
    }

    /// Applies a scroll-wheel delta (in detent steps, e.g. `+1.0` up / `-1.0` down),
    /// expands the HUD, resets the auto-hide timer, and returns the updated percentage.
    pub fn on_scroll(&mut self, delta_steps: f32) -> u8 {
        if !delta_steps.is_finite() {
            return self.percent;
        }
        let step = (delta_steps * 4.0).round() as i32;
        let next = (self.percent as i32 + step).clamp(0, 100) as u8;
        self.apply_percent(next);
        next
    }

    /// Sets percentage from a normalized `[0.0, 1.0]` slider ratio, expands the HUD,
    /// and returns the updated percentage `0..=100`.
    pub fn set_percent_from_ratio(&mut self, ratio: f32) -> u8 {
        let clamped_ratio = if ratio.is_finite() {
            ratio.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let next = (clamped_ratio * 100.0).round() as u8;
        self.apply_percent(next);
        next
    }

    /// Toggles mute state when in Volume mode and returns the new `muted` state.
    pub fn toggle_mute(&mut self) -> bool {
        if self.kind != HudKind::Volume {
            return false;
        }
        self.muted = !self.muted;
        let fill = if self.muted {
            0.0
        } else {
            self.percent as f32 / 100.0
        };
        self.bar_fill.animate_to(fill, Spring::KNOB_SETTLE);
        if self.enabled {
            self.launch_open();
            self.auto_hide_timer = self.auto_hide_duration;
        }
        self.muted
    }

    fn apply_percent(&mut self, next: u8) {
        let clamped = next.min(100);
        self.percent = clamped;
        if self.kind == HudKind::Volume {
            self.muted = clamped == 0;
        }
        let fill = if self.muted {
            0.0
        } else {
            clamped as f32 / 100.0
        };
        let spring = if self.is_dragging {
            Spring::INTERACTIVE
        } else {
            Spring::KNOB_SETTLE
        };
        self.bar_fill.animate_to(fill, spring);
        if self.kind == HudKind::Brightness {
            self.sun_angle
                .animate_to((clamped as f32 / 100.0) * TAU, spring);
        }
        if self.enabled {
            self.launch_open();
            self.auto_hide_timer = self.auto_hide_duration;
        }
    }

    fn launch_open(&mut self) {
        if self.expand.target < 0.5 {
            self.expand.launch_to(1.0, Spring::EMERGE, 1.0);
        }
        if self.visibility.target < 0.5 {
            self.visibility.launch_to(1.0, Spring::EMERGE, 1.0);
        }
    }

    fn launch_close(&mut self) {
        if self.expand.target > 0.5 {
            self.expand.launch_to(0.0, Spring::RECEDE, 1.0);
        }
        if self.visibility.target > 0.5 {
            self.visibility.launch_to(0.0, Spring::RECEDE, 1.0);
        }
    }

    /// Collapses the expanded HUD immediately back to invisible idle state (`Spring::RECEDE`).
    pub fn hide(&mut self) {
        self.auto_hide_timer = 0.0;
        self.is_hovered = false;
        self.is_dragging = false;
        self.launch_close();
    }

    /// Advances the auto-hide timer and Apple spring animations by `dt` seconds.
    /// Returns `true` while still animating or while an active auto-hide timer is running.
    pub fn tick(&mut self, dt: f32) -> bool {
        if dt > 0.0
            && dt.is_finite()
            && !self.is_hovered
            && !self.is_dragging
            && self.auto_hide_timer > 0.0
        {
            self.auto_hide_timer = (self.auto_hide_timer - dt).max(0.0);
            if self.auto_hide_timer == 0.0 {
                self.launch_close();
            }
        }
        self.visibility.tick(dt);
        self.expand.tick(dt);
        self.bar_fill.tick(dt);
        self.sun_angle.tick(dt);
        self.auto_hide_timer > 0.0
            || !self.visibility.is_settled()
            || !self.expand.is_settled()
            || !self.bar_fill.is_settled()
            || !self.sun_angle.is_settled()
    }

    /// Returns `true` when the edge overlay surface is awake, expanding, or receding.
    /// When idle, returns `false` so the host hides the HWND and keeps it 100% click-through.
    pub fn is_visible(&self) -> bool {
        self.enabled && self.is_expanded()
    }

    /// Returns `true` when the HUD is expanded (hovered, dragging, timer active, or animating).
    pub fn is_expanded(&self) -> bool {
        self.enabled
            && (self.is_hovered
                || self.is_dragging
                || self.auto_hide_timer > 0.0
                || self.expand.value > 0.01
                || self.expand.target > 0.5
                || self.visibility.value > 0.01
                || self.visibility.target > 0.5)
    }

    /// Invisible screen-edge hover trigger rectangle (in surface-local DIPs) used to wake
    /// the minimized overlay without rendering any visible pixels.
    pub fn wake_hit_rect(&self, surface_rect: Rect) -> Rect {
        let h = EDGE_WAKE_HEIGHT.min(surface_rect.height.max(EDGE_WAKE_HEIGHT));
        let y = surface_rect.y + ((surface_rect.height - h) * 0.5).max(0.0);
        let x = match self.edge {
            HudEdge::Left => surface_rect.x,
            HudEdge::Right => surface_rect.x + (surface_rect.width - EDGE_WAKE_WIDTH).max(0.0),
        };
        Rect::new(x, y, EDGE_WAKE_WIDTH, h)
    }

    /// Computes the current bounding rectangle: empty (`0x0`) when idle, or emerging flush
    /// from the anchored screen edge into the flared `64x216` silhouette.
    pub fn computed_rect(&self, surface_width: f32, surface_height: f32) -> Rect {
        let t = self.expand.value.clamp(0.0, 1.0);
        if !self.enabled || t <= 0.01 {
            let edge_x = match self.edge {
                HudEdge::Left => 0.0,
                HudEdge::Right => surface_width.max(0.0),
            };
            return Rect::new(edge_x, (surface_height * 0.5).max(0.0), 0.0, 0.0);
        }

        let w = EXPANDED_PANEL_WIDTH * (MIN_EMERGE_WIDTH_FRAC + (1.0 - MIN_EMERGE_WIDTH_FRAC) * t);
        let h =
            EXPANDED_PANEL_HEIGHT * (MIN_EMERGE_HEIGHT_FRAC + (1.0 - MIN_EMERGE_HEIGHT_FRAC) * t);
        let x = match self.edge {
            HudEdge::Left => 0.0,
            HudEdge::Right => (surface_width - w).max(0.0),
        };
        let y = ((surface_height - h) * 0.5).max(0.0);
        Rect::new(x, y, w, h)
    }

    /// Computes the full `64x216` expanded panel rectangle anchored flush to `self.edge`.
    pub fn expanded_rect(&self, surface_width: f32, surface_height: f32) -> Rect {
        let x = match self.edge {
            HudEdge::Left => 0.0,
            HudEdge::Right => (surface_width - EXPANDED_PANEL_WIDTH).max(0.0),
        };
        let y = ((surface_height - EXPANDED_PANEL_HEIGHT) * 0.5).max(0.0);
        Rect::new(x, y, EXPANDED_PANEL_WIDTH, EXPANDED_PANEL_HEIGHT)
    }

    /// Returns the vertical slider track rectangle when the HUD is expanded.
    pub fn slider_rect(&self, surface_rect: Rect) -> Option<Rect> {
        if !self.is_expanded() {
            return None;
        }
        let panel = if self.expand.value >= 0.45 {
            self.computed_rect(surface_rect.width, surface_rect.height)
        } else {
            self.expanded_rect(surface_rect.width, surface_rect.height)
        };
        let panel_abs_x = surface_rect.x + panel.x;
        let panel_abs_y = surface_rect.y + panel.y;
        let track_x = panel_abs_x + (panel.width - SLIDER_TRACK_WIDTH) * 0.5;
        let track_y = panel_abs_y + 44.0;
        let track_h = (panel.height - 104.0).max(40.0);
        Some(Rect::new(track_x, track_y, SLIDER_TRACK_WIDTH, track_h))
    }

    /// Hit-tests `(x, y)` against the expanded vertical slider bar, returning the
    /// normalized value in `0.0..=1.0` (`0.0` = bottom, `1.0` = top).
    pub fn hit_test_slider(&self, x: f32, y: f32, rect: Rect) -> Option<f32> {
        let track = self.slider_rect(rect)?;
        let within_x = x >= track.x - SLIDER_HIT_MARGIN_X
            && x <= track.x + track.width + SLIDER_HIT_MARGIN_X;
        let within_y = y >= track.y - SLIDER_HIT_MARGIN_Y
            && y <= track.y + track.height + SLIDER_HIT_MARGIN_Y;
        if !within_x || !within_y {
            return None;
        }
        let ratio = ((track.y + track.height - y) / track.height.max(1.0)).clamp(0.0, 1.0);
        Some(ratio)
    }

    /// Returns `true` when `(x, y)` hits the awake edge-flared HUD.
    pub fn hit_test_pill(&self, x: f32, y: f32, surface_rect: Rect) -> bool {
        if !self.is_visible() {
            return false;
        }
        let local_rect = self.computed_rect(surface_rect.width, surface_rect.height);
        if local_rect.width <= 0.0 || local_rect.height <= 0.0 {
            return false;
        }
        let rx = surface_rect.x + local_rect.x;
        let ry = surface_rect.y + local_rect.y;
        let (hx, hw) = match self.edge {
            HudEdge::Left => (surface_rect.x, local_rect.width + EDGE_HOVER_BRIDGE),
            HudEdge::Right => {
                let left_bound = (rx - EDGE_HOVER_BRIDGE).max(surface_rect.x);
                let right_bound = surface_rect.x + surface_rect.width;
                (left_bound, (right_bound - left_bound).max(0.0))
            }
        };
        let hy = ry - 6.0;
        let hh = local_rect.height + 12.0;
        x >= hx && x <= hx + hw && y >= hy && y <= hy + hh
    }

    /// Returns `true` when `(x, y)` hits the speaker mute button at the bottom of the expanded panel.
    pub fn hit_test_mute_button(&self, x: f32, y: f32, surface_rect: Rect) -> bool {
        if self.kind != HudKind::Volume || !self.is_expanded() {
            return false;
        }
        let panel = if self.expand.value >= 0.45 {
            self.computed_rect(surface_rect.width, surface_rect.height)
        } else {
            self.expanded_rect(surface_rect.width, surface_rect.height)
        };
        let icon_rect = Rect::new(
            surface_rect.x + panel.x + 8.0,
            surface_rect.y + panel.y + panel.height - 56.0,
            (panel.width - 16.0).max(24.0),
            36.0,
        );
        x >= icon_rect.x
            && x <= icon_rect.x + icon_rect.width
            && y >= icon_rect.y
            && y <= icon_rect.y + icon_rect.height
    }

    /// Renders either an invisible empty region (when idle) or the screen-edge curved-flare
    /// Liquid Glass slider contour using `surface.palette()`.
    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let vis_alpha = self.visibility.value.clamp(0.0, 1.0);
        let t = self.expand.value.clamp(0.0, 1.0);
        let edge_side = match self.edge {
            HudEdge::Left => EdgeSide::Left,
            HudEdge::Right => EdgeSide::Right,
        };

        if !self.enabled || vis_alpha <= 0.02 || t <= 0.02 {
            surface.set_edge_surface_region(
                Rect::new(0.0, 0.0, 0.0, 0.0),
                0.0,
                0.0,
                edge_side,
                false,
            );
            return Ok(());
        }

        let (sw, sh) = surface.dip_size();
        let rect = self.computed_rect(sw, sh);
        if rect.width <= 1.0 || rect.height <= 1.0 {
            surface.set_edge_surface_region(
                Rect::new(0.0, 0.0, 0.0, 0.0),
                0.0,
                0.0,
                edge_side,
                false,
            );
            return Ok(());
        }

        surface.set_edge_surface_region(rect, EDGE_EAR_WIDTH, PANEL_CORNER_RADIUS, edge_side, true);

        // Screen-edge curved-flare Liquid Glass contour matching the upper Notch
        let mut glass_style = LiquidGlassStyle::clear(PANEL_CORNER_RADIUS)
            .with_shadow(8.0, 2.0, 0.20 * vis_alpha);
        glass_style.hovered = self.is_hovered;
        glass_style.pressed = self.is_dragging;
        surface.draw_liquid_glass_edge_surface(
            rect,
            EDGE_EAR_WIDTH,
            PANEL_CORNER_RADIUS,
            edge_side,
            true,
            glass_style,
        )?;

        let content_alpha = ((t - 0.14) / 0.86).clamp(0.0, 1.0) * vis_alpha;
        if rect.width < 40.0 || content_alpha <= 0.02 {
            return Ok(());
        }

        let palette = surface.palette();

        // 1. Adaptive monochrome percentage readout (no "Speakers"/"Display" text)
        let pct_text = if self.kind == HudKind::Volume && self.muted {
            "0%".to_string()
        } else {
            format!("{}%", self.percent)
        };
        let pct_color = if self.kind == HudKind::Volume && self.muted {
            palette.text_secondary.with_alpha_mul(content_alpha)
        } else {
            palette.text_primary.with_alpha_mul(content_alpha)
        };
        let pct_rect = Rect::new(rect.x + 4.0, rect.y + 18.0, rect.width - 8.0, 18.0);
        surface.draw_text(&pct_text, pct_rect, 12.5, pct_color, TextAlign::Center)?;

        // 2. Monochrome Liquid Glass vertical slider bar
        let track = Rect::new(
            rect.x + (rect.width - SLIDER_TRACK_WIDTH) * 0.5,
            rect.y + 44.0,
            SLIDER_TRACK_WIDTH,
            (rect.height - 104.0).max(40.0),
        );
        let track_r = SLIDER_TRACK_WIDTH * 0.5;
        surface.fill_rounded_rect(
            track,
            track_r,
            palette.slider_track.with_alpha_mul(content_alpha),
        )?;
        let track_border = if self.is_dragging {
            palette.inner_rim.with_alpha_mul(content_alpha)
        } else {
            palette.hairline_border.with_alpha_mul(0.8 * content_alpha)
        };
        surface.stroke_rounded_rect(track, track_r, track_border, 1.0)?;

        let fill_ratio = self.bar_fill.value.clamp(0.0, 1.0);
        if fill_ratio > 0.01 {
            let fill_h = (track.height * fill_ratio).max(SLIDER_TRACK_WIDTH);
            let fill_rect = Rect::new(
                track.x,
                track.y + track.height - fill_h,
                track.width,
                fill_h,
            );
            let fill_color = if self.kind == HudKind::Volume && self.muted {
                palette.status_muted.with_alpha(0.55 * content_alpha)
            } else if self.is_dragging {
                palette.text_primary.with_alpha_mul(content_alpha)
            } else {
                palette.slider_fill.with_alpha_mul(content_alpha)
            };
            surface.fill_rounded_rect(fill_rect, track_r, fill_color)?;
        }

        // 3. Crisp SF-Symbol vector icon at bottom (spinning sun for Brightness)
        let icon_box = Rect::new(
            rect.x + (rect.width - 22.0) * 0.5,
            rect.y + rect.height - 48.0,
            22.0,
            22.0,
        );
        if self.kind == HudKind::Volume && self.is_hovered {
            let pill = Rect::new(
                rect.x + (rect.width - 32.0) * 0.5,
                rect.y + rect.height - 53.0,
                32.0,
                32.0,
            );
            surface.fill_rounded_rect(
                pill,
                16.0,
                palette.control_hover.with_alpha_mul(0.55 * content_alpha),
            )?;
        }
        match self.kind {
            HudKind::Volume => {
                let (symbol, icon_color) = if self.muted || self.percent == 0 {
                    (
                        SfSymbol::VolumeMuted,
                        palette.text_secondary.with_alpha_mul(content_alpha),
                    )
                } else {
                    (
                        SfSymbol::VolumeHigh,
                        palette.text_primary.with_alpha_mul(content_alpha),
                    )
                };
                surface.draw_sf_symbol(icon_box, symbol, icon_color, 1.6)?;
            }
            HudKind::Brightness => {
                surface.draw_brightness_sun(
                    icon_box,
                    self.sun_angle.value,
                    palette.text_primary.with_alpha_mul(content_alpha),
                    1.6,
                )?;
            }
        }

        Ok(())
    }
}

/// Paired Left-edge Volume & Right-edge Brightness overlay manager.
#[derive(Debug, Clone)]
pub struct DualEdgeOverlays {
    pub volume: OverlayHud,
    pub brightness: OverlayHud,
}

impl Default for DualEdgeOverlays {
    fn default() -> Self {
        Self::new()
    }
}

impl DualEdgeOverlays {
    /// Creates the canonical Left Volume + Right Brightness overlay pair.
    pub fn new() -> Self {
        Self {
            volume: OverlayHud::volume_left(),
            brightness: OverlayHud::brightness_right(),
        }
    }

    /// Updates Left Volume state and expands it if enabled.
    pub fn show_volume(&mut self, percent: u8, muted: bool, device_name: Option<String>) {
        self.volume.show_volume(percent, muted, device_name);
    }

    /// Updates Right Brightness state and expands it if enabled.
    pub fn show_brightness(&mut self, percent: u8) {
        self.brightness.show_brightness(percent);
    }

    /// Ticks both edge HUDs and returns `true` if either is animating or timing out.
    pub fn tick(&mut self, dt: f32) -> bool {
        let v = self.volume.tick(dt);
        let b = self.brightness.tick(dt);
        v || b
    }
}

/// Sanitizes an OS audio device name, rejecting raw endpoint GUIDs `{...}` and blanks.
pub fn sanitize_device_name(raw: Option<&str>) -> Option<String> {
    let trimmed = raw?.trim();
    if trimmed.is_empty() || (trimmed.starts_with('{') && trimmed.ends_with('}')) {
        return None;
    }
    Some(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ColorThemeMode;

    #[test]
    fn dual_edge_defaults_start_invisible_and_idle() {
        let pair = DualEdgeOverlays::new();
        assert_eq!(pair.volume.edge, HudEdge::Left);
        assert_eq!(pair.volume.kind, HudKind::Volume);
        assert_eq!(pair.brightness.edge, HudEdge::Right);
        assert_eq!(pair.brightness.kind, HudKind::Brightness);

        assert!(!pair.volume.is_visible());
        assert!(!pair.brightness.is_visible());
        assert!(!pair.volume.is_expanded());
        assert!(!pair.brightness.is_expanded());

        let idle_left = pair.volume.computed_rect(80.0, 240.0);
        let idle_right = pair.brightness.computed_rect(80.0, 240.0);
        assert_eq!(idle_left.width, 0.0);
        assert_eq!(idle_left.height, 0.0);
        assert_eq!(idle_right.width, 0.0);
        assert_eq!(idle_right.height, 0.0);
    }

    #[test]
    fn edge_hover_or_osd_emerges_flush_to_64x216_and_outside_hide_recedes_to_invisible() {
        let mut left = OverlayHud::volume_left();
        let mut right = OverlayHud::brightness_right();
        let surf_rect = Rect::new(0.0, 0.0, 80.0, 240.0);

        let left_wake = left.wake_hit_rect(surf_rect);
        let right_wake = right.wake_hit_rect(surf_rect);
        assert_eq!(left_wake.x, 0.0);
        assert_eq!(right_wake.x + right_wake.width, 80.0);

        left.set_hovered(true);
        right.show_brightness(75);
        for _ in 0..90 {
            left.tick(1.0 / 60.0);
            right.tick(1.0 / 60.0);
        }

        assert!(left.is_visible());
        assert!(right.is_visible());

        // Flush against left and right screen edges for curved-flare attachment
        let lr_exp = left.computed_rect(80.0, 240.0);
        assert_eq!(lr_exp.x, 0.0);
        assert!((lr_exp.width - EXPANDED_PANEL_WIDTH).abs() < 0.1);
        assert!((lr_exp.height - EXPANDED_PANEL_HEIGHT).abs() < 0.1);

        let rr_exp = right.computed_rect(80.0, 240.0);
        assert!((rr_exp.x - (80.0 - EXPANDED_PANEL_WIDTH)).abs() < 0.1);
        assert!((rr_exp.width - EXPANDED_PANEL_WIDTH).abs() < 0.1);
        assert!((rr_exp.height - EXPANDED_PANEL_HEIGHT).abs() < 0.1);

        // Brightness sun angle rotates cleanly toward 0.75 * TAU
        assert!((right.sun_angle.target - 0.75 * TAU).abs() < 1e-4);

        // Immediate outside-click collapse via hide()
        left.hide();
        for _ in 0..90 {
            left.tick(1.0 / 60.0);
        }
        assert!(!left.is_visible());
        assert!(!left.is_expanded());
        assert_eq!(left.computed_rect(80.0, 240.0).width, 0.0);
    }

    #[test]
    fn volume_and_brightness_clamp_and_set_targets() {
        let mut hud = OverlayHud::volume_left();
        assert!(!hud.is_expanded());

        hud.show_volume(140, false, Some("Headphones (USB)".into()));
        assert!(hud.is_expanded());
        assert_eq!(hud.kind, HudKind::Volume);
        assert_eq!(hud.percent, 100);
        assert!(!hud.muted);
        assert_eq!(hud.device_name.as_deref(), Some("Headphones (USB)"));
        assert_eq!(hud.bar_fill.target, 1.0);

        hud.show_volume(55, true, Some("{0.0.0.00000000}.{1234}".into()));
        assert!(hud.muted);
        assert_eq!(hud.device_name, None);
        assert_eq!(hud.bar_fill.target, 0.0);

        hud.show_brightness(80);
        assert_eq!(hud.kind, HudKind::Brightness);
        assert_eq!(hud.percent, 80);
        assert!(!hud.muted);
        assert_eq!(hud.device_name, None);
        assert!((hud.bar_fill.target - 0.8).abs() < 1e-5);
        assert!((hud.sun_angle.target - 0.8 * TAU).abs() < 1e-4);
    }

    #[test]
    fn auto_hide_timer_collapses_back_to_invisible_after_timeout() {
        let mut hud = OverlayHud::brightness_right();
        hud.show_brightness(60);

        for _ in 0..30 {
            hud.tick(1.0 / 60.0);
        }
        assert!(hud.is_expanded());
        assert!(hud.is_visible());
        assert!(hud.expand.value > 0.5);

        for _ in 0..240 {
            hud.tick(1.0 / 60.0);
        }
        assert!(!hud.is_expanded());
        assert!(!hud.is_visible());
        assert_eq!(hud.expand.value, 0.0);
        assert_eq!(hud.computed_rect(80.0, 240.0).width, 0.0);
    }

    #[test]
    fn hit_test_slider_mute_and_scroll_wheel_update_values() {
        let mut hud = OverlayHud::volume_left();
        let surface_rect = Rect::new(0.0, 0.0, 80.0, 240.0);

        assert_eq!(hud.hit_test_slider(34.0, 100.0, surface_rect), None);
        assert!(!hud.hit_test_pill(2.0, 120.0, surface_rect));

        hud.show_volume(40, false, Some("Speakers".into()));
        hud.expand.snap_to(1.0);
        hud.visibility.snap_to(1.0);
        assert!(hud.hit_test_pill(2.0, 120.0, surface_rect));
        assert!(hud.hit_test_pill(30.0, 120.0, surface_rect));

        let track = hud.slider_rect(surface_rect).expect("slider rect");
        let mid_x = track.x + track.width * 0.5;
        let top_ratio = hud.hit_test_slider(mid_x, track.y, surface_rect).unwrap();
        let mid_ratio = hud
            .hit_test_slider(mid_x, track.y + track.height * 0.5, surface_rect)
            .unwrap();
        let bot_ratio = hud
            .hit_test_slider(mid_x, track.y + track.height, surface_rect)
            .unwrap();

        assert!((top_ratio - 1.0).abs() < 1e-3);
        assert!((mid_ratio - 0.5).abs() < 1e-3);
        assert!(bot_ratio.abs() < 1e-3);
        assert_eq!(hud.hit_test_slider(79.0, track.y + 10.0, surface_rect), None);

        let panel = hud.expanded_rect(surface_rect.width, surface_rect.height);
        assert!(hud.hit_test_mute_button(
            panel.x + panel.width * 0.5,
            panel.y + panel.height - 38.0,
            surface_rect
        ));
        assert!(hud.toggle_mute());
        assert!(hud.muted);

        assert_eq!(hud.set_percent_from_ratio(0.75), 75);
        assert_eq!(hud.percent, 75);
        assert!(!hud.muted);
        assert_eq!(hud.on_scroll(2.0), 83);
        assert_eq!(hud.on_scroll(-30.0), 0);
        assert!(hud.muted);
    }

    #[test]
    fn dual_edge_overlays_render_invisible_idle_and_adaptive_flared_glass_without_error() {
        let surface = match NativeSurface::new("HimeOverlayTest", -2000, -2000, 80, 240) {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut pair = DualEdgeOverlays::new();

        // Idle: draws 0 pixels / 0 alpha and clears region
        surface.begin_draw();
        pair.volume.render(&surface).expect("idle left render");
        surface.end_draw_and_present().expect("idle left present");

        surface.begin_draw();
        pair.brightness.render(&surface).expect("idle right render");
        surface.end_draw_and_present().expect("idle right present");

        // Expanded edge-flared Liquid Glass under both LightGlass and DarkGlass polarities
        pair.show_volume(72, false, Some("Headphones (USB)".into()));
        pair.show_brightness(84);
        pair.volume.expand.snap_to(1.0);
        pair.volume.visibility.snap_to(1.0);
        pair.volume.bar_fill.snap_to(0.72);
        pair.brightness.expand.snap_to(1.0);
        pair.brightness.visibility.snap_to(1.0);
        pair.brightness.bar_fill.snap_to(0.84);
        pair.brightness.sun_angle.snap_to(0.84 * TAU);

        for mode in [ColorThemeMode::Dark, ColorThemeMode::Light] {
            surface.set_color_theme_mode(mode);
            surface.begin_draw();
            pair.volume.render(&surface).expect("expanded volume render");
            surface.end_draw_and_present().expect("expanded volume present");

            surface.begin_draw();
            pair.brightness
                .render(&surface)
                .expect("expanded brightness render");
            surface
                .end_draw_and_present()
                .expect("expanded brightness present");
        }
    }
}
