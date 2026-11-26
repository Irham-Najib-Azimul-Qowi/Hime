//! Native AI Chat & Approval Drawer rendered with Direct2D (`NativeSurface`).
//! Supports shelf file attachments, streaming assistant responses, and tool approval cards.

use serde::{Deserialize, Serialize};
use windows::core::Result;

use crate::animation::SpringValue;
use crate::d2d::{NativeSurface, Rect, TextAlign};
use crate::theme::{
    ACCENT_CYAN, GLASS_BORDER, GLASS_SURFACE, STATUS_ALERT, STATUS_SUCCESS, STATUS_WARNING,
    SURFACE_ELEVATED, TEXT_PRIMARY, TEXT_SECONDARY,
};

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
    ClearAttachments,
    SendPrompt,
    Approve(String),
    Deny(String),
}

/// Native AI Chat & Approval Drawer state + renderer.
pub struct AiDrawer {
    pub visible: bool,
    pub attachments: Vec<DrawerAttachment>,
    pub messages: Vec<ChatMessage>,
    pub approvals: Vec<ApprovalCard>,
    pub draft_prompt: String,
    open_spring: SpringValue,
    width: f32,
    height: f32,
}

impl AiDrawer {
    pub fn new() -> Self {
        Self {
            visible: false,
            attachments: Vec::new(),
            messages: Vec::new(),
            approvals: Vec::new(),
            draft_prompt: String::new(),
            open_spring: SpringValue::snappy(0.0),
            width: 440.0,
            height: 500.0,
        }
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
        if !self.visible {
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
        // Close button
        if x >= self.width - 42.0 && x <= self.width - 14.0 && y >= 14.0 && y <= 42.0 {
            return Some(DrawerAction::Close);
        }
        // Clear attachments chip
        if !self.attachments.is_empty()
            && x >= self.width - 110.0
            && x <= self.width - 20.0
            && y >= 50.0
            && y <= 76.0
        {
            return Some(DrawerAction::ClearAttachments);
        }
        // Approval buttons
        for (id, approve, deny) in self.approval_button_rects() {
            if point_in_rect(x, y, approve) {
                return Some(DrawerAction::Approve(id));
            }
            if point_in_rect(x, y, deny) {
                return Some(DrawerAction::Deny(id));
            }
        }
        // Send button
        let send = Rect::new(self.width - 88.0, self.height - 48.0, 68.0, 32.0);
        if point_in_rect(x, y, send) {
            return Some(DrawerAction::SendPrompt);
        }
        None
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
            "Hime AI Drawer",
            Rect::new(20.0, 14.0, 220.0, 28.0),
            15.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;
        surface.draw_text(
            "X",
            Rect::new(self.width - 42.0, 14.0, 28.0, 28.0),
            13.0,
            TEXT_SECONDARY,
            TextAlign::Center,
        )?;

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
                Rect::new(self.width - 110.0, 50.0, 90.0, 26.0),
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
            if card_y + 68.0 > panel.height - 56.0 {
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
        if panel.height >= 120.0 {
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

            let send_rect = Rect::new(self.width - 88.0, self.height - 48.0, 68.0, 32.0);
            surface.fill_rounded_rect(send_rect, 10.0, ACCENT_CYAN)?;
            surface.draw_text("Send", send_rect, 12.0, GLASS_SURFACE, TextAlign::Center)?;
        }

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
}
