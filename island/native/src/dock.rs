//! Native macOS-style floating Dock bar (`DockBar`) rendered via Direct2D.
//!
//! Supports:
//! - [`DockMode`] (`Fixed`, `AutoHide`, `Smart`): when hidden (`is_hidden`), collapses down
//!   into a sleek `80px x 4px` bottom-edge handle strip that wakes on cursor hover.
//! - Pinned apps (left) + grouped running apps (`HICON` -> premultiplied BGRA via
//!   `draw_bgra_bitmap`), separated by a thin vertical divider when both are present.
//! - White/cyan active indicator dot underneath running apps + multi-window count badge.
//! - Right-side utilities after vertical divider: Hime AI, Settings, and System Tray
//!   (`+N` badge + expandable tray flyout items).
//! - Pure white (`#FFFFFF`) hover tooltips and full `hit_test_action` / `hit_test_context_action`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::SpringValue;
use crate::d2d::{
    extract_window_or_exe_icon, HimeExpression, IconRgba, NativeSurface, Rect, TextAlign,
};
use crate::telemetry::{group_running_windows, RunningWindow};
use crate::theme::{
    Color, ACCENT_CYAN, CORNER_RADIUS, DOCK_SURFACE, GLASS_BORDER, NOTCH_BG_SOLID, STATUS_ALERT,
    SURFACE_ELEVATED, TEXT_PRIMARY, TEXT_SECONDARY,
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
const HANDLE_STRIP_WIDTH: f32 = 80.0;
const HANDLE_STRIP_HEIGHT: f32 = 4.0;
const HOVER_SCALE_TARGET: f32 = 1.22;
const MAX_RUNNING_GROUPS: usize = 12;
const AUTO_HIDE_GRACE_SECS: f32 = 1.0;
const TRAY_ITEM_SIZE: f32 = 24.0;
const TRAY_FLYOUT_PAD: f32 = 6.0;
const TRAY_FLYOUT_GAP: f32 = 6.0;

/// Dock visibility behavior mode matching `Dock.tsx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DockMode {
    #[default]
    Fixed,
    AutoHide,
    Smart,
}

/// System tray entry rendered inside the Dock's tray flyout strip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrayEntry {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub exe_path: String,
    #[serde(default)]
    pub hwnd: isize,
    #[serde(skip, default)]
    pub icon: Option<IconRgba>,
}

impl TrayEntry {
    /// Creates a new `TrayEntry`.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        exe_path: impl Into<String>,
        hwnd: isize,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            exe_path: exe_path.into(),
            hwnd,
            icon: None,
        }
    }
}

/// Clickable / context action emitted by [`DockBar::hit_test_action`] and [`DockBar::hit_test_context_action`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockAction {
    /// Focus or launch the application item at `index`.
    App(usize),
    /// Pin or unpin the application item at `index`.
    TogglePin(usize),
    /// Toggle the Hime AI Drawer.
    ToggleAiDrawer,
    /// Toggle the Hime Settings panel.
    ToggleSettings,
    /// Open or toggle the System Tray flyout / menu.
    ToggleTray,
    /// Restore or activate the System Tray item at `index`.
    ActivateTrayItem(usize),
    /// Wake / reveal an auto-hidden Dock from its bottom `80x4` handle strip.
    WakeHiddenDock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockHoverTarget {
    App(usize),
    AiDrawer,
    Settings,
    Tray,
    TrayItem(usize),
    HandleStrip,
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

/// Floating bottom Dock bar with auto-hide handle strip, pinned + grouped running apps,
/// real Win32 HICONs, System Tray badge + flyout items, and white hover tooltips.
#[derive(Debug, Clone)]
pub struct DockBar {
    items: Vec<DockItem>,
    icon_cache: HashMap<String, Option<IconRgba>>,
    tray_items: Vec<TrayEntry>,
    pub mode: DockMode,
    pub is_hidden: bool,
    pub is_overlapped: bool,
    pub is_edge_hovered: bool,
    pub ai_enabled: bool,
    pub tray_open: bool,
    pub system_tray_count: u32,
    pub visible: bool,
    pub visibility: SpringValue,
    pub show_progress: SpringValue,
    hovered_index: Option<usize>,
    hovered_target: Option<DockHoverTarget>,
    hide_grace_timer: f32,
}

impl Default for DockBar {
    fn default() -> Self {
        Self::new()
    }
}

impl DockBar {
    /// Creates an empty, visible `DockBar` in [`DockMode::Fixed`].
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            icon_cache: HashMap::new(),
            tray_items: Vec::new(),
            mode: DockMode::Fixed,
            is_hidden: false,
            is_overlapped: false,
            is_edge_hovered: false,
            ai_enabled: true,
            tray_open: false,
            system_tray_count: 0,
            visible: true,
            visibility: SpringValue::gentle(1.0),
            show_progress: SpringValue::snappy(1.0),
            hovered_index: None,
            hovered_target: None,
            hide_grace_timer: 0.0,
        }
    }

    pub fn items(&self) -> &[DockItem] {
        &self.items
    }

    pub fn tray_items(&self) -> &[TrayEntry] {
        &self.tray_items
    }

    pub fn hovered_index(&self) -> Option<usize> {
        self.hovered_index
    }

    /// Returns `true` when any app item, utility button, tray item, or handle strip is hovered.
    pub fn has_hover(&self) -> bool {
        self.hovered_target.is_some()
    }

    /// Returns the remaining auto-hide grace timer in seconds.
    pub fn hide_grace_timer(&self) -> f32 {
        self.hide_grace_timer
    }

    /// Returns `true` if any Dock spring or auto-hide grace timer is still active.
    pub fn is_animating(&self) -> bool {
        !self.visibility.is_settled()
            || !self.show_progress.is_settled()
            || self.hide_grace_timer > 0.0
            || self.items.iter().any(|it| !it.hover_scale.is_settled())
    }

    /// Clears hover state across app items, utility buttons, and tray items.
    pub fn clear_hover(&mut self) {
        let was_hovered = self.hovered_target.is_some();
        self.set_hovered(None);
        self.hovered_target = None;
        if was_hovered && self.mode != DockMode::Fixed {
            self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
        }
        self.recompute_hidden();
    }

    /// Returns the number of cached icon entries (keyed by `exe_path` / fallback `hwnd`).
    pub fn icon_cache_len(&self) -> usize {
        self.icon_cache.len()
    }

    /// Sets the dock display mode (`Fixed`, `AutoHide`, `Smart`).
    pub fn set_mode(&mut self, mode: DockMode) {
        self.mode = mode;
        if mode == DockMode::Fixed {
            self.hide_grace_timer = 0.0;
        }
        self.recompute_hidden();
    }

    /// Reports whether foreground windows overlap or maximize over the dock (`DockMode::Smart`).
    pub fn set_overlapped(&mut self, overlapped: bool) {
        self.is_overlapped = overlapped;
        self.recompute_hidden();
    }

    /// Reports whether the cursor is hovering the bottom screen edge wake zone.
    pub fn set_edge_hovered(&mut self, hovered: bool) {
        self.is_edge_hovered = hovered;
        if hovered {
            self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
        }
        self.recompute_hidden();
    }

    /// Shows or hides the Hime AI button in the right-side utility cluster.
    pub fn set_ai_enabled(&mut self, enabled: bool) {
        self.ai_enabled = enabled;
        if !enabled && self.hovered_target == Some(DockHoverTarget::AiDrawer) {
            self.hovered_target = None;
        }
    }

    /// Toggles the inline System Tray flyout strip and returns the new `tray_open` state.
    pub fn toggle_tray(&mut self) -> bool {
        self.tray_open = !self.tray_open;
        if self.tray_open {
            self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
        } else if matches!(self.hovered_target, Some(DockHoverTarget::TrayItem(_))) {
            self.hovered_target = None;
        }
        self.recompute_hidden();
        self.tray_open
    }

    /// Explicitly sets `tray_open`.
    pub fn set_tray_open(&mut self, open: bool) {
        if self.tray_open != open {
            self.toggle_tray();
        }
    }

    /// Replaces the System Tray item list, resolving HICONs once per entry and updating `system_tray_count`.
    pub fn set_tray_items(&mut self, mut items: Vec<TrayEntry>) {
        for entry in &mut items {
            if entry.icon.is_none() && (entry.hwnd != 0 || !entry.exe_path.trim().is_empty()) {
                entry.icon = self.resolve_icon(entry.hwnd, &entry.exe_path);
            }
        }
        self.system_tray_count = items.len() as u32;
        self.tray_items = items;
        if self.tray_items.is_empty() {
            self.tray_open = false;
        }
    }

    /// Recomputes `is_hidden` from `mode`, overlap state, hover state, and grace timer.
    pub fn recompute_hidden(&mut self) {
        let should_hide = match self.mode {
            DockMode::Fixed => false,
            DockMode::AutoHide => {
                !self.has_hover()
                    && !self.is_edge_hovered
                    && !self.tray_open
                    && self.hide_grace_timer <= 0.0
            }
            DockMode::Smart => {
                self.is_overlapped
                    && !self.has_hover()
                    && !self.is_edge_hovered
                    && !self.tray_open
                    && self.hide_grace_timer <= 0.0
            }
        };
        self.is_hidden = should_hide;
        self.show_progress
            .set_target(if should_hide { 0.0 } else { 1.0 });
        if should_hide {
            self.tray_open = false;
        }
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

    /// Adds or updates a dock item by `id`, keeping pinned items ordered before unpinned items.
    pub fn add_item(&mut self, mut item: DockItem) {
        if item.icon.is_none() && (!item.exe_path.is_empty() || !item.hwnds.is_empty()) {
            let primary = item.hwnds.first().copied().unwrap_or(0);
            item.icon = self.resolve_icon(primary, &item.exe_path);
        }
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
        self.partition_pinned_first();
    }

    /// Replaces the pinned apps list while preserving currently running unpinned groups.
    pub fn set_pinned_apps(&mut self, pinned: Vec<DockItem>) {
        self.items.retain(|it| !it.pinned && it.running);
        let mut new_pinned = Vec::with_capacity(pinned.len());
        for mut item in pinned {
            item.pinned = true;
            if item.icon.is_none() {
                let primary = item.hwnds.first().copied().unwrap_or(0);
                item.icon = self.resolve_icon(primary, &item.exe_path);
            }
            new_pinned.push(item);
        }
        new_pinned.append(&mut self.items);
        self.items = new_pinned;
    }

    /// Toggles the `pinned` state of the item at `index`, re-partitioning pinned items left
    /// and removing the item if it was unpinned while not running. Returns the new `pinned` state.
    pub fn toggle_pin_at(&mut self, index: usize) -> Option<bool> {
        let item = self.items.get_mut(index)?;
        let next_pinned = !item.pinned;
        item.pinned = next_pinned;
        if !next_pinned && !item.running {
            self.items.remove(index);
            self.hovered_index = None;
            if matches!(self.hovered_target, Some(DockHoverTarget::App(_))) {
                self.hovered_target = None;
            }
        } else {
            self.partition_pinned_first();
        }
        Some(next_pinned)
    }

    fn partition_pinned_first(&mut self) {
        let mut pinned = Vec::with_capacity(self.items.len());
        let mut unpinned = Vec::with_capacity(self.items.len());
        for item in self.items.drain(..) {
            if item.pinned {
                pinned.push(item);
            } else {
                unpinned.push(item);
            }
        }
        pinned.extend(unpinned);
        self.items = pinned;
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

        self.partition_pinned_first();

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
            self.tray_open = false;
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
        if valid.is_some() {
            self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
            self.recompute_hidden();
        }
        for (i, item) in self.items.iter_mut().enumerate() {
            let target = match valid {
                Some(h) if h == i => HOVER_SCALE_TARGET,
                Some(h) if h.abs_diff(i) == 1 => 1.08,
                _ => 1.0,
            };
            item.hover_scale.set_target(target);
        }
    }

    /// Performs hit-testing at `(x, y)` and updates hover state (items, utility buttons, tray items, and handle strip).
    pub fn set_hovered_at(&mut self, x: f32, y: f32, surface_rect: Rect) -> Option<usize> {
        if self.is_hidden && self.show_progress.value <= 0.2 {
            if point_in_rect(x, y, self.wake_hit_rect(surface_rect)) {
                self.set_edge_hovered(true);
                self.hovered_target = Some(DockHoverTarget::HandleStrip);
            } else {
                self.set_edge_hovered(false);
                self.clear_hover();
            }
            return None;
        }

        match self.hit_test_action(x, y, surface_rect) {
            Some(DockAction::App(idx)) => {
                self.set_hovered(Some(idx));
                Some(idx)
            }
            Some(DockAction::ToggleAiDrawer) => {
                self.set_hovered(None);
                self.hovered_target = Some(DockHoverTarget::AiDrawer);
                self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                self.recompute_hidden();
                None
            }
            Some(DockAction::ToggleSettings) => {
                self.set_hovered(None);
                self.hovered_target = Some(DockHoverTarget::Settings);
                self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                self.recompute_hidden();
                None
            }
            Some(DockAction::ToggleTray) => {
                self.set_hovered(None);
                self.hovered_target = Some(DockHoverTarget::Tray);
                self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                self.recompute_hidden();
                None
            }
            Some(DockAction::ActivateTrayItem(idx)) => {
                self.set_hovered(None);
                self.hovered_target = Some(DockHoverTarget::TrayItem(idx));
                self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                self.recompute_hidden();
                None
            }
            Some(DockAction::WakeHiddenDock) => {
                self.set_edge_hovered(true);
                self.hovered_target = Some(DockHoverTarget::HandleStrip);
                None
            }
            Some(DockAction::TogglePin(_)) | None => {
                self.clear_hover();
                None
            }
        }
    }

    /// Advances visibility, auto-hide grace timer, `show_progress`, and all item hover springs by `dt` seconds.
    /// Returns `true` if any spring or grace timer is still active.
    pub fn tick(&mut self, dt: f32) -> bool {
        if dt > 0.0
            && dt.is_finite()
            && self.mode != DockMode::Fixed
            && !self.has_hover()
            && !self.is_edge_hovered
            && !self.tray_open
            && self.hide_grace_timer > 0.0
        {
            self.hide_grace_timer = (self.hide_grace_timer - dt).max(0.0);
            if self.hide_grace_timer == 0.0 {
                self.recompute_hidden();
            }
        }

        self.visibility.tick(dt);
        self.show_progress.tick(dt);
        let mut animating = !self.visibility.is_settled()
            || !self.show_progress.is_settled()
            || self.hide_grace_timer > 0.0;
        for item in &mut self.items {
            item.hover_scale.tick(dt);
            if !item.hover_scale.is_settled() {
                animating = true;
            }
        }
        animating
    }

    fn has_pinned_running_divider(&self) -> bool {
        let has_pinned = self.items.iter().any(|it| it.pinned);
        let has_unpinned = self.items.iter().any(|it| !it.pinned);
        has_pinned && has_unpinned
    }

    fn pinned_count(&self) -> usize {
        self.items.iter().filter(|it| it.pinned).count()
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
            let base = count * ITEM_SLOT_SIZE + (count - 1.0) * ITEM_GAP;
            if self.has_pinned_running_divider() {
                base + DIVIDER_GAP + DIVIDER_WIDTH
            } else {
                base
            }
        }
    }

    fn utilities_width(&self) -> f32 {
        if self.ai_enabled {
            // Hime AI + gap + Settings + gap + Tray
            UTIL_BTN_SIZE * 2.0 + self.tray_btn_width() + ITEM_GAP * 2.0
        } else {
            // Settings + gap + Tray
            UTIL_BTN_SIZE + self.tray_btn_width() + ITEM_GAP
        }
    }

    /// Returns the `80px x 4px` bottom-edge handle strip rectangle used when `is_hidden`.
    pub fn handle_strip_rect(&self, surface_rect: Rect) -> Rect {
        let x = surface_rect.x + (surface_rect.width - HANDLE_STRIP_WIDTH) * 0.5;
        let y = surface_rect.y + surface_rect.height - HANDLE_STRIP_HEIGHT - 1.0;
        Rect::new(x, y, HANDLE_STRIP_WIDTH, HANDLE_STRIP_HEIGHT)
    }

    /// Returns a generous wake hit-test band around the `80x4` bottom handle strip.
    pub fn wake_hit_rect(&self, surface_rect: Rect) -> Rect {
        let strip = self.handle_strip_rect(surface_rect);
        Rect::new(
            strip.x - 20.0,
            (strip.y - 10.0).max(surface_rect.y),
            strip.width + 40.0,
            16.0,
        )
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
        let vis_offset = (1.0 - self.visibility.value.clamp(0.0, 1.0)) * 28.0;
        let hide_offset = (1.0 - self.show_progress.value.clamp(0.0, 1.0)) * (DOCK_HEIGHT + 16.0);
        let y = surface_rect.y + surface_rect.height - height - 8.0 + vis_offset + hide_offset;
        Rect::new(x, y, width, height)
    }

    /// Returns the bounding rectangles for each application item in order.
    pub fn item_rects(&self, surface_rect: Rect) -> Vec<Rect> {
        let dock = self.dock_rect(surface_rect);
        let mut rects = Vec::with_capacity(self.items.len());
        let base_y = dock.y + 7.0;
        let has_group_div = self.has_pinned_running_divider();
        let pinned_len = self.pinned_count();

        for (i, item) in self.items.iter().enumerate() {
            let scale = item.hover_scale.value.clamp(0.8, 1.4);
            let size = ICON_BASE_SIZE * scale;
            let group_extra = if has_group_div && i >= pinned_len {
                DIVIDER_GAP + DIVIDER_WIDTH
            } else {
                0.0
            };
            let cx = dock.x
                + DOCK_PAD_X
                + (i as f32) * (ITEM_SLOT_SIZE + ITEM_GAP)
                + group_extra
                + ITEM_SLOT_SIZE * 0.5;
            let x = cx - size * 0.5;
            let y = base_y + (ICON_BASE_SIZE - size) * 0.65;
            rects.push(Rect::new(x, y, size, size));
        }
        rects
    }

    /// Returns `(pinned_divider_x, utils_divider_x, ai_rect, settings_rect, tray_rect)`.
    fn utility_rects(&self, dock: Rect) -> (Option<f32>, Option<f32>, Option<Rect>, Rect, Rect) {
        let pinned_div_x = if self.has_pinned_running_divider() {
            let p = self.pinned_count() as f32;
            Some(dock.x + DOCK_PAD_X + p * ITEM_SLOT_SIZE + (p - 1.0) * ITEM_GAP + ITEM_GAP * 0.5 + DIVIDER_GAP * 0.5)
        } else {
            None
        };

        let items_w = self.items_span_width();
        let (utils_div_x, utils_start_x) = if self.items.is_empty() {
            (None, dock.x + DOCK_PAD_X)
        } else {
            let div_x = dock.x + DOCK_PAD_X + items_w + DIVIDER_GAP;
            (Some(div_x), div_x + DIVIDER_WIDTH + DIVIDER_GAP)
        };

        let btn_y = dock.y + (dock.height - UTIL_BTN_SIZE) * 0.5;
        let (ai_rect, settings_x) = if self.ai_enabled {
            let r = Rect::new(utils_start_x, btn_y, UTIL_BTN_SIZE, UTIL_BTN_SIZE);
            (Some(r), r.x + r.width + ITEM_GAP)
        } else {
            (None, utils_start_x)
        };

        let settings_rect = Rect::new(settings_x, btn_y, UTIL_BTN_SIZE, UTIL_BTN_SIZE);
        let tray_rect = Rect::new(
            settings_rect.x + settings_rect.width + ITEM_GAP,
            btn_y,
            self.tray_btn_width(),
            UTIL_BTN_SIZE,
        );
        (pinned_div_x, utils_div_x, ai_rect, settings_rect, tray_rect)
    }

    /// Computes the System Tray flyout bar rectangle when `tray_open && !tray_items.is_empty()`.
    pub fn tray_flyout_rect(&self, surface_rect: Rect, dock: Rect) -> Option<Rect> {
        if !self.tray_open || self.tray_items.is_empty() || self.is_hidden {
            return None;
        }
        let (_, _, _, _, tray_rect) = self.utility_rects(dock);
        let count = self.tray_items.len() as f32;
        let width = TRAY_FLYOUT_PAD * 2.0 + count * TRAY_ITEM_SIZE + (count - 1.0) * TRAY_FLYOUT_GAP;
        let height = TRAY_ITEM_SIZE + TRAY_FLYOUT_PAD * 2.0;
        let max_x = (surface_rect.x + surface_rect.width - width - 4.0).max(surface_rect.x + 4.0);
        let center_x = tray_rect.x + tray_rect.width * 0.5;
        let x = (center_x - width * 0.5).clamp(surface_rect.x + 4.0, max_x);
        let y = (dock.y - height - 6.0).max(surface_rect.y + 2.0);
        Some(Rect::new(x, y, width, height))
    }

    /// Computes item rectangles inside the open System Tray flyout bar.
    pub fn tray_item_rects(&self, surface_rect: Rect, dock: Rect) -> Vec<Rect> {
        let Some(flyout) = self.tray_flyout_rect(surface_rect, dock) else {
            return Vec::new();
        };
        self.tray_items
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let x = flyout.x + TRAY_FLYOUT_PAD + i as f32 * (TRAY_ITEM_SIZE + TRAY_FLYOUT_GAP);
                let y = flyout.y + TRAY_FLYOUT_PAD;
                Rect::new(x, y, TRAY_ITEM_SIZE, TRAY_ITEM_SIZE)
            })
            .collect()
    }

    /// Returns the index of the application item under `(x, y)`, if any.
    pub fn hit_test(&self, x: f32, y: f32, surface_rect: Rect) -> Option<usize> {
        if (!self.visible && self.visibility.value <= 0.01)
            || (self.is_hidden && self.show_progress.value <= 0.2)
        {
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

    /// Hit-tests all Dock targets: wake strip (when hidden), Tray flyout items, app items,
    /// and right-side utility buttons (`Hime AI`, `Settings`, `Tray`).
    pub fn hit_test_action(&self, x: f32, y: f32, surface_rect: Rect) -> Option<DockAction> {
        if !self.visible && self.visibility.value <= 0.01 {
            return None;
        }
        if self.is_hidden && self.show_progress.value <= 0.2 {
            return if point_in_rect(x, y, self.wake_hit_rect(surface_rect)) {
                Some(DockAction::WakeHiddenDock)
            } else {
                None
            };
        }

        let dock = self.dock_rect(surface_rect);
        if self.tray_open {
            for (idx, r) in self.tray_item_rects(surface_rect, dock).into_iter().enumerate() {
                if point_in_rect(x, y, r) {
                    return Some(DockAction::ActivateTrayItem(idx));
                }
            }
        }

        if let Some(idx) = self.hit_test(x, y, surface_rect) {
            return Some(DockAction::App(idx));
        }

        let (_, _, ai_rect, settings_rect, tray_rect) = self.utility_rects(dock);
        if let Some(ai) = ai_rect {
            if point_in_rect(x, y, ai) {
                return Some(DockAction::ToggleAiDrawer);
            }
        }
        if point_in_rect(x, y, settings_rect) {
            Some(DockAction::ToggleSettings)
        } else if point_in_rect(x, y, tray_rect) {
            Some(DockAction::ToggleTray)
        } else {
            None
        }
    }

    /// Right-click / context-action hit test (e.g. Pin/Unpin on an app icon, or Tray/Settings/AI).
    pub fn hit_test_context_action(&self, x: f32, y: f32, surface_rect: Rect) -> Option<DockAction> {
        if (!self.visible && self.visibility.value <= 0.01)
            || (self.is_hidden && self.show_progress.value <= 0.2)
        {
            return None;
        }
        if let Some(idx) = self.hit_test(x, y, surface_rect) {
            return Some(DockAction::TogglePin(idx));
        }
        self.hit_test_action(x, y, surface_rect)
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
                let (_, _, ai_rect, _, _) = self.utility_rects(dock);
                (ai_rect?, "Hime AI".to_string())
            }
            DockHoverTarget::Settings => {
                let (_, _, _, settings_rect, _) = self.utility_rects(dock);
                (settings_rect, "Settings".to_string())
            }
            DockHoverTarget::Tray => {
                let (_, _, _, _, tray_rect) = self.utility_rects(dock);
                let label = if self.system_tray_count > 0 {
                    format!("System Tray ({})", self.system_tray_count)
                } else {
                    "System Tray".to_string()
                };
                (tray_rect, label)
            }
            DockHoverTarget::TrayItem(idx) => {
                let item = self.tray_items.get(idx)?;
                let rects = self.tray_item_rects(surface_rect, dock);
                let r = *rects.get(idx)?;
                (r, item.label.clone())
            }
            DockHoverTarget::HandleStrip => return None,
        };

        let text_w = (label.chars().count() as f32 * 6.8 + 18.0).clamp(54.0, 210.0);
        let tip_h = 22.0;
        let anchor_y = if matches!(self.hovered_target, Some(DockHoverTarget::TrayItem(_))) {
            target_rect.y
        } else {
            dock.y
        };
        let tip_y = (anchor_y - tip_h - 6.0).max(2.0);
        let cx = target_rect.x + target_rect.width * 0.5;
        let max_x = (surface_rect.width - text_w - 4.0).max(4.0);
        let tip_x = (cx - text_w * 0.5).clamp(4.0, max_x);
        Some((Rect::new(tip_x, tip_y, text_w, tip_h), label))
    }

    /// Renders either the `80x4` bottom handle strip (when auto-hidden) or the full Dock capsule.
    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let alpha = self.visibility.value.clamp(0.0, 1.0);
        if alpha <= 0.01 {
            return Ok(());
        }

        let (sw, sh) = surface.dip_size();
        let surface_rect = Rect::new(0.0, 0.0, sw, sh);

        // Hidden mode: sleek 80px x 4px bottom-edge handle strip
        let show_t = self.show_progress.value.clamp(0.0, 1.0);
        if self.is_hidden && show_t <= 0.08 {
            let strip = self.handle_strip_rect(surface_rect);
            surface.set_rounded_region(strip, 2.0);
            surface.fill_rounded_rect(
                strip,
                2.0,
                Color::from_rgba_u8(255, 255, 255, 130).with_alpha(0.55 * alpha),
            )?;
            surface.stroke_rounded_rect(strip, 2.0, GLASS_BORDER.with_alpha(0.45 * alpha), 0.75)?;
            return Ok(());
        }

        let dock = self.dock_rect(surface_rect);
        let tray_flyout = self.tray_flyout_rect(surface_rect, dock);
        let tooltip = self.active_tooltip(surface_rect, dock);

        let mut regions = Vec::with_capacity(3);
        regions.push((dock, CORNER_RADIUS));
        if let Some(fly) = tray_flyout {
            regions.push((fly, 10.0));
        }
        if let Some((tip_rect, _)) = &tooltip {
            regions.push((*tip_rect, 7.0));
        }
        set_dock_regions(surface, &regions);

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

            // Active white/cyan indicator pill centered underneath running apps
            if item.running {
                let multi = item.hwnds.len() > 1;
                let dot_w = if multi { 10.0 } else { 5.0 };
                let dot_color = if multi {
                    ACCENT_CYAN.with_alpha(alpha)
                } else {
                    TEXT_PRIMARY.with_alpha(0.92 * alpha)
                };
                let dot = Rect::new(
                    rect.x + rect.width * 0.5 - dot_w * 0.5,
                    dock.y + dock.height - 6.5,
                    dot_w,
                    2.5,
                );
                surface.fill_rounded_rect(dot, 1.25, dot_color)?;
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

        // Thin vertical separators + right-side Hime AI / Settings / Tray controls
        let (pinned_div_x, utils_div_x, ai_rect, settings_rect, tray_rect) =
            self.utility_rects(dock);
        if let Some(px) = pinned_div_x {
            let div_rect = Rect::new(px, dock.y + (dock.height - 26.0) * 0.5, DIVIDER_WIDTH, 26.0);
            surface.fill_rounded_rect(div_rect, 0.5, GLASS_BORDER.with_alpha(0.24 * alpha))?;
        }
        if let Some(dx) = utils_div_x {
            let div_rect = Rect::new(dx, dock.y + (dock.height - 26.0) * 0.5, DIVIDER_WIDTH, 26.0);
            surface.fill_rounded_rect(div_rect, 0.5, GLASS_BORDER.with_alpha(0.24 * alpha))?;
        }

        self.render_utility_buttons(surface, ai_rect, settings_rect, tray_rect, alpha)?;

        // Expanded System Tray flyout strip (when toggled open)
        if let Some(flyout) = tray_flyout {
            surface.fill_rounded_rect(flyout, 10.0, NOTCH_BG_SOLID.with_alpha(0.95 * alpha))?;
            surface.stroke_rounded_rect(flyout, 10.0, GLASS_BORDER.with_alpha(0.28 * alpha), 1.0)?;
            let tray_rects = self.tray_item_rects(surface_rect, dock);
            for (idx, (entry, r)) in self.tray_items.iter().zip(tray_rects.iter()).enumerate() {
                let hovered = self.hovered_target == Some(DockHoverTarget::TrayItem(idx));
                if hovered {
                    surface.fill_rounded_rect(
                        *r,
                        6.0,
                        SURFACE_ELEVATED.with_alpha(0.95 * alpha),
                    )?;
                }
                if let Some(ref icon_rgba) = entry.icon {
                    surface.draw_bgra_bitmap(
                        *r,
                        icon_rgba.width,
                        icon_rgba.height,
                        &icon_rgba.bgra_premul,
                        5.0,
                    )?;
                } else {
                    let initial = entry
                        .label
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string();
                    surface.fill_rounded_rect(*r, 6.0, SURFACE_ELEVATED.with_alpha(0.8 * alpha))?;
                    surface.draw_text(
                        &initial,
                        *r,
                        11.0,
                        TEXT_PRIMARY.with_alpha(alpha),
                        TextAlign::Center,
                    )?;
                }
            }
        }

        // Hover tooltip pill above the hovered item (pure WHITE text #FFFFFF)
        if let Some((tip_rect, label)) = tooltip {
            surface.fill_rounded_rect(tip_rect, 7.0, NOTCH_BG_SOLID.with_alpha(0.96 * alpha))?;
            surface.stroke_rounded_rect(
                tip_rect,
                7.0,
                GLASS_BORDER.with_alpha(0.28 * alpha),
                1.0,
            )?;
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
        ai_rect: Option<Rect>,
        settings_rect: Rect,
        tray_rect: Rect,
        alpha: f32,
    ) -> Result<()> {
        let ai_hovered = self.hovered_target == Some(DockHoverTarget::AiDrawer);
        let settings_hovered = self.hovered_target == Some(DockHoverTarget::Settings);
        let tray_hovered = self.hovered_target == Some(DockHoverTarget::Tray) || self.tray_open;

        // 1. Hime AI button (expressive Hime eyes badge)
        if let Some(ai_r) = ai_rect {
            draw_util_bg(surface, ai_r, ai_hovered, alpha)?;
            surface.draw_hime_eyes(
                ai_r.x + ai_r.width * 0.5,
                ai_r.y + ai_r.height * 0.5,
                0.72,
                HimeExpression::Netral,
                0.0,
            )?;
        }

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

        // 3. System Tray button (badge + icon preview or crisp 2x2 tray dots)
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
            let cx = tray_rect.x + tray_rect.width * 0.5;
            let cy = tray_rect.y + tray_rect.height * 0.5;
            let dot_color = if tray_hovered {
                ACCENT_CYAN.with_alpha(alpha)
            } else {
                TEXT_PRIMARY.with_alpha(0.82 * alpha)
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

fn set_dock_regions(surface: &NativeSurface, rects: &[(Rect, f32)]) {
    surface.set_rounded_regions(rects);
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
            assert_eq!(
                dock.hit_test_context_action(cx, cy, surface_rect),
                Some(DockAction::TogglePin(idx))
            );
        }
        assert_eq!(
            dock.hit_test_id(rects[1].x + 5.0, rects[1].y + 5.0, surface_rect),
            Some("term")
        );
        assert_eq!(dock.hit_test(5.0, 5.0, surface_rect), None);

        let dock_rect = dock.dock_rect(surface_rect);
        assert_eq!(dock_rect.height, 54.0);
        let (pinned_div_x, utils_div_x, ai_rect, settings_rect, tray_rect) =
            dock.utility_rects(dock_rect);
        assert!(pinned_div_x.is_some());
        assert!(utils_div_x.is_some());
        let ai_rect = ai_rect.expect("ai button enabled by default");
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
    fn dock_modes_auto_hide_and_smart_collapse_to_80x4_handle_strip() {
        let mut dock = sample_dock();
        let surface_rect = Rect::new(0.0, 0.0, 800.0, 120.0);

        // Fixed mode never auto-hides even when overlapped.
        dock.set_mode(DockMode::Fixed);
        dock.set_overlapped(true);
        assert!(!dock.is_hidden);

        // Smart mode hides only when overlapped and unhovered past grace period.
        dock.set_mode(DockMode::Smart);
        assert!(dock.is_hidden);
        for _ in 0..120 {
            dock.tick(1.0 / 60.0);
        }
        assert_eq!(dock.show_progress.value, 0.0);

        let strip = dock.handle_strip_rect(surface_rect);
        assert_eq!(strip.width, 80.0);
        assert_eq!(strip.height, 4.0);
        assert_eq!(
            dock.hit_test_action(strip.x + 40.0, strip.y + 2.0, surface_rect),
            Some(DockAction::WakeHiddenDock)
        );

        // Edge hover wakes hidden Dock smoothly back up.
        dock.set_edge_hovered(true);
        assert!(!dock.is_hidden);
        for _ in 0..120 {
            dock.tick(1.0 / 60.0);
        }
        assert!((dock.show_progress.value - 1.0).abs() < 0.05);

        // Unhovering starts the 2.5s grace timer; after grace expires in AutoHide, it hides.
        dock.set_edge_hovered(false);
        dock.set_mode(DockMode::AutoHide);
        assert!(!dock.is_hidden);
        for _ in 0..240 {
            dock.tick(1.0 / 60.0);
        }
        assert!(dock.is_hidden);
        assert_eq!(dock.show_progress.value, 0.0);
    }

    #[test]
    fn dock_renders_full_capsule_tray_flyout_and_hidden_handle_strip_without_error() {
        let surface = match NativeSurface::new("HimeDockTest", -2000, -2000, 800, 120) {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut dock = sample_dock();
        dock.set_tray_items(vec![TrayEntry::new("discord", "Discord", "", 0)]);
        dock.toggle_tray();
        dock.set_hovered(Some(1));
        for _ in 0..60 {
            dock.tick(1.0 / 60.0);
        }

        surface.begin_draw();
        dock.render(&surface).expect("DockBar expanded + tray + tooltip render");
        surface.end_draw_and_present().expect("DockBar present");

        dock.clear_hover();
        dock.set_tray_open(false);
        dock.set_mode(DockMode::AutoHide);
        for _ in 0..240 {
            dock.tick(1.0 / 60.0);
        }
        assert!(dock.is_hidden);

        surface.begin_draw();
        dock.render(&surface).expect("DockBar 80x4 handle strip render");
        surface.end_draw_and_present().expect("DockBar strip present");
    }

    #[test]
    fn tray_items_and_pin_toggle_work_with_hit_test() {
        let mut dock = sample_dock();
        let surface_rect = Rect::new(0.0, 0.0, 800.0, 120.0);

        dock.set_tray_items(vec![
            TrayEntry::new("discord", "Discord", "", 0),
            TrayEntry::new("steam", "Steam", "", 0),
        ]);
        assert_eq!(dock.system_tray_count, 2);
        assert!(dock.toggle_tray());

        let dock_rect = dock.dock_rect(surface_rect);
        let tray_rects = dock.tray_item_rects(surface_rect, dock_rect);
        assert_eq!(tray_rects.len(), 2);
        assert_eq!(
            dock.hit_test_action(
                tray_rects[1].x + 5.0,
                tray_rects[1].y + 5.0,
                surface_rect
            ),
            Some(DockAction::ActivateTrayItem(1))
        );

        // Toggle pin on unpinned non-running browser -> pinned
        assert_eq!(dock.toggle_pin_at(2), Some(true));
        assert!(dock.items()[2].pinned);
        // Unpin non-running item removes it
        assert_eq!(dock.toggle_pin_at(2), Some(false));
        assert_eq!(dock.items().len(), 2);
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

    #[test]
    fn auto_hide_startup_collapse_wake_on_bottom_hover_and_leave_lifecycle() {
        let mut dock = sample_dock();
        let surface_rect = Rect::new(0.0, 0.0, 960.0, 124.0);

        // 1. Startup with AutoHide and cursor away: immediately targets hidden (0.0)
        dock.set_mode(DockMode::AutoHide);
        assert!(dock.is_hidden);
        assert_eq!(dock.hide_grace_timer(), 0.0);
        assert!(dock.is_animating());

        // Tick ~0.67s: show_progress settles to 0.0, dock_rect slides off-bottom, 80x4 strip active
        for _ in 0..40 {
            dock.tick(1.0 / 60.0);
        }
        assert!(dock.is_hidden);
        assert_eq!(dock.show_progress.value, 0.0);
        let hidden_rect = dock.dock_rect(surface_rect);
        assert!(hidden_rect.y > surface_rect.height);

        let strip = dock.handle_strip_rect(surface_rect);
        assert_eq!(strip.width, 80.0);
        assert_eq!(strip.height, 4.0);
        let wake = dock.wake_hit_rect(surface_rect);
        assert!(point_in_rect(strip.x + 40.0, strip.y + 2.0, wake));

        // 2. Wake on bottom-edge hover (over handle strip)
        dock.set_hovered_at(strip.x + 40.0, strip.y + 2.0, surface_rect);
        assert!(!dock.is_hidden);
        assert!(dock.is_edge_hovered);
        for _ in 0..60 {
            dock.tick(1.0 / 60.0);
        }
        assert!((dock.show_progress.value - 1.0).abs() < 0.05);
        let visible_rect = dock.dock_rect(surface_rect);
        assert!((visible_rect.y - 62.0).abs() < 2.0);

        // 3. Hover-leave starts grace timer, then collapses back to 80x4 strip
        dock.clear_hover();
        dock.set_edge_hovered(false);
        assert!(!dock.is_hidden);
        assert!(dock.hide_grace_timer() >= AUTO_HIDE_GRACE_SECS - 0.05);

        for _ in 0..240 {
            dock.tick(1.0 / 60.0);
        }
        assert!(dock.is_hidden);
        assert_eq!(dock.show_progress.value, 0.0);
        assert_eq!(dock.hide_grace_timer(), 0.0);
    }
}
