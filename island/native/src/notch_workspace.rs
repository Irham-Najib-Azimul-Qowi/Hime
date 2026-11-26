//! Pure-Rust Direct2D workspace panels and HUD banners for `NotchView`.
//!
//! Covers:
//! - Synced LRC lyrics parser & binary-search active line matcher (`parse_lrc_lines`, `active_lyric_index`).
//! - `TasksWorkspaceState`: compact task list, segmented filters, quick-add draft, toggle/delete hit-testing & rendering.
//! - `FocusWorkspaceState`: active routine/focus session controls (`Start`, `Complete`, `Skip`, `Emergency Pause`)
//!   and today's screen-time summary (`study_secs`, `distracting_secs`).
//! - `ProactiveBannerState` & `TranslateBannerState`: dark-glass HUD banner/card inside the Notch
//!   with Accept/Dismiss/Snooze and Close/OpenDrawer actions.

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::d2d::{HimeExpression, NativeSurface, Rect, TextAlign};
use crate::theme::{
    ACCENT_CYAN, GLASS_BORDER, GLASS_SURFACE, STATUS_ALERT, STATUS_SUCCESS, STATUS_WARNING,
    SURFACE_ELEVATED, TEXT_PRIMARY, TEXT_SECONDARY,
};

const CARD_BG: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 14);
const CARD_BG_ACTIVE: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 28);
const CARD_BORDER: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 26);
const TRACK_BG: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 34);
const HIME_CRIMSON: crate::theme::Color = crate::theme::Color::from_rgb_u8(0xE5, 0x09, 0x14);
const TEXT_MUTED: crate::theme::Color = crate::theme::Color::from_rgba_u8(255, 255, 255, 175);

/// Single timestamped LRC lyric line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LrcLine {
    pub ms: u64,
    pub text: String,
}

/// Parses raw LRC lyrics text into chronologically sorted [`LrcLine`] entries plus any
/// global `[offset:±ms]` tag value found in the file.
pub fn parse_lrc_with_offset(input: &str) -> (Vec<LrcLine>, i64) {
    let mut offset_ms: i64 = 0;
    let mut raw_lines: Vec<(u64, String)> = Vec::new();

    for raw_line in input.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(parsed_offset) = parse_lrc_offset_tag(trimmed) {
            offset_ms = parsed_offset;
            continue;
        }

        let mut rest = trimmed;
        let mut timestamps: Vec<u64> = Vec::new();
        while let Some((ts_ms, next_slice)) = take_leading_lrc_timestamp(rest) {
            timestamps.push(ts_ms);
            rest = next_slice;
        }

        if !timestamps.is_empty() {
            let text = rest.trim().to_string();
            for ts in timestamps {
                raw_lines.push((ts, text.clone()));
            }
        }
    }

    let mut lines: Vec<LrcLine> = raw_lines
        .into_iter()
        .map(|(ms, text)| LrcLine {
            ms: (ms as i64).saturating_add(offset_ms).max(0) as u64,
            text,
        })
        .collect();
    lines.sort_by_key(|line| line.ms);
    (lines, offset_ms)
}

/// Parses raw LRC text into a chronologically sorted list of [`LrcLine`]s.
pub fn parse_lrc_lines(input: &str) -> Vec<LrcLine> {
    parse_lrc_with_offset(input).0
}

/// Finds the active lyric line index for `position_ms + offset_ms` in `O(log N)` time.
///
/// Returns `None` if `lines` is empty or `position_ms + offset_ms` precedes the first line.
pub fn active_lyric_index(lines: &[LrcLine], position_ms: u64, offset_ms: i64) -> Option<usize> {
    if lines.is_empty() {
        return None;
    }
    let effective_ms = (position_ms as i64).saturating_add(offset_ms).max(0) as u64;
    if effective_ms < lines[0].ms {
        return None;
    }
    let idx = lines.partition_point(|line| line.ms <= effective_ms);
    idx.checked_sub(1)
}

fn parse_lrc_offset_tag(line: &str) -> Option<i64> {
    if !line.starts_with('[') || !line.ends_with(']') {
        return None;
    }
    let inner = &line[1..line.len() - 1];
    let (key, val) = inner.split_once(':')?;
    if !key.trim().eq_ignore_ascii_case("offset") {
        return None;
    }
    val.trim().parse::<i64>().ok()
}

fn take_leading_lrc_timestamp(s: &str) -> Option<(u64, &str)> {
    let s = s.trim_start();
    if !s.starts_with('[') {
        return None;
    }
    let close = s.find(']')?;
    let tag = &s[1..close];
    let (min_str, sec_part) = tag.split_once(':')?;
    if min_str.is_empty() || min_str.len() > 3 || !min_str.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let minutes: u64 = min_str.parse().ok()?;

    let (sec_str, sub_str) = match sec_part.split_once('.') {
        Some((sec, sub)) => (sec, Some(sub)),
        None => (sec_part, None),
    };
    if sec_str.len() != 2 || !sec_str.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let seconds: u64 = sec_str.parse().ok()?;
    if seconds >= 60 {
        return None;
    }

    let millis: u64 = match sub_str {
        None => 0,
        Some(sub) => {
            if sub.is_empty() || !sub.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            match sub.len() {
                1 => sub.parse::<u64>().ok()? * 100,
                2 => sub.parse::<u64>().ok()? * 10,
                _ => sub[..3].parse::<u64>().ok()?,
            }
        }
    };

    let total_ms = (minutes * 60 + seconds) * 1000 + millis;
    Some((total_ms, &s[close + 1..]))
}

/// Task priority matching Core's `ProductivityTaskRecord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

impl TaskPriority {
    pub fn short_label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Med",
            Self::High => "High",
            Self::Critical => "Crit",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Critical,
            Self::Critical => Self::Low,
        }
    }

    pub fn accent_color(self) -> crate::theme::Color {
        match self {
            Self::Low => TEXT_SECONDARY,
            Self::Medium => ACCENT_CYAN,
            Self::High => STATUS_WARNING,
            Self::Critical => STATUS_ALERT,
        }
    }
}

/// Task status matching Core columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Todo,
    InProgress,
    Review,
    Done,
}

impl TaskStatus {
    pub fn is_done(self) -> bool {
        matches!(self, Self::Done)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Todo => "Todo",
            Self::InProgress => "Active",
            Self::Review => "Review",
            Self::Done => "Done",
        }
    }
}

/// Segmented filter tab for [`TasksWorkspaceState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskTabFilter {
    #[default]
    Today,
    Upcoming,
    Done,
}

impl TaskTabFilter {
    pub fn label(self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::Upcoming => "Upcoming",
            Self::Done => "Done",
        }
    }
}

/// Compact task entry displayed in [`TasksWorkspaceState`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCardItem {
    pub id: String,
    pub title: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub due_label: Option<String>,
}

/// Action returned by [`TasksWorkspaceState::hit_test`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TasksHitAction {
    SelectTab(TaskTabFilter),
    ToggleTask(String),
    DeleteTask(String),
    CyclePriority,
    QuickAdd(String),
}

/// Computed Direct2D rectangles for one visible task row.
#[derive(Debug, Clone)]
pub struct TaskRowLayout {
    pub id: String,
    pub row_rect: Rect,
    pub toggle_rect: Rect,
    pub delete_rect: Rect,
}

/// Shared geometry for [`TasksWorkspaceState`] rendering and click hit-testing.
#[derive(Debug, Clone)]
pub struct TasksLayout {
    pub header_rect: Rect,
    pub progress_rect: Rect,
    pub tabs: [(TaskTabFilter, Rect); 3],
    pub list_rect: Rect,
    pub rows: Vec<TaskRowLayout>,
    pub quick_add_input: Rect,
    pub quick_add_priority_btn: Rect,
    pub quick_add_submit_btn: Rect,
}

/// State for the Notch Tasks workspace panel.
#[derive(Debug, Clone)]
pub struct TasksWorkspaceState {
    pub tasks: Vec<TaskCardItem>,
    pub active_tab: TaskTabFilter,
    pub quick_add_draft: String,
    pub quick_add_priority: TaskPriority,
    pub loading: bool,
    pub error: Option<String>,
    next_local_id: u64,
}

impl Default for TasksWorkspaceState {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            active_tab: TaskTabFilter::Today,
            quick_add_draft: String::new(),
            quick_add_priority: TaskPriority::Medium,
            loading: false,
            error: None,
            next_local_id: 1,
        }
    }
}

impl TasksWorkspaceState {
    pub fn filtered_tasks(&self) -> Vec<&TaskCardItem> {
        self.tasks
            .iter()
            .filter(|task| match self.active_tab {
                TaskTabFilter::Done => task.status.is_done(),
                TaskTabFilter::Upcoming => {
                    !task.status.is_done() && task.due_label.as_deref() == Some("Upcoming")
                }
                TaskTabFilter::Today => {
                    !task.status.is_done() && task.due_label.as_deref() != Some("Upcoming")
                }
            })
            .collect()
    }

    pub fn counts(&self) -> (usize, usize) {
        let done = self.tasks.iter().filter(|t| t.status.is_done()).count();
        let active = self.tasks.len().saturating_sub(done);
        (active, done)
    }

    pub fn toggle_task(&mut self, id: &str) -> bool {
        let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        task.status = if task.status.is_done() {
            TaskStatus::Todo
        } else {
            TaskStatus::Done
        };
        true
    }

    pub fn delete_task(&mut self, id: &str) -> bool {
        let before = self.tasks.len();
        self.tasks.retain(|t| t.id != id);
        self.tasks.len() != before
    }

    pub fn cycle_quick_add_priority(&mut self) {
        self.quick_add_priority = self.quick_add_priority.next();
    }

    pub fn commit_quick_add(&mut self) -> Option<TaskCardItem> {
        let title = self.quick_add_draft.trim().to_string();
        if title.is_empty() {
            return None;
        }
        let item = TaskCardItem {
            id: format!("local-task-{}", self.next_local_id),
            title,
            priority: self.quick_add_priority,
            status: TaskStatus::Todo,
            due_label: match self.active_tab {
                TaskTabFilter::Upcoming => Some("Upcoming".to_string()),
                _ => Some("Today".to_string()),
            },
        };
        self.next_local_id += 1;
        self.tasks.insert(0, item.clone());
        self.quick_add_draft.clear();
        Some(item)
    }

    pub fn layout(&self, body: Rect) -> TasksLayout {
        let header_rect = Rect::new(body.x, body.y, body.width - 78.0, 20.0);
        let progress_rect = Rect::new(body.x + body.width - 74.0, body.y, 74.0, 20.0);

        let tab_y = body.y + 24.0;
        let tab_h = 22.0;
        let tab_gap = 4.0;
        let tab_w = (body.width - tab_gap * 2.0) / 3.0;
        let tabs = [
            (TaskTabFilter::Today, Rect::new(body.x, tab_y, tab_w, tab_h)),
            (
                TaskTabFilter::Upcoming,
                Rect::new(body.x + tab_w + tab_gap, tab_y, tab_w, tab_h),
            ),
            (
                TaskTabFilter::Done,
                Rect::new(body.x + (tab_w + tab_gap) * 2.0, tab_y, tab_w, tab_h),
            ),
        ];

        let quick_h = 28.0;
        let quick_y = body.y + body.height - quick_h;
        let list_y = tab_y + tab_h + 6.0;
        let list_h = (quick_y - 6.0 - list_y).max(40.0);
        let list_rect = Rect::new(body.x, list_y, body.width, list_h);

        let row_h = 36.0;
        let row_gap = 5.0;
        let mut rows = Vec::new();
        for (idx, item) in self.filtered_tasks().into_iter().take(3).enumerate() {
            let ry = list_y + idx as f32 * (row_h + row_gap);
            if ry + row_h > list_y + list_h + 1.0 {
                break;
            }
            let row_rect = Rect::new(body.x, ry, body.width, row_h);
            let toggle_rect = Rect::new(row_rect.x + 8.0, row_rect.y + 8.0, 20.0, 20.0);
            let delete_rect = Rect::new(
                row_rect.x + row_rect.width - 28.0,
                row_rect.y + 7.0,
                22.0,
                22.0,
            );
            rows.push(TaskRowLayout {
                id: item.id.clone(),
                row_rect,
                toggle_rect,
                delete_rect,
            });
        }

        let prio_w = 48.0;
        let add_w = 56.0;
        let input_w = (body.width - prio_w - add_w - 10.0).max(80.0);
        let quick_add_input = Rect::new(body.x, quick_y, input_w, quick_h);
        let quick_add_priority_btn = Rect::new(body.x + input_w + 5.0, quick_y, prio_w, quick_h);
        let quick_add_submit_btn =
            Rect::new(body.x + input_w + prio_w + 10.0, quick_y, add_w, quick_h);

        TasksLayout {
            header_rect,
            progress_rect,
            tabs,
            list_rect,
            rows,
            quick_add_input,
            quick_add_priority_btn,
            quick_add_submit_btn,
        }
    }

    pub fn hit_test(&self, x: f32, y: f32, body: Rect) -> Option<TasksHitAction> {
        let layout = self.layout(body);
        for (tab, rect) in &layout.tabs {
            if point_in_rect(x, y, *rect) {
                return Some(TasksHitAction::SelectTab(*tab));
            }
        }
        for row in &layout.rows {
            if point_in_rect(x, y, row.delete_rect) {
                return Some(TasksHitAction::DeleteTask(row.id.clone()));
            }
            if point_in_rect(x, y, row.toggle_rect) {
                return Some(TasksHitAction::ToggleTask(row.id.clone()));
            }
        }
        if point_in_rect(x, y, layout.quick_add_priority_btn) {
            return Some(TasksHitAction::CyclePriority);
        }
        if point_in_rect(x, y, layout.quick_add_submit_btn) {
            let title = self.quick_add_draft.trim();
            if !title.is_empty() {
                return Some(TasksHitAction::QuickAdd(title.to_string()));
            }
        }
        None
    }

    pub fn render(&self, surface: &NativeSurface, body: Rect) -> Result<()> {
        let layout = self.layout(body);
        let (active_count, done_count) = self.counts();
        let total = active_count + done_count;

        surface.draw_text_bold(
            &format!("Tasks ({})", active_count),
            layout.header_rect,
            12.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.fill_rounded_rect(layout.progress_rect, 9.0, CARD_BG)?;
        surface.stroke_rounded_rect(layout.progress_rect, 9.0, CARD_BORDER, 1.0)?;
        surface.draw_text(
            &format!("{}/{} done", done_count, total),
            layout.progress_rect,
            9.0,
            STATUS_SUCCESS,
            TextAlign::Center,
        )?;

        for (tab, rect) in &layout.tabs {
            let active = *tab == self.active_tab;
            surface.fill_rounded_rect(*rect, 8.0, if active { CARD_BG_ACTIVE } else { CARD_BG })?;
            if active {
                surface.stroke_rounded_rect(*rect, 8.0, ACCENT_CYAN.with_alpha(0.75), 1.0)?;
                surface.draw_text_bold(tab.label(), *rect, 10.0, TEXT_PRIMARY, TextAlign::Center)?;
            } else {
                surface.draw_text(tab.label(), *rect, 10.0, TEXT_MUTED, TextAlign::Center)?;
            }
        }

        if let Some(err) = &self.error {
            surface.fill_rounded_rect(layout.list_rect, 10.0, STATUS_ALERT.with_alpha(0.14))?;
            surface.stroke_rounded_rect(layout.list_rect, 10.0, STATUS_ALERT.with_alpha(0.4), 1.0)?;
            surface.draw_text(err, layout.list_rect, 10.0, STATUS_ALERT, TextAlign::Center)?;
        } else if self.loading && layout.rows.is_empty() {
            surface.fill_rounded_rect(layout.list_rect, 10.0, CARD_BG)?;
            surface.draw_text("Loading tasks...", layout.list_rect, 10.5, TEXT_MUTED, TextAlign::Center)?;
        } else if layout.rows.is_empty() {
            surface.fill_rounded_rect(layout.list_rect, 10.0, CARD_BG)?;
            surface.stroke_rounded_rect(layout.list_rect, 10.0, CARD_BORDER, 1.0)?;
            let empty_msg = match self.active_tab {
                TaskTabFilter::Done => "No completed tasks yet",
                TaskTabFilter::Upcoming => "No upcoming tasks scheduled",
                TaskTabFilter::Today => "All caught up for today",
            };
            surface.draw_text(empty_msg, layout.list_rect, 10.5, TEXT_MUTED, TextAlign::Center)?;
        } else {
            let filtered = self.filtered_tasks();
            for (idx, row) in layout.rows.iter().enumerate() {
                let item = filtered[idx];
                let is_done = item.status.is_done();
                let prio_color = item.priority.accent_color();

                surface.fill_rounded_rect(row.row_rect, 9.0, CARD_BG)?;
                surface.stroke_rounded_rect(row.row_rect, 9.0, CARD_BORDER, 1.0)?;

                // Priority accent strip
                if matches!(item.priority, TaskPriority::High | TaskPriority::Critical) {
                    surface.fill_rounded_rect(
                        Rect::new(row.row_rect.x + 1.0, row.row_rect.y + 6.0, 3.0, row.row_rect.height - 12.0),
                        1.5,
                        prio_color,
                    )?;
                }

                // Circle checkbox
                let cx = row.toggle_rect.x + row.toggle_rect.width * 0.5;
                let cy = row.toggle_rect.y + row.toggle_rect.height * 0.5;
                if is_done {
                    surface.fill_circle(cx, cy, 8.0, STATUS_SUCCESS)?;
                    surface.draw_line(cx - 3.5, cy, cx - 0.8, cy + 3.0, TEXT_PRIMARY, 1.5)?;
                    surface.draw_line(cx - 0.8, cy + 3.0, cx + 4.0, cy - 3.0, TEXT_PRIMARY, 1.5)?;
                } else {
                    surface.stroke_circle(cx, cy, 8.0, TEXT_MUTED, 1.4)?;
                }

                // Title
                let badge_w = 42.0;
                let title_rect = Rect::new(
                    row.toggle_rect.x + row.toggle_rect.width + 8.0,
                    row.row_rect.y + 3.0,
                    (row.delete_rect.x - (row.toggle_rect.x + row.toggle_rect.width + 14.0) - badge_w)
                        .max(60.0),
                    row.row_rect.height - 6.0,
                );
                surface.draw_text(
                    &item.title,
                    title_rect,
                    10.5,
                    if is_done { TEXT_SECONDARY } else { TEXT_PRIMARY },
                    TextAlign::Leading,
                )?;

                // Status / Priority pill
                let chip_rect = Rect::new(
                    row.delete_rect.x - badge_w - 4.0,
                    row.row_rect.y + 9.0,
                    badge_w,
                    18.0,
                );
                let chip_label = if is_done {
                    item.status.label()
                } else {
                    item.priority.short_label()
                };
                surface.fill_rounded_rect(chip_rect, 6.0, prio_color.with_alpha(0.18))?;
                surface.draw_text(chip_label, chip_rect, 8.5, prio_color, TextAlign::Center)?;

                // Delete button
                surface.fill_rounded_rect(row.delete_rect, 6.0, CARD_BG)?;
                let dcx = row.delete_rect.x + row.delete_rect.width * 0.5;
                let dcy = row.delete_rect.y + row.delete_rect.height * 0.5;
                surface.draw_line(dcx - 3.2, dcy - 3.2, dcx + 3.2, dcy + 3.2, TEXT_MUTED, 1.3)?;
                surface.draw_line(dcx + 3.2, dcy - 3.2, dcx - 3.2, dcy + 3.2, TEXT_MUTED, 1.3)?;
            }
        }

        // Quick-add draft bar
        surface.fill_rounded_rect(layout.quick_add_input, 9.0, CARD_BG)?;
        surface.stroke_rounded_rect(layout.quick_add_input, 9.0, CARD_BORDER, 1.0)?;
        let draft_text = if self.quick_add_draft.trim().is_empty() {
            "+ Quick add task..."
        } else {
            self.quick_add_draft.as_str()
        };
        surface.draw_text(
            draft_text,
            Rect::new(
                layout.quick_add_input.x + 10.0,
                layout.quick_add_input.y,
                layout.quick_add_input.width - 16.0,
                layout.quick_add_input.height,
            ),
            10.0,
            if self.quick_add_draft.trim().is_empty() {
                TEXT_SECONDARY
            } else {
                TEXT_PRIMARY
            },
            TextAlign::Leading,
        )?;

        let p_color = self.quick_add_priority.accent_color();
        surface.fill_rounded_rect(layout.quick_add_priority_btn, 9.0, p_color.with_alpha(0.2))?;
        surface.stroke_rounded_rect(layout.quick_add_priority_btn, 9.0, p_color.with_alpha(0.6), 1.0)?;
        surface.draw_text_bold(
            self.quick_add_priority.short_label(),
            layout.quick_add_priority_btn,
            9.5,
            p_color,
            TextAlign::Center,
        )?;

        let can_submit = !self.quick_add_draft.trim().is_empty();
        surface.fill_rounded_rect(
            layout.quick_add_submit_btn,
            9.0,
            if can_submit {
                ACCENT_CYAN.with_alpha(0.28)
            } else {
                CARD_BG
            },
        )?;
        surface.stroke_rounded_rect(
            layout.quick_add_submit_btn,
            9.0,
            if can_submit { ACCENT_CYAN } else { CARD_BORDER },
            1.0,
        )?;
        surface.draw_text_bold(
            "+ Add",
            layout.quick_add_submit_btn,
            9.5,
            if can_submit { TEXT_PRIMARY } else { TEXT_SECONDARY },
            TextAlign::Center,
        )?;

        Ok(())
    }
}

/// Active focus session lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FocusSessionStatus {
    #[default]
    Idle,
    Active,
    Paused,
    Completed,
    Skipped,
}

impl FocusSessionStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Ready",
            Self::Active => "Focusing",
            Self::Paused => "Emergency Paused",
            Self::Completed => "Completed",
            Self::Skipped => "Skipped",
        }
    }
}

/// Action emitted by [`FocusWorkspaceState::hit_test`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusHitAction {
    Start,
    Complete,
    Skip,
    EmergencyPause,
}

/// Shared geometry for [`FocusWorkspaceState`].
#[derive(Debug, Clone)]
pub struct FocusLayout {
    pub header_rect: Rect,
    pub streak_badge: Rect,
    pub session_card: Rect,
    pub progress_bar: Rect,
    pub start_btn: Option<Rect>,
    pub complete_btn: Option<Rect>,
    pub skip_btn: Option<Rect>,
    pub emergency_pause_btn: Rect,
    pub screentime_card: Rect,
    pub screentime_bar: Rect,
}

/// State for the Notch Focus & Screen-Time workspace panel.
#[derive(Debug, Clone)]
pub struct FocusWorkspaceState {
    pub routine_id: String,
    pub routine_title: String,
    pub category: String,
    pub enforcement: String,
    pub status: FocusSessionStatus,
    pub remaining_secs: u32,
    pub total_secs: u32,
    pub streak_days: u32,
    pub emergency_paused: bool,
    pub study_secs: u64,
    pub neutral_secs: u64,
    pub distracting_secs: u64,
    accum_dt: f32,
}

impl Default for FocusWorkspaceState {
    fn default() -> Self {
        Self {
            routine_id: "default-focus".to_string(),
            routine_title: "Deep Work Block".to_string(),
            category: "Study".to_string(),
            enforcement: "tegas".to_string(),
            status: FocusSessionStatus::Idle,
            remaining_secs: 25 * 60,
            total_secs: 25 * 60,
            streak_days: 0,
            emergency_paused: false,
            study_secs: 0,
            neutral_secs: 0,
            distracting_secs: 0,
            accum_dt: 0.0,
        }
    }
}

impl FocusWorkspaceState {
    pub fn start(&mut self) {
        if self.remaining_secs == 0 || matches!(self.status, FocusSessionStatus::Completed | FocusSessionStatus::Skipped) {
            self.remaining_secs = self.total_secs.max(60);
        }
        self.status = FocusSessionStatus::Active;
        self.emergency_paused = false;
        self.accum_dt = 0.0;
    }

    pub fn complete(&mut self) {
        if self.status != FocusSessionStatus::Completed {
            self.streak_days = self.streak_days.saturating_add(1);
        }
        self.status = FocusSessionStatus::Completed;
        self.emergency_paused = false;
        self.remaining_secs = 0;
        self.accum_dt = 0.0;
    }

    pub fn skip(&mut self) {
        self.status = FocusSessionStatus::Skipped;
        self.emergency_paused = false;
        self.accum_dt = 0.0;
    }

    pub fn toggle_emergency_pause(&mut self) {
        self.emergency_paused = !self.emergency_paused;
        if self.emergency_paused {
            if self.status == FocusSessionStatus::Active {
                self.status = FocusSessionStatus::Paused;
            }
        } else if self.status == FocusSessionStatus::Paused {
            self.status = FocusSessionStatus::Active;
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if self.status != FocusSessionStatus::Active || self.emergency_paused || dt <= 0.0 {
            return false;
        }
        self.accum_dt += dt;
        let mut changed = false;
        while self.accum_dt >= 1.0 {
            self.accum_dt -= 1.0;
            if self.remaining_secs > 1 {
                self.remaining_secs -= 1;
                changed = true;
            } else {
                self.remaining_secs = 0;
                self.status = FocusSessionStatus::Completed;
                self.streak_days = self.streak_days.saturating_add(1);
                changed = true;
                break;
            }
        }
        changed
    }

    pub fn formatted_remaining(&self) -> String {
        format!("{:02}:{:02}", self.remaining_secs / 60, self.remaining_secs % 60)
    }

    pub fn progress_ratio(&self) -> f32 {
        if self.total_secs == 0 {
            return 0.0;
        }
        let elapsed = self.total_secs.saturating_sub(self.remaining_secs);
        (elapsed as f32 / self.total_secs as f32).clamp(0.0, 1.0)
    }

    pub fn screentime_total_secs(&self) -> u64 {
        self.study_secs
            .saturating_add(self.neutral_secs)
            .saturating_add(self.distracting_secs)
    }

    pub fn format_duration_short(secs: u64) -> String {
        let mins = secs / 60;
        if mins >= 60 {
            format!("{}h {}m", mins / 60, mins % 60)
        } else {
            format!("{}m", mins)
        }
    }

    pub fn layout(&self, body: Rect) -> FocusLayout {
        let header_rect = Rect::new(body.x, body.y, body.width - 74.0, 22.0);
        let streak_badge = Rect::new(body.x + body.width - 70.0, body.y, 70.0, 22.0);

        let session_card = Rect::new(body.x, body.y + 28.0, body.width, 114.0);
        let progress_bar = Rect::new(
            session_card.x + 12.0,
            session_card.y + 58.0,
            session_card.width - 24.0,
            6.0,
        );

        let btn_y = session_card.y + 74.0;
        let btn_h = 28.0;
        let gap = 6.0;
        let pause_w = 112.0;
        let emergency_pause_btn = Rect::new(
            session_card.x + session_card.width - 12.0 - pause_w,
            btn_y,
            pause_w,
            btn_h,
        );
        let left_area_w = (emergency_pause_btn.x - gap - (session_card.x + 12.0)).max(90.0);

        let (start_btn, complete_btn, skip_btn) = match self.status {
            FocusSessionStatus::Active | FocusSessionStatus::Paused => {
                let comp_w = (left_area_w - gap) * 0.58;
                let skip_w = (left_area_w - gap) * 0.42;
                (
                    None,
                    Some(Rect::new(session_card.x + 12.0, btn_y, comp_w, btn_h)),
                    Some(Rect::new(
                        session_card.x + 12.0 + comp_w + gap,
                        btn_y,
                        skip_w,
                        btn_h,
                    )),
                )
            }
            FocusSessionStatus::Idle
            | FocusSessionStatus::Completed
            | FocusSessionStatus::Skipped => (
                Some(Rect::new(session_card.x + 12.0, btn_y, left_area_w, btn_h)),
                None,
                None,
            ),
        };

        let screentime_y = session_card.y + session_card.height + 8.0;
        let screentime_h = (body.y + body.height - screentime_y).max(60.0);
        let screentime_card = Rect::new(body.x, screentime_y, body.width, screentime_h);
        let screentime_bar = Rect::new(
            screentime_card.x + 12.0,
            screentime_card.y + 30.0,
            screentime_card.width - 24.0,
            8.0,
        );

        FocusLayout {
            header_rect,
            streak_badge,
            session_card,
            progress_bar,
            start_btn,
            complete_btn,
            skip_btn,
            emergency_pause_btn,
            screentime_card,
            screentime_bar,
        }
    }

    pub fn hit_test(&self, x: f32, y: f32, body: Rect) -> Option<FocusHitAction> {
        let layout = self.layout(body);
        if let Some(btn) = layout.start_btn {
            if point_in_rect(x, y, btn) {
                return Some(FocusHitAction::Start);
            }
        }
        if let Some(btn) = layout.complete_btn {
            if point_in_rect(x, y, btn) {
                return Some(FocusHitAction::Complete);
            }
        }
        if let Some(btn) = layout.skip_btn {
            if point_in_rect(x, y, btn) {
                return Some(FocusHitAction::Skip);
            }
        }
        if point_in_rect(x, y, layout.emergency_pause_btn) {
            return Some(FocusHitAction::EmergencyPause);
        }
        None
    }

    pub fn render(&self, surface: &NativeSurface, body: Rect) -> Result<()> {
        let layout = self.layout(body);

        surface.draw_text_bold(
            &format!("Focus • {}", self.routine_title),
            layout.header_rect,
            11.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.fill_rounded_rect(layout.streak_badge, 9.0, STATUS_WARNING.with_alpha(0.18))?;
        surface.stroke_rounded_rect(layout.streak_badge, 9.0, STATUS_WARNING.with_alpha(0.5), 1.0)?;
        surface.draw_text_bold(
            &format!("{}d streak", self.streak_days),
            layout.streak_badge,
            9.0,
            STATUS_WARNING,
            TextAlign::Center,
        )?;

        // Session card
        surface.fill_rounded_rect(layout.session_card, 12.0, CARD_BG)?;
        surface.stroke_rounded_rect(
            layout.session_card,
            12.0,
            if self.emergency_paused {
                STATUS_ALERT.with_alpha(0.7)
            } else {
                CARD_BORDER
            },
            1.0,
        )?;

        let status_color = match self.status {
            FocusSessionStatus::Active => ACCENT_CYAN,
            FocusSessionStatus::Paused => STATUS_ALERT,
            FocusSessionStatus::Completed => STATUS_SUCCESS,
            FocusSessionStatus::Skipped => STATUS_WARNING,
            FocusSessionStatus::Idle => TEXT_MUTED,
        };
        surface.draw_text_bold(
            &self.formatted_remaining(),
            Rect::new(layout.session_card.x + 12.0, layout.session_card.y + 10.0, 120.0, 38.0),
            26.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.draw_text_bold(
            self.status.label(),
            Rect::new(
                layout.session_card.x + 136.0,
                layout.session_card.y + 12.0,
                layout.session_card.width - 148.0,
                18.0,
            ),
            10.5,
            status_color,
            TextAlign::Trailing,
        )?;
        surface.draw_text(
            &format!("{} • {}", self.category, self.enforcement),
            Rect::new(
                layout.session_card.x + 136.0,
                layout.session_card.y + 30.0,
                layout.session_card.width - 148.0,
                16.0,
            ),
            9.5,
            TEXT_MUTED,
            TextAlign::Trailing,
        )?;

        // Session progress bar
        surface.fill_rounded_rect(layout.progress_bar, 3.0, TRACK_BG)?;
        let fill_w = layout.progress_bar.width * self.progress_ratio();
        if fill_w > 0.5 {
            surface.fill_rounded_rect(
                Rect::new(
                    layout.progress_bar.x,
                    layout.progress_bar.y,
                    fill_w.max(6.0),
                    layout.progress_bar.height,
                ),
                3.0,
                status_color,
            )?;
        }

        // Action buttons
        if let Some(btn) = layout.start_btn {
            surface.fill_rounded_rect(btn, 9.0, ACCENT_CYAN.with_alpha(0.24))?;
            surface.stroke_rounded_rect(btn, 9.0, ACCENT_CYAN, 1.0)?;
            surface.draw_text_bold("Start", btn, 10.5, TEXT_PRIMARY, TextAlign::Center)?;
        }
        if let Some(btn) = layout.complete_btn {
            surface.fill_rounded_rect(btn, 9.0, STATUS_SUCCESS.with_alpha(0.24))?;
            surface.stroke_rounded_rect(btn, 9.0, STATUS_SUCCESS, 1.0)?;
            surface.draw_text_bold("Complete", btn, 10.0, TEXT_PRIMARY, TextAlign::Center)?;
        }
        if let Some(btn) = layout.skip_btn {
            surface.fill_rounded_rect(btn, 9.0, CARD_BG_ACTIVE)?;
            surface.stroke_rounded_rect(btn, 9.0, CARD_BORDER, 1.0)?;
            surface.draw_text("Skip", btn, 10.0, TEXT_MUTED, TextAlign::Center)?;
        }

        let pause_bg = if self.emergency_paused {
            HIME_CRIMSON
        } else {
            STATUS_ALERT.with_alpha(0.18)
        };
        surface.fill_rounded_rect(layout.emergency_pause_btn, 9.0, pause_bg)?;
        surface.stroke_rounded_rect(
            layout.emergency_pause_btn,
            9.0,
            STATUS_ALERT.with_alpha(0.75),
            1.0,
        )?;
        surface.draw_text_bold(
            if self.emergency_paused {
                "Resume"
            } else {
                "Emergency Pause"
            },
            layout.emergency_pause_btn,
            9.5,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        // Today's screen-time summary card
        surface.fill_rounded_rect(layout.screentime_card, 12.0, CARD_BG)?;
        surface.stroke_rounded_rect(layout.screentime_card, 12.0, CARD_BORDER, 1.0)?;
        let total = self.screentime_total_secs();
        surface.draw_text_bold(
            "Screen-Time Today",
            Rect::new(
                layout.screentime_card.x + 12.0,
                layout.screentime_card.y + 8.0,
                160.0,
                16.0,
            ),
            10.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            &format!("Total {}", Self::format_duration_short(total)),
            Rect::new(
                layout.screentime_card.x + 170.0,
                layout.screentime_card.y + 8.0,
                layout.screentime_card.width - 182.0,
                16.0,
            ),
            9.5,
            TEXT_MUTED,
            TextAlign::Trailing,
        )?;

        // Stacked breakdown bar
        surface.fill_rounded_rect(layout.screentime_bar, 4.0, TRACK_BG)?;
        if total > 0 {
            let study_w = layout.screentime_bar.width * (self.study_secs as f32 / total as f32);
            let neutral_w = layout.screentime_bar.width * (self.neutral_secs as f32 / total as f32);
            let dist_w = (layout.screentime_bar.width - study_w - neutral_w).max(0.0);
            let mut cursor = layout.screentime_bar.x;
            if study_w > 1.0 {
                surface.fill_rounded_rect(
                    Rect::new(cursor, layout.screentime_bar.y, study_w, layout.screentime_bar.height),
                    4.0,
                    STATUS_SUCCESS,
                )?;
                cursor += study_w;
            }
            if neutral_w > 1.0 {
                surface.fill_rounded_rect(
                    Rect::new(cursor, layout.screentime_bar.y, neutral_w, layout.screentime_bar.height),
                    2.0,
                    ACCENT_CYAN.with_alpha(0.6),
                )?;
                cursor += neutral_w;
            }
            if dist_w > 1.0 {
                surface.fill_rounded_rect(
                    Rect::new(cursor, layout.screentime_bar.y, dist_w, layout.screentime_bar.height),
                    4.0,
                    STATUS_ALERT,
                )?;
            }
        }

        let legend_y = layout.screentime_bar.y + 14.0;
        let half_w = (layout.screentime_card.width - 24.0) * 0.5;
        surface.draw_text(
            &format!("Study: {}", Self::format_duration_short(self.study_secs)),
            Rect::new(layout.screentime_card.x + 12.0, legend_y, half_w, 16.0),
            9.5,
            STATUS_SUCCESS,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            &format!(
                "Distracting: {}",
                Self::format_duration_short(self.distracting_secs)
            ),
            Rect::new(
                layout.screentime_card.x + 12.0 + half_w,
                legend_y,
                half_w,
                16.0,
            ),
            9.5,
            STATUS_ALERT,
            TextAlign::Trailing,
        )?;

        Ok(())
    }
}

/// Proactive interruption level matching `11-PROACTIVE-INTERACTION.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProactiveLevel {
    Bisik,
    #[default]
    Colek,
    Tanya,
    Penting,
}

impl ProactiveLevel {
    pub fn accent_color(self) -> crate::theme::Color {
        match self {
            Self::Bisik => ACCENT_CYAN,
            Self::Colek => STATUS_WARNING,
            Self::Tanya => ACCENT_CYAN,
            Self::Penting => STATUS_ALERT,
        }
    }
}

/// Action emitted by [`ProactiveBannerState::hit_test`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProactiveBannerAction {
    Accept(String),
    Dismiss(String),
    Snooze(String),
    OpenDrawer,
}

/// Shared layout rectangles for [`ProactiveBannerState`].
#[derive(Debug, Clone)]
pub struct ProactiveBannerLayout {
    pub card_rect: Rect,
    pub accept_btn: Rect,
    pub snooze_btn: Rect,
    pub dismiss_btn: Rect,
    pub open_drawer_btn: Rect,
}

/// Dark-glass HUD proactive suggestion banner inside the Notch.
#[derive(Debug, Clone)]
pub struct ProactiveBannerState {
    pub active: bool,
    pub id: String,
    pub situation_id: String,
    pub level: ProactiveLevel,
    pub title: String,
    pub text: String,
    pub expression: HimeExpression,
    pub accept_label: String,
    pub snooze_label: String,
    pub auto_dismiss_secs: Option<f32>,
}

impl Default for ProactiveBannerState {
    fn default() -> Self {
        Self {
            active: false,
            id: String::new(),
            situation_id: String::new(),
            level: ProactiveLevel::Colek,
            title: String::new(),
            text: String::new(),
            expression: HimeExpression::Netral,
            accept_label: "Accept".to_string(),
            snooze_label: "Snooze".to_string(),
            auto_dismiss_secs: None,
        }
    }
}

impl ProactiveBannerState {
    pub fn show(
        &mut self,
        id: impl Into<String>,
        situation_id: impl Into<String>,
        level: ProactiveLevel,
        title: impl Into<String>,
        text: impl Into<String>,
        expression: HimeExpression,
        auto_dismiss_secs: Option<f32>,
    ) {
        self.active = true;
        self.id = id.into();
        self.situation_id = situation_id.into();
        self.level = level;
        self.title = title.into();
        self.text = text.into();
        self.expression = expression;
        self.auto_dismiss_secs = auto_dismiss_secs;
    }

    pub fn dismiss(&mut self) {
        self.active = false;
        self.auto_dismiss_secs = None;
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.active || dt <= 0.0 {
            return false;
        }
        if let Some(rem) = self.auto_dismiss_secs.as_mut() {
            *rem -= dt;
            if *rem <= 0.0 {
                self.dismiss();
                return true;
            }
        }
        false
    }

    pub fn layout(&self, area: Rect) -> ProactiveBannerLayout {
        let card_rect = Rect::new(
            area.x + 8.0,
            area.y + 6.0,
            (area.width - 16.0).max(160.0),
            (area.height - 12.0).max(64.0),
        );
        let dismiss_btn = Rect::new(
            card_rect.x + card_rect.width - 26.0,
            card_rect.y + 8.0,
            18.0,
            18.0,
        );
        let btn_y = card_rect.y + card_rect.height - 30.0;
        let btn_h = 22.0;
        let accept_btn = Rect::new(card_rect.x + 10.0, btn_y, 74.0, btn_h);
        let snooze_btn = Rect::new(accept_btn.x + accept_btn.width + 6.0, btn_y, 68.0, btn_h);
        let open_drawer_btn = Rect::new(
            card_rect.x + card_rect.width - 90.0,
            btn_y,
            80.0,
            btn_h,
        );
        ProactiveBannerLayout {
            card_rect,
            accept_btn,
            snooze_btn,
            dismiss_btn,
            open_drawer_btn,
        }
    }

    pub fn hit_test(&self, x: f32, y: f32, area: Rect) -> Option<ProactiveBannerAction> {
        if !self.active {
            return None;
        }
        let layout = self.layout(area);
        if point_in_rect(x, y, layout.dismiss_btn) {
            return Some(ProactiveBannerAction::Dismiss(self.id.clone()));
        }
        if point_in_rect(x, y, layout.accept_btn) {
            return Some(ProactiveBannerAction::Accept(self.id.clone()));
        }
        if point_in_rect(x, y, layout.snooze_btn) {
            return Some(ProactiveBannerAction::Snooze(self.id.clone()));
        }
        if point_in_rect(x, y, layout.open_drawer_btn) {
            return Some(ProactiveBannerAction::OpenDrawer);
        }
        None
    }

    pub fn render(&self, surface: &NativeSurface, area: Rect) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        let layout = self.layout(area);
        let accent = self.level.accent_color();

        surface.fill_rounded_rect(layout.card_rect, 12.0, GLASS_SURFACE)?;
        surface.stroke_rounded_rect(layout.card_rect, 12.0, accent.with_alpha(0.55), 1.0)?;

        surface.draw_hime_eyes(
            layout.card_rect.x + 22.0,
            layout.card_rect.y + 20.0,
            0.8,
            self.expression,
            0.0,
        )?;

        let text_x = layout.card_rect.x + 42.0;
        let text_w = (layout.dismiss_btn.x - text_x - 6.0).max(80.0);
        surface.draw_text_bold(
            &self.title,
            Rect::new(text_x, layout.card_rect.y + 6.0, text_w, 16.0),
            11.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            &self.text,
            Rect::new(text_x, layout.card_rect.y + 23.0, text_w, 18.0),
            9.5,
            TEXT_MUTED,
            TextAlign::Leading,
        )?;

        // Dismiss (X)
        surface.fill_rounded_rect(layout.dismiss_btn, 6.0, CARD_BG)?;
        let dcx = layout.dismiss_btn.x + layout.dismiss_btn.width * 0.5;
        let dcy = layout.dismiss_btn.y + layout.dismiss_btn.height * 0.5;
        surface.draw_line(dcx - 3.0, dcy - 3.0, dcx + 3.0, dcy + 3.0, TEXT_MUTED, 1.3)?;
        surface.draw_line(dcx + 3.0, dcy - 3.0, dcx - 3.0, dcy + 3.0, TEXT_MUTED, 1.3)?;

        // Accept / Snooze / OpenDrawer
        surface.fill_rounded_rect(layout.accept_btn, 7.0, accent.with_alpha(0.28))?;
        surface.stroke_rounded_rect(layout.accept_btn, 7.0, accent, 1.0)?;
        surface.draw_text_bold(
            &self.accept_label,
            layout.accept_btn,
            9.5,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        surface.fill_rounded_rect(layout.snooze_btn, 7.0, CARD_BG)?;
        surface.stroke_rounded_rect(layout.snooze_btn, 7.0, GLASS_BORDER, 1.0)?;
        surface.draw_text(
            &self.snooze_label,
            layout.snooze_btn,
            9.5,
            TEXT_MUTED,
            TextAlign::Center,
        )?;

        surface.fill_rounded_rect(layout.open_drawer_btn, 7.0, CARD_BG_ACTIVE)?;
        surface.stroke_rounded_rect(layout.open_drawer_btn, 7.0, GLASS_BORDER, 1.0)?;
        surface.draw_text(
            "Open AI",
            layout.open_drawer_btn,
            9.5,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        Ok(())
    }
}

/// Action emitted by [`TranslateBannerState::hit_test`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslateBannerAction {
    Close,
    OpenDrawer,
}

/// Shared layout rectangles for [`TranslateBannerState`].
#[derive(Debug, Clone)]
pub struct TranslateBannerLayout {
    pub card_rect: Rect,
    pub close_btn: Rect,
    pub open_drawer_btn: Rect,
}

/// Dark-glass HUD translation card inside the Notch.
#[derive(Debug, Clone, Default)]
pub struct TranslateBannerState {
    pub active: bool,
    pub loading: bool,
    pub source_text: String,
    pub translated_text: String,
    pub target_lang: String,
    pub cached: bool,
    pub error: Option<String>,
    pub auto_close_secs: Option<f32>,
}

impl TranslateBannerState {
    pub fn show_loading(&mut self, source_text: impl Into<String>, target_lang: impl Into<String>) {
        self.active = true;
        self.loading = true;
        self.source_text = source_text.into();
        self.translated_text.clear();
        self.target_lang = target_lang.into();
        self.cached = false;
        self.error = None;
        self.auto_close_secs = None;
    }

    pub fn show_result(
        &mut self,
        source_text: impl Into<String>,
        translated_text: impl Into<String>,
        target_lang: impl Into<String>,
        cached: bool,
    ) {
        self.active = true;
        self.loading = false;
        self.source_text = source_text.into();
        self.translated_text = translated_text.into();
        self.target_lang = target_lang.into();
        self.cached = cached;
        self.error = None;
        self.auto_close_secs = Some(9.0);
    }

    pub fn show_error(&mut self, error: impl Into<String>) {
        self.active = true;
        self.loading = false;
        self.error = Some(error.into());
        self.auto_close_secs = Some(6.0);
    }

    pub fn close(&mut self) {
        self.active = false;
        self.loading = false;
        self.auto_close_secs = None;
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.active || self.loading || dt <= 0.0 {
            return false;
        }
        if let Some(rem) = self.auto_close_secs.as_mut() {
            *rem -= dt;
            if *rem <= 0.0 {
                self.close();
                return true;
            }
        }
        false
    }

    pub fn layout(&self, area: Rect) -> TranslateBannerLayout {
        let card_rect = Rect::new(
            area.x + 8.0,
            area.y + 6.0,
            (area.width - 16.0).max(160.0),
            (area.height - 12.0).max(64.0),
        );
        let btn_y = card_rect.y + card_rect.height - 28.0;
        let close_btn = Rect::new(card_rect.x + card_rect.width - 68.0, btn_y, 58.0, 20.0);
        let open_drawer_btn = Rect::new(close_btn.x - 96.0, btn_y, 90.0, 20.0);
        TranslateBannerLayout {
            card_rect,
            close_btn,
            open_drawer_btn,
        }
    }

    pub fn hit_test(&self, x: f32, y: f32, area: Rect) -> Option<TranslateBannerAction> {
        if !self.active {
            return None;
        }
        let layout = self.layout(area);
        if point_in_rect(x, y, layout.close_btn) {
            return Some(TranslateBannerAction::Close);
        }
        if point_in_rect(x, y, layout.open_drawer_btn) {
            return Some(TranslateBannerAction::OpenDrawer);
        }
        None
    }

    pub fn render(&self, surface: &NativeSurface, area: Rect) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        let layout = self.layout(area);
        surface.fill_rounded_rect(layout.card_rect, 12.0, SURFACE_ELEVATED)?;
        surface.stroke_rounded_rect(layout.card_rect, 12.0, ACCENT_CYAN.with_alpha(0.45), 1.0)?;

        let lang_badge = Rect::new(layout.card_rect.x + 10.0, layout.card_rect.y + 8.0, 46.0, 16.0);
        surface.fill_rounded_rect(lang_badge, 6.0, ACCENT_CYAN.with_alpha(0.2))?;
        let lang_label = if self.target_lang.trim().is_empty() {
            "TR"
        } else {
            self.target_lang.as_str()
        };
        surface.draw_text_bold(lang_label, lang_badge, 8.5, ACCENT_CYAN, TextAlign::Center)?;

        let source_rect = Rect::new(
            lang_badge.x + lang_badge.width + 8.0,
            layout.card_rect.y + 7.0,
            (layout.card_rect.width - 74.0).max(80.0),
            16.0,
        );
        surface.draw_text(
            &self.source_text,
            source_rect,
            9.5,
            TEXT_SECONDARY,
            TextAlign::Leading,
        )?;

        let main_rect = Rect::new(
            layout.card_rect.x + 10.0,
            layout.card_rect.y + 27.0,
            layout.card_rect.width - 20.0,
            24.0,
        );
        if self.loading {
            surface.draw_text("Translating...", main_rect, 11.0, TEXT_MUTED, TextAlign::Leading)?;
        } else if let Some(err) = &self.error {
            surface.draw_text(err, main_rect, 10.5, STATUS_ALERT, TextAlign::Leading)?;
        } else {
            surface.draw_text_bold(
                &self.translated_text,
                main_rect,
                11.5,
                TEXT_PRIMARY,
                TextAlign::Leading,
            )?;
        }

        if self.cached && !self.loading && self.error.is_none() {
            surface.draw_text(
                "cached",
                Rect::new(
                    layout.card_rect.x + 10.0,
                    layout.close_btn.y,
                    60.0,
                    layout.close_btn.height,
                ),
                8.5,
                TEXT_SECONDARY,
                TextAlign::Leading,
            )?;
        }

        surface.fill_rounded_rect(layout.open_drawer_btn, 7.0, ACCENT_CYAN.with_alpha(0.22))?;
        surface.stroke_rounded_rect(layout.open_drawer_btn, 7.0, ACCENT_CYAN, 1.0)?;
        surface.draw_text_bold(
            "Open Drawer",
            layout.open_drawer_btn,
            9.0,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        surface.fill_rounded_rect(layout.close_btn, 7.0, CARD_BG)?;
        surface.stroke_rounded_rect(layout.close_btn, 7.0, GLASS_BORDER, 1.0)?;
        surface.draw_text(
            "Close",
            layout.close_btn,
            9.0,
            TEXT_MUTED,
            TextAlign::Center,
        )?;

        Ok(())
    }
}

fn point_in_rect(x: f32, y: f32, rect: Rect) -> bool {
    x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height
}

#[cfg(test)]
mod tests {
    use super::*;

    fn center(r: Rect) -> (f32, f32) {
        (r.x + r.width * 0.5, r.y + r.height * 0.5)
    }

    #[test]
    fn parse_lrc_lines_handles_formats_multi_timestamps_and_offset() {
        let raw = "\
[ti:Nightcall]\n\
[offset:+500]\n\
[00:01.50][00:05.000]Chorus line\n\
[00:03.2]Middle line\n\
[00:00.10]Intro\n";
        let lines = parse_lrc_lines(raw);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], LrcLine { ms: 600, text: "Intro".into() });
        assert_eq!(lines[1], LrcLine { ms: 2000, text: "Chorus line".into() });
        assert_eq!(lines[2], LrcLine { ms: 3700, text: "Middle line".into() });
        assert_eq!(lines[3], LrcLine { ms: 5500, text: "Chorus line".into() });
    }

    #[test]
    fn active_lyric_index_applies_offset_and_binary_search() {
        let lines = vec![
            LrcLine { ms: 1000, text: "First".into() },
            LrcLine { ms: 3000, text: "Second".into() },
            LrcLine { ms: 6000, text: "Third".into() },
        ];
        assert_eq!(active_lyric_index(&lines, 500, 0), None);
        assert_eq!(active_lyric_index(&lines, 1000, 0), Some(0));
        assert_eq!(active_lyric_index(&lines, 2999, 0), Some(0));
        assert_eq!(active_lyric_index(&lines, 2500, 500), Some(1));
        assert_eq!(active_lyric_index(&lines, 6000, -1000), Some(1));
        assert_eq!(active_lyric_index(&lines, 9000, 0), Some(2));
    }

    #[test]
    fn tasks_workspace_hit_test_and_mutations_work() {
        let mut ws = TasksWorkspaceState::default();
        ws.quick_add_draft = "Review PR".into();
        ws.quick_add_priority = TaskPriority::High;
        let body = Rect::new(24.0, 44.0, 352.0, 222.0);

        let layout = ws.layout(body);
        let (ax, ay) = center(layout.quick_add_submit_btn);
        assert_eq!(
            ws.hit_test(ax, ay, body),
            Some(TasksHitAction::QuickAdd("Review PR".into()))
        );
        let added = ws.commit_quick_add().expect("should add draft task");
        assert_eq!(added.priority, TaskPriority::High);
        assert!(ws.quick_add_draft.is_empty());

        let layout2 = ws.layout(body);
        let (tx, ty) = center(layout2.rows[0].toggle_rect);
        assert_eq!(
            ws.hit_test(tx, ty, body),
            Some(TasksHitAction::ToggleTask(added.id.clone()))
        );
        assert!(ws.toggle_task(&added.id));
        assert_eq!(ws.counts(), (0, 1));

        let (tab_x, tab_y) = center(layout2.tabs[2].1);
        assert_eq!(
            ws.hit_test(tab_x, tab_y, body),
            Some(TasksHitAction::SelectTab(TaskTabFilter::Done))
        );
        ws.active_tab = TaskTabFilter::Done;

        let layout3 = ws.layout(body);
        let (dx, dy) = center(layout3.rows[0].delete_rect);
        assert_eq!(
            ws.hit_test(dx, dy, body),
            Some(TasksHitAction::DeleteTask(added.id.clone()))
        );
        assert!(ws.delete_task(&added.id));
        assert_eq!(ws.counts(), (0, 0));
    }

    #[test]
    fn focus_workspace_controls_and_screentime_work() {
        let mut focus = FocusWorkspaceState {
            study_secs: 3600,
            distracting_secs: 900,
            ..Default::default()
        };
        let body = Rect::new(24.0, 44.0, 352.0, 222.0);

        let l_idle = focus.layout(body);
        let (sx, sy) = center(l_idle.start_btn.expect("start button visible in Idle"));
        assert_eq!(focus.hit_test(sx, sy, body), Some(FocusHitAction::Start));
        focus.start();
        assert_eq!(focus.status, FocusSessionStatus::Active);

        focus.tick(2.0);
        assert_eq!(focus.remaining_secs, 25 * 60 - 2);

        let l_active = focus.layout(body);
        let (px, py) = center(l_active.emergency_pause_btn);
        assert_eq!(
            focus.hit_test(px, py, body),
            Some(FocusHitAction::EmergencyPause)
        );
        focus.toggle_emergency_pause();
        assert_eq!(focus.status, FocusSessionStatus::Paused);
        assert!(focus.emergency_paused);

        let (cx, cy) = center(l_active.complete_btn.expect("complete button visible"));
        assert_eq!(focus.hit_test(cx, cy, body), Some(FocusHitAction::Complete));
        focus.complete();
        assert_eq!(focus.status, FocusSessionStatus::Completed);
        assert_eq!(focus.streak_days, 1);

        focus.start();
        let l_restarted = focus.layout(body);
        let (kx, ky) = center(l_restarted.skip_btn.expect("skip button visible"));
        assert_eq!(focus.hit_test(kx, ky, body), Some(FocusHitAction::Skip));
        focus.skip();
        assert_eq!(focus.status, FocusSessionStatus::Skipped);
        assert_eq!(focus.screentime_total_secs(), 4500);
    }

    #[test]
    fn proactive_and_translate_banners_hit_test_and_auto_expire() {
        let area = Rect::new(0.0, 0.0, 380.0, 96.0);
        let mut proactive = ProactiveBannerState::default();
        proactive.show(
            "sug-1",
            "break_reminder",
            ProactiveLevel::Tanya,
            "Take a break",
            "You have focused for 50 minutes",
            HimeExpression::Senang,
            Some(6.0),
        );
        let p_layout = proactive.layout(area);
        let (ax, ay) = center(p_layout.accept_btn);
        assert_eq!(
            proactive.hit_test(ax, ay, area),
            Some(ProactiveBannerAction::Accept("sug-1".into()))
        );
        let (sx, sy) = center(p_layout.snooze_btn);
        assert_eq!(
            proactive.hit_test(sx, sy, area),
            Some(ProactiveBannerAction::Snooze("sug-1".into()))
        );
        let (dx, dy) = center(p_layout.dismiss_btn);
        assert_eq!(
            proactive.hit_test(dx, dy, area),
            Some(ProactiveBannerAction::Dismiss("sug-1".into()))
        );
        let (ox, oy) = center(p_layout.open_drawer_btn);
        assert_eq!(
            proactive.hit_test(ox, oy, area),
            Some(ProactiveBannerAction::OpenDrawer)
        );
        assert!(proactive.tick(7.0));
        assert!(!proactive.active);

        let mut translate = TranslateBannerState::default();
        translate.show_result("Hello", "Halo", "id", true);
        let t_layout = translate.layout(area);
        let (tcx, tcy) = center(t_layout.close_btn);
        assert_eq!(
            translate.hit_test(tcx, tcy, area),
            Some(TranslateBannerAction::Close)
        );
        let (tox, toy) = center(t_layout.open_drawer_btn);
        assert_eq!(
            translate.hit_test(tox, toy, area),
            Some(TranslateBannerAction::OpenDrawer)
        );
        assert!(translate.tick(10.0));
        assert!(!translate.active);
    }
}
