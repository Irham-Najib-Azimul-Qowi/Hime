//! Native Dual Edge Overlays (`OverlayHud`, `DualEdgeOverlays`) rendered via Direct2D.
//!
//! Styled cleanly for low overhead:
//! - **Minimized / Idle State:** 100% invisible (0 alpha, empty 0x0 region, click-through).
//! - **Left Edge (`HudEdge::Left`, `HudKind::Volume`):** Smoothly emerges (`Spring::EMERGE`)
//!   flush against the left screen edge with curved flare ears (`ear_w = 13`, `flares = true`)
//!   on edge hover, scroll, or volume OSD change over a plain black `#000000` background with
//!   zero border strokes or gradients.
//! - **Right Edge (`HudEdge::Right`, `HudKind::Brightness`):** Smoothly emerges flush against
//!   the right screen edge with curved flare ears over a plain black `#000000` background,
//!   displaying percentage, slider bar, and rotating `draw_brightness_sun` icon.

use std::f32::consts::TAU;

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::{Spring, SpringValue};
use crate::d2d::{EdgeSide, NativeSurface, Rect, SfSymbol, TextAlign};
use crate::theme::{AdaptivePalette, Color};

const DEFAULT_AUTO_HIDE_SECS: f32 = 2.5;
const ADJUST_DISPLAY_SECS: f32 = 0.85;
const EXPANDED_PANEL_WIDTH: f32 = 40.0;
const MIXER_PANEL_WIDTH: f32 = 210.0;
const EXPANDED_PANEL_HEIGHT: f32 = 216.0;
const EDGE_EAR_WIDTH: f32 = 13.0;
const PANEL_CORNER_RADIUS: f32 = 14.0;
const SLIDER_TRACK_WIDTH: f32 = 10.0;
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

/// Connected audio output device entry displayed in the Volume Side Notch mixer drawer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioDeviceSlot {
    pub id: String,
    pub name: String,
    pub percent: u8,
    pub muted: bool,
    pub is_default: bool,
}

/// Native Direct2D edge-anchored HUD for Volume (left) and Brightness (right).
///
/// Completely invisible (`0` alpha / empty region / click-through) when idle; emerges
/// with Apple `Spring::EMERGE` into a screen-edge flared `40x216` (or `210x216` when
/// audio mixer is open) contour on edge hover or OSD updates, and collapses cleanly with
/// `Spring::RECEDE` on timeout or outside click.
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
    pub icon_hovered: bool,
    pub mixer_open: bool,
    pub devices: Vec<AudioDeviceSlot>,
    pub visibility: SpringValue,
    pub expand: SpringValue,
    pub mixer_expand: SpringValue,
    pub bar_fill: SpringValue,
    pub sun_angle: SpringValue,
    auto_hide_timer: f32,
    auto_hide_duration: f32,
    adjust_timer: f32,
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
            icon_hovered: false,
            mixer_open: false,
            devices: Vec::new(),
            visibility: SpringValue::emerge(0.0),
            expand: SpringValue::emerge(0.0),
            mixer_expand: SpringValue::emerge(0.0),
            bar_fill: SpringValue::knob_settle(0.5),
            sun_angle: SpringValue::from_spring(0.5 * TAU, Spring::INTERACTIVE),
            auto_hide_timer: 0.0,
            auto_hide_duration: DEFAULT_AUTO_HIDE_SECS,
            adjust_timer: 0.0,
        }
    }

    /// Returns `true` while the user is actively dragging or shortly after a level adjustment.
    pub fn is_adjusting(&self) -> bool {
        self.is_dragging || self.adjust_timer > 0.0
    }

    /// Updates hover state for the bottom icon button (switches Volume icon to `SfSymbol::AudioMixer`).
    pub fn set_icon_hovered(&mut self, hovered: bool) {
        self.icon_hovered = hovered;
    }

    /// Replaces the connected audio devices displayed when the Left Volume mixer drawer is open.
    pub fn set_devices(&mut self, devices: Vec<AudioDeviceSlot>) {
        self.devices = devices;
    }

    /// Toggles the expanded audio device mixer drawer (Volume HUD only) and returns `mixer_open`.
    pub fn toggle_mixer(&mut self) -> bool {
        if self.kind != HudKind::Volume {
            return false;
        }
        self.set_mixer_open(!self.mixer_open);
        self.mixer_open
    }

    /// Explicitly opens or closes the audio device mixer drawer (Volume HUD only).
    pub fn set_mixer_open(&mut self, open: bool) {
        if self.kind != HudKind::Volume {
            return;
        }
        self.mixer_open = open;
        if open {
            if self.enabled {
                self.launch_open();
                self.auto_hide_timer = self.auto_hide_duration;
            }
            self.mixer_expand.launch_to(1.0, Spring::EMERGE, 1.0);
        } else {
            self.mixer_expand.launch_to(0.0, Spring::RECEDE, 1.0);
        }
    }

    /// Enables or disables this edge overlay. When disabled, it immediately collapses and hides.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.hide();
        }
    }

    fn set_volume_fields(&mut self, percent: u8, muted: bool, device_name: Option<String>) -> f32 {
        let clamped = percent.min(100);
        self.kind = HudKind::Volume;
        self.percent = clamped;
        self.muted = muted;
        self.device_name = sanitize_device_name(device_name.as_deref());
        if muted {
            0.0
        } else {
            clamped as f32 / 100.0
        }
    }

    /// Synchronizes Volume metrics without popping open an idle/hidden HUD.
    ///
    /// Used for external OS volume updates (trackpad gestures, keyboard keys, Windows mixer)
    /// so the left Volume Edge HUD only emerges on screen-edge hover while an already-visible
    /// HUD animates smoothly to the latest level.
    pub fn sync_volume_quiet(&mut self, percent: u8, muted: bool, device_name: Option<String>) {
        let fill = self.set_volume_fields(percent, muted, device_name);
        if self.is_visible() {
            self.bar_fill.animate_to(fill, Spring::KNOB_SETTLE);
        } else {
            self.bar_fill.snap_to(fill);
        }
    }

    /// Shows or updates the HUD in Volume mode and expands it with an auto-hide timer.
    pub fn show_volume(&mut self, percent: u8, muted: bool, device_name: Option<String>) {
        let fill = self.set_volume_fields(percent, muted, device_name);
        self.bar_fill.animate_to(fill, Spring::KNOB_SETTLE);
        if self.enabled {
            self.launch_open();
            self.auto_hide_timer = self.auto_hide_duration;
            self.adjust_timer = ADJUST_DISPLAY_SECS;
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
            self.adjust_timer = ADJUST_DISPLAY_SECS;
        }
    }

    /// Updates hover state. Hovering the edge wakes and emerges the edge-flared Liquid Glass HUD.
    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
        if hovered {
            if self.enabled {
                self.launch_open();
            }
        } else {
            self.icon_hovered = false;
            if !self.is_dragging && self.auto_hide_timer <= 0.0 {
                self.launch_close();
            }
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
            self.adjust_timer = ADJUST_DISPLAY_SECS;
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
            self.adjust_timer = ADJUST_DISPLAY_SECS;
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
        if self.mixer_open || self.mixer_expand.target > 0.5 {
            self.mixer_open = false;
            self.mixer_expand.launch_to(0.0, Spring::RECEDE, 1.0);
        }
        self.icon_hovered = false;
    }

    /// Collapses the expanded HUD immediately back to invisible idle state (`Spring::RECEDE`).
    pub fn hide(&mut self) {
        self.auto_hide_timer = 0.0;
        self.adjust_timer = 0.0;
        self.is_hovered = false;
        self.is_dragging = false;
        self.icon_hovered = false;
        self.mixer_open = false;
        self.launch_close();
    }

    /// Advances the auto-hide timer and Apple spring animations by `dt` seconds.
    /// Returns `true` while still animating or while an active auto-hide timer is running.
    pub fn tick(&mut self, dt: f32) -> bool {
        if dt > 0.0 && dt.is_finite() {
            if self.adjust_timer > 0.0 && !self.is_dragging {
                self.adjust_timer = (self.adjust_timer - dt).max(0.0);
            }
            if !self.is_hovered && !self.is_dragging && self.auto_hide_timer > 0.0 {
                self.auto_hide_timer = (self.auto_hide_timer - dt).max(0.0);
                if self.auto_hide_timer == 0.0 {
                    self.launch_close();
                }
            }
        }
        self.visibility.tick(dt);
        self.expand.tick(dt);
        self.mixer_expand.tick(dt);
        self.bar_fill.tick(dt);
        self.sun_angle.tick(dt);
        self.auto_hide_timer > 0.0
            || self.adjust_timer > 0.0
            || !self.visibility.is_settled()
            || !self.expand.is_settled()
            || !self.mixer_expand.is_settled()
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
                || self.mixer_open
                || self.auto_hide_timer > 0.0
                || self.expand.value > 0.01
                || self.expand.target > 0.5
                || self.mixer_expand.value > 0.01
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
    /// from the anchored screen edge into the flared `40x216` (or `210x216` mixer) silhouette.
    pub fn computed_rect(&self, surface_width: f32, surface_height: f32) -> Rect {
        let t = self.expand.value.clamp(0.0, 1.0);
        if !self.enabled || t <= 0.01 {
            let edge_x = match self.edge {
                HudEdge::Left => 0.0,
                HudEdge::Right => surface_width.max(0.0),
            };
            return Rect::new(edge_x, (surface_height * 0.5).max(0.0), 0.0, 0.0);
        }

        let mixer_t = if self.kind == HudKind::Volume {
            self.mixer_expand.value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let target_w = EXPANDED_PANEL_WIDTH + (MIXER_PANEL_WIDTH - EXPANDED_PANEL_WIDTH) * mixer_t;
        let w = target_w * (MIN_EMERGE_WIDTH_FRAC + (1.0 - MIN_EMERGE_WIDTH_FRAC) * t);
        let h =
            EXPANDED_PANEL_HEIGHT * (MIN_EMERGE_HEIGHT_FRAC + (1.0 - MIN_EMERGE_HEIGHT_FRAC) * t);
        let x = match self.edge {
            HudEdge::Left => 0.0,
            HudEdge::Right => (surface_width - w).max(0.0),
        };
        let y = ((surface_height - h) * 0.5).max(0.0);
        Rect::new(x, y, w, h)
    }

    /// Computes the full `40x216` (or `210x216` when mixer is open) expanded panel rectangle anchored flush to `self.edge`.
    pub fn expanded_rect(&self, surface_width: f32, surface_height: f32) -> Rect {
        let target_w = if self.kind == HudKind::Volume && self.mixer_open {
            MIXER_PANEL_WIDTH
        } else {
            EXPANDED_PANEL_WIDTH
        };
        let x = match self.edge {
            HudEdge::Left => 0.0,
            HudEdge::Right => (surface_width - target_w).max(0.0),
        };
        let y = ((surface_height - EXPANDED_PANEL_HEIGHT) * 0.5).max(0.0);
        Rect::new(x, y, target_w, EXPANDED_PANEL_HEIGHT)
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
        let strip_w = panel.width.min(EXPANDED_PANEL_WIDTH);
        let strip_x = match self.edge {
            HudEdge::Left => surface_rect.x + panel.x,
            HudEdge::Right => surface_rect.x + panel.x + (panel.width - strip_w),
        };
        let panel_abs_y = surface_rect.y + panel.y;
        let track_x = strip_x + (strip_w - SLIDER_TRACK_WIDTH) * 0.5;
        let track_y = panel_abs_y + 20.0;
        let track_h = (panel.height - 76.0).max(40.0);
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

    /// Returns the bottom icon button hit rectangle in `surface_rect` coordinates.
    pub fn icon_button_rect(&self, surface_rect: Rect) -> Option<Rect> {
        if !self.is_expanded() {
            return None;
        }
        let panel = if self.expand.value >= 0.45 {
            self.computed_rect(surface_rect.width, surface_rect.height)
        } else {
            self.expanded_rect(surface_rect.width, surface_rect.height)
        };
        let strip_w = panel.width.min(EXPANDED_PANEL_WIDTH);
        let strip_x = match self.edge {
            HudEdge::Left => surface_rect.x + panel.x,
            HudEdge::Right => surface_rect.x + panel.x + (panel.width - strip_w),
        };
        Some(Rect::new(
            strip_x + 4.0,
            surface_rect.y + panel.y + panel.height - 52.0,
            (strip_w - 8.0).max(24.0),
            36.0,
        ))
    }

    /// Returns `true` when `(x, y)` hits the bottom icon button of the expanded panel.
    pub fn hit_test_icon_button(&self, x: f32, y: f32, surface_rect: Rect) -> bool {
        let Some(icon_rect) = self.icon_button_rect(surface_rect) else {
            return false;
        };
        x >= icon_rect.x
            && x <= icon_rect.x + icon_rect.width
            && y >= icon_rect.y
            && y <= icon_rect.y + icon_rect.height
    }

    /// Returns `true` when `(x, y)` hits the speaker button at the bottom of the expanded Volume panel.
    pub fn hit_test_mute_button(&self, x: f32, y: f32, surface_rect: Rect) -> bool {
        if self.kind != HudKind::Volume {
            return false;
        }
        self.hit_test_icon_button(x, y, surface_rect)
    }

    /// Computes `(device_id, slider_rect)` pairs for connected audio devices in the open mixer drawer.
    pub fn device_slider_rects(&self, surface_rect: Rect) -> Vec<(String, Rect)> {
        if self.kind != HudKind::Volume || !self.is_expanded() || !self.mixer_open {
            return Vec::new();
        }
        let panel = if self.expand.value >= 0.45 {
            self.computed_rect(surface_rect.width, surface_rect.height)
        } else {
            self.expanded_rect(surface_rect.width, surface_rect.height)
        };
        if panel.width <= EXPANDED_PANEL_WIDTH + 40.0 {
            return Vec::new();
        }
        let drawer_x = surface_rect.x + panel.x + EXPANDED_PANEL_WIDTH + 10.0;
        let drawer_w = (panel.width - EXPANDED_PANEL_WIDTH - 22.0).max(40.0);
        let start_y = surface_rect.y + panel.y + 40.0;
        let row_h = 34.0_f32;
        let max_rows = ((panel.height - 54.0) / row_h).floor().max(0.0) as usize;
        self.devices
            .iter()
            .take(max_rows)
            .enumerate()
            .map(|(i, dev)| {
                let row_y = start_y + (i as f32) * row_h;
                let bar = Rect::new(drawer_x, row_y + 18.0, drawer_w, 8.0);
                (dev.id.clone(), bar)
            })
            .collect()
    }

    /// Hit-tests `(x, y)` against a connected audio device's volume bar in the open mixer drawer,
    /// returning `(device_id, target_percent)`.
    pub fn hit_test_device_slider(&self, x: f32, y: f32, surface_rect: Rect) -> Option<(String, u8)> {
        for (id, bar) in self.device_slider_rects(surface_rect) {
            if x >= bar.x - 4.0
                && x <= bar.x + bar.width + 4.0
                && y >= bar.y - 14.0
                && y <= bar.y + bar.height + 8.0
            {
                let ratio = ((x - bar.x) / bar.width.max(1.0)).clamp(0.0, 1.0);
                let pct = (ratio * 100.0).round() as u8;
                return Some((id, pct));
            }
        }
        None
    }

    /// Renders either an invisible empty region (when idle) or the screen-edge curved-flare
    /// Plain black `#000000` slider contour without strokes, glass blur, or gradients.
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

        // Screen-edge curved-flare contour on plain solid black (#000000), zero stroke/glass
        surface.draw_edge_surface(
            rect,
            EDGE_EAR_WIDTH,
            PANEL_CORNER_RADIUS,
            edge_side,
            true,
            Color::rgba(0.0, 0.0, 0.0, vis_alpha),
            None,
        )?;

        let content_alpha = ((t - 0.14) / 0.86).clamp(0.0, 1.0) * vis_alpha;
        if rect.width < 24.0 || content_alpha <= 0.02 {
            return Ok(());
        }

        let palette = AdaptivePalette::dark_glass();
        let strip_w = rect.width.min(EXPANDED_PANEL_WIDTH);
        let strip_x = match self.edge {
            HudEdge::Left => rect.x,
            HudEdge::Right => rect.x + (rect.width - strip_w),
        };

        // 1. Flat borderless vertical slider bar (top-aligned in the thin 40px strip)
        let track = Rect::new(
            strip_x + (strip_w - SLIDER_TRACK_WIDTH) * 0.5,
            rect.y + 20.0,
            SLIDER_TRACK_WIDTH,
            (rect.height - 76.0).max(40.0),
        );
        let track_r = SLIDER_TRACK_WIDTH * 0.5;
        surface.fill_rounded_rect(
            track,
            track_r,
            palette.slider_track.with_alpha_mul(content_alpha),
        )?;

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

        // 2. Bottom slot: show % text while adjusting, otherwise show crisp SF-Symbol vector icon
        let bottom_center_y = rect.y + rect.height - 34.0;
        if self.is_adjusting() {
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
            let pct_rect = Rect::new(strip_x + 2.0, bottom_center_y - 10.0, strip_w - 4.0, 20.0);
            surface.draw_text(&pct_text, pct_rect, 11.5, pct_color, TextAlign::Center)?;
        } else {
            let icon_box = Rect::new(
                strip_x + (strip_w - 20.0) * 0.5,
                bottom_center_y - 10.0,
                20.0,
                20.0,
            );
            if self.kind == HudKind::Volume
                && (self.icon_hovered || self.mixer_open)
                && content_alpha > 0.75
            {
                let pill = Rect::new(
                    strip_x + (strip_w - 28.0) * 0.5,
                    bottom_center_y - 14.0,
                    28.0,
                    28.0,
                );
                let pill_fill = if self.mixer_open {
                    palette.control_active
                } else {
                    palette.control_hover
                };
                surface.fill_rounded_rect(pill, 14.0, pill_fill)?;
            }
            match self.kind {
                HudKind::Volume => {
                    let (symbol, icon_color) = if self.icon_hovered || self.mixer_open {
                        (
                            SfSymbol::AudioMixer,
                            palette.text_primary.with_alpha_mul(content_alpha),
                        )
                    } else if self.muted || self.percent == 0 {
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
        }

        // 3. Expanded Left Volume Audio Mixer Drawer (per-device volume bars)
        let mixer_t = if self.kind == HudKind::Volume {
            self.mixer_expand.value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        if mixer_t > 0.25 && rect.width > EXPANDED_PANEL_WIDTH + 40.0 {
            let drawer_alpha = (((mixer_t - 0.25) / 0.75).clamp(0.0, 1.0)) * content_alpha;
            let div_x = rect.x + EXPANDED_PANEL_WIDTH;
            surface.fill_rounded_rect(
                Rect::new(div_x, rect.y + 18.0, 1.0, (rect.height - 36.0).max(20.0)),
                0.5,
                palette.divider.with_alpha_mul(drawer_alpha),
            )?;

            let drawer_x = div_x + 10.0;
            let drawer_w = (rect.width - EXPANDED_PANEL_WIDTH - 22.0).max(40.0);
            surface.draw_text_bold(
                "Audio Devices",
                Rect::new(drawer_x, rect.y + 16.0, drawer_w, 16.0),
                11.0,
                palette.text_secondary.with_alpha_mul(drawer_alpha),
                TextAlign::Leading,
            )?;

            if self.devices.is_empty() {
                surface.draw_text(
                    "No audio devices",
                    Rect::new(drawer_x, rect.y + 44.0, drawer_w, rect.height - 64.0),
                    11.0,
                    palette.text_secondary.with_alpha_mul(drawer_alpha),
                    TextAlign::Center,
                )?;
            } else {
                let start_y = rect.y + 40.0;
                let row_h = 34.0_f32;
                let max_rows = ((rect.height - 54.0) / row_h).floor().max(0.0) as usize;
                for (idx, dev) in self.devices.iter().take(max_rows).enumerate() {
                    let row_y = start_y + (idx as f32) * row_h;
                    let name_w = (drawer_w - 34.0).max(24.0);
                    let name_color = if dev.is_default {
                        palette.text_primary.with_alpha_mul(drawer_alpha)
                    } else {
                        palette.text_secondary.with_alpha_mul(drawer_alpha)
                    };
                    surface.draw_text(
                        &dev.name,
                        Rect::new(drawer_x, row_y, name_w, 15.0),
                        10.5,
                        name_color,
                        TextAlign::Leading,
                    )?;
                    let pct_str = if dev.muted {
                        "Muted".to_string()
                    } else {
                        format!("{}%", dev.percent.min(100))
                    };
                    surface.draw_text(
                        &pct_str,
                        Rect::new(drawer_x + drawer_w - 34.0, row_y, 34.0, 15.0),
                        10.0,
                        palette.text_secondary.with_alpha_mul(drawer_alpha),
                        TextAlign::Trailing,
                    )?;

                    let bar_rect = Rect::new(drawer_x, row_y + 18.0, drawer_w, 7.0);
                    surface.fill_rounded_rect(
                        bar_rect,
                        3.5,
                        palette.slider_track.with_alpha_mul(drawer_alpha),
                    )?;
                    let dev_fill = if dev.muted {
                        0.0
                    } else {
                        (dev.percent.min(100) as f32) / 100.0
                    };
                    if dev_fill > 0.01 {
                        let fill_w = (bar_rect.width * dev_fill).max(7.0);
                        surface.fill_rounded_rect(
                            Rect::new(bar_rect.x, bar_rect.y, fill_w, bar_rect.height),
                            3.5,
                            palette.slider_fill.with_alpha_mul(drawer_alpha),
                        )?;
                    }
                }
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

    /// Synchronizes Left Volume state quietly without popping open an idle/hidden HUD.
    pub fn sync_volume_quiet(&mut self, percent: u8, muted: bool, device_name: Option<String>) {
        self.volume.sync_volume_quiet(percent, muted, device_name);
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
    fn dual_edge_overlays_render_invisible_idle_and_plain_black_without_error() {
        use crate::d2d::BackdropBuffer;
        use crate::theme::{ColorThemeMode, ThemePolarity};

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

        // Expanded edge-flared overlays render plain black under both dark and bright host surfaces
        pair.show_volume(72, false, Some("Headphones (USB)".into()));
        pair.show_brightness(84);
        pair.volume.expand.snap_to(1.0);
        pair.volume.visibility.snap_to(1.0);
        pair.volume.bar_fill.snap_to(0.72);
        pair.brightness.expand.snap_to(1.0);
        pair.brightness.visibility.snap_to(1.0);
        pair.brightness.bar_fill.snap_to(0.84);
        pair.brightness.sun_angle.snap_to(0.84 * TAU);

        let dark_bg = BackdropBuffer::from_bgra_premul(4, 4, vec![0x14, 0x14, 0x18, 0xFF].repeat(16))
            .expect("dark backdrop");
        let bright_bg =
            BackdropBuffer::from_bgra_premul(4, 4, vec![0xF0, 0xF2, 0xF6, 0xFF].repeat(16))
                .expect("bright backdrop");

        surface.set_color_theme_mode(ColorThemeMode::Auto);
        surface.set_backdrop(Some(dark_bg));
        assert_eq!(surface.theme_polarity(), ThemePolarity::DarkGlass);

        surface.begin_draw();
        pair.volume.render(&surface).expect("dark volume render");
        surface.end_draw_and_present().expect("dark volume present");
        surface.begin_draw();
        pair.brightness
            .render(&surface)
            .expect("dark brightness render");
        surface
            .end_draw_and_present()
            .expect("dark brightness present");

        surface.set_backdrop(Some(bright_bg));

        surface.begin_draw();
        pair.volume.render(&surface).expect("bright-bg volume render");
        surface.end_draw_and_present().expect("bright-bg volume present");
        surface.begin_draw();
        pair.brightness
            .render(&surface)
            .expect("bright-bg brightness render");
        surface
            .end_draw_and_present()
            .expect("bright-bg brightness present");
    }

    #[test]
    fn sync_volume_quiet_updates_metrics_without_popping_hidden_hud() {
        let mut hud = OverlayHud::volume_left();
        assert!(!hud.is_visible());
        assert!(!hud.is_expanded());

        hud.sync_volume_quiet(68, false, Some("Speakers".into()));
        assert!(!hud.is_visible());
        assert!(!hud.is_expanded());
        assert_eq!(hud.percent, 68);
        assert!(!hud.muted);
        assert_eq!(hud.device_name.as_deref(), Some("Speakers"));
        assert!((hud.bar_fill.value - 0.68).abs() < 1e-4);

        // Once hovered onto the left screen edge, it emerges showing the synced level
        hud.set_hovered(true);
        for _ in 0..20 {
            hud.tick(1.0 / 60.0);
        }
        assert!(hud.is_visible());
        assert!(hud.is_expanded());

        // Subsequent external sync while visible updates target smoothly without breaking visibility
        hud.sync_volume_quiet(25, true, Some("Headphones".into()));
        assert!(hud.is_visible());
        assert_eq!(hud.percent, 25);
        assert!(hud.muted);
        assert_eq!(hud.bar_fill.target, 0.0);
    }

    #[test]
    fn adjusting_percentage_swap_and_volume_mixer_drawer_work() {
        let mut hud = OverlayHud::volume_left();
        let surf_rect = Rect::new(0.0, 0.0, 240.0, 240.0);

        assert!(!hud.is_adjusting());
        assert_eq!(EXPANDED_PANEL_WIDTH, 40.0);
        assert_eq!(PANEL_CORNER_RADIUS, 14.0);

        hud.show_volume(18, false, Some("Speakers".into()));
        assert!(hud.is_adjusting());
        for _ in 0..90 {
            hud.tick(1.0 / 60.0);
        }
        assert!(!hud.is_adjusting());

        hud.set_dragging(true);
        assert!(hud.is_adjusting());
        hud.set_dragging(false);
        assert!(hud.is_adjusting());

        hud.set_devices(vec![
            AudioDeviceSlot {
                id: "dev-1".into(),
                name: "Speakers".into(),
                percent: 65,
                muted: false,
                is_default: true,
            },
            AudioDeviceSlot {
                id: "dev-2".into(),
                name: "Headphones".into(),
                percent: 40,
                muted: false,
                is_default: false,
            },
        ]);

        hud.set_icon_hovered(true);
        assert!(hud.icon_hovered);
        let icon_rect = hud.icon_button_rect(surf_rect).expect("icon rect");
        assert!(hud.hit_test_icon_button(
            icon_rect.x + icon_rect.width * 0.5,
            icon_rect.y + icon_rect.height * 0.5,
            surf_rect
        ));
        assert!(hud.toggle_mixer());
        assert!(hud.mixer_open);

        for _ in 0..90 {
            hud.tick(1.0 / 60.0);
        }
        let expanded = hud.computed_rect(240.0, 240.0);
        assert!((expanded.width - MIXER_PANEL_WIDTH).abs() < 0.5);

        let dev_rects = hud.device_slider_rects(surf_rect);
        assert_eq!(dev_rects.len(), 2);
        let (first_id, first_bar) = &dev_rects[0];
        assert_eq!(first_id, "dev-1");
        let hit = hud
            .hit_test_device_slider(
                first_bar.x + first_bar.width * 0.5,
                first_bar.y + first_bar.height * 0.5,
                surf_rect,
            )
            .expect("device slider hit");
        assert_eq!(hit.0, "dev-1");
        assert!((hit.1 as i32 - 50).abs() <= 1);
    }
}
