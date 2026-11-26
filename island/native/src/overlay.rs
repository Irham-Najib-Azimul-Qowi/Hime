//! Native Dual Edge Overlays (`OverlayHud`, `DualEdgeOverlays`) rendered via Direct2D.
//!
//! - **Left Edge (`HudEdge::Left`, `HudKind::Volume`):** Always-present `6x56` edge handle
//!   pill that smoothly expands on hover, scroll, or volume change into a `68x210` vertical
//!   glass capsule with a cyan/white gradient slider, crisp vector speaker icon, white `%`,
//!   and friendly audio device label.
//! - **Right Edge (`HudEdge::Right`, `HudKind::Brightness`):** Always-present `6x56` edge
//!   handle pill that smoothly expands into a `68x210` vertical glass capsule with a
//!   yellow-orange/white gradient slider, crisp rotating vector sun icon, and white `%`.

use serde::{Deserialize, Serialize};
use windows::core::Result;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_BEGIN_HOLLOW, D2D1_FIGURE_END_CLOSED,
    D2D1_FIGURE_END_OPEN, D2D1_GRADIENT_STOP,
};
use windows::Win32::Graphics::Direct2D::{
    ID2D1StrokeStyle, D2D1_CAP_STYLE_ROUND, D2D1_ELLIPSE, D2D1_EXTEND_MODE_CLAMP, D2D1_GAMMA_2_2,
    D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES, D2D1_ROUNDED_RECT, D2D1_STROKE_STYLE_PROPERTIES1,
};
use windows_numerics::Vector2 as D2D_POINT_2F;

use crate::animation::SpringValue;
use crate::d2d::{EdgeSide, NativeSurface, Rect, TextAlign};
use crate::theme::{
    Color, ACCENT_CYAN, DOCK_SURFACE, GLASS_BORDER, NOTCH_BG_SOLID, STATUS_OFFLINE,
    STATUS_WARNING, SURFACE_ELEVATED, TEXT_PRIMARY, TEXT_SECONDARY,
};

const DEFAULT_AUTO_HIDE_SECS: f32 = 2.5;
const IDLE_PILL_WIDTH: f32 = 6.0;
const IDLE_PILL_HEIGHT: f32 = 56.0;
const EXPANDED_PANEL_WIDTH: f32 = 68.0;
const EXPANDED_PANEL_HEIGHT: f32 = 210.0;
const SLIDER_TRACK_WIDTH: f32 = 12.0;
const SLIDER_HIT_MARGIN_X: f32 = 10.0;
const SLIDER_HIT_MARGIN_Y: f32 = 4.0;
const EDGE_WAKE_MARGIN: f32 = 6.0;

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

/// Native Direct2D edge-anchored HUD for Volume (left) and Brightness (right)
/// with always-present `6x56` idle edge pill and spring-expanded `68x210` slider capsule.
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
    /// resting as a `6x56` idle edge pill.
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
            visibility: SpringValue::gentle(1.0),
            expand: SpringValue::snappy(0.0),
            bar_fill: SpringValue::snappy(0.5),
            auto_hide_timer: 0.0,
            auto_hide_duration: DEFAULT_AUTO_HIDE_SECS,
        }
    }

    /// Enables or disables this edge overlay. When disabled, even the idle pill is hidden.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.visibility.set_target(if enabled { 1.0 } else { 0.0 });
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
        self.expand.set_target(1.0);
        self.bar_fill
            .set_target(if muted { 0.0 } else { clamped as f32 / 100.0 });
        if self.enabled {
            self.visibility.set_target(1.0);
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
        self.expand.set_target(1.0);
        self.bar_fill.set_target(clamped as f32 / 100.0);
        if self.enabled {
            self.visibility.set_target(1.0);
            self.auto_hide_timer = self.auto_hide_duration;
        }
    }

    /// Updates hover state. Hovering expands the edge pill into the `68x210` slider capsule.
    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
        if hovered {
            if self.enabled {
                self.expand.set_target(1.0);
            }
        } else if !self.is_dragging && self.auto_hide_timer <= 0.0 {
            self.expand.set_target(0.0);
        }
    }

    /// Updates active slider drag state, keeping the panel open while dragging.
    pub fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
        if dragging {
            if self.enabled {
                self.expand.set_target(1.0);
            }
        } else {
            self.auto_hide_timer = self.auto_hide_duration;
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
        self.bar_fill
            .set_target(if self.muted { 0.0 } else { self.percent as f32 / 100.0 });
        if self.enabled {
            self.expand.set_target(1.0);
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
        self.bar_fill
            .set_target(if self.muted { 0.0 } else { clamped as f32 / 100.0 });
        if self.enabled {
            self.expand.set_target(1.0);
            self.auto_hide_timer = self.auto_hide_duration;
        }
    }

    /// Collapses the expanded panel back to the `6x56` idle edge pill.
    pub fn hide(&mut self) {
        self.auto_hide_timer = 0.0;
        self.is_hovered = false;
        self.is_dragging = false;
        self.expand.set_target(0.0);
    }

    /// Advances the auto-hide timer and spring animations by `dt` seconds.
    /// Returns `true` while still animating or while an active auto-hide timer is running.
    pub fn tick(&mut self, dt: f32) -> bool {
        if dt > 0.0 && dt.is_finite() && !self.is_hovered && !self.is_dragging && self.auto_hide_timer > 0.0 {
            self.auto_hide_timer = (self.auto_hide_timer - dt).max(0.0);
            if self.auto_hide_timer == 0.0 {
                self.expand.set_target(0.0);
            }
        }
        self.visibility.tick(dt);
        self.expand.tick(dt);
        self.bar_fill.tick(dt);
        self.auto_hide_timer > 0.0
            || !self.visibility.is_settled()
            || !self.expand.is_settled()
            || !self.bar_fill.is_settled()
    }

    /// Returns `true` when the edge overlay surface (idle pill or expanded capsule) should be shown.
    pub fn is_visible(&self) -> bool {
        self.enabled && (self.visibility.value > 0.01 || self.visibility.target > 0.01)
    }

    /// Returns `true` when the HUD is expanded (hovered, dragging, timer active, or animating open).
    pub fn is_expanded(&self) -> bool {
        self.enabled
            && (self.is_hovered
                || self.is_dragging
                || self.auto_hide_timer > 0.0
                || self.expand.value > 0.05
                || self.expand.target > 0.5)
    }

    /// Computes the current bounding rectangle (interpolated between `6x56` idle pill and
    /// `68x210` expanded capsule) anchored flush to the configured screen edge.
    pub fn computed_rect(&self, surface_width: f32, surface_height: f32) -> Rect {
        let t = self.expand.value.clamp(0.0, 1.0);
        let w = IDLE_PILL_WIDTH + (EXPANDED_PANEL_WIDTH - IDLE_PILL_WIDTH) * t;
        let h = IDLE_PILL_HEIGHT + (EXPANDED_PANEL_HEIGHT - IDLE_PILL_HEIGHT) * t;
        self.edge_anchored_rect(surface_width, surface_height, w, h)
    }

    /// Computes the full `68x210` expanded panel rectangle anchored flush to `self.edge`.
    pub fn expanded_rect(&self, surface_width: f32, surface_height: f32) -> Rect {
        self.edge_anchored_rect(
            surface_width,
            surface_height,
            EXPANDED_PANEL_WIDTH,
            EXPANDED_PANEL_HEIGHT,
        )
    }

    fn edge_anchored_rect(
        &self,
        surface_width: f32,
        surface_height: f32,
        width: f32,
        height: f32,
    ) -> Rect {
        let x = match self.edge {
            HudEdge::Left => 0.0,
            HudEdge::Right => (surface_width - width).max(0.0),
        };
        let y = ((surface_height - height) * 0.5).max(0.0);
        Rect::new(x, y, width, height)
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
        let track_y = panel_abs_y + 34.0;
        let track_h = (panel.height - 88.0).max(40.0);
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

    /// Returns `true` when `(x, y)` hits the idle pill or expanded capsule (with a small edge wake margin).
    pub fn hit_test_pill(&self, x: f32, y: f32, surface_rect: Rect) -> bool {
        if !self.is_visible() {
            return false;
        }
        let local_rect = self.computed_rect(surface_rect.width, surface_rect.height);
        let rx = surface_rect.x + local_rect.x;
        let ry = surface_rect.y + local_rect.y;
        let (hx, hw) = match self.edge {
            HudEdge::Left => (rx, local_rect.width + EDGE_WAKE_MARGIN),
            HudEdge::Right => (
                (rx - EDGE_WAKE_MARGIN).max(surface_rect.x),
                local_rect.width + EDGE_WAKE_MARGIN,
            ),
        };
        let hy = ry - 4.0;
        let hh = local_rect.height + 8.0;
        x >= hx && x <= hx + hw && y >= hy && y <= hy + hh
    }

    /// Returns `true` when `(x, y)` hits the icon button at the bottom of the expanded panel.
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
            surface_rect.x + panel.x + 12.0,
            surface_rect.y + panel.y + panel.height - 50.0,
            panel.width - 24.0,
            30.0,
        );
        x >= icon_rect.x
            && x <= icon_rect.x + icon_rect.width
            && y >= icon_rect.y
            && y <= icon_rect.y + icon_rect.height
    }

    /// Renders either the sleek `6x56` idle edge pill or the expanded `68x210` glass capsule.
    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let vis_alpha = self.visibility.value.clamp(0.0, 1.0);
        if !self.enabled || vis_alpha <= 0.01 {
            return Ok(());
        }

        let (sw, sh) = surface.dip_size();
        let t = self.expand.value.clamp(0.0, 1.0);
        let rect = self.computed_rect(sw, sh);
        let edge_side = match self.edge {
            HudEdge::Left => EdgeSide::Left,
            HudEdge::Right => EdgeSide::Right,
        };
        let corner_r = 3.0 + 15.0 * t;

        surface.set_edge_surface_region(rect, 0.0, corner_r, edge_side, false);

        if t < 0.22 {
            // Idle Edge Pill (`6x56`) flush against left/right edge
            let pill_bg = NOTCH_BG_SOLID.with_alpha(0.92 * vis_alpha);
            surface.draw_edge_surface(
                rect,
                0.0,
                corner_r,
                edge_side,
                false,
                pill_bg,
                Some(GLASS_BORDER.with_alpha(0.45 * vis_alpha)),
            )?;

            let accent_color = match self.kind {
                HudKind::Volume if self.muted => STATUS_OFFLINE,
                HudKind::Volume => ACCENT_CYAN,
                HudKind::Brightness => STATUS_WARNING,
            };
            let bar_x = match self.edge {
                HudEdge::Left => rect.x + 1.5,
                HudEdge::Right => rect.x + rect.width - 3.5,
            };
            let indicator = Rect::new(bar_x, rect.y + 12.0, 2.0, (rect.height - 24.0).max(8.0));
            surface.fill_rounded_rect(indicator, 1.0, accent_color.with_alpha(0.88 * vis_alpha))?;
            return Ok(());
        }

        // Expanded Edge Panel (`68x210`)
        let content_alpha = ((t - 0.15) / 0.85).clamp(0.0, 1.0) * vis_alpha;
        surface.draw_edge_surface(
            rect,
            0.0,
            corner_r,
            edge_side,
            false,
            DOCK_SURFACE.with_alpha(0.95 * vis_alpha),
            Some(GLASS_BORDER.with_alpha(0.32 * vis_alpha)),
        )?;

        if rect.width < 40.0 || content_alpha <= 0.02 {
            return Ok(());
        }

        // 1. White percentage header (`#FFFFFF`)
        let pct_text = if self.kind == HudKind::Volume && self.muted {
            "0%".to_string()
        } else {
            format!("{}%", self.percent)
        };
        let pct_rect = Rect::new(rect.x + 4.0, rect.y + 10.0, rect.width - 8.0, 18.0);
        surface.draw_text(
            &pct_text,
            pct_rect,
            13.0,
            TEXT_PRIMARY.with_alpha(content_alpha),
            TextAlign::Center,
        )?;

        // 2. Vertical gradient slider bar
        let track = Rect::new(
            rect.x + (rect.width - SLIDER_TRACK_WIDTH) * 0.5,
            rect.y + 34.0,
            SLIDER_TRACK_WIDTH,
            (rect.height - 88.0).max(40.0),
        );
        let track_r = SLIDER_TRACK_WIDTH * 0.5;
        surface.fill_rounded_rect(track, track_r, SURFACE_ELEVATED.with_alpha(0.85 * content_alpha))?;
        let track_border = match self.kind {
            HudKind::Volume if !self.muted => ACCENT_CYAN.with_alpha(0.28 * content_alpha),
            HudKind::Brightness => STATUS_WARNING.with_alpha(0.28 * content_alpha),
            _ => GLASS_BORDER.with_alpha(0.22 * content_alpha),
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
            self.fill_vertical_gradient_bar(surface, fill_rect, track_r, content_alpha)?;
        }

        // 3. Crisp vector icon below slider
        let icon_cx = rect.x + rect.width * 0.5;
        let icon_cy = rect.y + rect.height - 35.0;
        match self.kind {
            HudKind::Volume => draw_speaker_icon(
                surface,
                icon_cx,
                icon_cy,
                16.0,
                self.muted,
                if self.muted {
                    STATUS_OFFLINE.with_alpha(content_alpha)
                } else {
                    TEXT_PRIMARY.with_alpha(content_alpha)
                },
            )?,
            HudKind::Brightness => draw_sun_icon(
                surface,
                icon_cx,
                icon_cy,
                16.0,
                self.percent as f32 / 100.0,
                Color::from_rgb_u8(0xFF, 0xB7, 0x03).with_alpha(content_alpha),
            )?,
        }

        // 4. Friendly device / mode subtitle at bottom
        let subtitle = match self.kind {
            HudKind::Volume => {
                if self.muted {
                    "Muted".to_string()
                } else {
                    short_device_label(self.device_name.as_deref())
                }
            }
            HudKind::Brightness => "Display".to_string(),
        };
        let sub_rect = Rect::new(rect.x + 4.0, rect.y + rect.height - 20.0, rect.width - 8.0, 14.0);
        surface.draw_text(
            &subtitle,
            sub_rect,
            9.5,
            TEXT_SECONDARY.with_alpha(0.92 * content_alpha),
            TextAlign::Center,
        )?;

        Ok(())
    }

    fn fill_vertical_gradient_bar(
        &self,
        surface: &NativeSurface,
        fill_rect: Rect,
        radius: f32,
        alpha: f32,
    ) -> Result<()> {
        let stops = match self.kind {
            HudKind::Volume if self.muted => [
                D2D1_GRADIENT_STOP {
                    position: 0.0,
                    color: STATUS_OFFLINE.with_alpha(alpha).to_d2d(),
                },
                D2D1_GRADIENT_STOP {
                    position: 0.5,
                    color: STATUS_OFFLINE.with_alpha(alpha).to_d2d(),
                },
                D2D1_GRADIENT_STOP {
                    position: 1.0,
                    color: STATUS_OFFLINE.with_alpha(0.8 * alpha).to_d2d(),
                },
            ],
            HudKind::Volume => [
                // Cyan / white vertical gradient
                D2D1_GRADIENT_STOP {
                    position: 0.0,
                    color: Color::from_rgb_u8(0xFF, 0xFF, 0xFF).with_alpha(alpha).to_d2d(),
                },
                D2D1_GRADIENT_STOP {
                    position: 0.35,
                    color: Color::from_rgb_u8(0x38, 0xD6, 0xF5).with_alpha(alpha).to_d2d(),
                },
                D2D1_GRADIENT_STOP {
                    position: 1.0,
                    color: Color::from_rgb_u8(0x00, 0x96, 0xFF).with_alpha(alpha).to_d2d(),
                },
            ],
            HudKind::Brightness => [
                // Yellow-orange / white vertical gradient
                D2D1_GRADIENT_STOP {
                    position: 0.0,
                    color: Color::from_rgb_u8(0xFF, 0xFF, 0xFF).with_alpha(alpha).to_d2d(),
                },
                D2D1_GRADIENT_STOP {
                    position: 0.35,
                    color: Color::from_rgb_u8(0xFF, 0xB7, 0x03).with_alpha(alpha).to_d2d(),
                },
                D2D1_GRADIENT_STOP {
                    position: 1.0,
                    color: Color::from_rgb_u8(0xFB, 0x85, 0x00).with_alpha(alpha).to_d2d(),
                },
            ],
        };

        let target = surface.render_target();
        unsafe {
            let collection = target.CreateGradientStopCollection(
                &stops,
                D2D1_GAMMA_2_2,
                D2D1_EXTEND_MODE_CLAMP,
            )?;
            let props = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                startPoint: D2D_POINT_2F {
                    X: fill_rect.x,
                    Y: fill_rect.y,
                },
                endPoint: D2D_POINT_2F {
                    X: fill_rect.x,
                    Y: fill_rect.y + fill_rect.height,
                },
            };
            let brush = target.CreateLinearGradientBrush(&props, None, &collection)?;
            let rr = D2D1_ROUNDED_RECT {
                rect: fill_rect.to_d2d(),
                radiusX: radius,
                radiusY: radius,
            };
            target.FillRoundedRectangle(&rr, &brush);
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

fn short_device_label(raw: Option<&str>) -> String {
    let Some(name) = sanitize_device_name(raw) else {
        return "Audio".to_string();
    };
    let concise = name
        .split(" (")
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&name);
    if concise.chars().count() <= 10 {
        concise.to_string()
    } else {
        let mut out: String = concise.chars().take(9).collect();
        out.push('…');
        out
    }
}

fn draw_speaker_icon(
    surface: &NativeSurface,
    cx: f32,
    cy: f32,
    size: f32,
    muted: bool,
    color: Color,
) -> Result<()> {
    let s = size.max(10.0) * 0.5;
    let target = surface.render_target();
    let factory = surface.d2d_factory();
    unsafe {
        let brush = target.CreateSolidColorBrush(&color.to_d2d(), None)?;
        let stroke_props = D2D1_STROKE_STYLE_PROPERTIES1 {
            startCap: D2D1_CAP_STYLE_ROUND,
            endCap: D2D1_CAP_STYLE_ROUND,
            ..Default::default()
        };
        let round_cap: ID2D1StrokeStyle = factory.CreateStrokeStyle(&stroke_props, None)?.into();

        // Speaker cone polygon
        let geo = factory.CreatePathGeometry()?;
        let sink = geo.Open()?;
        sink.BeginFigure(pt(cx - s * 0.72, cy - s * 0.28), D2D1_FIGURE_BEGIN_FILLED);
        sink.AddLine(pt(cx - s * 0.32, cy - s * 0.28));
        sink.AddLine(pt(cx + s * 0.08, cy - s * 0.62));
        sink.AddLine(pt(cx + s * 0.08, cy + s * 0.62));
        sink.AddLine(pt(cx - s * 0.32, cy + s * 0.28));
        sink.AddLine(pt(cx - s * 0.72, cy + s * 0.28));
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        sink.Close()?;
        target.FillGeometry(&geo, &brush, None);

        let stroke_w = (size * 0.1).max(1.3);
        if muted {
            target.DrawLine(
                pt(cx + s * 0.26, cy - s * 0.3),
                pt(cx + s * 0.76, cy + s * 0.3),
                &brush,
                stroke_w,
                Some(&round_cap),
            );
            target.DrawLine(
                pt(cx + s * 0.76, cy - s * 0.3),
                pt(cx + s * 0.26, cy + s * 0.3),
                &brush,
                stroke_w,
                Some(&round_cap),
            );
        } else {
            for r in [s * 0.36, s * 0.64] {
                let wave = factory.CreatePathGeometry()?;
                let wsink = wave.Open()?;
                wsink.BeginFigure(pt(cx + r * 0.65, cy - r * 0.72), D2D1_FIGURE_BEGIN_HOLLOW);
                wsink.AddLine(pt(cx + r * 0.95, cy));
                wsink.AddLine(pt(cx + r * 0.65, cy + r * 0.72));
                wsink.EndFigure(D2D1_FIGURE_END_OPEN);
                wsink.Close()?;
                target.DrawGeometry(&wave, &brush, stroke_w, Some(&round_cap));
            }
        }
    }
    Ok(())
}

fn draw_sun_icon(
    surface: &NativeSurface,
    cx: f32,
    cy: f32,
    size: f32,
    brightness_ratio: f32,
    color: Color,
) -> Result<()> {
    let s = size.max(10.0) * 0.5;
    let angle_offset = brightness_ratio.clamp(0.0, 1.0) * std::f32::consts::PI;
    let target = surface.render_target();
    let factory = surface.d2d_factory();
    unsafe {
        let brush = target.CreateSolidColorBrush(&color.to_d2d(), None)?;
        let stroke_props = D2D1_STROKE_STYLE_PROPERTIES1 {
            startCap: D2D1_CAP_STYLE_ROUND,
            endCap: D2D1_CAP_STYLE_ROUND,
            ..Default::default()
        };
        let round_cap: ID2D1StrokeStyle = factory.CreateStrokeStyle(&stroke_props, None)?.into();
        let stroke_w = (size * 0.1).max(1.3);

        target.DrawEllipse(
            &D2D1_ELLIPSE {
                point: pt(cx, cy),
                radiusX: s * 0.38,
                radiusY: s * 0.38,
            },
            &brush,
            stroke_w,
            None,
        );

        for i in 0..8 {
            let theta = angle_offset + (i as f32) * (std::f32::consts::PI * 0.25);
            let (sin_t, cos_t) = theta.sin_cos();
            let r0 = s * 0.62;
            let r1 = s * 0.88;
            target.DrawLine(
                pt(cx + cos_t * r0, cy + sin_t * r0),
                pt(cx + cos_t * r1, cy + sin_t * r1),
                &brush,
                stroke_w,
                Some(&round_cap),
            );
        }
    }
    Ok(())
}

#[inline]
const fn pt(x: f32, y: f32) -> D2D_POINT_2F {
    D2D_POINT_2F { X: x, Y: y }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_edge_defaults_left_volume_and_right_brightness() {
        let pair = DualEdgeOverlays::new();
        assert_eq!(pair.volume.edge, HudEdge::Left);
        assert_eq!(pair.volume.kind, HudKind::Volume);
        assert_eq!(pair.brightness.edge, HudEdge::Right);
        assert_eq!(pair.brightness.kind, HudKind::Brightness);

        assert!(pair.volume.is_visible());
        assert!(pair.brightness.is_visible());
        assert!(!pair.volume.is_expanded());
        assert!(!pair.brightness.is_expanded());
    }

    #[test]
    fn idle_pill_is_6x56_flush_and_expands_to_68x210_on_hover_or_show() {
        let mut left = OverlayHud::volume_left();
        let lr_idle = left.computed_rect(120.0, 240.0);
        assert_eq!(lr_idle.x, 0.0);
        assert_eq!(lr_idle.width, 6.0);
        assert_eq!(lr_idle.height, 56.0);

        let mut right = OverlayHud::brightness_right();
        let rr_idle = right.computed_rect(120.0, 240.0);
        assert_eq!(rr_idle.x, 120.0 - 6.0);
        assert_eq!(rr_idle.width, 6.0);
        assert_eq!(rr_idle.height, 56.0);

        left.set_hovered(true);
        right.show_brightness(75);
        for _ in 0..120 {
            left.tick(1.0 / 60.0);
            right.tick(1.0 / 60.0);
        }

        let lr_exp = left.computed_rect(120.0, 240.0);
        assert_eq!(lr_exp.x, 0.0);
        assert!((lr_exp.width - 68.0).abs() < 0.1);
        assert!((lr_exp.height - 210.0).abs() < 0.1);

        let rr_exp = right.computed_rect(120.0, 240.0);
        assert!((rr_exp.x - (120.0 - 68.0)).abs() < 0.1);
        assert!((rr_exp.width - 68.0).abs() < 0.1);
        assert!((rr_exp.height - 210.0).abs() < 0.1);
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
    }

    #[test]
    fn auto_hide_timer_collapses_back_to_idle_pill_after_timeout() {
        let mut hud = OverlayHud::brightness_right();
        hud.show_brightness(60);

        // Advance partway; HUD should remain expanded.
        for _ in 0..30 {
            hud.tick(1.0 / 60.0);
        }
        assert!(hud.is_expanded());
        assert!(hud.expand.value > 0.5);

        // Advance past auto-hide duration + settle time.
        for _ in 0..240 {
            hud.tick(1.0 / 60.0);
        }
        assert!(!hud.is_expanded());
        assert!(hud.is_visible());
        assert_eq!(hud.expand.value, 0.0);
        assert_eq!(hud.computed_rect(100.0, 240.0).width, 6.0);
    }

    #[test]
    fn hit_test_slider_and_scroll_wheel_update_values() {
        let mut hud = OverlayHud::volume_left();
        let surface_rect = Rect::new(0.0, 0.0, 100.0, 240.0);

        // Collapsed idle pill has no active slider hit target.
        assert_eq!(hud.hit_test_slider(34.0, 100.0, surface_rect), None);
        assert!(hud.hit_test_pill(2.0, 120.0, surface_rect));

        hud.show_volume(40, false, Some("Speakers".into()));
        hud.expand.snap_to(1.0);

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
        assert_eq!(hud.hit_test_slider(95.0, track.y + 10.0, surface_rect), None);

        assert_eq!(hud.set_percent_from_ratio(0.75), 75);
        assert_eq!(hud.percent, 75);
        assert_eq!(hud.on_scroll(2.0), 83);
        assert_eq!(hud.on_scroll(-30.0), 0);
        assert!(hud.muted);
    }

    #[test]
    fn dual_edge_overlays_render_idle_and_expanded_gradients_without_error() {
        let surface = match NativeSurface::new("HimeOverlayTest", -2000, -2000, 120, 240) {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut pair = DualEdgeOverlays::new();

        // Idle 6x56 edge pills
        surface.begin_draw();
        pair.volume.render(&surface).expect("idle left pill render");
        surface.end_draw_and_present().expect("idle left pill present");

        surface.begin_draw();
        pair.brightness.render(&surface).expect("idle right pill render");
        surface.end_draw_and_present().expect("idle right pill present");

        // Expanded 68x210 gradient slider capsules
        pair.show_volume(72, false, Some("Headphones (USB)".into()));
        pair.show_brightness(84);
        pair.volume.expand.snap_to(1.0);
        pair.volume.bar_fill.snap_to(0.72);
        pair.brightness.expand.snap_to(1.0);
        pair.brightness.bar_fill.snap_to(0.84);

        surface.begin_draw();
        pair.volume.render(&surface).expect("expanded volume render");
        surface.end_draw_and_present().expect("expanded volume present");

        surface.begin_draw();
        pair.brightness.render(&surface).expect("expanded brightness render");
        surface.end_draw_and_present().expect("expanded brightness present");
    }
}
