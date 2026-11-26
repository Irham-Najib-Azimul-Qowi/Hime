//! Native AI Chat & Approval Drawer rendered with Direct2D (`NativeSurface`).
//! Supports shelf file attachments, streaming assistant responses, and tool approval cards.

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::SpringValue;
use crate::d2d::{NativeSurface, Rect, TextAlign};
use crate::theme::{
    ACCENT_CYAN, GLASS_BORDER, GLASS_SURFACE, STATUS_ALERT, STATUS_OFFLINE, STATUS_SUCCESS,
    STATUS_WARNING, SURFACE_ELEVATED, TEXT_PRIMARY, TEXT_SECONDARY,
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
    open_spring: SpringValue,
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
            open_spring: SpringValue::snappy(0.0),
            width: DRAWER_WIDTH,
            height: DRAWER_HEIGHT,
        }
    }

    pub fn select_tab(&mut self, tab: DrawerTab) {
        self.active_tab = tab;
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
        self.open_spring.set_target(1.0);
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.open_spring.set_target(0.0);
    }

    pub fn toggle(&mut self) {
        if self.visible {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let was_settled = self.open_spring.is_settled();
        self.open_spring.tick(dt);
        !was_settled || !self.open_spring.is_settled()
    }

    pub fn is_animating(&self) -> bool {
        !self.open_spring.is_settled()
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
            return Ok(());
        }

        let panel = Rect::new(0.0, 0.0, self.width, self.height * progress.max(0.2));
        surface.set_rounded_region(panel, 18.0);
        surface.fill_rounded_rect(panel, 18.0, GLASS_SURFACE)?;
        surface.stroke_rounded_rect(panel, 18.0, GLASS_BORDER, 1.0)?;

        surface.draw_text(
            "Hime AI",
            Rect::new(20.0, 14.0, 92.0, 28.0),
            15.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        // Tab switcher in header
        for (tab, rect) in self.tab_rects() {
            let active = self.active_tab == tab;
            surface.fill_rounded_rect(
                rect,
                8.0,
                if active {
                    ACCENT_CYAN.with_alpha(0.20)
                } else {
                    SURFACE_ELEVATED.with_alpha(0.65)
                },
            )?;
            surface.stroke_rounded_rect(
                rect,
                8.0,
                if active {
                    ACCENT_CYAN.with_alpha(0.75)
                } else {
                    GLASS_BORDER
                },
                1.0,
            )?;
            surface.draw_text(
                tab.label(),
                rect,
                11.5,
                if active { TEXT_PRIMARY } else { TEXT_SECONDARY },
                TextAlign::Center,
            )?;
        }

        surface.draw_text(
            "X",
            self.close_rect(),
            13.0,
            TEXT_SECONDARY,
            TextAlign::Center,
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
            let attach_bar = Rect::new(20.0, 50.0, self.width - 40.0, 26.0);
            surface.fill_rounded_rect(attach_bar, 8.0, SURFACE_ELEVATED)?;
            let summary = format!("{} shelf attachment(s)", self.attachments.len());
            surface.draw_text(
                &summary,
                Rect::new(30.0, 50.0, self.width - 150.0, 26.0),
                11.5,
                ACCENT_CYAN,
                TextAlign::Leading,
            )?;
            surface.draw_text(
                "Clear",
                self.clear_attachments_rect(),
                11.5,
                TEXT_SECONDARY,
                TextAlign::Center,
            )?;
        }

        // Recent messages
        let mut msg_y = if self.attachments.is_empty() { 54.0 } else { 86.0 };
        let max_msg_y = 330.0;
        for msg in self.messages.iter().rev().take(5).rev() {
            if msg_y + 44.0 > max_msg_y {
                break;
            }
            let bubble = Rect::new(20.0, msg_y, self.width - 40.0, 44.0);
            surface.fill_rounded_rect(bubble, 10.0, SURFACE_ELEVATED)?;
            let prefix = match msg.role {
                ChatRole::User => "You: ",
                ChatRole::Assistant => {
                    if msg.streaming {
                        "Hime (typing...): "
                    } else {
                        "Hime: "
                    }
                }
                ChatRole::System => "System: ",
            };
            let line = format!("{prefix}{}", msg.content);
            let color = match msg.role {
                ChatRole::User => TEXT_PRIMARY,
                ChatRole::Assistant => ACCENT_CYAN,
                ChatRole::System => TEXT_SECONDARY,
            };
            surface.draw_text(
                &line,
                Rect::new(30.0, msg_y, self.width - 60.0, 44.0),
                12.0,
                color,
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
            surface.fill_rounded_rect(rect, 12.0, SURFACE_ELEVATED)?;
            surface.stroke_rounded_rect(rect, 12.0, STATUS_WARNING, 1.0)?;
            surface.draw_text(
                &format!("Approve {}: {}", card.tool_name, card.summary),
                Rect::new(30.0, card_y + 6.0, self.width - 60.0, 24.0),
                12.0,
                TEXT_PRIMARY,
                TextAlign::Leading,
            )?;

            let approve = Rect::new(self.width - 172.0, card_y + 36.0, 72.0, 26.0);
            let deny = Rect::new(self.width - 92.0, card_y + 36.0, 72.0, 26.0);
            surface.fill_rounded_rect(approve, 6.0, STATUS_SUCCESS)?;
            surface.draw_text("Approve", approve, 11.5, TEXT_PRIMARY, TextAlign::Center)?;
            surface.fill_rounded_rect(deny, 6.0, STATUS_ALERT)?;
            surface.draw_text("Deny", deny, 11.5, TEXT_PRIMARY, TextAlign::Center)?;
            card_y += 76.0;
        }

        // Input footer
        if panel_height >= 120.0 {
            let input_rect = Rect::new(20.0, self.height - 48.0, self.width - 116.0, 32.0);
            surface.fill_rounded_rect(input_rect, 10.0, SURFACE_ELEVATED)?;
            let prompt_display;
            let prompt_text = if self.draft_prompt.is_empty() {
                "Ask Hime AI... (type & press Enter)"
            } else {
                prompt_display = format!("{}|", self.draft_prompt);
                prompt_display.as_str()
            };
            let prompt_color = if self.draft_prompt.is_empty() {
                TEXT_SECONDARY
            } else {
                TEXT_PRIMARY
            };
            surface.draw_text(
                prompt_text,
                Rect::new(30.0, self.height - 48.0, self.width - 136.0, 32.0),
                12.0,
                prompt_color,
                TextAlign::Leading,
            )?;

            let send_rect = self.send_button_rect();
            surface.fill_rounded_rect(send_rect, 10.0, ACCENT_CYAN)?;
            surface.draw_text("Send", send_rect, 12.0, GLASS_SURFACE, TextAlign::Center)?;
        }

        Ok(())
    }

    fn render_translate_tab(&self, surface: &NativeSurface, panel_height: f32) -> Result<()> {
        if panel_height < 140.0 {
            return Ok(());
        }

        surface.draw_text(
            "Selection Translate",
            Rect::new(20.0, 56.0, 220.0, 26.0),
            13.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        let lang_badge = Rect::new(self.width - 130.0, 56.0, 110.0, 24.0);
        surface.fill_rounded_rect(lang_badge, 8.0, ACCENT_CYAN.with_alpha(0.16))?;
        surface.stroke_rounded_rect(lang_badge, 8.0, ACCENT_CYAN.with_alpha(0.45), 1.0)?;
        surface.draw_text(
            &format!("Target: {}", self.translation.target_lang.to_uppercase()),
            lang_badge,
            11.0,
            ACCENT_CYAN,
            TextAlign::Center,
        )?;

        let source_card = Rect::new(20.0, 92.0, self.width - 40.0, 112.0);
        surface.fill_rounded_rect(source_card, 12.0, SURFACE_ELEVATED)?;
        surface.stroke_rounded_rect(source_card, 12.0, GLASS_BORDER, 1.0)?;
        surface.draw_text(
            &format!("SOURCE ({})", self.translation.source_lang.to_uppercase()),
            Rect::new(32.0, 100.0, self.width - 64.0, 18.0),
            10.5,
            TEXT_SECONDARY,
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
        surface.fill_rounded_rect(result_card, 12.0, SURFACE_ELEVATED)?;
        surface.stroke_rounded_rect(result_card, 12.0, ACCENT_CYAN.with_alpha(0.35), 1.0)?;
        let header_label = if self.translation.cached {
            "TRANSLATION • CACHED"
        } else {
            "TRANSLATION"
        };
        surface.draw_text(
            header_label,
            Rect::new(32.0, 224.0, self.width - 64.0, 18.0),
            10.5,
            ACCENT_CYAN,
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

        let clear_rect = self.translate_clear_rect();
        surface.fill_rounded_rect(clear_rect, 8.0, SURFACE_ELEVATED)?;
        surface.stroke_rounded_rect(clear_rect, 8.0, GLASS_BORDER, 1.0)?;
        surface.draw_text("Clear", clear_rect, 11.5, TEXT_SECONDARY, TextAlign::Center)?;

        let copy_rect = self.translate_copy_rect();
        let has_result = !self.translation.translated_text.is_empty();
        surface.fill_rounded_rect(
            copy_rect,
            8.0,
            if has_result {
                ACCENT_CYAN.with_alpha(0.22)
            } else {
                STATUS_OFFLINE.with_alpha(0.22)
            },
        )?;
        surface.stroke_rounded_rect(
            copy_rect,
            8.0,
            if has_result {
                ACCENT_CYAN.with_alpha(0.6)
            } else {
                GLASS_BORDER
            },
            1.0,
        )?;
        surface.draw_text("Copy", copy_rect, 11.5, TEXT_PRIMARY, TextAlign::Center)?;

        if let Some(notice) = &self.translation.status_notice {
            surface.draw_text(
                notice,
                Rect::new(20.0, 434.0, self.width - 40.0, 22.0),
                11.0,
                STATUS_SUCCESS,
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
        let dev_card = Rect::new(20.0, 56.0, self.width - 40.0, 194.0);
        surface.fill_rounded_rect(dev_card, 12.0, SURFACE_ELEVATED)?;
        surface.stroke_rounded_rect(dev_card, 12.0, GLASS_BORDER, 1.0)?;
        surface.draw_text(
            "Developer",
            Rect::new(34.0, 68.0, 180.0, 22.0),
            13.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        let branch_pill = Rect::new(self.width - 174.0, 66.0, 140.0, 24.0);
        surface.fill_rounded_rect(branch_pill, 6.0, ACCENT_CYAN.with_alpha(0.16))?;
        surface.draw_text(
            &self.developer.branch,
            branch_pill,
            11.0,
            ACCENT_CYAN,
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
            STATUS_SUCCESS
        } else {
            STATUS_ALERT
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

        let dev_btn = self.dev_check_rect();
        surface.fill_rounded_rect(dev_btn, 8.0, ACCENT_CYAN.with_alpha(0.20))?;
        surface.stroke_rounded_rect(dev_btn, 8.0, ACCENT_CYAN.with_alpha(0.55), 1.0)?;
        surface.draw_text(
            "Run Dev Check",
            dev_btn,
            11.5,
            TEXT_PRIMARY,
            TextAlign::Center,
        )?;

        // Documents card
        let docs_card = Rect::new(20.0, 266.0, self.width - 40.0, 174.0);
        surface.fill_rounded_rect(docs_card, 12.0, SURFACE_ELEVATED)?;
        surface.stroke_rounded_rect(docs_card, 12.0, GLASS_BORDER, 1.0)?;
        surface.draw_text(
            "Documents",
            Rect::new(34.0, 278.0, 180.0, 22.0),
            13.5,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        let total_jobs =
            self.documents.active_jobs + self.documents.completed_jobs + self.documents.failed_jobs;
        let jobs_pill = Rect::new(self.width - 144.0, 276.0, 110.0, 24.0);
        surface.fill_rounded_rect(jobs_pill, 6.0, STATUS_SUCCESS.with_alpha(0.16))?;
        surface.draw_text(
            &format!("{total_jobs} job(s)"),
            jobs_pill,
            11.0,
            STATUS_SUCCESS,
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

        let docs_btn = self.refresh_docs_rect();
        surface.fill_rounded_rect(docs_btn, 8.0, STATUS_SUCCESS.with_alpha(0.20))?;
        surface.stroke_rounded_rect(docs_btn, 8.0, STATUS_SUCCESS.with_alpha(0.55), 1.0)?;
        surface.draw_text(
            "Refresh Docs",
            docs_btn,
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
}
