//! Apple Liquid Glass monochrome & semantic color tokens for Hime native shell.
//!
//! Follows Apple Human Interface Guidelines (Liquid Glass + Color):
//! - Obsidian smoked glass surfaces (`GLASS_REGULAR`, `GLASS_CLEAR`, `GLASS_ELEVATED`)
//! - Monochrome label hierarchy (`TEXT_PRIMARY`, `TEXT_SECONDARY`, `TEXT_TERTIARY`, `TEXT_QUATERNARY`)
//! - Subtle specular rims & hairline dividers (`GLASS_HAIRLINE`, `DIVIDER_HAIRLINE`)
//! - Single restrained system accent (`SYSTEM_BLUE` `#0A84FF`) reserved for active states & selected tabs
//! - Semantic live status indicators (`STATUS_LIVE_GREEN`, `STATUS_LIVE_RED`, `STATUS_LIVE_AMBER`)

use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

/// RGBA color represented in sRGB normalized `[0.0, 1.0]` floats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Creates a color from normalized `[0.0, 1.0]` components.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates an opaque color from `0..=255` RGB channels.
    pub const fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

    /// Creates a color from `0..=255` RGBA channels.
    pub const fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Parses a `#RRGGBB` or `#RRGGBBAA` hex string at runtime.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let s = hex.strip_prefix('#').unwrap_or(hex);
        match s.len() {
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                Some(Self::from_rgb_u8(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                let a = u8::from_str_radix(&s[6..8], 16).ok()?;
                Some(Self::from_rgba_u8(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Returns a copy of this color with a different alpha channel.
    pub const fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    /// Returns a copy of this color with its alpha scaled by `factor` (clamped to `0.0..=1.0`).
    pub fn with_alpha_mul(self, factor: f32) -> Self {
        Self {
            a: (self.a * factor).clamp(0.0, 1.0),
            ..self
        }
    }

    /// Linearly interpolates between `self` and `to` (`t` clamped to `0.0..=1.0`).
    pub fn lerp(self, to: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let inv = 1.0 - t;
        Self {
            r: self.r * inv + to.r * t,
            g: self.g * inv + to.g * t,
            b: self.b * inv + to.b * t,
            a: self.a * inv + to.a * t,
        }
    }

    /// Converts to Direct2D's `D2D1_COLOR_F`.
    pub const fn to_d2d(self) -> D2D1_COLOR_F {
        D2D1_COLOR_F {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

// ---------------------------------------------------------------------------
// Apple Liquid Glass Materials (Obsidian Dark Mode)
// ---------------------------------------------------------------------------

/// `Glass.regular`: obsidian smoked glass for panels, menus, settings, and text-heavy surfaces (~77% alpha).
pub const GLASS_REGULAR: Color = Color::from_rgba_u8(0x14, 0x14, 0x16, 0xC4);

/// `Glass.clear`: lighter translucent smoked glass for floating pills and media/HUD controls (~62% alpha).
pub const GLASS_CLEAR: Color = Color::from_rgba_u8(0x10, 0x10, 0x12, 0x9E);

/// Elevated inner card / grouped section surface (`#202024` at ~82% alpha).
pub const GLASS_ELEVATED: Color = Color::from_rgba_u8(0x20, 0x20, 0x24, 0xD1);

/// Subtle translucent control fill (`rgba(255, 255, 255, 0.08)`).
pub const GLASS_CONTROL: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x14);

/// Hovered translucent control fill (`rgba(255, 255, 255, 0.14)`).
pub const GLASS_CONTROL_HOVER: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x24);

/// Pressed / active monochrome control fill (`rgba(255, 255, 255, 0.20)`).
pub const GLASS_CONTROL_ACTIVE: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x33);

/// Top specular rim highlight on Liquid Glass bevels (`rgba(255, 255, 255, 0.26)`).
pub const GLASS_SPECULAR_TOP: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x42);

/// Opposite-edge specular catchlight (`rgba(255, 255, 255, 0.08)`).
pub const GLASS_SPECULAR_BOTTOM: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x14);

/// Soft inner shadow rim inside the glass border (`rgba(0, 0, 0, 0.24)`).
pub const GLASS_INNER_SHADE: Color = Color::from_rgba_u8(0x00, 0x00, 0x00, 0x3D);

/// Crisp 1px Liquid Glass hairline border (`rgba(255, 255, 255, 0.16)`).
pub const GLASS_HAIRLINE: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x29);

/// Soft outer drop shadow base (`rgba(0, 0, 0, 0.30)`).
pub const GLASS_SHADOW: Color = Color::from_rgba_u8(0x00, 0x00, 0x00, 0x4D);

/// Subtle section & toolbar hairline divider (`rgba(255, 255, 255, 0.10)`).
pub const DIVIDER_HAIRLINE: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x1A);

/// Subtle slider track background (`rgba(255, 255, 255, 0.14)`).
pub const SLIDER_TRACK: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x24);

/// Crisp monochrome slider active fill (`rgba(255, 255, 255, 0.92)`).
pub const SLIDER_FILL: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0xEB);

// ---------------------------------------------------------------------------
// Monochrome Apple HIG Label Hierarchy
// ---------------------------------------------------------------------------

/// Primary label color (`#FFFFFF`).
pub const TEXT_PRIMARY: Color = Color::from_rgb_u8(0xFF, 0xFF, 0xFF);

/// Secondary label color (`rgba(235, 235, 245, 0.62)` — Apple `secondaryLabel` dark).
pub const TEXT_SECONDARY: Color = Color::from_rgba_u8(0xEB, 0xEB, 0xF5, 0x9E);

/// Tertiary label color (`rgba(235, 235, 245, 0.34)` — Apple `tertiaryLabel` dark).
pub const TEXT_TERTIARY: Color = Color::from_rgba_u8(0xEB, 0xEB, 0xF5, 0x57);

/// Quaternary label color (`rgba(235, 235, 245, 0.18)` — Apple `quaternaryLabel` dark).
pub const TEXT_QUATERNARY: Color = Color::from_rgba_u8(0xEB, 0xEB, 0xF5, 0x2E);

// ---------------------------------------------------------------------------
// Single System Accent + Semantic Live Indicators
// ---------------------------------------------------------------------------

/// Apple dark-mode `systemBlue` (`#0A84FF`) — reserved for active states & selected tabs.
pub const SYSTEM_BLUE: Color = Color::from_rgb_u8(0x0A, 0x84, 0xFF);

/// Soft Apple blue glass tint (`#0A84FF` at 24% opacity) for selected controls/tabs.
pub const SYSTEM_BLUE_TINT: Color = Color::from_rgba_u8(0x0A, 0x84, 0xFF, 0x3D);

/// Live healthy / active / charging status indicator (`#30D158`, Apple `systemGreen` dark).
pub const STATUS_LIVE_GREEN: Color = Color::from_rgb_u8(0x30, 0xD1, 0x58);

/// Live critical / recording / destructive status indicator (`#FF453A`, Apple `systemRed` dark).
pub const STATUS_LIVE_RED: Color = Color::from_rgb_u8(0xFF, 0x45, 0x3A);

/// Live caution / warning status indicator (`#FF9F0A`, Apple `systemOrange` dark).
pub const STATUS_LIVE_AMBER: Color = Color::from_rgb_u8(0xFF, 0x9F, 0x0A);

/// Muted / offline / inactive indicator (`#8E8E93`, Apple `systemGray`).
pub const STATUS_MUTED: Color = Color::from_rgb_u8(0x8E, 0x8E, 0x93);

// ---------------------------------------------------------------------------
// Backwards-Compatible Token Aliases (mapped cleanly to the Apple HIG palette)
// ---------------------------------------------------------------------------

/// Obsidian smoked notch surface (`#0D0D0F` at ~86% opacity).
pub const NOTCH_SURFACE: Color = Color::from_rgba_u8(0x0D, 0x0D, 0x0F, 0xDB);

/// Regular smoked Liquid Glass surface alias.
pub const NOTCH_BG_SOLID: Color = GLASS_REGULAR;

/// Regular smoked Liquid Glass surface alias.
pub const GLASS_SURFACE: Color = GLASS_REGULAR;

/// Floating Dock Liquid Glass surface alias.
pub const DOCK_SURFACE: Color = GLASS_REGULAR;

/// Elevated card/chip fill alias.
pub const SURFACE_ELEVATED: Color = GLASS_ELEVATED;

/// Subtle 1px hairline glass border alias.
pub const GLASS_BORDER: Color = GLASS_HAIRLINE;

/// Active system tint alias (`#0A84FF`).
pub const EDGE_ACCENT: Color = SYSTEM_BLUE;

/// Live alert edge accent alias (`#FF453A`).
pub const EDGE_ACCENT_RED: Color = STATUS_LIVE_RED;

/// System accent alias (`#0A84FF`, replacing legacy cyan).
pub const ACCENT_CYAN: Color = SYSTEM_BLUE;

/// System accent soft glow alias.
pub const ACCENT_GLOW: Color = SYSTEM_BLUE_TINT;

/// Semantic amber status alias.
pub const STATUS_WARNING: Color = STATUS_LIVE_AMBER;

/// Semantic red alert status alias.
pub const STATUS_ALERT: Color = STATUS_LIVE_RED;

/// Neutral muted/offline status alias.
pub const STATUS_OFFLINE: Color = STATUS_MUTED;

/// Semantic green live status alias.
pub const STATUS_SUCCESS: Color = STATUS_LIVE_GREEN;

/// Fully transparent color for clearing premultiplied 32-bit layered surfaces.
pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

/// Default notch corner radius in DIPs.
pub const CORNER_RADIUS: f32 = 18.0;

/// Safe inner padding in DIPs for expanded surfaces.
pub const SAFE_PADDING: f32 = 24.0;

/// Default UI font family on Windows 11/10.
pub const FONT_FAMILY: &str = "Segoe UI";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parsing_matches_tokens() {
        assert_eq!(Color::from_hex("#FFFFFF"), Some(TEXT_PRIMARY));
        assert_eq!(Color::from_hex("#0A84FF"), Some(SYSTEM_BLUE));
        assert_eq!(Color::from_hex("#30D158"), Some(STATUS_LIVE_GREEN));
        assert_eq!(Color::from_hex("#FF453A"), Some(STATUS_LIVE_RED));
        assert_eq!(Color::from_hex("invalid"), None);
    }

    #[test]
    fn color_lerp_and_alpha_mul_work_cleanly() {
        let mid = Color::from_rgb_u8(0, 0, 0).lerp(TEXT_PRIMARY, 0.5);
        assert!((mid.r - 0.5).abs() < 1e-4 && (mid.a - 1.0).abs() < 1e-4);
        assert!((TEXT_PRIMARY.with_alpha_mul(0.25).a - 0.25).abs() < 1e-4);
    }

    #[test]
    fn geometry_constants_match_spec() {
        assert_eq!(CORNER_RADIUS, 18.0);
        assert_eq!(SAFE_PADDING, 24.0);
    }
}
