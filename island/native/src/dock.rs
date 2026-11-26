//! Native macOS-style floating Dock bar (`DockBar`) rendered via Direct2D Liquid Glass.
//!
//! Supports:
//! - [`DockMode`] (`Fixed`, `AutoHide`, `Smart`): when hidden (`is_hidden`), collapses to a
//!   100% invisible, 0-pixel / `0x0`-region state that wakes smoothly on bottom-edge cursor hover.
//! - Pinned apps (left) + grouped running apps (`HICON` -> premultiplied BGRA via
//!   `draw_bgra_bitmap`), separated by a subtle hairline divider when both are present.
//! - Monochrome active indicator dot underneath running apps + multi-window count badge.
//! - Right-side utility cluster after hairline divider: Hime AI (when enabled), Spaces
//!   (Windows Virtual Desktop Switcher with Liquid Glass flyout), and System Tray (`+N` badge
//!   + expandable tray flyout).
//! - Apple Liquid Glass tooltips and full `hit_test_action` / `hit_test_context_action`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::{Spring, SpringValue};
use crate::d2d::{
    extract_window_or_exe_icon, HimeExpression, IconRgba, LiquidGlassStyle, NativeSurface, Rect,
    SfSymbol, TextAlign,
};
use crate::telemetry::{group_running_windows, RunningWindow};
use crate::theme::{
    CORNER_RADIUS, DIVIDER_HAIRLINE, STATUS_LIVE_RED, SYSTEM_BLUE, TEXT_PRIMARY, TEXT_SECONDARY,
    TEXT_TERTIARY,
};

const DOCK_HEIGHT: f32 = 54.0;
const ICON_BASE_SIZE: f32 = 36.0;
const ICON_CORNER_RADIUS: f32 = 9.0;
const ITEM_GAP: f32 = 6.0;
const DOCK_PAD_X: f32 = 12.0;
const DIVIDER_WIDTH: f32 = 1.0;
const DIVIDER_GAP: f32 = 8.0;
const UTIL_BTN_SIZE: f32 = 34.0;
const TRAY_BTN_BASE_WIDTH: f32 = 34.0;
const TRAY_BTN_BADGE_WIDTH: f32 = 46.0;
const HANDLE_STRIP_WIDTH: f32 = 80.0;
const HANDLE_STRIP_HEIGHT: f32 = 4.0;
const MIN_MAGNIFY_SCALE: f32 = 1.0;
const MAX_MAGNIFY_SCALE: f32 = 1.65;
const MAGNIFY_EFFECT_WIDTH: f32 = 190.0;
const BOUNCE_DURATION_SECS: f32 = 0.38;
const MAX_RUNNING_GROUPS: usize = 12;
const AUTO_HIDE_GRACE_SECS: f32 = 1.0;
const TRAY_ITEM_SIZE: f32 = 24.0;
const TRAY_FLYOUT_PAD: f32 = 6.0;
const TRAY_FLYOUT_GAP: f32 = 6.0;
const SPACES_FLYOUT_PAD: f32 = 5.0;
const SPACES_FLYOUT_GAP: f32 = 5.0;
const SPACES_PILL_HEIGHT: f32 = 26.0;
const SPACES_NEW_BTN_SIZE: f32 = 26.0;

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

/// Virtual Desktop ("Space") entry rendered inside the Dock's Spaces switcher flyout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceEntry {
    pub id: String,
    pub name: String,
    pub index: usize,
    pub active: bool,
}

impl SpaceEntry {
    /// Creates a new `SpaceEntry`.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        index: usize,
        active: bool,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            index,
            active,
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
    /// Toggle the Virtual Desktops ("Spaces") flyout.
    ToggleSpaces,
    /// Switch to the Virtual Desktop at `index`.
    SwitchDesktop(usize),
    /// Create a new Virtual Desktop.
    NewDesktop,
    /// Open or toggle the System Tray flyout / menu.
    ToggleTray,
    /// Restore or activate the System Tray item at `index`.
    ActivateTrayItem(usize),
    /// Wake / reveal an auto-hidden Dock from its bottom-edge wake region.
    WakeHiddenDock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockHoverTarget {
    App(usize),
    AiDrawer,
    Spaces,
    SpaceItem(usize),
    NewSpace,
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
    #[serde(skip, default)]
    pub bounce_timer: f32,
    #[serde(skip, default)]
    pub bounce_amplitude: f32,
}

fn default_item_scale() -> SpringValue {
    SpringValue::interactive(1.0)
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
            hover_scale: SpringValue::interactive(1.0),
            bounce_timer: 0.0,
            bounce_amplitude: 0.0,
        }
    }
}

/// Floating bottom Dock bar rendered in Apple Liquid Glass with invisible AutoHide/Smart
/// minimization, pinned + grouped running apps, real Win32 HICONs, Virtual Desktop Switcher
/// ("Spaces") flyout, System Tray badge + flyout, and Liquid Glass hover tooltips.
#[derive(Debug, Clone)]
pub struct DockBar {
    items: Vec<DockItem>,
    icon_cache: HashMap<String, Option<IconRgba>>,
    tray_items: Vec<TrayEntry>,
    spaces: Vec<SpaceEntry>,
    pub mode: DockMode,
    pub is_hidden: bool,
    pub is_overlapped: bool,
    pub is_edge_hovered: bool,
    pub ai_enabled: bool,
    pub tray_open: bool,
    pub spaces_open: bool,
    pub system_tray_count: u32,
    pub visible: bool,
    pub visibility: SpringValue,
    pub show_progress: SpringValue,
    tooltip_progress: SpringValue,
    spaces_flyout_progress: SpringValue,
    tray_flyout_progress: SpringValue,
    hovered_index: Option<usize>,
    hovered_target: Option<DockHoverTarget>,
    hover_mouse_x: Option<f32>,
    hide_grace_timer: f32,
}

impl Default for DockBar {
    fn default() -> Self {
        Self::new()
    }
}

impl DockBar {
    /// Creates an empty, visible `DockBar` in [`DockMode::Fixed`] with a default `"Desktop 1"` space.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            icon_cache: HashMap::new(),
            tray_items: Vec::new(),
            spaces: vec![SpaceEntry::new("desktop-0", "Desktop 1", 0, true)],
            mode: DockMode::Fixed,
            is_hidden: false,
            is_overlapped: false,
            is_edge_hovered: false,
            ai_enabled: true,
            tray_open: false,
            spaces_open: false,
            system_tray_count: 0,
            visible: true,
            visibility: SpringValue::emerge(1.0),
            show_progress: SpringValue::emerge(1.0),
            tooltip_progress: SpringValue::emerge(0.0),
            spaces_flyout_progress: SpringValue::emerge(0.0),
            tray_flyout_progress: SpringValue::emerge(0.0),
            hovered_index: None,
            hovered_target: None,
            hover_mouse_x: None,
            hide_grace_timer: 0.0,
        }
    }

    pub fn items(&self) -> &[DockItem] {
        &self.items
    }

    pub fn tray_items(&self) -> &[TrayEntry] {
        &self.tray_items
    }

    pub fn spaces(&self) -> &[SpaceEntry] {
        &self.spaces
    }

    pub fn hovered_index(&self) -> Option<usize> {
        self.hovered_index
    }

    /// Returns `true` when any app item, magnification band, utility button, flyout item, or wake zone is hovered.
    pub fn has_hover(&self) -> bool {
        self.hovered_target.is_some() || self.hover_mouse_x.is_some()
    }

    /// Returns the remaining auto-hide grace timer in seconds.
    pub fn hide_grace_timer(&self) -> f32 {
        self.hide_grace_timer
    }

    /// Returns `true` if any Dock spring, magnification spring, click bounce, or auto-hide grace timer is still active.
    pub fn is_animating(&self) -> bool {
        !self.visibility.is_settled()
            || !self.show_progress.is_settled()
            || !self.tooltip_progress.is_settled()
            || !self.spaces_flyout_progress.is_settled()
            || !self.tray_flyout_progress.is_settled()
            || self.hide_grace_timer > 0.0
            || self
                .items
                .iter()
                .any(|it| !it.hover_scale.is_settled() || it.bounce_timer > 0.0)
    }

    /// Triggers a macOS-style vertical click bounce animation on the item at `index`.
    pub fn trigger_bounce(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            let scale = item.hover_scale.value;
            item.bounce_amplitude = if scale > 1.3 {
                ICON_BASE_SIZE * 0.24
            } else {
                ICON_BASE_SIZE * 0.17
            };
            item.bounce_timer = BOUNCE_DURATION_SECS;
        }
    }

    fn should_show_tooltip_for(&self, target: Option<DockHoverTarget>) -> bool {
        match target {
            None | Some(DockHoverTarget::HandleStrip) => false,
            Some(DockHoverTarget::Spaces) => !self.spaces_open,
            Some(DockHoverTarget::Tray) => !self.tray_open,
            Some(_) => true,
        }
    }

    fn refresh_tooltip_target(&mut self) {
        let show_tip = self.should_show_tooltip_for(self.hovered_target);
        let tip_goal = if show_tip { 1.0 } else { 0.0 };
        if (self.tooltip_progress.target - tip_goal).abs() > 0.001 {
            if show_tip {
                self.tooltip_progress.launch_to(1.0, Spring::EMERGE, 1.0);
            } else {
                self.tooltip_progress.launch_to(0.0, Spring::RECEDE, 1.0);
            }
        }
    }

    fn set_hover_target(&mut self, target: Option<DockHoverTarget>) {
        self.hovered_target = target;
        self.refresh_tooltip_target();
    }

    /// Clears hover state across app items, utility buttons, flyout items, and wake region.
    pub fn clear_hover(&mut self) {
        let was_hovered = self.hovered_target.is_some() || self.hover_mouse_x.is_some();
        self.hovered_index = None;
        self.hover_mouse_x = None;
        self.set_hover_target(None);
        self.update_magnification_targets();
        if was_hovered && self.mode != DockMode::Fixed {
            self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
        }
        self.recompute_hidden();
    }

    /// Immediately hides a non-fixed Dock (e.g. when the user clicks outside it).
    pub fn force_hide_now(&mut self) {
        if self.mode == DockMode::Fixed {
            return;
        }
        self.hovered_index = None;
        self.hover_mouse_x = None;
        self.set_hover_target(None);
        self.is_edge_hovered = false;
        self.set_tray_open(false);
        self.set_spaces_open(false);
        self.hide_grace_timer = 0.0;
        for item in &mut self.items {
            item.hover_scale.snap_to(1.0);
            item.bounce_timer = 0.0;
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
            self.set_hover_target(None);
        }
    }

    /// Toggles the inline System Tray flyout strip (mutually exclusive with Spaces) and returns `tray_open`.
    pub fn toggle_tray(&mut self) -> bool {
        self.tray_open = !self.tray_open;
        if self.tray_open {
            if self.spaces_open {
                self.spaces_open = false;
                self.spaces_flyout_progress.launch_to(0.0, Spring::RECEDE, 1.0);
            }
            self.tray_flyout_progress.launch_to(1.0, Spring::EMERGE, 1.0);
            self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
        } else {
            self.tray_flyout_progress.launch_to(0.0, Spring::RECEDE, 1.0);
            if matches!(self.hovered_target, Some(DockHoverTarget::TrayItem(_))) {
                self.set_hover_target(None);
            }
        }
        self.refresh_tooltip_target();
        self.recompute_hidden();
        self.tray_open
    }

    /// Explicitly sets `tray_open`.
    pub fn set_tray_open(&mut self, open: bool) {
        if self.tray_open != open {
            self.toggle_tray();
        }
    }

    /// Toggles the Windows Virtual Desktops ("Spaces") flyout (mutually exclusive with Tray) and returns `spaces_open`.
    pub fn toggle_spaces(&mut self) -> bool {
        self.spaces_open = !self.spaces_open;
        if self.spaces_open {
            if self.tray_open {
                self.tray_open = false;
                self.tray_flyout_progress.launch_to(0.0, Spring::RECEDE, 1.0);
            }
            self.spaces_flyout_progress.launch_to(1.0, Spring::EMERGE, 1.0);
            self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
        } else {
            self.spaces_flyout_progress.launch_to(0.0, Spring::RECEDE, 1.0);
            if matches!(
                self.hovered_target,
                Some(DockHoverTarget::SpaceItem(_) | DockHoverTarget::NewSpace)
            ) {
                self.set_hover_target(None);
            }
        }
        self.refresh_tooltip_target();
        self.recompute_hidden();
        self.spaces_open
    }

    /// Explicitly sets `spaces_open`.
    pub fn set_spaces_open(&mut self, open: bool) {
        if self.spaces_open != open {
            self.toggle_spaces();
        }
    }

    /// Replaces the Virtual Desktops list displayed inside the Spaces flyout.
    pub fn set_spaces(&mut self, spaces: Vec<SpaceEntry>) {
        self.spaces = spaces;
        if let Some(DockHoverTarget::SpaceItem(idx)) = self.hovered_target {
            if idx >= self.spaces.len() {
                self.set_hover_target(None);
            }
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
        if self.tray_items.is_empty() && self.tray_open {
            self.set_tray_open(false);
        }
    }

    /// Recomputes `is_hidden` from `mode`, overlap state, hover state, open flyouts, and grace timer.
    pub fn recompute_hidden(&mut self) {
        let should_hide = match self.mode {
            DockMode::Fixed => false,
            DockMode::AutoHide => {
                !self.has_hover()
                    && !self.is_edge_hovered
                    && !self.tray_open
                    && !self.spaces_open
                    && self.hide_grace_timer <= 0.0
            }
            DockMode::Smart => {
                self.is_overlapped
                    && !self.has_hover()
                    && !self.is_edge_hovered
                    && !self.tray_open
                    && !self.spaces_open
                    && self.hide_grace_timer <= 0.0
            }
        };
        self.is_hidden = should_hide;
        let goal = if should_hide { 0.0 } else { 1.0 };
        if (self.show_progress.target - goal).abs() > 0.001 {
            if should_hide {
                self.show_progress.launch_to(0.0, Spring::RECEDE, 1.0);
            } else {
                self.show_progress.launch_to(1.0, Spring::EMERGE, 1.0);
            }
        }
        if should_hide {
            if self.tray_open {
                self.tray_open = false;
                self.tray_flyout_progress.launch_to(0.0, Spring::RECEDE, 1.0);
            }
            if self.spaces_open {
                self.spaces_open = false;
                self.spaces_flyout_progress.launch_to(0.0, Spring::RECEDE, 1.0);
            }
        }
    }

    /// Resolves a 64px BGRA icon from cache or extracts it once via [`extract_window_or_exe_icon`].
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

        let extracted = extract_window_or_exe_icon(hwnd, trimmed, 64);
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
                self.set_hover_target(None);
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
                    self.set_hover_target(None);
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
                    self.set_hover_target(None);
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

    /// Updates `visible` and animates `visibility` with Apple `EMERGE` / `RECEDE` springs.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        if visible {
            self.visibility.launch_to(1.0, Spring::EMERGE, 1.0);
        } else {
            self.visibility.launch_to(0.0, Spring::RECEDE, 1.0);
            self.set_tray_open(false);
            self.set_spaces_open(false);
            self.clear_hover();
        }
    }

    /// Updates the system tray badge count.
    pub fn set_system_tray_count(&mut self, count: u32) {
        self.system_tray_count = count;
    }

    fn resting_item_center_x(&self, index: usize) -> f32 {
        let has_group_div = self.has_pinned_running_divider();
        let pinned_len = self.pinned_count();
        let group_extra = if has_group_div && index >= pinned_len {
            DIVIDER_GAP + DIVIDER_WIDTH
        } else {
            0.0
        };
        (index as f32) * (ICON_BASE_SIZE + ITEM_GAP) + group_extra + ICON_BASE_SIZE * 0.5
    }

    fn resting_item_span(&self, index: usize) -> (f32, f32) {
        let has_group_div = self.has_pinned_running_divider();
        let pinned_len = self.pinned_count();
        let group_extra = if has_group_div && index >= pinned_len {
            DIVIDER_GAP + DIVIDER_WIDTH
        } else {
            0.0
        };
        let left = (index as f32) * (ICON_BASE_SIZE + ITEM_GAP) + group_extra;
        (left, left + ICON_BASE_SIZE)
    }

    /// Authentic macOS cosine-based magnification targets driven by Apple `Spring` curves.
    fn update_magnification_targets(&mut self) {
        let Some(mouse_pos) = self.hover_mouse_x else {
            let return_spring = Spring::new(0.26, 0.0);
            for item in &mut self.items {
                item.hover_scale.animate_to(MIN_MAGNIFY_SCALE, return_spring);
            }
            return;
        };

        let min_x = mouse_pos - MAGNIFY_EFFECT_WIDTH * 0.5;
        let max_x = mouse_pos + MAGNIFY_EFFECT_WIDTH * 0.5;
        let has_group_div = self.has_pinned_running_divider();
        let pinned_len = self.pinned_count();

        for (i, item) in self.items.iter_mut().enumerate() {
            let group_extra = if has_group_div && i >= pinned_len {
                DIVIDER_GAP + DIVIDER_WIDTH
            } else {
                0.0
            };
            let normal_center =
                (i as f32) * (ICON_BASE_SIZE + ITEM_GAP) + group_extra + ICON_BASE_SIZE * 0.5;
            let target = if normal_center < min_x || normal_center > max_x {
                MIN_MAGNIFY_SCALE
            } else {
                let theta = (((normal_center - min_x) / MAGNIFY_EFFECT_WIDTH)
                    * 2.0
                    * std::f32::consts::PI)
                    .clamp(0.0, 2.0 * std::f32::consts::PI);
                let scale_factor = (1.0 - theta.cos()) * 0.5;
                MIN_MAGNIFY_SCALE + scale_factor * (MAX_MAGNIFY_SCALE - MIN_MAGNIFY_SCALE)
            };
            item.hover_scale.animate_to(target, Spring::INTERACTIVE);
        }
    }

    /// Maps screen cursor `x` inside `surface_rect` back onto resting item X coordinates
    /// so peak magnification tracks accurately under the cursor with zero layout oscillation.
    fn map_cursor_x_to_resting_x(&self, x: f32, surface_rect: Rect) -> f32 {
        let rects = self.item_rects(surface_rect);
        if rects.is_empty() {
            return 0.0;
        }
        let first_r = rects[0];
        let (first_rest_l, _) = self.resting_item_span(0);
        if x <= first_r.x {
            return first_rest_l - (first_r.x - x);
        }
        for i in 0..rects.len() {
            let r = rects[i];
            let (rest_l, rest_r) = self.resting_item_span(i);
            if x <= r.x + r.width {
                let u = ((x - r.x) / r.width.max(1.0)).clamp(0.0, 1.0);
                return rest_l + u * (rest_r - rest_l);
            }
            if i + 1 < rects.len() {
                let next_r = rects[i + 1];
                if x < next_r.x {
                    let (next_rest_l, _) = self.resting_item_span(i + 1);
                    let gap_u = ((x - (r.x + r.width)) / (next_r.x - (r.x + r.width)).max(1.0))
                        .clamp(0.0, 1.0);
                    return rest_r + gap_u * (next_rest_l - rest_r);
                }
            }
        }
        let last_idx = rects.len() - 1;
        let last_r = rects[last_idx];
        let (_, last_rest_r) = self.resting_item_span(last_idx);
        last_rest_r + (x - (last_r.x + last_r.width))
    }

    /// Sets the hovered item index and updates cosine magnification wave targets.
    pub fn set_hovered(&mut self, index: Option<usize>) {
        let valid = index.filter(|&i| i < self.items.len());
        self.hovered_index = valid;
        self.hover_mouse_x = valid.map(|i| self.resting_item_center_x(i));
        self.set_hover_target(valid.map(DockHoverTarget::App));
        if valid.is_some() {
            self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
            self.recompute_hidden();
        }
        self.update_magnification_targets();
    }

    /// Performs hit-testing at `(x, y)` and updates hover state, continuous cosine wave magnification,
    /// utility buttons, Spaces/Tray flyout items, and bottom-edge wake region.
    pub fn set_hovered_at(&mut self, x: f32, y: f32, surface_rect: Rect) -> Option<usize> {
        if self.is_hidden && self.show_progress.value <= 0.2 {
            if point_in_rect(x, y, self.wake_hit_rect(surface_rect)) {
                self.set_edge_hovered(true);
                self.set_hover_target(Some(DockHoverTarget::HandleStrip));
            } else {
                self.set_edge_hovered(false);
                self.clear_hover();
            }
            return None;
        }

        match self.hit_test_action(x, y, surface_rect) {
            Some(DockAction::App(idx)) => {
                let rest_x = self.map_cursor_x_to_resting_x(x, surface_rect);
                self.hovered_index = Some(idx);
                self.hover_mouse_x = Some(rest_x);
                self.set_hover_target(Some(DockHoverTarget::App(idx)));
                self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                self.recompute_hidden();
                self.update_magnification_targets();
                Some(idx)
            }
            Some(DockAction::ToggleAiDrawer) => {
                self.hovered_index = None;
                self.hover_mouse_x = None;
                self.set_hover_target(Some(DockHoverTarget::AiDrawer));
                self.update_magnification_targets();
                self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                self.recompute_hidden();
                None
            }
            Some(DockAction::ToggleSpaces) => {
                self.hovered_index = None;
                self.hover_mouse_x = None;
                self.set_hover_target(Some(DockHoverTarget::Spaces));
                self.update_magnification_targets();
                self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                self.recompute_hidden();
                None
            }
            Some(DockAction::SwitchDesktop(idx)) => {
                self.hovered_index = None;
                self.hover_mouse_x = None;
                self.set_hover_target(Some(DockHoverTarget::SpaceItem(idx)));
                self.update_magnification_targets();
                self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                self.recompute_hidden();
                None
            }
            Some(DockAction::NewDesktop) => {
                self.hovered_index = None;
                self.hover_mouse_x = None;
                self.set_hover_target(Some(DockHoverTarget::NewSpace));
                self.update_magnification_targets();
                self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                self.recompute_hidden();
                None
            }
            Some(DockAction::ToggleTray) => {
                self.hovered_index = None;
                self.hover_mouse_x = None;
                self.set_hover_target(Some(DockHoverTarget::Tray));
                self.update_magnification_targets();
                self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                self.recompute_hidden();
                None
            }
            Some(DockAction::ActivateTrayItem(idx)) => {
                self.hovered_index = None;
                self.hover_mouse_x = None;
                self.set_hover_target(Some(DockHoverTarget::TrayItem(idx)));
                self.update_magnification_targets();
                self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                self.recompute_hidden();
                None
            }
            Some(DockAction::WakeHiddenDock) => {
                self.set_edge_hovered(true);
                self.set_hover_target(Some(DockHoverTarget::HandleStrip));
                None
            }
            Some(DockAction::TogglePin(_)) | None => {
                let dock = self.dock_rect(surface_rect);
                let rects = self.item_rects(surface_rect);
                let in_items_band = !rects.is_empty() && {
                    let left = rects.first().map(|r| r.x - 6.0).unwrap_or(dock.x);
                    let right = rects.last().map(|r| r.x + r.width + 6.0).unwrap_or(dock.x);
                    let top = dock.y - (MAX_MAGNIFY_SCALE - 1.0) * ICON_BASE_SIZE - 8.0;
                    let bottom = dock.y + dock.height;
                    x >= left && x <= right && y >= top && y <= bottom
                };
                if in_items_band {
                    let rest_x = self.map_cursor_x_to_resting_x(x, surface_rect);
                    self.hovered_index = None;
                    self.hover_mouse_x = Some(rest_x);
                    self.set_hover_target(None);
                    self.hide_grace_timer = AUTO_HIDE_GRACE_SECS;
                    self.recompute_hidden();
                    self.update_magnification_targets();
                } else {
                    self.clear_hover();
                }
                None
            }
        }
    }

    /// Advances visibility, auto-hide grace timer, `show_progress`, flyout & tooltip springs,
    /// analytic magnification springs, and click bounce animation by `dt` seconds.
    /// Returns `true` if any animation or grace timer is still active.
    pub fn tick(&mut self, dt: f32) -> bool {
        if dt > 0.0
            && dt.is_finite()
            && self.mode != DockMode::Fixed
            && !self.has_hover()
            && !self.is_edge_hovered
            && !self.tray_open
            && !self.spaces_open
            && self.hide_grace_timer > 0.0
        {
            self.hide_grace_timer = (self.hide_grace_timer - dt).max(0.0);
            if self.hide_grace_timer == 0.0 {
                self.recompute_hidden();
            }
        }

        self.visibility.tick(dt);
        self.show_progress.tick(dt);
        self.tooltip_progress.tick(dt);
        self.spaces_flyout_progress.tick(dt);
        self.tray_flyout_progress.tick(dt);

        let mut animating = !self.visibility.is_settled()
            || !self.show_progress.is_settled()
            || !self.tooltip_progress.is_settled()
            || !self.spaces_flyout_progress.is_settled()
            || !self.tray_flyout_progress.is_settled()
            || self.hide_grace_timer > 0.0;

        for item in &mut self.items {
            item.hover_scale.tick(dt);
            if !item.hover_scale.is_settled() {
                animating = true;
            }

            if item.bounce_timer > 0.0 && dt > 0.0 && dt.is_finite() {
                item.bounce_timer = (item.bounce_timer - dt).max(0.0);
                if item.bounce_timer > 0.0 {
                    animating = true;
                }
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

    fn layout_items_internal(&self, dock_x: f32, dock_y: f32) -> (Vec<Rect>, Option<f32>, f32) {
        if self.items.is_empty() {
            return (Vec::new(), None, 0.0);
        }
        let has_group_div = self.has_pinned_running_divider();
        let pinned_len = self.pinned_count();
        let bottom_y = dock_y + DOCK_HEIGHT - 10.5;
        let mut rects = Vec::with_capacity(self.items.len());
        let mut pinned_div_x = None;
        let mut cursor = 0.0_f32;

        for (i, item) in self.items.iter().enumerate() {
            if has_group_div && i == pinned_len {
                pinned_div_x =
                    Some(dock_x + DOCK_PAD_X + cursor - ITEM_GAP * 0.5 + DIVIDER_GAP * 0.5);
                cursor += DIVIDER_GAP + DIVIDER_WIDTH;
            }
            let scale = item.hover_scale.value.clamp(0.8, MAX_MAGNIFY_SCALE + 0.05);
            let size = ICON_BASE_SIZE * scale;
            let bounce_offset = if item.bounce_timer > 0.0 {
                let p = 1.0 - (item.bounce_timer / BOUNCE_DURATION_SECS).clamp(0.0, 1.0);
                item.bounce_amplitude * (p * std::f32::consts::PI).sin()
            } else {
                0.0
            };
            let x = dock_x + DOCK_PAD_X + cursor;
            let y = bottom_y - size - bounce_offset;
            rects.push(Rect::new(x, y, size, size));
            cursor += size + ITEM_GAP;
        }
        let span_width = (cursor - ITEM_GAP).max(0.0);
        (rects, pinned_div_x, span_width)
    }

    fn items_span_width(&self) -> f32 {
        if self.items.is_empty() {
            return 0.0;
        }
        let has_group_div = self.has_pinned_running_divider();
        let pinned_len = self.pinned_count();
        let mut cursor = 0.0_f32;
        for (i, item) in self.items.iter().enumerate() {
            if has_group_div && i == pinned_len {
                cursor += DIVIDER_GAP + DIVIDER_WIDTH;
            }
            let scale = item.hover_scale.value.clamp(0.8, MAX_MAGNIFY_SCALE + 0.05);
            cursor += ICON_BASE_SIZE * scale + ITEM_GAP;
        }
        (cursor - ITEM_GAP).max(0.0)
    }

    fn utilities_width(&self) -> f32 {
        if self.ai_enabled {
            // Hime AI + gap + Spaces + gap + Tray
            UTIL_BTN_SIZE * 2.0 + self.tray_btn_width() + ITEM_GAP * 2.0
        } else {
            // Spaces + gap + Tray
            UTIL_BTN_SIZE + self.tray_btn_width() + ITEM_GAP
        }
    }

    /// Returns the `80px x 4px` bottom-edge wake reference rectangle used when `is_hidden`.
    pub fn handle_strip_rect(&self, surface_rect: Rect) -> Rect {
        let x = surface_rect.x + (surface_rect.width - HANDLE_STRIP_WIDTH) * 0.5;
        let y = surface_rect.y + surface_rect.height - HANDLE_STRIP_HEIGHT - 1.0;
        Rect::new(x, y, HANDLE_STRIP_WIDTH, HANDLE_STRIP_HEIGHT)
    }

    /// Returns a generous wake hit-test band along the bottom edge when hidden (invisible visually).
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

    /// Returns the bounding rectangles for each application item using dynamic bottom-origin macOS magnification.
    pub fn item_rects(&self, surface_rect: Rect) -> Vec<Rect> {
        let dock = self.dock_rect(surface_rect);
        self.layout_items_internal(dock.x, dock.y).0
    }

    /// Returns `(pinned_divider_x, utils_divider_x, ai_rect, spaces_rect, tray_rect)`.
    fn utility_rects(&self, dock: Rect) -> (Option<f32>, Option<f32>, Option<Rect>, Rect, Rect) {
        let (_, pinned_div_x, items_w) = self.layout_items_internal(dock.x, dock.y);

        let (utils_div_x, utils_start_x) = if self.items.is_empty() {
            (None, dock.x + DOCK_PAD_X)
        } else {
            let div_x = dock.x + DOCK_PAD_X + items_w + DIVIDER_GAP;
            (Some(div_x), div_x + DIVIDER_WIDTH + DIVIDER_GAP)
        };

        let btn_y = dock.y + (dock.height - UTIL_BTN_SIZE) * 0.5;
        let (ai_rect, spaces_x) = if self.ai_enabled {
            let r = Rect::new(utils_start_x, btn_y, UTIL_BTN_SIZE, UTIL_BTN_SIZE);
            (Some(r), r.x + r.width + ITEM_GAP)
        } else {
            (None, utils_start_x)
        };

        let spaces_rect = Rect::new(spaces_x, btn_y, UTIL_BTN_SIZE, UTIL_BTN_SIZE);
        let tray_rect = Rect::new(
            spaces_rect.x + spaces_rect.width + ITEM_GAP,
            btn_y,
            self.tray_btn_width(),
            UTIL_BTN_SIZE,
        );
        (pinned_div_x, utils_div_x, ai_rect, spaces_rect, tray_rect)
    }

    /// Computes the System Tray flyout bar rectangle when open or receding.
    pub fn tray_flyout_rect(&self, surface_rect: Rect, dock: Rect) -> Option<Rect> {
        let tray_t = self.tray_flyout_progress.value.clamp(0.0, 1.0);
        if self.tray_items.is_empty()
            || (self.is_hidden && self.show_progress.value <= 0.08)
            || (!self.tray_open && tray_t <= 0.02)
        {
            return None;
        }
        let (_, _, _, _, tray_rect) = self.utility_rects(dock);
        let count = self.tray_items.len() as f32;
        let width = TRAY_FLYOUT_PAD * 2.0 + count * TRAY_ITEM_SIZE + (count - 1.0) * TRAY_FLYOUT_GAP;
        let height = TRAY_ITEM_SIZE + TRAY_FLYOUT_PAD * 2.0;
        let max_x = (surface_rect.x + surface_rect.width - width - 4.0).max(surface_rect.x + 4.0);
        let center_x = tray_rect.x + tray_rect.width * 0.5;
        let x = (center_x - width * 0.5).clamp(surface_rect.x + 4.0, max_x);
        let slide_y = (1.0 - tray_t) * 8.0;
        let y = (dock.y - height - 6.0 + slide_y).max(surface_rect.y + 2.0);
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

    fn space_pill_widths(&self, max_flyout_width: f32) -> Vec<f32> {
        if self.spaces.is_empty() {
            return Vec::new();
        }
        let raw: Vec<f32> = self
            .spaces
            .iter()
            .map(|s| (s.name.chars().count() as f32 * 6.5 + 24.0).clamp(62.0, 120.0))
            .collect();
        let n = raw.len();
        let fixed_chrome = SPACES_FLYOUT_PAD * 2.0
            + (n.saturating_sub(1) as f32) * SPACES_FLYOUT_GAP
            + SPACES_FLYOUT_GAP
            + DIVIDER_WIDTH
            + SPACES_FLYOUT_GAP
            + SPACES_NEW_BTN_SIZE;
        let avail_pills = (max_flyout_width - fixed_chrome).max(40.0 * n as f32);
        let raw_sum: f32 = raw.iter().sum();
        if raw_sum <= avail_pills || raw_sum <= 0.0 {
            raw
        } else {
            let scale = avail_pills / raw_sum;
            raw.into_iter().map(|w| (w * scale).max(36.0)).collect()
        }
    }

    /// Computes the Windows Virtual Desktops ("Spaces") Liquid Glass flyout bar above the Dock.
    pub fn spaces_flyout_rect(&self, surface_rect: Rect, dock: Rect) -> Option<Rect> {
        let spaces_t = self.spaces_flyout_progress.value.clamp(0.0, 1.0);
        if (self.is_hidden && self.show_progress.value <= 0.08)
            || (!self.spaces_open && spaces_t <= 0.02)
        {
            return None;
        }
        let (_, _, _, spaces_rect, _) = self.utility_rects(dock);
        let max_w = (surface_rect.width - 12.0).max(80.0);
        let pill_widths = self.space_pill_widths(max_w);
        let pills_w: f32 = pill_widths.iter().sum();
        let gaps_w = if pill_widths.is_empty() {
            0.0
        } else {
            (pill_widths.len().saturating_sub(1) as f32) * SPACES_FLYOUT_GAP
                + SPACES_FLYOUT_GAP * 2.0
                + DIVIDER_WIDTH
        };
        let width = (SPACES_FLYOUT_PAD * 2.0 + pills_w + gaps_w + SPACES_NEW_BTN_SIZE).min(max_w);
        let height = SPACES_PILL_HEIGHT + SPACES_FLYOUT_PAD * 2.0;
        let max_x = (surface_rect.x + surface_rect.width - width - 4.0).max(surface_rect.x + 4.0);
        let center_x = spaces_rect.x + spaces_rect.width * 0.5;
        let x = (center_x - width * 0.5).clamp(surface_rect.x + 4.0, max_x);
        let slide_y = (1.0 - spaces_t) * 8.0;
        let y = (dock.y - height - 6.0 + slide_y).max(surface_rect.y + 2.0);
        Some(Rect::new(x, y, width, height))
    }

    /// Computes item pill rectangles inside the open Spaces flyout bar.
    pub fn space_item_rects(&self, surface_rect: Rect, dock: Rect) -> Vec<Rect> {
        let Some(flyout) = self.spaces_flyout_rect(surface_rect, dock) else {
            return Vec::new();
        };
        let pill_widths = self.space_pill_widths((surface_rect.width - 12.0).max(80.0));
        let mut cursor = flyout.x + SPACES_FLYOUT_PAD;
        let y = flyout.y + SPACES_FLYOUT_PAD;
        pill_widths
            .into_iter()
            .map(|w| {
                let r = Rect::new(cursor, y, w, SPACES_PILL_HEIGHT);
                cursor += w + SPACES_FLYOUT_GAP;
                r
            })
            .collect()
    }

    /// Computes the `+` New Desktop pill rectangle inside the open Spaces flyout bar.
    pub fn new_space_rect(&self, surface_rect: Rect, dock: Rect) -> Option<Rect> {
        let flyout = self.spaces_flyout_rect(surface_rect, dock)?;
        let x = flyout.x + flyout.width - SPACES_FLYOUT_PAD - SPACES_NEW_BTN_SIZE;
        let y = flyout.y + SPACES_FLYOUT_PAD;
        Some(Rect::new(x, y, SPACES_NEW_BTN_SIZE, SPACES_PILL_HEIGHT))
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

    /// Hit-tests all Dock targets: wake region (when hidden), Spaces & Tray flyouts,
    /// app items, and right-side utility buttons (`Hime AI`, `Spaces`, `Tray`).
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
        if self.spaces_open {
            if let Some(new_r) = self.new_space_rect(surface_rect, dock) {
                if point_in_rect(x, y, new_r) {
                    return Some(DockAction::NewDesktop);
                }
            }
            for (idx, r) in self
                .space_item_rects(surface_rect, dock)
                .into_iter()
                .enumerate()
            {
                if point_in_rect(x, y, r) {
                    return Some(DockAction::SwitchDesktop(idx));
                }
            }
        }
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

        let (_, _, ai_rect, spaces_rect, tray_rect) = self.utility_rects(dock);
        if let Some(ai) = ai_rect {
            if point_in_rect(x, y, ai) {
                return Some(DockAction::ToggleAiDrawer);
            }
        }
        if point_in_rect(x, y, spaces_rect) {
            Some(DockAction::ToggleSpaces)
        } else if point_in_rect(x, y, tray_rect) {
            Some(DockAction::ToggleTray)
        } else {
            None
        }
    }

    /// Right-click / context-action hit test (e.g. Pin/Unpin on an app icon, or Spaces/Tray/AI).
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
        let target = self.hovered_target?;
        let is_flyout_item = matches!(
            target,
            DockHoverTarget::TrayItem(_)
                | DockHoverTarget::SpaceItem(_)
                | DockHoverTarget::NewSpace
        );
        let (target_rect, label) = match target {
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
            DockHoverTarget::Spaces => {
                if self.spaces_open {
                    return None;
                }
                let (_, _, _, spaces_rect, _) = self.utility_rects(dock);
                (spaces_rect, "Spaces".to_string())
            }
            DockHoverTarget::SpaceItem(idx) => {
                let space = self.spaces.get(idx)?;
                let rects = self.space_item_rects(surface_rect, dock);
                let r = *rects.get(idx)?;
                (r, space.name.clone())
            }
            DockHoverTarget::NewSpace => {
                let r = self.new_space_rect(surface_rect, dock)?;
                (r, "New Desktop".to_string())
            }
            DockHoverTarget::Tray => {
                if self.tray_open {
                    return None;
                }
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
        let anchor_y = if is_flyout_item {
            target_rect.y
        } else {
            target_rect.y.min(dock.y)
        };
        let tip_y = (anchor_y - tip_h - 6.0).max(2.0);
        let cx = target_rect.x + target_rect.width * 0.5;
        let max_x = (surface_rect.width - text_w - 4.0).max(4.0);
        let tip_x = (cx - text_w * 0.5).clamp(4.0, max_x);
        let tip_rect = Rect::new(tip_x, tip_y, text_w, tip_h);

        // Never let a dock-row tooltip overlap an open Spaces or Tray flyout above the Dock.
        if !is_flyout_item {
            if let Some(fly) = self.spaces_flyout_rect(surface_rect, dock) {
                if rects_overlap(tip_rect, fly, 4.0) {
                    return None;
                }
            }
            if let Some(fly) = self.tray_flyout_rect(surface_rect, dock) {
                if rects_overlap(tip_rect, fly, 4.0) {
                    return None;
                }
            }
        }

        Some((tip_rect, label))
    }

    /// Renders either the 100% invisible minimized state (when auto-hidden) or the Apple Liquid Glass Dock capsule.
    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let alpha = self.visibility.value.clamp(0.0, 1.0);
        let show_t = self.show_progress.value.clamp(0.0, 1.0);
        if alpha <= 0.01 || (self.is_hidden && show_t <= 0.08) {
            surface.set_rounded_region(Rect::new(0.0, 0.0, 0.0, 0.0), 0.0);
            return Ok(());
        }

        let (sw, sh) = surface.dip_size();
        let surface_rect = Rect::new(0.0, 0.0, sw, sh);

        let dock = self.dock_rect(surface_rect);
        let rects = self.item_rects(surface_rect);
        let tray_flyout = self.tray_flyout_rect(surface_rect, dock);
        let spaces_flyout = self.spaces_flyout_rect(surface_rect, dock);
        let tooltip = self.active_tooltip(surface_rect, dock);

        let mut regions = Vec::with_capacity(4 + rects.len());
        regions.push((dock, CORNER_RADIUS));
        for r in &rects {
            if r.y < dock.y {
                regions.push((
                    Rect::new(r.x - 2.0, r.y - 2.0, r.width + 4.0, r.height + 4.0),
                    ICON_CORNER_RADIUS,
                ));
            }
        }
        if let Some(fly) = tray_flyout {
            regions.push((fly, 10.0));
        }
        if let Some(fly) = spaces_flyout {
            regions.push((fly, 12.0));
        }
        if let Some((tip_rect, _)) = &tooltip {
            regions.push((*tip_rect, 7.0));
        }
        set_dock_regions(surface, &regions);

        // Apple Liquid Glass Dock capsule (clear smoked glass + diagonal specular rim + soft outer shadow)
        surface.draw_liquid_glass_panel(
            dock,
            LiquidGlassStyle::clear(CORNER_RADIUS).with_shadow(12.0, 3.0, 0.24 * alpha * show_t),
        )?;

        // Application items (crystal-clear BGRA icons + soft clean shadow, zero dark hover borders)
        for (i, (item, rect)) in self.items.iter().zip(rects.iter()).enumerate() {
            let is_hovered = self.hovered_target == Some(DockHoverTarget::App(i));
            let scale = item.hover_scale.value;
            if scale > 1.04 || item.bounce_timer > 0.0 {
                let elev = ((scale - 1.0) / (MAX_MAGNIFY_SCALE - 1.0)).clamp(0.0, 1.0);
                let shadow_a =
                    (0.24 * elev + if item.bounce_timer > 0.0 { 0.10 } else { 0.0 }) * alpha;
                surface.draw_soft_shadow(
                    *rect,
                    ICON_CORNER_RADIUS,
                    4.0 + 6.0 * elev,
                    2.5,
                    shadow_a,
                )?;
            }

            if let Some(ref icon_rgba) = item.icon {
                surface.draw_bgra_bitmap(
                    *rect,
                    icon_rgba.width,
                    icon_rgba.height,
                    &icon_rgba.bgra_premul,
                    ICON_CORNER_RADIUS,
                )?;
            } else {
                surface.draw_liquid_glass_button(
                    *rect,
                    ICON_CORNER_RADIUS,
                    None,
                    is_hovered,
                    false,
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
                    (15.0 * scale.clamp(1.0, 1.35)).round(),
                    TEXT_PRIMARY.with_alpha(alpha),
                    TextAlign::Center,
                )?;
            }

            // macOS-style active indicator dot centered underneath running apps
            if item.running {
                let multi = item.hwnds.len() > 1;
                let dot_w = if multi { 10.0 } else { 4.5 };
                let dot_h = if multi { 2.8 } else { 4.5 };
                let dot_r = dot_h * 0.5;
                let dot_color = if multi {
                    SYSTEM_BLUE.with_alpha(alpha)
                } else {
                    TEXT_PRIMARY.with_alpha(0.92 * alpha)
                };
                let dot = Rect::new(
                    rect.x + rect.width * 0.5 - dot_w * 0.5,
                    dock.y + dock.height - 6.5 - (dot_h - 2.8) * 0.5,
                    dot_w,
                    dot_h,
                );
                surface.fill_rounded_rect(dot, dot_r, dot_color)?;
            }

            // Multi-window / notification count badge
            if item.badge_count > 0 {
                let badge = Rect::new(rect.x + rect.width - 12.0, rect.y - 2.0, 15.0, 15.0);
                surface.fill_rounded_rect(badge, 7.5, STATUS_LIVE_RED.with_alpha(alpha))?;
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

        // Hairline separators + right-side Hime AI / Spaces / System Tray controls
        let (pinned_div_x, utils_div_x, ai_rect, spaces_rect, tray_rect) =
            self.utility_rects(dock);
        if let Some(px) = pinned_div_x {
            let div_rect = Rect::new(px, dock.y + (dock.height - 26.0) * 0.5, DIVIDER_WIDTH, 26.0);
            surface.fill_rounded_rect(div_rect, 0.5, DIVIDER_HAIRLINE.with_alpha_mul(alpha))?;
        }
        if let Some(dx) = utils_div_x {
            let div_rect = Rect::new(dx, dock.y + (dock.height - 26.0) * 0.5, DIVIDER_WIDTH, 26.0);
            surface.fill_rounded_rect(div_rect, 0.5, DIVIDER_HAIRLINE.with_alpha_mul(alpha))?;
        }

        self.render_utility_buttons(surface, ai_rect, spaces_rect, tray_rect, alpha)?;

        // Expanded Virtual Desktops ("Spaces") Liquid Glass flyout
        if let Some(flyout) = spaces_flyout {
            let fly_t = self.spaces_flyout_progress.value.clamp(0.0, 1.0);
            let fly_alpha = alpha * fly_t;
            if fly_alpha > 0.02 {
                surface.draw_liquid_glass_panel(
                    flyout,
                    LiquidGlassStyle::regular(12.0).with_shadow(10.0, 3.0, 0.24 * fly_alpha),
                )?;
                let space_rects = self.space_item_rects(surface_rect, dock);
                for (idx, (entry, r)) in self.spaces.iter().zip(space_rects.iter()).enumerate() {
                    let hovered = self.hovered_target == Some(DockHoverTarget::SpaceItem(idx));
                    let tint = if entry.active {
                        Some(SYSTEM_BLUE.with_alpha_mul(fly_alpha))
                    } else {
                        None
                    };
                    surface.draw_liquid_glass_button(*r, 7.0, tint, hovered, false)?;

                    let dot_r = Rect::new(r.x + 8.0, r.y + (r.height - 5.0) * 0.5, 5.0, 5.0);
                    let dot_color = if entry.active {
                        SYSTEM_BLUE.with_alpha(fly_alpha)
                    } else {
                        TEXT_TERTIARY.with_alpha(0.55 * fly_alpha)
                    };
                    surface.fill_rounded_rect(dot_r, 2.5, dot_color)?;

                    let label_rect =
                        Rect::new(r.x + 15.0, r.y, (r.width - 20.0).max(12.0), r.height);
                    let text_color = if entry.active || hovered {
                        TEXT_PRIMARY.with_alpha(fly_alpha)
                    } else {
                        TEXT_SECONDARY.with_alpha(fly_alpha)
                    };
                    surface.draw_text(&entry.name, label_rect, 11.0, text_color, TextAlign::Leading)?;
                }

                if let Some(new_r) = self.new_space_rect(surface_rect, dock) {
                    let div_x = new_r.x - SPACES_FLYOUT_GAP - DIVIDER_WIDTH;
                    surface.fill_rounded_rect(
                        Rect::new(div_x, flyout.y + 8.0, DIVIDER_WIDTH, flyout.height - 16.0),
                        0.5,
                        DIVIDER_HAIRLINE.with_alpha_mul(fly_alpha),
                    )?;
                    let new_hovered = self.hovered_target == Some(DockHoverTarget::NewSpace);
                    surface.draw_liquid_glass_button(new_r, 7.0, None, new_hovered, false)?;
                    let plus_rect = Rect::new(
                        new_r.x + 5.0,
                        new_r.y + 5.0,
                        (new_r.width - 10.0).max(8.0),
                        (new_r.height - 10.0).max(8.0),
                    );
                    surface.draw_sf_symbol(
                        plus_rect,
                        SfSymbol::Plus,
                        TEXT_PRIMARY.with_alpha(if new_hovered { 1.0 } else { 0.90 } * fly_alpha),
                        1.6,
                    )?;
                }
            }
        }

        // Expanded System Tray Liquid Glass flyout strip (when toggled open)
        if let Some(flyout) = tray_flyout {
            let tray_t = self.tray_flyout_progress.value.clamp(0.0, 1.0);
            let tray_alpha = alpha * tray_t;
            if tray_alpha > 0.02 {
                surface.draw_liquid_glass_panel(
                    flyout,
                    LiquidGlassStyle::regular(10.0).with_shadow(8.0, 2.5, 0.24 * tray_alpha),
                )?;
                let tray_rects = self.tray_item_rects(surface_rect, dock);
                for (idx, (entry, r)) in self.tray_items.iter().zip(tray_rects.iter()).enumerate() {
                    let hovered = self.hovered_target == Some(DockHoverTarget::TrayItem(idx));
                    if hovered {
                        surface.draw_liquid_glass_button(*r, 6.0, None, true, false)?;
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
                        if !hovered {
                            surface.draw_liquid_glass_button(*r, 6.0, None, false, false)?;
                        }
                        surface.draw_text(
                            &initial,
                            *r,
                            11.0,
                            TEXT_PRIMARY.with_alpha(tray_alpha),
                            TextAlign::Center,
                        )?;
                    }
                }
            }
        }

        // Liquid Glass hover tooltip pill above the hovered item
        if let Some((mut tip_rect, label)) = tooltip {
            let tip_t = self.tooltip_progress.value.clamp(0.0, 1.0);
            if tip_t > 0.02 {
                tip_rect.y += (1.0 - tip_t) * 4.0;
                surface.draw_liquid_glass_panel(
                    tip_rect,
                    LiquidGlassStyle::regular(7.0).with_shadow(6.0, 2.0, 0.22 * alpha * tip_t),
                )?;
                surface.draw_text(
                    &label,
                    tip_rect,
                    11.5,
                    TEXT_PRIMARY.with_alpha(alpha * tip_t),
                    TextAlign::Center,
                )?;
            }
        }

        Ok(())
    }

    fn render_utility_buttons(
        &self,
        surface: &NativeSurface,
        ai_rect: Option<Rect>,
        spaces_rect: Rect,
        tray_rect: Rect,
        alpha: f32,
    ) -> Result<()> {
        let ai_hovered = self.hovered_target == Some(DockHoverTarget::AiDrawer);
        let spaces_hovered = self.hovered_target == Some(DockHoverTarget::Spaces);
        let tray_hovered = self.hovered_target == Some(DockHoverTarget::Tray);

        // 1. Hime AI button (expressive Hime eyes badge)
        if let Some(ai_r) = ai_rect {
            surface.draw_liquid_glass_button(ai_r, 10.0, None, ai_hovered, false)?;
            surface.draw_hime_eyes(
                ai_r.x + ai_r.width * 0.5,
                ai_r.y + ai_r.height * 0.5,
                0.72,
                HimeExpression::Netral,
                0.0,
            )?;
        }

        // 2. Spaces (Windows Virtual Desktop Switcher) button (`SfSymbol::Desktops`)
        let spaces_tint = if self.spaces_open {
            Some(SYSTEM_BLUE.with_alpha_mul(alpha))
        } else {
            None
        };
        surface.draw_liquid_glass_button(
            spaces_rect,
            10.0,
            spaces_tint,
            spaces_hovered,
            self.spaces_open,
        )?;
        let spaces_icon = Rect::new(
            spaces_rect.x + 8.0,
            spaces_rect.y + 8.0,
            spaces_rect.width - 16.0,
            spaces_rect.height - 16.0,
        );
        surface.draw_sf_symbol(
            spaces_icon,
            SfSymbol::Desktops,
            TEXT_PRIMARY.with_alpha(if spaces_hovered || self.spaces_open { 1.0 } else { 0.88 } * alpha),
            1.5,
        )?;

        // 3. System Tray button (`SfSymbol::TrayGrid` + optional `+N` count)
        let tray_tint = if self.tray_open {
            Some(SYSTEM_BLUE.with_alpha_mul(alpha))
        } else {
            None
        };
        surface.draw_liquid_glass_button(
            tray_rect,
            10.0,
            tray_tint,
            tray_hovered,
            self.tray_open,
        )?;
        let tray_fg =
            TEXT_PRIMARY.with_alpha(if tray_hovered || self.tray_open { 1.0 } else { 0.88 } * alpha);
        if self.system_tray_count > 0 {
            let icon_r = Rect::new(tray_rect.x + 6.0, tray_rect.y + 10.0, 14.0, 14.0);
            surface.draw_sf_symbol(icon_r, SfSymbol::TrayGrid, tray_fg, 1.4)?;
            let count_r = Rect::new(
                tray_rect.x + 20.0,
                tray_rect.y,
                (tray_rect.width - 23.0).max(16.0),
                tray_rect.height,
            );
            surface.draw_text(
                &format!("+{}", self.system_tray_count),
                count_r,
                10.5,
                tray_fg,
                TextAlign::Center,
            )?;
        } else {
            let icon_r = Rect::new(
                tray_rect.x + 9.0,
                tray_rect.y + 9.0,
                tray_rect.width - 18.0,
                tray_rect.height - 18.0,
            );
            surface.draw_sf_symbol(icon_r, SfSymbol::TrayGrid, tray_fg, 1.5)?;
        }

        Ok(())
    }
}

fn point_in_rect(x: f32, y: f32, r: Rect) -> bool {
    x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height
}

fn rects_overlap(a: Rect, b: Rect, pad: f32) -> bool {
    a.x < b.x + b.width + pad
        && a.x + a.width + pad > b.x
        && a.y < b.y + b.height + pad
        && a.y + a.height + pad > b.y
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
        let (pinned_div_x, utils_div_x, ai_rect, spaces_rect, tray_rect) =
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
                spaces_rect.x + spaces_rect.width * 0.5,
                spaces_rect.y + spaces_rect.height * 0.5,
                surface_rect
            ),
            Some(DockAction::ToggleSpaces)
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
    fn dock_utility_cluster_excludes_settings_and_exposes_spaces_tooltip() {
        let mut dock = sample_dock();
        let surface_rect = Rect::new(0.0, 0.0, 1080.0, 136.0);
        let dock_rect = dock.dock_rect(surface_rect);
        let (_, _, ai_rect, spaces_rect, tray_rect) = dock.utility_rects(dock_rect);
        assert!(ai_rect.is_some());

        // When AI is disabled, only Spaces and Tray remain in the utility cluster.
        dock.set_ai_enabled(false);
        let dock_no_ai = dock.dock_rect(surface_rect);
        let (_, _, no_ai, spaces_no_ai, tray_no_ai) = dock.utility_rects(dock_no_ai);
        assert!(no_ai.is_none());
        assert_eq!(
            dock.hit_test_action(
                spaces_no_ai.x + spaces_no_ai.width * 0.5,
                spaces_no_ai.y + spaces_no_ai.height * 0.5,
                surface_rect
            ),
            Some(DockAction::ToggleSpaces)
        );
        assert_eq!(
            dock.hit_test_action(
                tray_no_ai.x + tray_no_ai.width * 0.5,
                tray_no_ai.y + tray_no_ai.height * 0.5,
                surface_rect
            ),
            Some(DockAction::ToggleTray)
        );

        // Hovering Spaces button sets "Spaces" tooltip.
        dock.set_ai_enabled(true);
        dock.set_hovered_at(
            spaces_rect.x + spaces_rect.width * 0.5,
            spaces_rect.y + spaces_rect.height * 0.5,
            surface_rect,
        );
        let dock_rect = dock.dock_rect(surface_rect);
        let (_, tip) = dock
            .active_tooltip(surface_rect, dock_rect)
            .expect("Spaces button should produce a tooltip");
        assert_eq!(tip, "Spaces");
        assert_ne!(
            dock.hit_test_action(
                tray_rect.x + tray_rect.width * 0.5,
                tray_rect.y + tray_rect.height * 0.5,
                surface_rect
            ),
            Some(DockAction::ToggleSpaces)
        );
    }

    #[test]
    fn spaces_flyout_toggle_switch_new_desktop_hit_testing_and_mutual_exclusivity() {
        let mut dock = sample_dock();
        let surface_rect = Rect::new(0.0, 0.0, 1080.0, 136.0);
        dock.set_spaces(vec![
            SpaceEntry::new("d0", "Desktop 1", 0, false),
            SpaceEntry::new("d1", "Work", 1, true),
            SpaceEntry::new("d2", "Media", 2, false),
        ]);
        dock.set_tray_items(vec![TrayEntry::new("discord", "Discord", "", 0)]);

        // Opening Tray then Spaces closes Tray (mutually exclusive).
        assert!(dock.toggle_tray());
        assert!(dock.tray_open);
        assert!(!dock.spaces_open);

        assert!(dock.toggle_spaces());
        assert!(dock.spaces_open);
        assert!(!dock.tray_open);

        for _ in 0..30 {
            dock.tick(1.0 / 60.0);
        }

        let dock_rect = dock.dock_rect(surface_rect);
        let flyout = dock
            .spaces_flyout_rect(surface_rect, dock_rect)
            .expect("spaces flyout rect should be present when spaces_open");
        assert!(flyout.y + flyout.height < dock_rect.y);

        let item_rects = dock.space_item_rects(surface_rect, dock_rect);
        assert_eq!(item_rects.len(), 3);
        for (idx, r) in item_rects.iter().enumerate() {
            assert_eq!(
                dock.hit_test_action(r.x + r.width * 0.5, r.y + r.height * 0.5, surface_rect),
                Some(DockAction::SwitchDesktop(idx))
            );
        }

        // Hovering a desktop pill displays its desktop label tooltip.
        dock.set_hovered_at(
            item_rects[1].x + item_rects[1].width * 0.5,
            item_rects[1].y + item_rects[1].height * 0.5,
            surface_rect,
        );
        let (_, tip) = dock
            .active_tooltip(surface_rect, dock_rect)
            .expect("hovered desktop pill should show desktop label tooltip");
        assert_eq!(tip, "Work");

        // `+` New Desktop pill hit-tests to DockAction::NewDesktop and shows "New Desktop" tooltip.
        let new_r = dock
            .new_space_rect(surface_rect, dock_rect)
            .expect("new desktop pill rect should exist");
        assert_eq!(
            dock.hit_test_action(
                new_r.x + new_r.width * 0.5,
                new_r.y + new_r.height * 0.5,
                surface_rect
            ),
            Some(DockAction::NewDesktop)
        );
        dock.set_hovered_at(
            new_r.x + new_r.width * 0.5,
            new_r.y + new_r.height * 0.5,
            surface_rect,
        );
        let (_, new_tip) = dock
            .active_tooltip(surface_rect, dock_rect)
            .expect("hovered + button should show New Desktop tooltip");
        assert_eq!(new_tip, "New Desktop");

        // Opening Tray closes Spaces.
        assert!(dock.toggle_tray());
        assert!(dock.tray_open);
        assert!(!dock.spaces_open);
    }

    #[test]
    fn flyout_open_suppresses_overlapping_button_and_dock_row_tooltips() {
        let mut dock = sample_dock();
        let surface_rect = Rect::new(0.0, 0.0, 1080.0, 136.0);
        dock.set_spaces(vec![
            SpaceEntry::new("d0", "Desktop 1", 0, true),
            SpaceEntry::new("d1", "Desktop 2", 1, false),
        ]);
        dock.set_tray_items(vec![TrayEntry::new("discord", "Discord", "", 0)]);

        let dock_rect = dock.dock_rect(surface_rect);
        let (_, _, _, spaces_rect, tray_rect) = dock.utility_rects(dock_rect);

        // Hovering Spaces when closed shows "Spaces" tooltip.
        dock.set_hovered_at(
            spaces_rect.x + spaces_rect.width * 0.5,
            spaces_rect.y + spaces_rect.height * 0.5,
            surface_rect,
        );
        assert_eq!(
            dock.active_tooltip(surface_rect, dock_rect).map(|(_, t)| t),
            Some("Spaces".to_string())
        );

        // Opening Spaces while cursor stays on the Spaces button suppresses "Spaces" tooltip.
        assert!(dock.toggle_spaces());
        for _ in 0..30 {
            dock.tick(1.0 / 60.0);
        }
        assert_eq!(dock.active_tooltip(surface_rect, dock_rect), None);

        // Hovering a desktop pill inside the Spaces flyout shows its tooltip cleanly ABOVE the flyout.
        let flyout = dock
            .spaces_flyout_rect(surface_rect, dock_rect)
            .expect("spaces flyout rect");
        let item_rects = dock.space_item_rects(surface_rect, dock_rect);
        dock.set_hovered_at(
            item_rects[0].x + item_rects[0].width * 0.5,
            item_rects[0].y + item_rects[0].height * 0.5,
            surface_rect,
        );
        let (tip_rect, tip_label) = dock
            .active_tooltip(surface_rect, dock_rect)
            .expect("space pill tooltip");
        assert_eq!(tip_label, "Desktop 1");
        assert!(tip_rect.y + tip_rect.height <= flyout.y);

        // Same suppression for Tray button when Tray flyout is open.
        assert!(dock.toggle_tray());
        for _ in 0..30 {
            dock.tick(1.0 / 60.0);
        }
        dock.set_hovered_at(
            tray_rect.x + tray_rect.width * 0.5,
            tray_rect.y + tray_rect.height * 0.5,
            surface_rect,
        );
        assert_eq!(dock.active_tooltip(surface_rect, dock_rect), None);
    }

    #[test]
    fn dock_modes_auto_hide_and_smart_collapse_to_invisible_wake_band() {
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

        // Unhovering starts the grace timer; after grace expires in AutoHide, it hides.
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
    fn dock_renders_liquid_glass_spaces_tray_and_invisible_auto_hide_without_error() {
        let surface = match NativeSurface::new("HimeDockTest", -2000, -2000, 1080, 136) {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut dock = sample_dock();
        dock.set_spaces(vec![
            SpaceEntry::new("d0", "Desktop 1", 0, true),
            SpaceEntry::new("d1", "Desktop 2", 1, false),
        ]);
        dock.set_tray_items(vec![TrayEntry::new("discord", "Discord", "", 0)]);
        dock.toggle_spaces();
        dock.set_hovered(Some(1));
        for _ in 0..60 {
            dock.tick(1.0 / 60.0);
        }

        surface.begin_draw();
        dock.render(&surface)
            .expect("DockBar Liquid Glass + Spaces flyout + tooltip render");
        surface
            .end_draw_and_present()
            .expect("DockBar Spaces present");

        dock.toggle_tray();
        for _ in 0..60 {
            dock.tick(1.0 / 60.0);
        }
        surface.begin_draw();
        dock.render(&surface)
            .expect("DockBar Liquid Glass + Tray flyout render");
        surface.end_draw_and_present().expect("DockBar Tray present");

        dock.clear_hover();
        dock.set_tray_open(false);
        dock.set_spaces_open(false);
        dock.set_mode(DockMode::AutoHide);
        for _ in 0..240 {
            dock.tick(1.0 / 60.0);
        }
        assert!(dock.is_hidden);
        assert_eq!(dock.show_progress.value, 0.0);

        surface.begin_draw();
        dock.render(&surface)
            .expect("DockBar invisible AutoHide render");
        surface
            .end_draw_and_present()
            .expect("DockBar invisible AutoHide present");
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
        assert!(
            dock.items()[1].hover_scale.value > 1.60,
            "hovered item should reach cosine peak ~1.65, got {}",
            dock.items()[1].hover_scale.value
        );
        assert!(
            dock.items()[0].hover_scale.value > 1.25
                && dock.items()[0].hover_scale.value < dock.items()[1].hover_scale.value,
            "left neighbor should follow cosine bell curve, got {}",
            dock.items()[0].hover_scale.value
        );
        assert!(
            dock.items()[2].hover_scale.value > 1.15
                && dock.items()[2].hover_scale.value < dock.items()[1].hover_scale.value,
            "right neighbor should follow cosine bell curve, got {}",
            dock.items()[2].hover_scale.value
        );

        dock.set_hovered(None);
        for _ in 0..90 {
            dock.tick(1.0 / 60.0);
        }
        assert!((dock.items()[1].hover_scale.value - 1.0).abs() < 0.01);
        assert!((dock.items()[0].hover_scale.value - 1.0).abs() < 0.01);
        assert!((dock.items()[2].hover_scale.value - 1.0).abs() < 0.01);
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

        // Tick ~0.67s: show_progress settles to 0.0 via Spring::RECEDE, dock_rect slides off-bottom
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

        // 2. Wake on bottom-edge hover
        dock.set_hovered_at(strip.x + 40.0, strip.y + 2.0, surface_rect);
        assert!(!dock.is_hidden);
        assert!(dock.is_edge_hovered);
        for _ in 0..60 {
            dock.tick(1.0 / 60.0);
        }
        assert!((dock.show_progress.value - 1.0).abs() < 0.05);
        let visible_rect = dock.dock_rect(surface_rect);
        assert!((visible_rect.y - 62.0).abs() < 2.0);

        // 3. Hover-leave starts grace timer, then collapses back to invisible
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

    #[test]
    fn force_hide_now_collapses_auto_hide_dock_and_flyouts_immediately_without_grace_delay() {
        let mut dock = sample_dock();
        dock.set_mode(DockMode::AutoHide);
        dock.set_edge_hovered(true);
        dock.set_spaces_open(true);
        assert!(!dock.is_hidden);
        assert!(dock.spaces_open);
        assert!(dock.hide_grace_timer() > 0.0);

        dock.force_hide_now();
        assert!(dock.is_hidden);
        assert!(!dock.tray_open);
        assert!(!dock.spaces_open);
        assert_eq!(dock.hide_grace_timer(), 0.0);
    }

    #[test]
    fn macos_magnification_expands_dock_width_and_anchors_bottom_origin() {
        let mut dock = sample_dock();
        let surface_rect = Rect::new(0.0, 0.0, 1080.0, 136.0);
        let rest_dock = dock.dock_rect(surface_rect);
        let rest_rects = dock.item_rects(surface_rect);
        let rest_bottom = rest_rects[1].y + rest_rects[1].height;

        dock.set_hovered(Some(1));
        for _ in 0..60 {
            dock.tick(1.0 / 60.0);
        }

        let mag_dock = dock.dock_rect(surface_rect);
        let mag_rects = dock.item_rects(surface_rect);
        let mag_bottom = mag_rects[1].y + mag_rects[1].height;

        // Container expands dynamically to accommodate magnified icons
        assert!(
            mag_dock.width > rest_dock.width + 20.0,
            "dock width should expand on magnification: rest={} mag={}",
            rest_dock.width,
            mag_dock.width
        );
        // Bottom edge stays anchored while icon grows upward above dock.y
        assert!((mag_bottom - rest_bottom).abs() < 0.05);
        assert!(mag_rects[1].y < mag_dock.y);
        // Adjacent icons never overlap
        assert!(mag_rects[0].x + mag_rects[0].width < mag_rects[1].x);
        assert!(mag_rects[1].x + mag_rects[1].width < mag_rects[2].x);
    }

    #[test]
    fn click_bounce_lifts_icon_upward_and_settles_back_to_baseline() {
        let mut dock = sample_dock();
        let surface_rect = Rect::new(0.0, 0.0, 1080.0, 136.0);
        let base_y = dock.item_rects(surface_rect)[0].y;

        dock.trigger_bounce(0);
        assert!(dock.is_animating());

        // Advance to half of bounce duration (~peak lift)
        for _ in 0..11 {
            dock.tick(1.0 / 60.0);
        }
        let peak_y = dock.item_rects(surface_rect)[0].y;
        assert!(
            peak_y < base_y - 3.0,
            "icon should bounce upward above baseline: base={} peak={}",
            base_y,
            peak_y
        );

        // Advance past bounce end -> settles cleanly back to baseline
        for _ in 0..30 {
            dock.tick(1.0 / 60.0);
        }
        let end_y = dock.item_rects(surface_rect)[0].y;
        assert!((end_y - base_y).abs() < 0.01);
        assert!(!dock.is_animating());
    }
}
