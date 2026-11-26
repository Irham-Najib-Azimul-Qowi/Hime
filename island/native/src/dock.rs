//! Native macOS-style floating Dock bar (`DockBar`) rendered via Direct2D.

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::SpringValue;
use crate::d2d::{NativeSurface, Rect, TextAlign};
use crate::theme::{
    ACCENT_CYAN, CORNER_RADIUS, GLASS_BORDER, GLASS_SURFACE, STATUS_ALERT, SURFACE_ELEVATED,
    TEXT_PRIMARY, TEXT_SECONDARY,
};

const ITEM_BASE_SIZE: f32 = 42.0;
const ITEM_GAP: f32 = 10.0;
const DOCK_PAD_X: f32 = 14.0;
const DOCK_PAD_Y: f32 = 10.0;
const TRAY_CHIP_WIDTH: f32 = 52.0;
const HOVER_SCALE_TARGET: f32 = 1.22;

/// An application entry inside [`DockBar`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockItem {
    pub id: String,
    pub label: String,
    pub executable: String,
    pub pinned: bool,
    pub running: bool,
    pub badge_count: u32,
    #[serde(skip, default = "default_item_scale")]
    pub hover_scale: SpringValue,
}

fn default_item_scale() -> SpringValue {
    SpringValue::snappy(1.0)
}

impl DockItem {
    /// Creates a new dock item at resting scale `1.0`.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        executable: impl Into<String>,
        pinned: bool,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            executable: executable.into(),
            pinned,
            running: false,
            badge_count: 0,
            hover_scale: SpringValue::snappy(1.0),
        }
    }
}

/// Floating bottom Dock bar with spring hover magnification and hit-testing.
#[derive(Debug, Clone)]
pub struct DockBar {
    items: Vec<DockItem>,
    pub system_tray_count: u32,
    pub visible: bool,
    pub visibility: SpringValue,
    hovered_index: Option<usize>,
}

impl Default for DockBar {
    fn default() -> Self {
        Self::new()
    }
}

impl DockBar {
    /// Creates an empty, visible `DockBar`.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            system_tray_count: 0,
            visible: true,
            visibility: SpringValue::gentle(1.0),
            hovered_index: None,
        }
    }

    pub fn items(&self) -> &[DockItem] {
        &self.items
    }

    pub fn hovered_index(&self) -> Option<usize> {
        self.hovered_index
    }

    /// Adds or updates a dock item by `id`.
    pub fn add_item(&mut self, item: DockItem) {
        if let Some(existing) = self.items.iter_mut().find(|it| it.id == item.id) {
            existing.label = item.label;
            existing.executable = item.executable;
            existing.pinned = item.pinned;
            existing.running = item.running;
            existing.badge_count = item.badge_count;
        } else {
            self.items.push(item);
        }
    }

    /// Removes an item by `id`. Returns `true` if an item was removed.
    pub fn remove_item(&mut self, id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|it| it.id != id);
        if let Some(idx) = self.hovered_index {
            if idx >= self.items.len() {
                self.hovered_index = None;
            }
        }
        self.items.len() != before
    }

    /// Updates running state for an item by `id`.
    pub fn set_running(&mut self, id: &str, running: bool) {
        if let Some(item) = self.items.iter_mut().find(|it| it.id == id) {
            item.running = running;
        }
    }

    /// Updates `visible` and animates `visibility` target.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        self.visibility.set_target(if visible { 1.0 } else { 0.0 });
        if !visible {
            self.set_hovered(None);
        }
    }

    /// Updates the system tray badge count.
    pub fn set_system_tray_count(&mut self, count: u32) {
        self.system_tray_count = count;
    }

    /// Sets the hovered item index and updates item spring targets.
    pub fn set_hovered(&mut self, index: Option<usize>) {
        let valid = index.filter(|&i| i < self.items.len());
        self.hovered_index = valid;
        for (i, item) in self.items.iter_mut().enumerate() {
            let target = match valid {
                Some(h) if h == i => HOVER_SCALE_TARGET,
                Some(h) if h.abs_diff(i) == 1 => 1.08,
                _ => 1.0,
            };
            item.hover_scale.set_target(target);
        }
    }

    /// Performs hit-testing at `(x, y)` and updates hover state accordingly.
    pub fn set_hovered_at(&mut self, x: f32, y: f32, surface_rect: Rect) -> Option<usize> {
        let hit = self.hit_test(x, y, surface_rect);
        self.set_hovered(hit);
        hit
    }

    /// Advances visibility and all item hover springs by `dt` seconds.
    /// Returns `true` if any spring is still moving.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.visibility.tick(dt);
        let mut animating = !self.visibility.is_settled();
        for item in &mut self.items {
            item.hover_scale.tick(dt);
            if !item.hover_scale.is_settled() {
                animating = true;
            }
        }
        animating
    }

    /// Computes the outer dock container rectangle centered horizontally inside `surface_rect`.
    pub fn dock_rect(&self, surface_rect: Rect) -> Rect {
        let count = self.items.len() as f32;
        let items_width = if self.items.is_empty() {
            ITEM_BASE_SIZE
        } else {
            count * ITEM_BASE_SIZE + (count - 1.0) * ITEM_GAP
        };
        let tray_extra = if self.system_tray_count > 0 {
            ITEM_GAP + TRAY_CHIP_WIDTH
        } else {
            0.0
        };
        let width = DOCK_PAD_X * 2.0 + items_width + tray_extra;
        let height = DOCK_PAD_Y * 2.0 + ITEM_BASE_SIZE + 8.0;
        let x = surface_rect.x + (surface_rect.width - width) * 0.5;
        let y_offset = (1.0 - self.visibility.value.clamp(0.0, 1.0)) * 28.0;
        let y = surface_rect.y + surface_rect.height - height - 8.0 + y_offset;
        Rect::new(x, y, width, height)
    }

    /// Returns the bounding rectangles for each item in order.
    pub fn item_rects(&self, surface_rect: Rect) -> Vec<Rect> {
        let dock = self.dock_rect(surface_rect);
        let mut rects = Vec::with_capacity(self.items.len());
        let base_y = dock.y + DOCK_PAD_Y;

        for (i, item) in self.items.iter().enumerate() {
            let scale = item.hover_scale.value.clamp(0.8, 1.4);
            let size = ITEM_BASE_SIZE * scale;
            let cx = dock.x + DOCK_PAD_X + (i as f32) * (ITEM_BASE_SIZE + ITEM_GAP) + ITEM_BASE_SIZE * 0.5;
            let x = cx - size * 0.5;
            let y = base_y + (ITEM_BASE_SIZE - size);
            rects.push(Rect::new(x, y, size, size));
        }
        rects
    }

    /// Returns the index of the item under `(x, y)`, if any.
    pub fn hit_test(&self, x: f32, y: f32, surface_rect: Rect) -> Option<usize> {
        if !self.visible && self.visibility.value <= 0.01 {
            return None;
        }
        self.item_rects(surface_rect).into_iter().position(|r| {
            x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height
        })
    }

    /// Returns the `id` of the item under `(x, y)`, if any.
    pub fn hit_test_id(&self, x: f32, y: f32, surface_rect: Rect) -> Option<&str> {
        self.hit_test(x, y, surface_rect)
            .and_then(|idx| self.items.get(idx))
            .map(|it| it.id.as_str())
    }

    /// Renders the dock bar onto `surface`.
    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let alpha = self.visibility.value.clamp(0.0, 1.0);
        if alpha <= 0.01 {
            return Ok(());
        }

        let (sw, sh) = surface.size();
        let surface_rect = Rect::new(0.0, 0.0, sw as f32, sh as f32);
        let dock = self.dock_rect(surface_rect);

        surface.fill_rounded_rect(dock, CORNER_RADIUS, GLASS_SURFACE.with_alpha(0.92 * alpha))?;
        surface.stroke_rounded_rect(
            dock,
            CORNER_RADIUS,
            GLASS_BORDER.with_alpha(0.18 * alpha),
            1.0,
        )?;

        let rects = self.item_rects(surface_rect);
        for (i, (item, rect)) in self.items.iter().zip(rects.iter()).enumerate() {
            let is_hovered = self.hovered_index == Some(i);
            let tile_bg = if is_hovered {
                SURFACE_ELEVATED.with_alpha(0.98 * alpha)
            } else {
                SURFACE_ELEVATED.with_alpha(0.85 * alpha)
            };
            surface.fill_rounded_rect(*rect, 12.0, tile_bg)?;
            if is_hovered {
                surface.stroke_rounded_rect(*rect, 12.0, ACCENT_CYAN.with_alpha(0.75 * alpha), 1.5)?;
            }

            let initial = item
                .label
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();
            surface.draw_text(
                &initial,
                *rect,
                16.0,
                TEXT_PRIMARY.with_alpha(alpha),
                TextAlign::Center,
            )?;

            if item.running {
                let dot = Rect::new(
                    rect.x + rect.width * 0.5 - 3.0,
                    dock.y + dock.height - 8.0,
                    6.0,
                    3.0,
                );
                surface.fill_rounded_rect(dot, 1.5, ACCENT_CYAN.with_alpha(alpha))?;
            }

            if item.badge_count > 0 {
                let badge = Rect::new(rect.x + rect.width - 14.0, rect.y - 2.0, 16.0, 16.0);
                surface.fill_rounded_rect(badge, 8.0, STATUS_ALERT.with_alpha(alpha))?;
                let txt = if item.badge_count > 99 {
                    "99+".to_string()
                } else {
                    item.badge_count.to_string()
                };
                surface.draw_text(&txt, badge, 9.0, TEXT_PRIMARY.with_alpha(alpha), TextAlign::Center)?;
            }
        }

        if self.system_tray_count > 0 {
            let tray_rect = Rect::new(
                dock.x + dock.width - DOCK_PAD_X - TRAY_CHIP_WIDTH,
                dock.y + DOCK_PAD_Y + 6.0,
                TRAY_CHIP_WIDTH,
                ITEM_BASE_SIZE - 12.0,
            );
            surface.fill_rounded_rect(tray_rect, 10.0, SURFACE_ELEVATED.with_alpha(0.8 * alpha))?;
            surface.draw_text(
                &format!("+{}", self.system_tray_count),
                tray_rect,
                12.0,
                TEXT_SECONDARY.with_alpha(alpha),
                TextAlign::Center,
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_dock() -> DockBar {
        let mut dock = DockBar::new();
        dock.add_item(DockItem::new("files", "Files", "explorer.exe", true));
        dock.add_item(DockItem::new("term", "Terminal", "wt.exe", true));
        dock.add_item(DockItem::new("browser", "Browser", "msedge.exe", false));
        dock.set_running("term", true);
        dock.set_system_tray_count(4);
        dock
    }

    #[test]
    fn hit_test_matches_rendered_item_rects() {
        let dock = sample_dock();
        let surface_rect = Rect::new(0.0, 0.0, 800.0, 120.0);
        let rects = dock.item_rects(surface_rect);
        assert_eq!(rects.len(), 3);

        for (idx, r) in rects.iter().enumerate() {
            let cx = r.x + r.width * 0.5;
            let cy = r.y + r.height * 0.5;
            assert_eq!(dock.hit_test(cx, cy, surface_rect), Some(idx));
        }
        assert_eq!(dock.hit_test_id(rects[1].x + 5.0, rects[1].y + 5.0, surface_rect), Some("term"));
        assert_eq!(dock.hit_test(5.0, 5.0, surface_rect), None);
    }

    #[test]
    fn hover_spring_scales_target_and_neighbors() {
        let mut dock = sample_dock();
        dock.set_hovered(Some(1));
        for _ in 0..90 {
            dock.tick(1.0 / 60.0);
        }
        assert!(dock.items()[1].hover_scale.value > 1.18);
        assert!(dock.items()[0].hover_scale.value > 1.05);
        assert!((dock.items()[2].hover_scale.value - 1.08).abs() < 0.05);

        dock.set_hovered(None);
        for _ in 0..90 {
            dock.tick(1.0 / 60.0);
        }
        assert!((dock.items()[1].hover_scale.value - 1.0).abs() < 0.02);
    }

    #[test]
    fn hide_disables_hit_testing_and_animates_visibility() {
        let mut dock = sample_dock();
        let surface_rect = Rect::new(0.0, 0.0, 800.0, 120.0);
        dock.set_visible(false);
        for _ in 0..120 {
            dock.tick(1.0 / 60.0);
        }
        assert_eq!(dock.visibility.value, 0.0);
        let rects = dock.item_rects(surface_rect);
        assert_eq!(dock.hit_test(rects[0].x + 10.0, rects[0].y + 10.0, surface_rect), None);
    }
}
