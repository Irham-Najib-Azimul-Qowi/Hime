//! Native 3-edge Dynamic Island / Notch surface (`NotchView`) rendered via Direct2D.

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::SpringValue;
use crate::d2d::{HimeExpression, NativeSurface, Rect, TextAlign};
use crate::theme::{
    ACCENT_CYAN, CORNER_RADIUS, GLASS_BORDER, NOTCH_SURFACE, SAFE_PADDING, STATUS_ALERT,
    STATUS_OFFLINE, STATUS_SUCCESS, STATUS_WARNING, SURFACE_ELEVATED, TEXT_PRIMARY, TEXT_SECONDARY,
};

const HIDDEN_BAR_THICKNESS: f32 = 3.0;
const TOP_COLLAPSED_WIDTH: f32 = 320.0;
const TOP_COLLAPSED_HEIGHT: f32 = 38.0;
const TOP_EXPANDED_WIDTH: f32 = 520.0;
const TOP_EXPANDED_HEIGHT: f32 = 164.0;
const SIDE_COLLAPSED_WIDTH: f32 = 54.0;
const SIDE_COLLAPSED_HEIGHT: f32 = 220.0;
const SIDE_EXPANDED_WIDTH: f32 = 300.0;
const SIDE_EXPANDED_HEIGHT: f32 = 360.0;

/// Screen edge where the Dynamic Island / Notch is anchored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotchEdge {
    #[default]
    Top,
    Left,
    Right,
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

/// Native Direct2D Dynamic Island / Notch view supporting Top, Left, and Right edges.
#[derive(Debug, Clone)]
pub struct NotchView {
    pub edge: NotchEdge,
    pub is_hidden: bool,
    pub is_expanded: bool,
    pub clock_hours: u8,
    pub clock_minutes: u8,
    pub net: NetStats,
    pub battery: BatteryStatus,
    pub expression: HimeExpression,
    pub blink_progress: f32,
    pub music: Option<MusicInfo>,
    pub ncm_center_alert: Option<String>,
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
        let (w, h) = target_dimensions(edge, false, false);
        Self {
            edge,
            is_hidden: false,
            is_expanded: false,
            clock_hours: 12,
            clock_minutes: 0,
            net: NetStats::default(),
            battery: BatteryStatus::default(),
            expression: HimeExpression::Netral,
            blink_progress: 0.0,
            music: None,
            ncm_center_alert: None,
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
        }
        self.sync_targets();
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        if self.is_hidden && expanded {
            self.is_hidden = false;
        }
        self.is_expanded = expanded;
        self.sync_targets();
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

    pub fn set_ncm_center_alert(&mut self, alert: Option<String>) {
        self.ncm_center_alert = alert.filter(|s| !s.trim().is_empty());
    }

    /// Formats clock text horizontally (`HH:MM`) for Top edge or vertically (`HH\nMM`) for Left/Right.
    pub fn formatted_clock(&self) -> String {
        match self.edge {
            NotchEdge::Top => format!("{:02}:{:02}", self.clock_hours, self.clock_minutes),
            NotchEdge::Left | NotchEdge::Right => {
                format!("{:02}\n{:02}", self.clock_hours, self.clock_minutes)
            }
        }
    }

    /// Advances size and expansion springs by `dt` seconds.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.width_spring.tick(dt);
        self.height_spring.tick(dt);
        self.expand_spring.tick(dt);
        !self.width_spring.is_settled()
            || !self.height_spring.is_settled()
            || !self.expand_spring.is_settled()
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

    /// Renders the notch onto `surface`.
    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let (sw, sh) = surface.size();
        let rect = self.computed_rect(sw as f32, sh as f32);

        if self.is_hidden && self.width_spring.is_settled() && self.height_spring.is_settled() {
            let bar_color = if self.net.is_internet_down {
                STATUS_OFFLINE
            } else {
                ACCENT_CYAN.with_alpha(0.75)
            };
            surface.fill_rounded_rect(rect, 1.5, bar_color)?;
            return Ok(());
        }

        let radius = if self.is_hidden { 2.0 } else { CORNER_RADIUS };
        surface.fill_rounded_rect(rect, radius, NOTCH_SURFACE)?;
        surface.stroke_rounded_rect(rect, radius, GLASS_BORDER, 1.0)?;

        if self.edge == NotchEdge::Top && !self.is_hidden {
            surface.draw_edge_surface_ears(rect, 12.0, NOTCH_SURFACE)?;
        }

        if self.is_hidden {
            return Ok(());
        }

        let inner = self.content_rect(sw as f32, sh as f32);
        match self.edge {
            NotchEdge::Top => self.render_top_contents(surface, rect, inner)?,
            NotchEdge::Left | NotchEdge::Right => self.render_side_contents(surface, rect, inner)?,
        }

        Ok(())
    }

    fn render_top_contents(
        &self,
        surface: &NativeSurface,
        outer: Rect,
        inner: Rect,
    ) -> Result<()> {
        // Center Hime mascot eyes
        let eye_cx = outer.x + outer.width * 0.5;
        let eye_cy = outer.y + 19.0;
        surface.draw_hime_eyes(eye_cx, eye_cy, 1.0, self.expression, self.blink_progress)?;

        // Left: Clock
        let clock_rect = Rect::new(inner.x, outer.y + 6.0, 68.0, 26.0);
        surface.draw_text(
            &self.formatted_clock(),
            clock_rect,
            13.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        // Right: Net speed (or offline icon-only) + Battery
        let net_rect = Rect::new(inner.x + inner.width - 132.0, outer.y + 6.0, 74.0, 26.0);
        if self.net.is_internet_down {
            surface.draw_text(
                "⊘",
                net_rect,
                13.0,
                STATUS_OFFLINE,
                TextAlign::Trailing,
            )?;
        } else {
            surface.draw_text(
                &format!("↓{}", format_net_speed(self.net.down_kbps)),
                net_rect,
                11.0,
                ACCENT_CYAN,
                TextAlign::Trailing,
            )?;
        }

        if self.battery.present {
            let bat_color = if self.battery.charging {
                STATUS_SUCCESS
            } else if self.battery.percent <= 20 {
                STATUS_WARNING
            } else {
                TEXT_SECONDARY
            };
            let bat_rect = Rect::new(inner.x + inner.width - 52.0, outer.y + 6.0, 52.0, 26.0);
            surface.draw_text(
                &format!("{}%", self.battery.percent),
                bat_rect,
                12.0,
                bat_color,
                TextAlign::Trailing,
            )?;
        }

        // Expanded or NCM center alert area
        if let Some(alert) = &self.ncm_center_alert {
            let alert_rect = Rect::new(inner.x, outer.y + 42.0, inner.width, 28.0);
            surface.fill_rounded_rect(alert_rect, 10.0, STATUS_ALERT.with_alpha(0.18))?;
            surface.draw_text(alert, alert_rect, 12.0, STATUS_ALERT, TextAlign::Center)?;
        }

        if self.is_expanded || self.expand_spring.value > 0.25 {
            if let Some(music) = &self.music {
                let card_y = outer.y + if self.ncm_center_alert.is_some() { 76.0 } else { 46.0 };
                let card = Rect::new(inner.x, card_y, inner.width, 64.0);
                surface.fill_rounded_rect(card, 12.0, SURFACE_ELEVATED)?;

                let title_rect = Rect::new(card.x + 12.0, card.y + 6.0, card.width - 24.0, 20.0);
                surface.draw_text(
                    &format!("{} — {}", music.title, music.artist),
                    title_rect,
                    12.5,
                    TEXT_PRIMARY,
                    TextAlign::Leading,
                )?;

                if let Some(lyric) = &music.lyric_line {
                    let lyric_rect = Rect::new(card.x + 12.0, card.y + 30.0, card.width - 24.0, 22.0);
                    surface.draw_text(lyric, lyric_rect, 11.5, ACCENT_CYAN, TextAlign::Leading)?;
                }
            }
        }

        Ok(())
    }

    fn render_side_contents(
        &self,
        surface: &NativeSurface,
        outer: Rect,
        inner: Rect,
    ) -> Result<()> {
        let cx = outer.x + outer.width * 0.5;
        surface.draw_hime_eyes(cx, outer.y + 26.0, 0.9, self.expression, self.blink_progress)?;

        let clock_rect = Rect::new(outer.x + 4.0, outer.y + 46.0, outer.width - 8.0, 42.0);
        surface.draw_text(
            &self.formatted_clock(),
            clock_rect,
            13.0,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        let net_rect = Rect::new(outer.x + 4.0, outer.y + 98.0, outer.width - 8.0, 24.0);
        if self.net.is_internet_down {
            surface.draw_text("⊘", net_rect, 13.0, STATUS_OFFLINE, TextAlign::Center)?;
        } else {
            surface.draw_text(
                &format_net_speed(self.net.down_kbps),
                net_rect,
                10.5,
                ACCENT_CYAN,
                TextAlign::Center,
            )?;
        }

        if self.battery.present {
            let bat_rect = Rect::new(outer.x + 4.0, outer.y + outer.height - 34.0, outer.width - 8.0, 22.0);
            surface.draw_text(
                &format!("{}%", self.battery.percent),
                bat_rect,
                11.0,
                TEXT_SECONDARY,
                TextAlign::Center,
            )?;
        }

        if self.is_expanded {
            if let Some(alert) = &self.ncm_center_alert {
                let alert_rect = Rect::new(inner.x, outer.y + 132.0, inner.width, 26.0);
                surface.fill_rounded_rect(alert_rect, 8.0, STATUS_ALERT.with_alpha(0.18))?;
                surface.draw_text(alert, alert_rect, 11.5, STATUS_ALERT, TextAlign::Center)?;
            }
            if let Some(music) = &self.music {
                let card = Rect::new(inner.x, outer.y + 166.0, inner.width, 64.0);
                surface.fill_rounded_rect(card, 12.0, SURFACE_ELEVATED)?;
                surface.draw_text(
                    &music.title,
                    Rect::new(card.x + 10.0, card.y + 8.0, card.width - 20.0, 20.0),
                    12.0,
                    TEXT_PRIMARY,
                    TextAlign::Leading,
                )?;
                if let Some(lyric) = &music.lyric_line {
                    surface.draw_text(
                        lyric,
                        Rect::new(card.x + 10.0, card.y + 32.0, card.width - 20.0, 20.0),
                        11.0,
                        ACCENT_CYAN,
                        TextAlign::Leading,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn sync_targets(&mut self) {
        let (w, h) = target_dimensions(self.edge, self.is_hidden, self.is_expanded);
        self.width_spring.set_target(w);
        self.height_spring.set_target(h);
        self.expand_spring
            .set_target(if self.is_expanded && !self.is_hidden { 1.0 } else { 0.0 });
    }
}

fn target_dimensions(edge: NotchEdge, is_hidden: bool, is_expanded: bool) -> (f32, f32) {
    match (edge, is_hidden, is_expanded) {
        (NotchEdge::Top, true, _) => (120.0, HIDDEN_BAR_THICKNESS),
        (NotchEdge::Left | NotchEdge::Right, true, _) => (HIDDEN_BAR_THICKNESS, 120.0),
        (NotchEdge::Top, false, false) => (TOP_COLLAPSED_WIDTH, TOP_COLLAPSED_HEIGHT),
        (NotchEdge::Top, false, true) => (TOP_EXPANDED_WIDTH, TOP_EXPANDED_HEIGHT),
        (NotchEdge::Left | NotchEdge::Right, false, false) => {
            (SIDE_COLLAPSED_WIDTH, SIDE_COLLAPSED_HEIGHT)
        }
        (NotchEdge::Left | NotchEdge::Right, false, true) => {
            (SIDE_EXPANDED_WIDTH, SIDE_EXPANDED_HEIGHT)
        }
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

    #[test]
    fn hidden_mode_collapses_to_3px_flush_bar() {
        let mut notch = NotchView::new(NotchEdge::Top);
        notch.set_hidden(true);
        for _ in 0..120 {
            notch.tick(1.0 / 60.0);
        }
        let top_rect = notch.computed_rect(1920.0, 1080.0);
        assert_eq!(top_rect.y, 0.0);
        assert!((top_rect.height - 3.0).abs() < 0.1);

        notch.set_edge(NotchEdge::Right);
        for _ in 0..120 {
            notch.tick(1.0 / 60.0);
        }
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
        for _ in 0..120 {
            notch.tick(1.0 / 60.0);
        }
        let outer = notch.computed_rect(800.0, 400.0);
        let inner = notch.content_rect(800.0, 400.0);
        assert_eq!(inner.x - outer.x, 24.0);
        assert_eq!(outer.width - inner.width, 48.0);
    }
}
