//! Native macOS-style floating Dock bar (`DockBar`) rendered via Direct2D.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use windows::core::Result;
use windows::Win32::Graphics::Gdi::{
    CombineRgn, CreateRoundRectRgn, DeleteObject, SetWindowRgn, RGN_OR,
};

use crate::animation::SpringValue;
use crate::d2d::{
    extract_window_or_exe_icon, HimeExpression, IconRgba, NativeSurface, Rect, TextAlign,
};
use crate::telemetry::{group_running_windows, RunningWindow};
use crate::theme::{
    ACCENT_CYAN, CORNER_RADIUS, DOCK_SURFACE, GLASS_BORDER, STATUS_ALERT, SURFACE_ELEVATED,
    TEXT_PRIMARY, TEXT_SECONDARY,
};

const DOCK_HEIGHT: f32 = 54.0;
const ICON_BASE_SIZE: f32 = 34.0;
const ICON_CORNER_RADIUS: f32 = 8.0;
const ITEM_SLOT_SIZE: f32 = 38.0;
const ITEM_GAP: f32 = 8.0;
const DOCK_PAD_X: f32 = 12.0;
const DIVIDER_WIDTH: f32 = 1.0;
const DIVIDER_GAP: f32 = 8.0;
const UTIL_BTN_SIZE: f32 = 34.0;
const TRAY_BTN_BASE_WIDTH: f32 = 34.0;
const TRAY_BTN_BADGE_WIDTH: f32 = 44.0;
const HOVER_SCALE_TARGET: f32 = 1.22;
const MAX_RUNNING_GROUPS: usize = 12;

/// Clickable action emitted by [`DockBar::hit_test_action`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockAction {
    /// Focus or launch the application item at `index`.
    App(usize),
    /// Toggle the Hime AI Drawer.
    ToggleAiDrawer,
    /// Toggle the Hime Settings panel.
    ToggleSettings,
    /// Open the System Tray menu.
    ToggleTray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockHoverTarget {
    App(usize),
    AiDrawer,
    Settings,
    Tray,
}

/// An application entry inside [`DockBar`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockItem {
    pub id: String,
    pub label: String,
    pub executable: String,
    #[serde(default)]
    pub exe_path: String,
    pub pinned: bool,
    pub running: bool,
    pub badge_count: u32,
    #[serde(default)]
    pub hwnds: Vec<isize>,
    #[serde(skip, default)]
    pub icon: Option<IconRgba>,
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
            exe_path: String::new(),
            pinned,
            running: false,
            badge_count: 0,
            hwnds: Vec::new(),
            icon: None,
            hover_scale: SpringValue::snappy(1.0),
        }
    }
}

/// Floating bottom Dock bar with real app icons, grouping, spring hover zoom, and tooltips.
#[derive(Debug, Clone)]
pub struct DockBar {
    items: Vec<DockItem>,
    icon_cache: HashMap<String, Option<IconRgba>>,
    pub system_tray_count: u32,
    pub visible: bool,
    pub visibility: SpringValue,
    hovered_index: Option<usize>,
    hovered_target: Option<DockHoverTarget>,
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
            icon_cache: HashMap::new(),
            system_tray_count: 0,
            visible: true,
            visibility: SpringValue::gentle(1.0),
            hovered_index: None,
            hovered_target: None,
        }
    }

    pub fn items(&self) -> &[DockItem] {
        &self.items
    }

    pub fn hovered_index(&self) -> Option<usize> {
        self.hovered_index
    }

    /// Returns `true` when any app item or utility button is currently hovered.
    pub fn has_hover(&self) -> bool {
        self.hovered_target.is_some()
    }

    /// Clears hover state across app items and utility buttons.
    pub fn clear_hover(&mut self) {
        self.set_hovered(None);
        self.hovered_target = None;
    }

    /// Returns the number of cached icon entries (keyed by `exe_path` / fallback `hwnd`).
    pub fn icon_cache_len(&self) -> usize {
        self.icon_cache.len()
    }

    /// Resolves a 48px BGRA icon from cache or extracts it once via [`extract_window_or_exe_icon`].
    pub fn resolve_icon(&mut self, hwnd: isize, exe_path: &str) -> Option<IconRgba> {
        let trimmed = exe_path.trim();
        let cache_key = if !trimmed.is_empty() {
            trimmed.to_lowercase()
        } else if hwnd != 0 {
            format!("hwnd:{hwnd}")
        } else {
            return None;
        };

        if let Some(cached) = self.icon_cache.get(&cache_key) {
            return cached.clone();
        }

        let extracted = extract_window_or_exe_icon(hwnd, trimmed, 48);
        self.icon_cache.insert(cache_key, extracted.clone());
        extracted
    }

    /// Seeds or overrides a cached icon for `exe_path` (primarily for tests or custom icons).
    pub fn cache_icon_for_path(&mut self, exe_path: &str, icon: Option<IconRgba>) {
        let key = exe_path.trim().to_lowercase();
        if !key.is_empty() {
            self.icon_cache.insert(key, icon);
        }
    }

    /// Adds or updates a dock item by `id`.
    pub fn add_item(&mut self, item: DockItem) {
        if let Some(existing) = self.items.iter_mut().find(|it| it.id == item.id) {
            existing.label = item.label;
            existing.executable = item.executable;
            if !item.exe_path.is_empty() {
                existing.exe_path = item.exe_path;
            }
            existing.pinned = item.pinned;
            existing.running = item.running;
            existing.badge_count = item.badge_count;
            existing.hwnds = item.hwnds;
            if item.icon.is_some() {
                existing.icon = item.icon;
            }
        } else {
            self.items.push(item);
        }
    }

    /// Synchronizes live [`RunningWindow`]s into `DockBar`, grouping by `executable.to_lowercase()`,
    /// preserving pinned items and spring hover state, and caching extracted icons once per `exe_path`.
    pub fn sync_running_windows(&mut self, windows: &[RunningWindow]) {
        let groups = group_running_windows(windows);
        let mut consumed = vec![false; groups.len()];

        // 1. Update pinned items first so pinned slots remain anchored in place.
        let mut pinned_updates: Vec<(usize, bool, u32, Vec<isize>, String)> = Vec::new();
        for (item_idx, item) in self.items.iter().enumerate() {
            if !item.pinned {
                continue;
            }
            let matched = groups.iter().enumerate().find(|(g_idx, g)| {
                !consumed[*g_idx]
                    && (item.id.eq_ignore_ascii_case(&g.id)
                        || (!item.executable.is_empty()
                            && item.executable.eq_ignore_ascii_case(&g.executable)))
            });
            if let Some((g_idx, g)) = matched {
                consumed[g_idx] = true;
                pinned_updates.push((
                    item_idx,
                    true,
                    g.badge_count,
                    g.hwnds.clone(),
                    g.exe_path.clone(),
                ));
            } else {
                pinned_updates.push((item_idx, false, 0, Vec::new(), String::new()));
            }
        }

        for (item_idx, running, badge_count, hwnds, exe_path) in pinned_updates {
            let need_icon = self.items[item_idx].icon.is_none();
            let primary_hwnd = hwnds.first().copied().unwrap_or(0);
            let resolved_path = if !exe_path.is_empty() {
                exe_path.clone()
            } else {
                self.items[item_idx].exe_path.clone()
            };
            let extracted = if need_icon && (primary_hwnd != 0 || !resolved_path.is_empty()) {
                self.resolve_icon(primary_hwnd, &resolved_path)
            } else {
                None
            };
            let item = &mut self.items[item_idx];
            item.running = running;
            item.badge_count = badge_count;
            item.hwnds = hwnds;
            if !exe_path.is_empty() {
                item.exe_path = exe_path;
            }
            if item.icon.is_none() {
                item.icon = extracted;
            }
        }

        // 2. Collect remaining (unpinned) groups up to MAX_RUNNING_GROUPS.
        let remaining: Vec<DockItem> = groups
            .into_iter()
            .enumerate()
            .filter_map(|(idx, g)| if !consumed[idx] { Some(g) } else { None })
            .take(MAX_RUNNING_GROUPS)
            .collect();

        // Remove stale unpinned items whose grouped executable exited.
        self.items
            .retain(|it| it.pinned || remaining.iter().any(|g| g.id == it.id));

        // 3. Upsert remaining groups in order, extracting icon at most once per exe_path.
        for group in remaining {
            let primary_hwnd = group.hwnds.first().copied().unwrap_or(0);
            if let Some(existing_idx) = self.items.iter().position(|it| it.id == group.id) {
                let need_icon = self.items[existing_idx].icon.is_none();
                let icon = if need_icon {
                    self.resolve_icon(primary_hwnd, &group.exe_path)
                } else {
                    None
                };
                let existing = &mut self.items[existing_idx];
                existing.label = group.label;
                existing.executable = group.executable;
                if !group.exe_path.is_empty() {
                    existing.exe_path = group.exe_path;
                }
                existing.running = true;
                existing.badge_count = group.badge_count;
                existing.hwnds = group.hwnds;
                if existing.icon.is_none() {
                    existing.icon = icon;
                }
            } else {
                let mut new_item = group;
                new_item.icon = self.resolve_icon(primary_hwnd, &new_item.exe_path);
                self.items.push(new_item);
            }
        }

        if let Some(idx) = self.hovered_index {
            if idx >= self.items.len() {
                self.hovered_index = None;
                if matches!(self.hovered_target, Some(DockHoverTarget::App(_))) {
                    self.hovered_target = None;
                }
            }
        }
    }

    /// Removes an item by `id`. Returns `true` if an item was removed.
    pub fn remove_item(&mut self, id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|it| it.id != id);
        if let Some(idx) = self.hovered_index {
            if idx >= self.items.len() {
                self.hovered_index = None;
                if matches!(self.hovered_target, Some(DockHoverTarget::App(_))) {
                    self.hovered_target = None;
                }
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
            self.clear_hover();
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
        self.hovered_target = valid.map(DockHoverTarget::App);
        for (i, item) in self.items.iter_mut().enumerate() {
            let target = match valid {
                Some(h) if h == i => HOVER_SCALE_TARGET,
                Some(h) if h.abs_diff(i) == 1 => 1.08,
                _ => 1.0,
            };
            item.hover_scale.set_target(target);
        }
    }

    /// Performs hit-testing at `(x, y)` and updates hover state (items + utility buttons).
    pub fn set_hovered_at(&mut self, x: f32, y: f32, surface_rect: Rect) -> Option<usize> {
        match self.hit_test_action(x, y, surface_rect) {
            Some(DockAction::App(idx)) => {
                self.set_hovered(Some(idx));
                Some(idx)
            }
            Some(DockAction::ToggleAiDrawer) => {
                self.set_hovered(None);
                self.hovered_target = Some(DockHoverTarget::AiDrawer);
                None
            }
            Some(DockAction::ToggleSettings) => {
                self.set_hovered(None);
                self.hovered_target = Some(DockHoverTarget::Settings);
                None
            }
            Some(DockAction::ToggleTray) => {
                self.set_hovered(None);
                self.hovered_target = Some(DockHoverTarget::Tray);
                None
            }
            None => {
                self.clear_hover();
                None
            }
        }
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

    fn tray_btn_width(&self) -> f32 {
        if self.system_tray_count > 0 {
            TRAY_BTN_BADGE_WIDTH
        } else {
            TRAY_BTN_BASE_WIDTH
        }
    }

    fn items_span_width(&self) -> f32 {
        if self.items.is_empty() {
            0.0
        } else {
            let count = self.items.len() as f32;
            count * ITEM_SLOT_SIZE + (count - 1.0) * ITEM_GAP
        }
    }

    fn utilities_width(&self) -> f32 {
        // Hime AI + gap + Settings + gap + Tray
        UTIL_BTN_SIZE * 2.0 + self.tray_btn_width() + ITEM_GAP * 2.0
    }

    /// Computes the outer dock container rectangle centered horizontally inside `surface_rect`.
    pub fn dock_rect(&self, surface_rect: Rect) -> Rect {
        let items_w = self.items_span_width();
        let divider_extra = if self.items.is_empty() {
            0.0
        } else {
            DIVIDER_GAP * 2.0 + DIVIDER_WIDTH
        };
        let width = DOCK_PAD_X * 2.0 + items_w + divider_extra + self.utilities_width();
        let height = DOCK_HEIGHT;
        let x = surface_rect.x + (surface_rect.width - width) * 0.5;
        let y_offset = (1.0 - self.visibility.value.clamp(0.0, 1.0)) * 28.0;
        let y = surface_rect.y + surface_rect.height - height - 8.0 + y_offset;
        Rect::new(x, y, width, height)
    }

    /// Returns the bounding rectangles for each application item in order.
    pub fn item_rects(&self, surface_rect: Rect) -> Vec<Rect> {
        let dock = self.dock_rect(surface_rect);
        let mut rects = Vec::with_capacity(self.items.len());
        let base_y = dock.y + 7.0;

        for (i, item) in self.items.iter().enumerate() {
            let scale = item.hover_scale.value.clamp(0.8, 1.4);
            let size = ICON_BASE_SIZE * scale;
            let cx = dock.x
                + DOCK_PAD_X
                + (i as f32) * (ITEM_SLOT_SIZE + ITEM_GAP)
                + ITEM_SLOT_SIZE * 0.5;
            let x = cx - size * 0.5;
            let y = base_y + (ICON_BASE_SIZE - size) * 0.65;
            rects.push(Rect::new(x, y, size, size));
        }
        rects
    }

    /// Returns `(divider_x, ai_rect, settings_rect, tray_rect)` for the right-side controls.
    fn utility_rects(&self, dock: Rect) -> (Option<f32>, Rect, Rect, Rect) {
        let items_w = self.items_span_width();
        let (divider_x, utils_start_x) = if self.items.is_empty() {
            (None, dock.x + DOCK_PAD_X)
        } else {
            let div_x = dock.x + DOCK_PAD_X + items_w + DIVIDER_GAP;
            (Some(div_x), div_x + DIVIDER_WIDTH + DIVIDER_GAP)
        };

        let btn_y = dock.y + (dock.height - UTIL_BTN_SIZE) * 0.5;
        let ai_rect = Rect::new(utils_start_x, btn_y, UTIL_BTN_SIZE, UTIL_BTN_SIZE);
        let settings_rect = Rect::new(
            ai_rect.x + ai_rect.width + ITEM_GAP,
            btn_y,
            UTIL_BTN_SIZE,
            UTIL_BTN_SIZE,
        );
        let tray_rect = Rect::new(
            settings_rect.x + settings_rect.width + ITEM_GAP,
            btn_y,
            self.tray_btn_width(),
            UTIL_BTN_SIZE,
        );
        (divider_x, ai_rect, settings_rect, tray_rect)
    }

    /// Returns the index of the application item under `(x, y)`, if any.
    pub fn hit_test(&self, x: f32, y: f32, surface_rect: Rect) -> Option<usize> {
        if !self.visible && self.visibility.value <= 0.01 {
            return None;
        }
        self.item_rects(surface_rect)
            .into_iter()
            .position(|r| point_in_rect(x, y, r))
    }

    /// Returns the `id` of the application item under `(x, y)`, if any.
    pub fn hit_test_id(&self, x: f32, y: f32, surface_rect: Rect) -> Option<&str> {
        self.hit_test(x, y, surface_rect)
            .and_then(|idx| self.items.get(idx))
            .map(|it| it.id.as_str())
    }

    /// Hit-tests both application items and right-side utility buttons (`Hime AI`, `Settings`, `Tray`).
    pub fn hit_test_action(&self, x: f32, y: f32, surface_rect: Rect) -> Option<DockAction> {
        if !self.visible && self.visibility.value <= 0.01 {
            return None;
        }
        if let Some(idx) = self.hit_test(x, y, surface_rect) {
            return Some(DockAction::App(idx));
        }
        let dock = self.dock_rect(surface_rect);
        let (_, ai_rect, settings_rect, tray_rect) = self.utility_rects(dock);
        if point_in_rect(x, y, ai_rect) {
            Some(DockAction::ToggleAiDrawer)
        } else if point_in_rect(x, y, settings_rect) {
            Some(DockAction::ToggleSettings)
        } else if point_in_rect(x, y, tray_rect) {
            Some(DockAction::ToggleTray)
        } else {
            None
        }
    }

    fn active_tooltip(&self, surface_rect: Rect, dock: Rect) -> Option<(Rect, String)> {
        let (target_rect, label) = match self.hovered_target? {
            DockHoverTarget::App(idx) => {
                let item = self.items.get(idx)?;
                let rects = self.item_rects(surface_rect);
                let r = *rects.get(idx)?;
                (r, item.label.clone())
            }
            DockHoverTarget::AiDrawer => {
                let (_, ai_rect, _, _) = self.utility_rects(dock);
                (ai_rect, "Hime AI".to_string())
            }
            DockHoverTarget::Settings => {
                let (_, _, settings_rect, _) = self.utility_rects(dock);
                (settings_rect, "Settings".to_string())
            }
            DockHoverTarget::Tray => {
                let (_, _, _, tray_rect) = self.utility_rects(dock);
                let label = if self.system_tray_count > 0 {
                    format!("System Tray ({})", self.system_tray_count)
                } else {
                    "System Tray".to_string()
                };
                (tray_rect, label)
            }
        };

        let text_w = (label.chars().count() as f32 * 6.8 + 18.0).clamp(54.0, 210.0);
        let tip_h = 22.0;
        let tip_y = (dock.y - tip_h - 6.0).max(2.0);
        let cx = target_rect.x + target_rect.width * 0.5;
        let max_x = (surface_rect.width - text_w - 4.0).max(4.0);
        let tip_x = (cx - text_w * 0.5).clamp(4.0, max_x);
        Some((Rect::new(tip_x, tip_y, text_w, tip_h), label))
    }

    /// Renders the dock bar onto `surface`.
    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let alpha = self.visibility.value.clamp(0.0, 1.0);
        if alpha <= 0.01 {
            return Ok(());
        }

        let (sw, sh) = surface.dip_size();
        let surface_rect = Rect::new(0.0, 0.0, sw, sh);
        let dock = self.dock_rect(surface_rect);
        let tooltip = self.active_tooltip(surface_rect, dock);

        if let Some((tip_rect, _)) = &tooltip {
            set_dock_and_tooltip_region(surface, dock, CORNER_RADIUS, *tip_rect, 7.0);
        } else {
            surface.set_rounded_region(dock, CORNER_RADIUS);
        }

        // Main floating dark capsule (`DOCK_SURFACE`, height 54px, radius 18px)
        surface.fill_rounded_rect(
            dock,
            CORNER_RADIUS,
            DOCK_SURFACE.with_alpha(DOCK_SURFACE.a * alpha),
        )?;
        surface.stroke_rounded_rect(
            dock,
            CORNER_RADIUS,
            GLASS_BORDER.with_alpha(0.22 * alpha),
            1.0,
        )?;

        // Application items (real Win32 BGRA icons + fallback letter tiles)
        let rects = self.item_rects(surface_rect);
        for (i, (item, rect)) in self.items.iter().zip(rects.iter()).enumerate() {
            let is_hovered = self.hovered_target == Some(DockHoverTarget::App(i));

            if let Some(ref icon_rgba) = item.icon {
                surface.draw_bgra_bitmap(
                    *rect,
                    icon_rgba.width,
                    icon_rgba.height,
                    &icon_rgba.bgra_premul,
                    ICON_CORNER_RADIUS,
                )?;
            } else {
                let tile_bg = if is_hovered {
                    SURFACE_ELEVATED.with_alpha(0.98 * alpha)
                } else {
                    SURFACE_ELEVATED.with_alpha(0.85 * alpha)
                };
                surface.fill_rounded_rect(*rect, ICON_CORNER_RADIUS, tile_bg)?;
                surface.stroke_rounded_rect(
                    *rect,
                    ICON_CORNER_RADIUS,
                    if is_hovered {
                        ACCENT_CYAN.with_alpha(0.75 * alpha)
                    } else {
                        GLASS_BORDER.with_alpha(0.22 * alpha)
                    },
                    if is_hovered { 1.4 } else { 1.0 },
                )?;
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
                    15.0,
                    TEXT_PRIMARY.with_alpha(alpha),
                    TextAlign::Center,
                )?;
            }

            // Active cyan indicator dot (#00D2FF) centered underneath running apps
            if item.running {
                let dot = Rect::new(
                    rect.x + rect.width * 0.5 - 2.5,
                    dock.y + dock.height - 6.5,
                    5.0,
                    2.5,
                );
                surface.fill_rounded_rect(dot, 1.25, ACCENT_CYAN.with_alpha(alpha))?;
            }

            // Multi-window / notification count badge
            if item.badge_count > 0 {
                let badge = Rect::new(rect.x + rect.width - 12.0, rect.y - 2.0, 15.0, 15.0);
                surface.fill_rounded_rect(badge, 7.5, STATUS_ALERT.with_alpha(alpha))?;
                let txt = if item.badge_count > 99 {
                    "99+".to_string()
                } else {
                    item.badge_count.to_string()
                };
                surface.draw_text(
                    &txt,
                    badge,
                    8.5,
                    TEXT_PRIMARY.with_alpha(alpha),
                    TextAlign::Center,
                )?;
            }
        }

        // Thin vertical separator + right-side Hime AI / Settings / Tray controls
        let (divider_x, ai_rect, settings_rect, tray_rect) = self.utility_rects(dock);
        if let Some(dx) = divider_x {
            let div_rect = Rect::new(dx, dock.y + (dock.height - 26.0) * 0.5, DIVIDER_WIDTH, 26.0);
            surface.fill_rounded_rect(div_rect, 0.5, GLASS_BORDER.with_alpha(0.24 * alpha))?;
        }

        self.render_utility_buttons(surface, ai_rect, settings_rect, tray_rect, alpha)?;

        // Hover tooltip pill above the hovered item
        if let Some((tip_rect, label)) = tooltip {
            surface.fill_rounded_rect(tip_rect, 7.0, DOCK_SURFACE.with_alpha(0.96 * alpha))?;
            surface.stroke_rounded_rect(tip_rect, 7.0, GLASS_BORDER.with_alpha(0.26 * alpha), 1.0)?;
            surface.draw_text(
                &label,
                tip_rect,
                11.5,
                TEXT_PRIMARY.with_alpha(alpha),
                TextAlign::Center,
            )?;
        }

        Ok(())
    }

    fn render_utility_buttons(
        &self,
        surface: &NativeSurface,
        ai_rect: Rect,
        settings_rect: Rect,
        tray_rect: Rect,
        alpha: f32,
    ) -> Result<()> {
        let ai_hovered = self.hovered_target == Some(DockHoverTarget::AiDrawer);
        let settings_hovered = self.hovered_target == Some(DockHoverTarget::Settings);
        let tray_hovered = self.hovered_target == Some(DockHoverTarget::Tray);

        // 1. Hime AI button (expressive Hime eyes badge)
        draw_util_bg(surface, ai_rect, ai_hovered, alpha)?;
        surface.draw_hime_eyes(
            ai_rect.x + ai_rect.width * 0.5,
            ai_rect.y + ai_rect.height * 0.5,
            0.72,
            HimeExpression::Netral,
            0.0,
        )?;

        // 2. Settings button (3 crisp horizontal slider tracks + knobs)
        draw_util_bg(surface, settings_rect, settings_hovered, alpha)?;
        let track_x = settings_rect.x + 8.0;
        let track_w = settings_rect.width - 16.0;
        let knob_offsets = [0.68_f32, 0.25_f32, 0.58_f32];
        for (idx, knob_ratio) in knob_offsets.iter().enumerate() {
            let ty = settings_rect.y + 10.0 + idx as f32 * 6.5;
            surface.fill_rounded_rect(
                Rect::new(track_x, ty, track_w, 2.0),
                1.0,
                TEXT_SECONDARY.with_alpha(0.65 * alpha),
            )?;
            let kx = track_x + (track_w - 4.5) * knob_ratio;
            surface.fill_rounded_rect(
                Rect::new(kx, ty - 1.25, 4.5, 4.5),
                2.25,
                if settings_hovered {
                    ACCENT_CYAN.with_alpha(alpha)
                } else {
                    TEXT_PRIMARY.with_alpha(alpha)
                },
            )?;
        }

        // 3. System Tray button
        draw_util_bg(surface, tray_rect, tray_hovered, alpha)?;
        if self.system_tray_count > 0 {
            surface.draw_text(
                &format!("+{}", self.system_tray_count),
                tray_rect,
                11.5,
                TEXT_PRIMARY.with_alpha(alpha),
                TextAlign::Center,
            )?;
        } else {
            // Crisp 2x2 tray dots glyph
            let cx = tray_rect.x + tray_rect.width * 0.5;
            let cy = tray_rect.y + tray_rect.height * 0.5;
            let dot_color = if tray_hovered {
                ACCENT_CYAN.with_alpha(alpha)
            } else {
                TEXT_SECONDARY.with_alpha(alpha)
            };
            for dx in [-4.5_f32, 1.5_f32] {
                for dy in [-4.5_f32, 1.5_f32] {
                    surface.fill_rounded_rect(Rect::new(cx + dx, cy + dy, 3.2, 3.2), 1.6, dot_color)?;
                }
            }
        }

        Ok(())
    }
}

fn draw_util_bg(surface: &NativeSurface, rect: Rect, hovered: bool, alpha: f32) -> Result<()> {
    let bg = if hovered {
        SURFACE_ELEVATED.with_alpha(0.98 * alpha)
    } else {
        SURFACE_ELEVATED.with_alpha(0.65 * alpha)
    };
    surface.fill_rounded_rect(rect, 10.0, bg)?;
    if hovered {
        surface.stroke_rounded_rect(rect, 10.0, ACCENT_CYAN.with_alpha(0.75 * alpha), 1.2)?;
    } else {
        surface.stroke_rounded_rect(rect, 10.0, GLASS_BORDER.with_alpha(0.16 * alpha), 1.0)?;
    }
    Ok(())
}

fn point_in_rect(x: f32, y: f32, r: Rect) -> bool {
    x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height
}

fn set_dock_and_tooltip_region(
    surface: &NativeSurface,
    dock: Rect,
    dock_radius: f32,
    tooltip: Rect,
    tip_radius: f32,
) {
    let scale = (surface.dpi() / 96.0).max(0.5);
    let make_rgn = |r: Rect, rad: f32| unsafe {
        let left = (r.x * scale).round() as i32;
        let top = (r.y * scale).round() as i32;
        let right = ((r.x + r.width) * scale).round() as i32 + 1;
        let bottom = ((r.y + r.height) * scale).round() as i32 + 1;
        let diam = ((rad * 2.0) * scale).round().max(0.0) as i32;
        CreateRoundRectRgn(left, top, right, bottom, diam, diam)
    };

    unsafe {
        let rgn_dock = make_rgn(dock, dock_radius);
        let rgn_tip = make_rgn(tooltip, tip_radius);
        if !rgn_dock.is_invalid() && !rgn_tip.is_invalid() {
            let _ = CombineRgn(Some(rgn_dock), Some(rgn_dock), Some(rgn_tip), RGN_OR);
            let _ = DeleteObject(rgn_tip.into());
            let _ = SetWindowRgn(surface.hwnd(), Some(rgn_dock), true);
        } else {
            if !rgn_dock.is_invalid() {
                let _ = DeleteObject(rgn_dock.into());
            }
            if !rgn_tip.is_invalid() {
                let _ = DeleteObject(rgn_tip.into());
            }
            surface.set_rounded_region(dock, dock_radius);
        }
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
    fn hit_test_matches_rendered_item_rects_and_utility_buttons() {
        let dock = sample_dock();
        let surface_rect = Rect::new(0.0, 0.0, 800.0, 120.0);
        let rects = dock.item_rects(surface_rect);
        assert_eq!(rects.len(), 3);

        for (idx, r) in rects.iter().enumerate() {
            let cx = r.x + r.width * 0.5;
            let cy = r.y + r.height * 0.5;
            assert_eq!(dock.hit_test(cx, cy, surface_rect), Some(idx));
            assert_eq!(
                dock.hit_test_action(cx, cy, surface_rect),
                Some(DockAction::App(idx))
            );
        }
        assert_eq!(
            dock.hit_test_id(rects[1].x + 5.0, rects[1].y + 5.0, surface_rect),
            Some("term")
        );
        assert_eq!(dock.hit_test(5.0, 5.0, surface_rect), None);

        let dock_rect = dock.dock_rect(surface_rect);
        assert_eq!(dock_rect.height, 54.0);
        let (div_x, ai_rect, settings_rect, tray_rect) = dock.utility_rects(dock_rect);
        assert!(div_x.is_some());
        assert_eq!(
            dock.hit_test_action(
                ai_rect.x + ai_rect.width * 0.5,
                ai_rect.y + ai_rect.height * 0.5,
                surface_rect
            ),
            Some(DockAction::ToggleAiDrawer)
        );
        assert_eq!(
            dock.hit_test_action(
                settings_rect.x + settings_rect.width * 0.5,
                settings_rect.y + settings_rect.height * 0.5,
                surface_rect
            ),
            Some(DockAction::ToggleSettings)
        );
        assert_eq!(
            dock.hit_test_action(
                tray_rect.x + tray_rect.width * 0.5,
                tray_rect.y + tray_rect.height * 0.5,
                surface_rect
            ),
            Some(DockAction::ToggleTray)
        );
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
        assert_eq!(
            dock.hit_test(rects[0].x + 10.0, rects[0].y + 10.0, surface_rect),
            None
        );
    }

    #[test]
    fn sync_running_windows_groups_and_caches_icons_once_per_exe_path() {
        let mut dock = DockBar::new();
        let dummy_icon = IconRgba {
            width: 2,
            height: 2,
            bgra_premul: vec![255; 16],
        };
        let exe_a = r"C:\Apps\FakeA.exe";
        dock.cache_icon_for_path(exe_a, Some(dummy_icon.clone()));

        let windows = vec![
            RunningWindow {
                hwnd: 11,
                title: "Window 1".to_string(),
                app_name: "FakeA".to_string(),
                executable: "FakeA.exe".to_string(),
                exe_path: exe_a.to_string(),
            },
            RunningWindow {
                hwnd: 12,
                title: "Window 2".to_string(),
                app_name: "FakeA".to_string(),
                executable: "fakea.exe".to_string(),
                exe_path: exe_a.to_string(),
            },
            RunningWindow {
                hwnd: 13,
                title: "Window 3".to_string(),
                app_name: "FakeA".to_string(),
                executable: "FAKEA.EXE".to_string(),
                exe_path: exe_a.to_string(),
            },
        ];

        dock.sync_running_windows(&windows);
        assert_eq!(dock.items().len(), 1);
        assert_eq!(dock.items()[0].badge_count, 3);
        assert_eq!(dock.items()[0].hwnds, vec![11, 12, 13]);
        assert_eq!(dock.items()[0].icon, Some(dummy_icon));
        assert_eq!(dock.icon_cache_len(), 1);

        // Hover state survives subsequent syncs of the same grouped executable.
        dock.set_hovered(Some(0));
        dock.tick(1.0 / 60.0);
        dock.sync_running_windows(&windows);
        assert_eq!(dock.hovered_index(), Some(0));
        assert_eq!(dock.icon_cache_len(), 1);
    }
}
