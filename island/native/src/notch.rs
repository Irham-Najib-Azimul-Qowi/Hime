//! Native 3-edge Dynamic Island / Notch surface (`NotchView`) rendered via Direct2D.

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::SpringValue;
use crate::d2d::{EdgeSide, HimeExpression, NativeSurface, Rect, TextAlign};
use crate::theme::{
    ACCENT_CYAN, CORNER_RADIUS, DOCK_SURFACE, GLASS_BORDER, NOTCH_BG_SOLID, SAFE_PADDING,
    STATUS_ALERT, STATUS_OFFLINE, STATUS_SUCCESS, STATUS_WARNING, SURFACE_ELEVATED, TEXT_PRIMARY,
    TEXT_SECONDARY,
};

const HIDDEN_BAR_THICKNESS: f32 = 4.0;
const TOP_COLLAPSED_WIDTH: f32 = 256.0;
const TOP_ALERT_WIDTH: f32 = 340.0;
const TOP_COLLAPSED_HEIGHT: f32 = 36.0;
const TOP_EXPANDED_WIDTH: f32 = 396.0;
const TOP_EXPANDED_HEIGHT: f32 = 236.0;
const SIDE_COLLAPSED_WIDTH: f32 = 44.0;
const SIDE_COLLAPSED_HEIGHT: f32 = 196.0;
const SIDE_ALERT_HEIGHT: f32 = 256.0;
const SIDE_EXPANDED_WIDTH: f32 = 300.0;
const SIDE_EXPANDED_HEIGHT: f32 = 360.0;
const EAR_WIDTH: f32 = 13.0;

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
        let (w, h) = target_dimensions(edge, false, false, false);
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
        self.sync_targets();
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
        let (sw, sh) = surface.dip_size();
        let rect = self.computed_rect(sw, sh);
        let edge_side = match self.edge {
            NotchEdge::Top => EdgeSide::Top,
            NotchEdge::Left => EdgeSide::Left,
            NotchEdge::Right => EdgeSide::Right,
        };

        if self.is_hidden && self.width_spring.is_settled() && self.height_spring.is_settled() {
            let bar_color = if self.net.is_internet_down {
                STATUS_OFFLINE
            } else {
                ACCENT_CYAN.with_alpha(0.82)
            };
            surface.set_rounded_region(rect, 2.0);
            surface.fill_rounded_rect(rect, 2.0, bar_color)?;
            return Ok(());
        }

        let radius = if self.is_hidden { 2.0 } else { CORNER_RADIUS };
        let flares = !self.is_hidden;
        surface.set_edge_surface_region(rect, EAR_WIDTH, radius, edge_side, flares);
        surface.draw_edge_surface(
            rect,
            EAR_WIDTH,
            radius,
            edge_side,
            flares,
            NOTCH_BG_SOLID,
            Some(GLASS_BORDER),
        )?;

        if self.is_hidden {
            return Ok(());
        }

        let inner = self.content_rect(sw, sh);
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
        let is_playing = self.music.as_ref().is_some_and(|m| m.is_playing);
        let has_alert = self.ncm_center_alert.is_some()
            || self.net.is_internet_down
            || self.expression != HimeExpression::Netral;

        // Left slot (x = +18..+96): Music visualizer when playing, else Wi-Fi + net speed
        if is_playing {
            surface.draw_music_visualizer(
                outer.x + 22.0,
                outer.y + 10.0,
                16.0,
                &[0.55, 0.92, 0.48, 0.80],
                true,
            )?;
            let speed_rect = Rect::new(outer.x + 42.0, outer.y + 6.0, 54.0, 24.0);
            surface.draw_text("PLAY", speed_rect, 10.0, ACCENT_CYAN, TextAlign::Leading)?;
        } else {
            let wifi_color = if self.net.is_internet_down {
                STATUS_ALERT
            } else {
                ACCENT_CYAN
            };
            surface.draw_wifi_icon(
                outer.x + 26.0,
                outer.y + 18.0,
                13.0,
                !self.net.is_internet_down,
                self.net.is_internet_down,
                wifi_color,
            )?;
            if !self.net.is_internet_down {
                let speed_rect = Rect::new(outer.x + 36.0, outer.y + 6.0, 62.0, 24.0);
                surface.draw_text(
                    &format!("↓{}", format_net_speed(self.net.down_kbps)),
                    speed_rect,
                    10.5,
                    TEXT_SECONDARY,
                    TextAlign::Leading,
                )?;
            }
        }

        // Center slot: Bold white clock by default; HimeEyes + short alert when alert/offline/non-neutral
        let center_rect = Rect::new(
            outer.x + 96.0,
            outer.y + 5.0,
            (outer.width - 192.0).max(48.0),
            26.0,
        );
        if has_alert {
            let eye_cx = if self.ncm_center_alert.is_some() {
                center_rect.x + 18.0
            } else {
                outer.x + outer.width * 0.5
            };
            surface.draw_hime_eyes(eye_cx, outer.y + 18.0, 0.88, self.expression, self.blink_progress)?;
            if let Some(alert) = &self.ncm_center_alert {
                let msg_rect = Rect::new(
                    center_rect.x + 34.0,
                    outer.y + 6.0,
                    (center_rect.width - 36.0).max(40.0),
                    24.0,
                );
                surface.draw_text(alert, msg_rect, 11.0, STATUS_ALERT, TextAlign::Leading)?;
            }
        } else {
            surface.draw_text(
                &self.formatted_clock(),
                center_rect,
                14.0,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;
        }

        // Right slot (x = outer.width - 96 .. -18): Battery % + crisp vector battery icon
        if self.battery.present {
            let bat_color = if self.battery.charging {
                STATUS_SUCCESS
            } else if self.battery.percent <= 20 {
                STATUS_WARNING
            } else {
                TEXT_PRIMARY
            };
            let bat_text_rect = Rect::new(outer.x + outer.width - 84.0, outer.y + 6.0, 38.0, 24.0);
            surface.draw_text(
                &format!("{}%", self.battery.percent),
                bat_text_rect,
                11.0,
                bat_color,
                TextAlign::Trailing,
            )?;
            surface.draw_battery_icon(
                outer.x + outer.width - 42.0,
                outer.y + 12.5,
                20.0,
                11.0,
                self.battery.percent,
                self.battery.charging,
            )?;
        }

        // Expanded Command Center panel below the 36px header
        if self.is_expanded || self.expand_spring.value > 0.25 {
            let tiles = [
                ("Wi-Fi", if self.net.is_internet_down { STATUS_ALERT } else { ACCENT_CYAN }),
                ("Bluetooth", TEXT_PRIMARY),
                ("AI Chat", ACCENT_CYAN),
                ("Settings", TEXT_SECONDARY),
                ("Vol/Bright", TEXT_SECONDARY),
            ];
            let gap = 8.0;
            let tile_w = ((inner.width - gap * 4.0) / 5.0).max(44.0);
            let tile_y = outer.y + 46.0;
            for (idx, (label, accent)) in tiles.iter().enumerate() {
                let tx = inner.x + (idx as f32) * (tile_w + gap);
                let tile_rect = Rect::new(tx, tile_y, tile_w, 54.0);
                surface.fill_rounded_rect(tile_rect, 12.0, DOCK_SURFACE)?;
                surface.stroke_rounded_rect(tile_rect, 12.0, GLASS_BORDER, 1.0)?;
                surface.draw_text(label, tile_rect, 10.5, *accent, TextAlign::Center)?;
            }

            if let Some(music) = &self.music {
                let card = Rect::new(inner.x, outer.y + 112.0, inner.width, 76.0);
                surface.fill_rounded_rect(card, 14.0, SURFACE_ELEVATED)?;
                surface.stroke_rounded_rect(card, 14.0, GLASS_BORDER, 1.0)?;

                surface.draw_music_visualizer(
                    card.x + 14.0,
                    card.y + 14.0,
                    18.0,
                    &[0.5, 0.9, 0.65, 0.85],
                    music.is_playing,
                )?;
                let title_rect = Rect::new(card.x + 38.0, card.y + 10.0, card.width - 52.0, 22.0);
                surface.draw_text(
                    &format!("{} — {}", music.title, music.artist),
                    title_rect,
                    12.5,
                    TEXT_PRIMARY,
                    TextAlign::Leading,
                )?;

                if let Some(lyric) = &music.lyric_line {
                    let lyric_rect = Rect::new(card.x + 14.0, card.y + 40.0, card.width - 28.0, 24.0);
                    surface.draw_text(lyric, lyric_rect, 11.5, ACCENT_CYAN, TextAlign::Leading)?;
                }
            } else {
                let status_card = Rect::new(inner.x, outer.y + 112.0, inner.width, 76.0);
                surface.fill_rounded_rect(status_card, 14.0, SURFACE_ELEVATED)?;
                surface.stroke_rounded_rect(status_card, 14.0, GLASS_BORDER, 1.0)?;
                surface.draw_text(
                    "Command Center • No Media Playing",
                    status_card,
                    11.5,
                    TEXT_SECONDARY,
                    TextAlign::Center,
                )?;
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
        let has_alert = self.ncm_center_alert.is_some()
            || self.net.is_internet_down
            || self.expression != HimeExpression::Netral;

        // Top: Wi-Fi icon + speed
        let wifi_color = if self.net.is_internet_down {
            STATUS_ALERT
        } else {
            ACCENT_CYAN
        };
        surface.draw_wifi_icon(
            cx,
            outer.y + 24.0,
            12.0,
            !self.net.is_internet_down,
            self.net.is_internet_down,
            wifi_color,
        )?;
        if !self.net.is_internet_down {
            let net_rect = Rect::new(outer.x + 2.0, outer.y + 38.0, outer.width - 4.0, 20.0);
            surface.draw_text(
                &format_net_speed(self.net.down_kbps),
                net_rect,
                9.0,
                TEXT_SECONDARY,
                TextAlign::Center,
            )?;
        }

        // Center: Vertical clock (HH\nMM) or HimeEyes
        if has_alert {
            surface.draw_hime_eyes(cx, outer.y + outer.height * 0.5, 0.8, self.expression, self.blink_progress)?;
        } else {
            let clock_rect = Rect::new(outer.x + 2.0, outer.y + outer.height * 0.5 - 24.0, outer.width - 4.0, 48.0);
            surface.draw_text(
                &self.formatted_clock(),
                clock_rect,
                13.0,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;
        }

        // Bottom: Battery icon + %
        if self.battery.present {
            surface.draw_battery_icon(
                cx - 10.0,
                outer.y + outer.height - 46.0,
                20.0,
                10.0,
                self.battery.percent,
                self.battery.charging,
            )?;
            let bat_rect = Rect::new(outer.x + 2.0, outer.y + outer.height - 32.0, outer.width - 4.0, 18.0);
            surface.draw_text(
                &format!("{}%", self.battery.percent),
                bat_rect,
                10.0,
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
        let has_alert = self.ncm_center_alert.is_some();
        let (w, h) = target_dimensions(self.edge, self.is_hidden, self.is_expanded, has_alert);
        self.width_spring.set_target(w);
        self.height_spring.set_target(h);
        self.expand_spring
            .set_target(if self.is_expanded && !self.is_hidden { 1.0 } else { 0.0 });
    }
}

fn target_dimensions(
    edge: NotchEdge,
    is_hidden: bool,
    is_expanded: bool,
    has_alert: bool,
) -> (f32, f32) {
    match (edge, is_hidden, is_expanded) {
        (NotchEdge::Top, true, _) => (48.0, HIDDEN_BAR_THICKNESS),
        (NotchEdge::Left | NotchEdge::Right, true, _) => (HIDDEN_BAR_THICKNESS, 48.0),
        (NotchEdge::Top, false, false) => (
            if has_alert {
                TOP_ALERT_WIDTH
            } else {
                TOP_COLLAPSED_WIDTH
            },
            TOP_COLLAPSED_HEIGHT,
        ),
        (NotchEdge::Top, false, true) => (TOP_EXPANDED_WIDTH, TOP_EXPANDED_HEIGHT),
        (NotchEdge::Left | NotchEdge::Right, false, false) => (
            SIDE_COLLAPSED_WIDTH,
            if has_alert {
                SIDE_ALERT_HEIGHT
            } else {
                SIDE_COLLAPSED_HEIGHT
            },
        ),
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
        assert!((top_rect.height - 4.0).abs() < 0.1);

        notch.set_edge(NotchEdge::Right);
        for _ in 0..120 {
            notch.tick(1.0 / 60.0);
        }
        let right_rect = notch.computed_rect(1920.0, 1080.0);
        assert!((right_rect.width - 4.0).abs() < 0.1);
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
