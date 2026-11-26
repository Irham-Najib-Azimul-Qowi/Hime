//! Native AI Chat & Approval Drawer rendered with Direct2D (`NativeSurface`).
//! Styled with Apple Liquid Glass, monochrome typography hierarchy, `SfSymbol` vector strokes,
//! and smooth Apple `Spring` (`EMERGE`/`RECEDE`/`SMOOTH`) transitions.

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::{Spring, SpringValue};
use crate::d2d::{LiquidGlassStyle, NativeSurface, Rect, SfSymbol, TextAlign};
use crate::theme::{
    DIVIDER_HAIRLINE, GLASS_CONTROL, GLASS_HAIRLINE, STATUS_LIVE_AMBER, STATUS_LIVE_GREEN,
    STATUS_LIVE_RED, SYSTEM_BLUE, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY,
};

pub const DRAWER_WIDTH: f32 = 440.0;
pub const DRAWER_HEIGHT: f32 = 500.0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DrawerTab {
    #[default]
    Chat,
    Translate,
    Workspace,
}

impl DrawerTab {
    pub const ALL: [Self; 3] = [Self::Chat, Self::Translate, Self::Workspace];

    pub fn label(self) -> &'static str {
        match self {
            Self::Chat => "Chat",
            Self::Translate => "Translate",
            Self::Workspace => "Workspace",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DrawerTranslationState {
    pub source_text: String,
    pub translated_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub cached: bool,
    pub status_notice: Option<String>,
}

impl Default for DrawerTranslationState {
    fn default() -> Self {
        Self {
            source_text: String::new(),
            translated_text: String::new(),
            source_lang: "auto".to_string(),
            target_lang: "ID".to_string(),
            cached: false,
            status_notice: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeveloperWorkspaceSummary {
    pub branch: String,
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub untracked_count: usize,
    pub is_clean: bool,
    pub secret_findings_count: usize,
    pub last_check_status: Option<String>,
}

impl Default for DeveloperWorkspaceSummary {
    fn default() -> Self {
        Self {
            branch: "main".to_string(),
            staged_count: 0,
            unstaged_count: 0,
            untracked_count: 0,
            is_clean: true,
            secret_findings_count: 0,
            last_check_status: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentsWorkspaceSummary {
    pub active_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
    pub latest_job_label: Option<String>,
}

impl Default for DocumentsWorkspaceSummary {
    fn default() -> Self {
        Self {
            active_jobs: 0,
            completed_jobs: 0,
            failed_jobs: 0,
            latest_job_label: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DrawerAttachment {
    pub id: String,
    pub name: String,
    pub path: String,
    pub kind: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub id: String,
    pub role: ChatRole,
    pub content: String,
    pub streaming: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalCard {
    pub id: String,
    pub tool_name: String,
    pub summary: String,
    pub risk_level: String,
    pub status: ApprovalStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawerAction {
    Close,
    SelectTab(DrawerTab),
    ClearAttachments,
    SendPrompt,
    Approve(String),
    Deny(String),
    ClearTranslation,
    CopyTranslation,
    RunDevCheck,
    RefreshDocs,
}

/// Native AI Chat & Approval Drawer state + renderer.
pub struct AiDrawer {
    pub visible: bool,
    pub active_tab: DrawerTab,
    pub attachments: Vec<DrawerAttachment>,
    pub messages: Vec<ChatMessage>,
    pub approvals: Vec<ApprovalCard>,
    pub draft_prompt: String,
    pub translation: DrawerTranslationState,
    pub developer: DeveloperWorkspaceSummary,
    pub documents: DocumentsWorkspaceSummary,
    pub pointer_pos: Option<(f32, f32)>,
    pub pointer_pressed: bool,
    open_spring: SpringValue,
    tab_spring: SpringValue,
    press_spring: SpringValue,
    width: f32,
    height: f32,
}

impl AiDrawer {
    pub fn new() -> Self {
        Self {
            visible: false,
            active_tab: DrawerTab::Chat,
            attachments: Vec::new(),
            messages: Vec::new(),
            approvals: Vec::new(),
            draft_prompt: String::new(),
            translation: DrawerTranslationState::default(),
            developer: DeveloperWorkspaceSummary::default(),
            documents: DocumentsWorkspaceSummary::default(),
            pointer_pos: None,
            pointer_pressed: false,
            open_spring: SpringValue::from_spring(0.0, Spring::EMERGE),
            tab_spring: SpringValue::from_spring(0.0, Spring::SMOOTH),
            press_spring: SpringValue::from_spring(1.0, Spring::KNOB_SETTLE),
            width: DRAWER_WIDTH,
            height: DRAWER_HEIGHT,
        }
    }

    pub fn select_tab(&mut self, tab: DrawerTab) {
        self.active_tab = tab;
        if let Some(idx) = DrawerTab::ALL.iter().position(|&t| t == tab) {
            self.tab_spring.animate_to(idx as f32, Spring::SMOOTH);
        }
    }

    pub fn set_pointer_hover(&mut self, pos: Option<(f32, f32)>) {
        self.pointer_pos = pos;
    }

    pub fn set_pointer_pressed(&mut self, pressed: bool) {
        self.pointer_pressed = pressed;
        self.press_spring
            .animate_to(if pressed { 0.97 } else { 1.0 }, Spring::KNOB_SETTLE);
    }

    fn is_hovered(&self, rect: Rect) -> bool {
        self.pointer_pos
            .is_some_and(|(x, y)| point_in_rect(x, y, rect))
    }

    fn is_pressed(&self, rect: Rect) -> bool {
        self.pointer_pressed && self.is_hovered(rect)
    }

    fn button_rect(&self, rect: Rect) -> Rect {
        if self.is_pressed(rect) {
            let scale = self.press_spring.value.clamp(0.95, 1.0);
            let dx = rect.width * (1.0 - scale) * 0.5;
            let dy = rect.height * (1.0 - scale) * 0.5;
            Rect::new(
                rect.x + dx,
                rect.y + dy,
                (rect.width - dx * 2.0).max(4.0),
                (rect.height - dy * 2.0).max(4.0),
            )
        } else {
            rect
        }
    }

    pub fn set_translation(
        &mut self,
        source_text: impl Into<String>,
        translated_text: impl Into<String>,
        target_lang: impl Into<String>,
        cached: bool,
    ) {
        self.translation.source_text = source_text.into();
        self.translation.translated_text = translated_text.into();
        self.translation.target_lang = target_lang.into();
        self.translation.cached = cached;
        self.translation.status_notice = None;
    }

    pub fn clear_translation(&mut self) {
        self.translation.source_text.clear();
        self.translation.translated_text.clear();
        self.translation.cached = false;
        self.translation.status_notice = None;
    }

    pub fn mark_translation_copied(&mut self) {
        self.translation.status_notice = Some("Copied translation".to_string());
    }

    pub fn set_developer_summary(&mut self, summary: DeveloperWorkspaceSummary) {
        self.developer = summary;
    }

    pub fn set_documents_summary(&mut self, summary: DocumentsWorkspaceSummary) {
        self.documents = summary;
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.open_spring.launch_to(1.0, Spring::EMERGE, 1.0);
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.open_spring.launch_to(0.0, Spring::RECEDE, 1.0);
    }

    pub fn toggle(&mut self) {
        if self.visible {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let was_settled = self.open_spring.is_settled()
            && self.tab_spring.is_settled()
            && self.press_spring.is_settled();
        self.open_spring.tick(dt);
        self.tab_spring.tick(dt);
        self.press_spring.tick(dt);
        !was_settled
            || !self.open_spring.is_settled()
            || !self.tab_spring.is_settled()
            || !self.press_spring.is_settled()
    }

    pub fn is_animating(&self) -> bool {
        !self.open_spring.is_settled()
            || !self.tab_spring.is_settled()
            || !self.press_spring.is_settled()
    }

    pub fn attach_items(&mut self, items: Vec<DrawerAttachment>) {
        for item in items {
            if !self.attachments.iter().any(|existing| existing.id == item.id) {
                self.attachments.push(item);
            }
        }
    }

    pub fn clear_attachments(&mut self) {
        self.attachments.clear();
    }

    pub fn push_user_message(&mut self, id: impl Into<String>, content: impl Into<String>) {
        self.messages.push(ChatMessage {
            id: id.into(),
            role: ChatRole::User,
            content: content.into(),
            streaming: false,
        });
    }

    pub fn begin_assistant_stream(&mut self, id: impl Into<String>) {
        self.messages.push(ChatMessage {
            id: id.into(),
            role: ChatRole::Assistant,
            content: String::new(),
            streaming: true,
        });
    }

    pub fn append_stream_chunk(&mut self, chunk: &str) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == ChatRole::Assistant && last.streaming {
                last.content.push_str(chunk);
                return;
            }
        }
        self.messages.push(ChatMessage {
            id: format!("stream-{}", self.messages.len() + 1),
            role: ChatRole::Assistant,
            content: chunk.to_string(),
            streaming: true,
        });
    }

    pub fn finish_assistant_stream(&mut self) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == ChatRole::Assistant {
                last.streaming = false;
            }
        }
    }

    pub fn push_approval_card(&mut self, card: ApprovalCard) {
        self.approvals.push(card);
    }

    pub fn resolve_approval(&mut self, id: &str, approved: bool) -> Option<ApprovalCard> {
        let card = self.approvals.iter_mut().find(|c| c.id == id)?;
        card.status = if approved {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Denied
        };
        Some(card.clone())
    }

    fn close_rect(&self) -> Rect {
        Rect::new(self.width - 42.0, 14.0, 28.0, 28.0)
    }

    pub fn tab_rects(&self) -> [(DrawerTab, Rect); 3] {
        let start_x = 120.0;
        let tab_w = 86.0;
        let gap = 4.0;
        let y = 14.0;
        let h = 26.0;
        [
            (DrawerTab::Chat, Rect::new(start_x, y, tab_w, h)),
            (
                DrawerTab::Translate,
                Rect::new(start_x + tab_w + gap, y, tab_w, h),
            ),
            (
                DrawerTab::Workspace,
                Rect::new(start_x + (tab_w + gap) * 2.0, y, tab_w, h),
            ),
        ]
    }

    fn clear_attachments_rect(&self) -> Rect {
        Rect::new(self.width - 110.0, 50.0, 90.0, 26.0)
    }

    fn send_button_rect(&self) -> Rect {
        Rect::new(self.width - 88.0, self.height - 48.0, 68.0, 32.0)
    }

    fn translate_clear_rect(&self) -> Rect {
        Rect::new(20.0, 392.0, 96.0, 32.0)
    }

    fn translate_copy_rect(&self) -> Rect {
        Rect::new(self.width - 126.0, 392.0, 106.0, 32.0)
    }

    fn dev_check_rect(&self) -> Rect {
        Rect::new(self.width - 154.0, 198.0, 120.0, 32.0)
    }

    fn refresh_docs_rect(&self) -> Rect {
        Rect::new(self.width - 154.0, 390.0, 120.0, 32.0)
    }

    fn approval_button_rects(&self) -> Vec<(String, Rect, Rect)> {
        let mut out = Vec::new();
        let mut y = 340.0;
        for card in self.approvals.iter().filter(|c| c.status == ApprovalStatus::Pending) {
            let approve = Rect::new(self.width - 172.0, y + 36.0, 72.0, 26.0);
            let deny = Rect::new(self.width - 92.0, y + 36.0, 72.0, 26.0);
            out.push((card.id.clone(), approve, deny));
            y += 76.0;
        }
        out
    }

    pub fn handle_key_down(&mut self, vk: u32) -> Option<DrawerAction> {
        if !self.visible {
            return None;
        }
        const VK_ESCAPE: u32 = 0x1B;
        if vk == VK_ESCAPE {
            self.close();
            return Some(DrawerAction::Close);
        }
        None
    }

    pub fn handle_char(&mut self, ch: char) -> Option<DrawerAction> {
        if !self.visible || self.active_tab != DrawerTab::Chat {
            return None;
        }
        match ch {
            '\r' | '\n' => {
                if !self.draft_prompt.trim().is_empty() || !self.attachments.is_empty() {
                    Some(DrawerAction::SendPrompt)
                } else {
                    None
                }
            }
            '\x08' => {
                self.draft_prompt.pop();
                None
            }
            '\x1b' => None,
            c if c.is_control() => None,
            c => {
                self.draft_prompt.push(c);
                None
            }
        }
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<DrawerAction> {
        if !self.visible {
            return None;
        }
        if point_in_rect(x, y, self.close_rect()) {
            return Some(DrawerAction::Close);
        }
        for (tab, rect) in self.tab_rects() {
            if point_in_rect(x, y, rect) {
                return Some(DrawerAction::SelectTab(tab));
            }
        }
        match self.active_tab {
            DrawerTab::Chat => {
                if !self.attachments.is_empty() && point_in_rect(x, y, self.clear_attachments_rect())
                {
                    return Some(DrawerAction::ClearAttachments);
                }
                for (id, approve, deny) in self.approval_button_rects() {
                    if point_in_rect(x, y, approve) {
                        return Some(DrawerAction::Approve(id));
                    }
                    if point_in_rect(x, y, deny) {
                        return Some(DrawerAction::Deny(id));
                    }
                }
                if point_in_rect(x, y, self.send_button_rect()) {
                    return Some(DrawerAction::SendPrompt);
                }
                None
            }
            DrawerTab::Translate => {
                if point_in_rect(x, y, self.translate_clear_rect()) {
                    return Some(DrawerAction::ClearTranslation);
                }
                if point_in_rect(x, y, self.translate_copy_rect()) {
                    return Some(DrawerAction::CopyTranslation);
                }
                None
            }
            DrawerTab::Workspace => {
                if point_in_rect(x, y, self.dev_check_rect()) {
                    return Some(DrawerAction::RunDevCheck);
                }
                if point_in_rect(x, y, self.refresh_docs_rect()) {
                    return Some(DrawerAction::RefreshDocs);
                }
                None
            }
        }
    }

    /// Hit-tests `(x, y)` and applies local drawer state transitions (tab switches, close,
    /// translation clear/copy notice, attachment clear, approval resolution).
    pub fn handle_click(&mut self, x: f32, y: f32) -> Option<DrawerAction> {
        let action = self.hit_test(x, y)?;
        match &action {
            DrawerAction::Close => self.close(),
            DrawerAction::SelectTab(tab) => self.select_tab(*tab),
            DrawerAction::ClearAttachments => self.clear_attachments(),
            DrawerAction::Approve(id) => {
                self.resolve_approval(id, true);
            }
            DrawerAction::Deny(id) => {
                self.resolve_approval(id, false);
            }
            DrawerAction::ClearTranslation => self.clear_translation(),
            DrawerAction::CopyTranslation => self.mark_translation_copied(),
            DrawerAction::SendPrompt
            | DrawerAction::RunDevCheck
            | DrawerAction::RefreshDocs => {}
        }
        Some(action)
    }

    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let progress = self.open_spring.value.clamp(0.0, 1.0);
        if progress <= 0.01 && !self.visible {
            surface.set_rounded_region(Rect::new(0.0, 0.0, 0.0, 0.0), 0.0);
            return Ok(());
        }

        // Emerge/recede without collapsing from scale(0) — start from 0.94 scale with soft vertical slide.
        let scale_factor = 0.94 + 0.06 * progress;
        let pw = self.width * scale_factor;
        let ph = self.height * scale_factor;
        let px = (self.width - pw) * 0.5;
        let py = (1.0 - progress) * 8.0;
        let panel = Rect::new(px, py, pw, ph);

        surface.set_rounded_region(panel, 18.0);
        surface.draw_liquid_glass_panel(panel, LiquidGlassStyle::regular(18.0))?;

        // Header brand icon + monochrome title
        let brand_icon = Rect::new(px + 18.0, py + 17.0, 16.0, 16.0);
        surface.draw_sf_symbol(brand_icon, SfSymbol::Sparkle, SYSTEM_BLUE, 1.5)?;
        surface.draw_text_bold(
            "Hime AI",
            Rect::new(px + 38.0, py + 12.0, 78.0, 28.0),
            14.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        // Segmented Liquid Glass tab switcher with smooth spring-gliding active pill
        let tab_rects = self.tab_rects();
        let seg_x = tab_rects[0].1.x - 3.0;
        let seg_y = tab_rects[0].1.y - 2.0;
        let seg_w = (tab_rects[2].1.x + tab_rects[2].1.width) - tab_rects[0].1.x + 6.0;
        let seg_h = tab_rects[0].1.height + 4.0;
        let seg_container = Rect::new(seg_x, seg_y, seg_w, seg_h);
        surface.fill_rounded_rect(seg_container, 10.0, GLASS_CONTROL)?;
        surface.stroke_rounded_rect(seg_container, 10.0, GLASS_HAIRLINE, 0.8)?;

        // Sliding active tab indicator driven by `tab_spring`
        let slide_idx = self.tab_spring.value.clamp(0.0, 2.0);
        let lo = (slide_idx.floor() as usize).min(2);
        let hi = (slide_idx.ceil() as usize).min(2);
        let frac = slide_idx - lo as f32;
        let r_lo = tab_rects[lo].1;
        let r_hi = tab_rects[hi].1;
        let active_pill = Rect::new(
            r_lo.x + (r_hi.x - r_lo.x) * frac,
            r_lo.y,
            r_lo.width,
            r_lo.height,
        );
        surface.draw_liquid_glass_panel(
            active_pill,
            LiquidGlassStyle::tinted(SYSTEM_BLUE, 8.0).without_shadow(),
        )?;

        for (tab, rect) in tab_rects {
            let active = self.active_tab == tab;
            let hovered = !active && self.is_hovered(rect);
            if hovered {
                surface.fill_rounded_rect(
                    rect,
                    8.0,
                    crate::theme::GLASS_CONTROL_HOVER.with_alpha_mul(0.6),
                )?;
            }
            let icon = match tab {
                DrawerTab::Chat => SfSymbol::Sparkle,
                DrawerTab::Translate => SfSymbol::Globe,
                DrawerTab::Workspace => SfSymbol::Terminal,
            };
            let fg = if active { TEXT_PRIMARY } else { TEXT_SECONDARY };
            let icon_rect = Rect::new(rect.x + 7.0, rect.y + (rect.height - 12.0) * 0.5, 12.0, 12.0);
            surface.draw_sf_symbol(icon_rect, icon, fg, 1.3)?;
            let label_rect = Rect::new(rect.x + 20.0, rect.y, rect.width - 23.0, rect.height);
            if active {
                surface.draw_text_bold(tab.label(), label_rect, 11.0, fg, TextAlign::Center)?;
            } else {
                surface.draw_text(tab.label(), label_rect, 11.0, fg, TextAlign::Center)?;
            }
        }

        // Close button (`SfSymbol::CloseX`, no ASCII `"X"`)
        let close_r = self.button_rect(self.close_rect());
        surface.draw_liquid_glass_button(
            close_r,
            9.0,
            None,
            self.is_hovered(self.close_rect()),
            self.is_pressed(self.close_rect()),
        )?;
        let close_icon = Rect::new(close_r.x + 7.0, close_r.y + 7.0, 14.0, 14.0);
        surface.draw_sf_symbol(close_icon, SfSymbol::CloseX, TEXT_SECONDARY, 1.5)?;

        // Subtle hairline divider below header
        surface.draw_line(
            px + 18.0,
            py + 48.0,
            px + pw - 18.0,
            py + 48.0,
            DIVIDER_HAIRLINE,
            1.0,
        )?;

        match self.active_tab {
            DrawerTab::Chat => self.render_chat_tab(surface, panel.height)?,
            DrawerTab::Translate => self.render_translate_tab(surface, panel.height)?,
            DrawerTab::Workspace => self.render_workspace_tab(surface, panel.height)?,
        }

        Ok(())
    }

    fn render_chat_tab(&self, surface: &NativeSurface, panel_height: f32) -> Result<()> {
        // Attachments bar from FileShelf
        if !self.attachments.is_empty() {
            let attach_bar = Rect::new(20.0, 56.0, self.width - 40.0, 28.0);
            surface.draw_liquid_glass_panel(
                attach_bar,
                LiquidGlassStyle::elevated(9.0).without_shadow(),
            )?;
            surface.draw_sf_symbol(
                Rect::new(28.0, 63.0, 14.0, 14.0),
                SfSymbol::FolderDrop,
                SYSTEM_BLUE,
                1.4,
            )?;
            let summary = format!("{} shelf attachment(s)", self.attachments.len());
            surface.draw_text(
                &summary,
                Rect::new(48.0, 56.0, self.width - 168.0, 28.0),
                11.5,
                TEXT_PRIMARY,
                TextAlign::Leading,
            )?;
            let clear_raw = self.clear_attachments_rect();
            let clear_r = self.button_rect(clear_raw);
            surface.draw_liquid_glass_button(
                clear_r,
                7.0,
                None,
                self.is_hovered(clear_raw),
                self.is_pressed(clear_raw),
            )?;
            surface.draw_sf_symbol(
                Rect::new(clear_r.x + 8.0, clear_r.y + 7.0, 12.0, 12.0),
                SfSymbol::Trash,
                TEXT_SECONDARY,
                1.3,
            )?;
            surface.draw_text(
                "Clear",
                Rect::new(clear_r.x + 20.0, clear_r.y, clear_r.width - 24.0, clear_r.height),
                11.0,
                TEXT_SECONDARY,
                TextAlign::Center,
            )?;
        }

        // Recent messages
        let mut msg_y = if self.attachments.is_empty() { 58.0 } else { 92.0 };
        let max_msg_y = 330.0;
        for msg in self.messages.iter().rev().take(5).rev() {
            if msg_y + 44.0 > max_msg_y {
                break;
            }
            let bubble = Rect::new(20.0, msg_y, self.width - 40.0, 44.0);
            let style = match msg.role {
                ChatRole::User => LiquidGlassStyle::elevated(11.0).without_shadow(),
                ChatRole::Assistant => LiquidGlassStyle::clear(11.0).without_shadow(),
                ChatRole::System => LiquidGlassStyle::elevated(11.0).without_shadow(),
            };
            surface.draw_liquid_glass_panel(bubble, style)?;

            let role_label = match msg.role {
                ChatRole::User => "You",
                ChatRole::Assistant => {
                    if msg.streaming {
                        "Hime • Typing"
                    } else {
                        "Hime"
                    }
                }
                ChatRole::System => "System",
            };
            let role_color = match msg.role {
                ChatRole::Assistant => SYSTEM_BLUE,
                ChatRole::User => TEXT_SECONDARY,
                ChatRole::System => TEXT_TERTIARY,
            };
            surface.draw_text_bold(
                role_label,
                Rect::new(32.0, msg_y + 4.0, self.width - 64.0, 15.0),
                10.0,
                role_color,
                TextAlign::Leading,
            )?;
            surface.draw_text(
                &msg.content,
                Rect::new(32.0, msg_y + 19.0, self.width - 64.0, 21.0),
                12.0,
                if msg.role == ChatRole::System {
                    TEXT_SECONDARY
                } else {
                    TEXT_PRIMARY
                },
                TextAlign::Leading,
            )?;
            msg_y += 52.0;
        }

        // Pending approval cards
        let mut card_y = 340.0;
        for card in self
            .approvals
            .iter()
            .filter(|c| c.status == ApprovalStatus::Pending)
        {
            if card_y + 68.0 > panel_height - 56.0 {
                break;
            }
            let rect = Rect::new(20.0, card_y, self.width - 40.0, 68.0);
            surface.draw_liquid_glass_panel(
                rect,
                LiquidGlassStyle::elevated(12.0).without_shadow(),
            )?;
            surface.stroke_rounded_rect(rect, 12.0, STATUS_LIVE_AMBER.with_alpha(0.55), 1.0)?;

            surface.draw_sf_symbol(
                Rect::new(30.0, card_y + 10.0, 14.0, 14.0),
                SfSymbol::ShieldCheck,
                STATUS_LIVE_AMBER,
                1.4,
            )?;
            surface.draw_text_bold(
                &format!("Approve {}: {}", card.tool_name, card.summary),
                Rect::new(50.0, card_y + 7.0, self.width - 80.0, 22.0),
                11.5,
                TEXT_PRIMARY,
                TextAlign::Leading,
            )?;

            let approve_raw = Rect::new(self.width - 172.0, card_y + 36.0, 72.0, 26.0);
            let deny_raw = Rect::new(self.width - 92.0, card_y + 36.0, 72.0, 26.0);
            let approve = self.button_rect(approve_raw);
            let deny = self.button_rect(deny_raw);

            surface.draw_liquid_glass_button(
                approve,
                7.0,
                Some(STATUS_LIVE_GREEN),
                self.is_hovered(approve_raw),
                self.is_pressed(approve_raw),
            )?;
            surface.draw_sf_symbol(
                Rect::new(approve.x + 6.0, approve.y + 7.0, 11.0, 11.0),
                SfSymbol::Check,
                TEXT_PRIMARY,
                1.4,
            )?;
            surface.draw_text_bold(
                "Allow",
                Rect::new(approve.x + 17.0, approve.y, approve.width - 20.0, approve.height),
                11.0,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;

            surface.draw_liquid_glass_button(
                deny,
                7.0,
                Some(STATUS_LIVE_RED),
                self.is_hovered(deny_raw),
                self.is_pressed(deny_raw),
            )?;
            surface.draw_sf_symbol(
                Rect::new(deny.x + 8.0, deny.y + 7.5, 10.0, 10.0),
                SfSymbol::CloseX,
                TEXT_PRIMARY,
                1.4,
            )?;
            surface.draw_text(
                "Deny",
                Rect::new(deny.x + 18.0, deny.y, deny.width - 22.0, deny.height),
                11.0,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;
            card_y += 76.0;
        }

        // Input footer
        if panel_height >= 120.0 {
            let input_rect = Rect::new(20.0, self.height - 48.0, self.width - 116.0, 32.0);
            surface.draw_liquid_glass_panel(
                input_rect,
                LiquidGlassStyle::elevated(10.0).without_shadow(),
            )?;
            let prompt_text = if self.draft_prompt.is_empty() {
                "Ask Hime AI..."
            } else {
                self.draft_prompt.as_str()
            };
            let prompt_color = if self.draft_prompt.is_empty() {
                TEXT_TERTIARY
            } else {
                TEXT_PRIMARY
            };
            surface.draw_text(
                prompt_text,
                Rect::new(32.0, self.height - 48.0, self.width - 140.0, 32.0),
                12.0,
                prompt_color,
                TextAlign::Leading,
            )?;
            if !self.draft_prompt.is_empty() {
                let approx_caret_x =
                    (34.0 + self.draft_prompt.chars().count() as f32 * 6.4).min(input_rect.x + input_rect.width - 10.0);
                surface.draw_line(
                    approx_caret_x,
                    input_rect.y + 8.0,
                    approx_caret_x,
                    input_rect.y + input_rect.height - 8.0,
                    SYSTEM_BLUE,
                    1.5,
                )?;
            }

            let send_raw = self.send_button_rect();
            let send_rect = self.button_rect(send_raw);
            let can_send = !self.draft_prompt.trim().is_empty() || !self.attachments.is_empty();
            surface.draw_liquid_glass_button(
                send_rect,
                10.0,
                if can_send { Some(SYSTEM_BLUE) } else { None },
                self.is_hovered(send_raw),
                self.is_pressed(send_raw),
            )?;
            surface.draw_sf_symbol(
                Rect::new(send_rect.x + 9.0, send_rect.y + 10.0, 12.0, 12.0),
                SfSymbol::SendArrow,
                TEXT_PRIMARY,
                1.5,
            )?;
            surface.draw_text_bold(
                "Send",
                Rect::new(send_rect.x + 21.0, send_rect.y, send_rect.width - 25.0, send_rect.height),
                11.5,
                TEXT_PRIMARY,
                TextAlign::Center,
            )?;
        }

        Ok(())
    }

    fn render_translate_tab(&self, surface: &NativeSurface, panel_height: f32) -> Result<()> {
        if panel_height < 140.0 {
            return Ok(());
        }

        surface.draw_text_bold(
            "Selection Translate",
            Rect::new(20.0, 58.0, 220.0, 24.0),
            13.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        let lang_badge = Rect::new(self.width - 130.0, 58.0, 110.0, 24.0);
        surface.fill_rounded_rect(lang_badge, 8.0, GLASS_CONTROL)?;
        surface.stroke_rounded_rect(lang_badge, 8.0, GLASS_HAIRLINE, 0.8)?;
        surface.draw_sf_symbol(
            Rect::new(lang_badge.x + 8.0, lang_badge.y + 6.0, 12.0, 12.0),
            SfSymbol::Globe,
            TEXT_SECONDARY,
            1.3,
        )?;
        surface.draw_text(
            &format!("Target: {}", self.translation.target_lang.to_uppercase()),
            Rect::new(lang_badge.x + 22.0, lang_badge.y, lang_badge.width - 26.0, lang_badge.height),
            10.5,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        let source_card = Rect::new(20.0, 92.0, self.width - 40.0, 112.0);
        surface.draw_liquid_glass_panel(
            source_card,
            LiquidGlassStyle::elevated(12.0).without_shadow(),
        )?;
        surface.draw_text(
            &format!("SOURCE ({})", self.translation.source_lang.to_uppercase()),
            Rect::new(32.0, 100.0, self.width - 64.0, 18.0),
            10.5,
            TEXT_TERTIARY,
            TextAlign::Leading,
        )?;
        let source_body = if self.translation.source_text.is_empty() {
            "Highlight text and trigger Translate Selection"
        } else {
            self.translation.source_text.as_str()
        };
        let source_color = if self.translation.source_text.is_empty() {
            TEXT_SECONDARY
        } else {
            TEXT_PRIMARY
        };
        surface.draw_text(
            source_body,
            Rect::new(32.0, 122.0, self.width - 64.0, 74.0),
            12.0,
            source_color,
            TextAlign::Leading,
        )?;

        let result_card = Rect::new(20.0, 216.0, self.width - 40.0, 160.0);
        surface.draw_liquid_glass_panel(
            result_card,
            LiquidGlassStyle::elevated(12.0).without_shadow(),
        )?;
        let header_label = if self.translation.cached {
            "TRANSLATION • CACHED"
        } else {
            "TRANSLATION"
        };
        surface.draw_text(
            header_label,
            Rect::new(32.0, 224.0, self.width - 64.0, 18.0),
            10.5,
            if self.translation.cached {
                SYSTEM_BLUE
            } else {
                TEXT_TERTIARY
            },
            TextAlign::Leading,
        )?;
        let translated_body = if self.translation.translated_text.is_empty() {
            "No translation yet"
        } else {
            self.translation.translated_text.as_str()
        };
        let translated_color = if self.translation.translated_text.is_empty() {
            TEXT_SECONDARY
        } else {
            TEXT_PRIMARY
        };
        surface.draw_text(
            translated_body,
            Rect::new(32.0, 246.0, self.width - 64.0, 120.0),
            13.0,
            translated_color,
            TextAlign::Leading,
        )?;

        let clear_raw = self.translate_clear_rect();
        let clear_rect = self.button_rect(clear_raw);
        surface.draw_liquid_glass_button(
            clear_rect,
            8.0,
            None,
            self.is_hovered(clear_raw),
            self.is_pressed(clear_raw),
        )?;
        surface.draw_sf_symbol(
            Rect::new(clear_rect.x + 12.0, clear_rect.y + 9.0, 13.0, 13.0),
            SfSymbol::Trash,
            TEXT_SECONDARY,
            1.3,
        )?;
        surface.draw_text(
            "Clear",
            Rect::new(clear_rect.x + 26.0, clear_rect.y, clear_rect.width - 32.0, clear_rect.height),
            11.5,
            TEXT_SECONDARY,
            TextAlign::Center,
        )?;

        let copy_raw = self.translate_copy_rect();
        let copy_rect = self.button_rect(copy_raw);
        let has_result = !self.translation.translated_text.is_empty();
        surface.draw_liquid_glass_button(
            copy_rect,
            8.0,
            if has_result { Some(SYSTEM_BLUE) } else { None },
            self.is_hovered(copy_raw),
            self.is_pressed(copy_raw),
        )?;
        surface.draw_sf_symbol(
            Rect::new(copy_rect.x + 14.0, copy_rect.y + 9.0, 13.0, 13.0),
            SfSymbol::Copy,
            if has_result { TEXT_PRIMARY } else { TEXT_SECONDARY },
            1.4,
        )?;
        surface.draw_text_bold(
            "Copy",
            Rect::new(copy_rect.x + 28.0, copy_rect.y, copy_rect.width - 34.0, copy_rect.height),
            11.5,
            if has_result { TEXT_PRIMARY } else { TEXT_SECONDARY },
            TextAlign::Center,
        )?;

        if let Some(notice) = &self.translation.status_notice {
            surface.draw_text(
                notice,
                Rect::new(20.0, 434.0, self.width - 40.0, 22.0),
                11.0,
                STATUS_LIVE_GREEN,
                TextAlign::Leading,
            )?;
        }

        Ok(())
    }

    fn render_workspace_tab(&self, surface: &NativeSurface, panel_height: f32) -> Result<()> {
        if panel_height < 160.0 {
            return Ok(());
        }

        // Developer card
        let dev_card = Rect::new(20.0, 58.0, self.width - 40.0, 192.0);
        surface.draw_liquid_glass_panel(
            dev_card,
            LiquidGlassStyle::elevated(12.0).without_shadow(),
        )?;
        surface.draw_sf_symbol(
            Rect::new(34.0, 71.0, 15.0, 15.0),
            SfSymbol::Terminal,
            TEXT_PRIMARY,
            1.4,
        )?;
        surface.draw_text_bold(
            "Developer",
            Rect::new(54.0, 68.0, 160.0, 22.0),
            13.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        let branch_pill = Rect::new(self.width - 174.0, 66.0, 140.0, 24.0);
        surface.fill_rounded_rect(branch_pill, 7.0, GLASS_CONTROL)?;
        surface.stroke_rounded_rect(branch_pill, 7.0, GLASS_HAIRLINE, 0.8)?;
        surface.draw_text(
            &self.developer.branch,
            branch_pill,
            11.0,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        let git_summary = if self.developer.is_clean {
            "Git Status: Clean working tree".to_string()
        } else {
            format!(
                "Git Status: {} staged • {} modified • {} untracked",
                self.developer.staged_count,
                self.developer.unstaged_count,
                self.developer.untracked_count
            )
        };
        surface.draw_text(
            &git_summary,
            Rect::new(34.0, 102.0, self.width - 68.0, 22.0),
            11.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        let scan_color = if self.developer.secret_findings_count == 0 {
            STATUS_LIVE_GREEN
        } else {
            STATUS_LIVE_RED
        };
        let scan_summary = format!(
            "Secret Scan: {} finding(s)",
            self.developer.secret_findings_count
        );
        surface.draw_text(
            &scan_summary,
            Rect::new(34.0, 134.0, self.width - 68.0, 22.0),
            11.5,
            scan_color,
            TextAlign::Leading,
        )?;

        if let Some(status) = &self.developer.last_check_status {
            surface.draw_text(
                status,
                Rect::new(34.0, 202.0, self.width - 200.0, 24.0),
                11.0,
                TEXT_SECONDARY,
                TextAlign::Leading,
            )?;
        }

        let dev_raw = self.dev_check_rect();
        let dev_btn = self.button_rect(dev_raw);
        surface.draw_liquid_glass_button(
            dev_btn,
            8.0,
            None,
            self.is_hovered(dev_raw),
            self.is_pressed(dev_raw),
        )?;
        surface.draw_sf_symbol(
            Rect::new(dev_btn.x + 9.0, dev_btn.y + 9.0, 13.0, 13.0),
            SfSymbol::ShieldCheck,
            TEXT_PRIMARY,
            1.3,
        )?;
        surface.draw_text(
            "Dev Check",
            Rect::new(dev_btn.x + 22.0, dev_btn.y, dev_btn.width - 26.0, dev_btn.height),
            11.5,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        // Documents card
        let docs_card = Rect::new(20.0, 266.0, self.width - 40.0, 174.0);
        surface.draw_liquid_glass_panel(
            docs_card,
            LiquidGlassStyle::elevated(12.0).without_shadow(),
        )?;
        surface.draw_sf_symbol(
            Rect::new(34.0, 281.0, 15.0, 15.0),
            SfSymbol::Document,
            TEXT_PRIMARY,
            1.4,
        )?;
        surface.draw_text_bold(
            "Documents",
            Rect::new(54.0, 278.0, 160.0, 22.0),
            13.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        let total_jobs =
            self.documents.active_jobs + self.documents.completed_jobs + self.documents.failed_jobs;
        let jobs_pill = Rect::new(self.width - 144.0, 276.0, 110.0, 24.0);
        surface.fill_rounded_rect(jobs_pill, 7.0, GLASS_CONTROL)?;
        surface.stroke_rounded_rect(jobs_pill, 7.0, GLASS_HAIRLINE, 0.8)?;
        surface.draw_text(
            &format!("{total_jobs} job(s)"),
            jobs_pill,
            11.0,
            TEXT_SECONDARY,
            TextAlign::Center,
        )?;

        let jobs_summary = format!(
            "Jobs: {} active • {} completed • {} failed",
            self.documents.active_jobs,
            self.documents.completed_jobs,
            self.documents.failed_jobs
        );
        surface.draw_text(
            &jobs_summary,
            Rect::new(34.0, 314.0, self.width - 68.0, 22.0),
            11.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        let latest_doc = self
            .documents
            .latest_job_label
            .as_deref()
            .unwrap_or("No recent document jobs");
        surface.draw_text(
            latest_doc,
            Rect::new(34.0, 346.0, self.width - 68.0, 22.0),
            11.0,
            TEXT_SECONDARY,
            TextAlign::Leading,
        )?;

        let docs_raw = self.refresh_docs_rect();
        let docs_btn = self.button_rect(docs_raw);
        surface.draw_liquid_glass_button(
            docs_btn,
            8.0,
            None,
            self.is_hovered(docs_raw),
            self.is_pressed(docs_raw),
        )?;
        surface.draw_sf_symbol(
            Rect::new(docs_btn.x + 9.0, docs_btn.y + 9.0, 13.0, 13.0),
            SfSymbol::Refresh,
            TEXT_PRIMARY,
            1.3,
        )?;
        surface.draw_text(
            "Refresh",
            Rect::new(docs_btn.x + 22.0, docs_btn.y, docs_btn.width - 26.0, docs_btn.height),
            11.5,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        Ok(())
    }
}

impl Default for AiDrawer {
    fn default() -> Self {
        Self::new()
    }
}

fn point_in_rect(x: f32, y: f32, r: Rect) -> bool {
    x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaming_text_accumulates_and_finishes() {
        let mut drawer = AiDrawer::new();
        drawer.push_user_message("u1", "Summarize shelf");
        drawer.begin_assistant_stream("a1");
        drawer.append_stream_chunk("Hello ");
        drawer.append_stream_chunk("world");
        assert!(drawer.messages[1].streaming);
        assert_eq!(drawer.messages[1].content, "Hello world");

        drawer.finish_assistant_stream();
        assert!(!drawer.messages[1].streaming);
    }

    #[test]
    fn shelf_attachments_dedup_and_clear() {
        let mut drawer = AiDrawer::new();
        let item = DrawerAttachment {
            id: "f1".to_string(),
            name: "spec.md".to_string(),
            path: "C:\\spec.md".to_string(),
            kind: "file".to_string(),
        };
        drawer.attach_items(vec![item.clone(), item]);
        assert_eq!(drawer.attachments.len(), 1);
        drawer.clear_attachments();
        assert!(drawer.attachments.is_empty());
    }

    #[test]
    fn approval_card_hit_test_and_resolution() {
        let mut drawer = AiDrawer::new();
        drawer.open();
        drawer.push_approval_card(ApprovalCard {
            id: "apr-42".to_string(),
            tool_name: "run_shell".to_string(),
            summary: "cargo test".to_string(),
            risk_level: "medium".to_string(),
            status: ApprovalStatus::Pending,
        });

        let action = drawer.hit_test(drawer.width - 140.0, 340.0 + 42.0);
        assert_eq!(action, Some(DrawerAction::Approve("apr-42".to_string())));

        let updated = drawer.resolve_approval("apr-42", true).unwrap();
        assert_eq!(updated.status, ApprovalStatus::Approved);
    }

    #[test]
    fn keyboard_input_updates_draft_and_handles_enter_backspace_escape() {
        let mut drawer = AiDrawer::new();
        // Ignored while hidden
        assert_eq!(drawer.handle_char('a'), None);
        assert!(drawer.draft_prompt.is_empty());

        drawer.open();
        assert_eq!(drawer.handle_char('h'), None);
        assert_eq!(drawer.handle_char('i'), None);
        assert_eq!(drawer.handle_char('!'), None);
        assert_eq!(drawer.draft_prompt, "hi!");

        // Backspace removes last char
        assert_eq!(drawer.handle_char('\x08'), None);
        assert_eq!(drawer.draft_prompt, "hi");

        // Enter triggers SendPrompt
        assert_eq!(drawer.handle_char('\r'), Some(DrawerAction::SendPrompt));

        // Escape closes drawer
        assert_eq!(drawer.handle_key_down(0x1B), Some(DrawerAction::Close));
        assert!(!drawer.visible);
    }

    fn rect_in_bounds(r: Rect) -> bool {
        r.x >= 0.0
            && r.y >= 0.0
            && r.x + r.width <= DRAWER_WIDTH
            && r.y + r.height <= DRAWER_HEIGHT
    }

    fn rects_overlap(a: Rect, b: Rect) -> bool {
        a.x < b.x + b.width
            && a.x + a.width > b.x
            && a.y < b.y + b.height
            && a.y + a.height > b.y
    }

    #[test]
    fn drawer_tabs_switch_and_stay_within_440x500_bounds() {
        let mut drawer = AiDrawer::new();
        drawer.open();

        let close = drawer.close_rect();
        assert!(rect_in_bounds(close));

        let tabs = drawer.tab_rects();
        for (i, (tab, rect)) in tabs.iter().enumerate() {
            assert!(rect_in_bounds(*rect), "tab {:?} out of bounds", tab);
            assert!(!rects_overlap(*rect, close), "tab {:?} overlaps close", tab);
            for j in (i + 1)..tabs.len() {
                assert!(
                    !rects_overlap(*rect, tabs[j].1),
                    "tabs {:?} and {:?} overlap",
                    tab,
                    tabs[j].0
                );
            }

            let cx = rect.x + rect.width * 0.5;
            let cy = rect.y + rect.height * 0.5;
            assert_eq!(drawer.handle_click(cx, cy), Some(DrawerAction::SelectTab(*tab)));
            assert_eq!(drawer.active_tab, *tab);
        }

        // All action button rects fit inside 440x500
        for r in [
            drawer.clear_attachments_rect(),
            drawer.send_button_rect(),
            drawer.translate_clear_rect(),
            drawer.translate_copy_rect(),
            drawer.dev_check_rect(),
            drawer.refresh_docs_rect(),
        ] {
            assert!(rect_in_bounds(r), "action rect out of bounds: {:?}", r);
        }
        assert!(!rects_overlap(
            drawer.translate_clear_rect(),
            drawer.translate_copy_rect()
        ));
        assert!(!rects_overlap(
            drawer.dev_check_rect(),
            drawer.refresh_docs_rect()
        ));
    }

    #[test]
    fn translate_tab_hit_test_and_clear_copy_actions() {
        let mut drawer = AiDrawer::new();
        drawer.open();
        drawer.set_translation("Hello world", "Halo dunia", "id", true);

        let translate_tab = drawer.tab_rects()[1].1;
        assert_eq!(
            drawer.handle_click(
                translate_tab.x + translate_tab.width * 0.5,
                translate_tab.y + translate_tab.height * 0.5
            ),
            Some(DrawerAction::SelectTab(DrawerTab::Translate))
        );
        assert_eq!(drawer.active_tab, DrawerTab::Translate);

        // Typing is ignored outside Chat tab
        assert_eq!(drawer.handle_char('x'), None);
        assert!(drawer.draft_prompt.is_empty());

        // Chat Send button does not fire on Translate tab
        let send = drawer.send_button_rect();
        assert_eq!(drawer.hit_test(send.x + 10.0, send.y + 10.0), None);

        // Copy action
        let copy = drawer.translate_copy_rect();
        assert_eq!(
            drawer.handle_click(copy.x + 10.0, copy.y + 10.0),
            Some(DrawerAction::CopyTranslation)
        );
        assert_eq!(
            drawer.translation.status_notice.as_deref(),
            Some("Copied translation")
        );

        // Clear action
        let clear = drawer.translate_clear_rect();
        assert_eq!(
            drawer.handle_click(clear.x + 10.0, clear.y + 10.0),
            Some(DrawerAction::ClearTranslation)
        );
        assert!(drawer.translation.source_text.is_empty());
        assert!(drawer.translation.translated_text.is_empty());
        assert!(!drawer.translation.cached);
    }

    #[test]
    fn workspace_tab_developer_and_documents_hit_testing() {
        let mut drawer = AiDrawer::new();
        drawer.open();
        drawer.set_developer_summary(DeveloperWorkspaceSummary {
            branch: "feat/native".to_string(),
            staged_count: 1,
            unstaged_count: 2,
            untracked_count: 1,
            is_clean: false,
            secret_findings_count: 3,
            last_check_status: Some("3 secret findings".to_string()),
        });
        drawer.set_documents_summary(DocumentsWorkspaceSummary {
            active_jobs: 1,
            completed_jobs: 4,
            failed_jobs: 0,
            latest_job_label: Some("merged_report.pdf".to_string()),
        });

        let ws_tab = drawer.tab_rects()[2].1;
        assert_eq!(
            drawer.handle_click(
                ws_tab.x + ws_tab.width * 0.5,
                ws_tab.y + ws_tab.height * 0.5
            ),
            Some(DrawerAction::SelectTab(DrawerTab::Workspace))
        );

        let dev_btn = drawer.dev_check_rect();
        assert_eq!(
            drawer.hit_test(dev_btn.x + 12.0, dev_btn.y + 12.0),
            Some(DrawerAction::RunDevCheck)
        );

        let docs_btn = drawer.refresh_docs_rect();
        assert_eq!(
            drawer.hit_test(docs_btn.x + 12.0, docs_btn.y + 12.0),
            Some(DrawerAction::RefreshDocs)
        );

        // Close works from Workspace tab too
        let close = drawer.close_rect();
        assert_eq!(
            drawer.handle_click(close.x + 6.0, close.y + 6.0),
            Some(DrawerAction::Close)
        );
        assert!(!drawer.visible);
    }

    #[test]
    fn drawer_uses_apple_springs_and_renders_liquid_glass_with_sf_symbols() {
        let mut drawer = AiDrawer::new();
        drawer.open();
        assert!(drawer.is_animating());
        for _ in 0..90 {
            drawer.tick(1.0 / 60.0);
        }
        assert!(!drawer.is_animating());

        drawer.push_user_message("u1", "Summarize workspace");
        drawer.begin_assistant_stream("a1");
        drawer.append_stream_chunk("Ready with Liquid Glass.");
        drawer.finish_assistant_stream();
        drawer.attach_items(vec![DrawerAttachment {
            id: "s1".to_string(),
            name: "design.md".to_string(),
            path: "C:\\design.md".to_string(),
            kind: "Document".to_string(),
        }]);
        drawer.push_approval_card(ApprovalCard {
            id: "apr-1".to_string(),
            tool_name: "git_status".to_string(),
            summary: "Inspect repository".to_string(),
            risk_level: "low".to_string(),
            status: ApprovalStatus::Pending,
        });

        let surface = match NativeSurface::new("AiDrawerLiquidTest", -2200, -2200, 440, 500) {
            Ok(s) => s,
            Err(_) => return,
        };
        for tab in DrawerTab::ALL {
            drawer.select_tab(tab);
            for _ in 0..45 {
                drawer.tick(1.0 / 60.0);
            }
            surface.begin_draw();
            drawer.render(&surface).expect("AiDrawer::render should succeed");
            surface.end_draw_and_present().expect("present should succeed");
        }
    }
}
