//! Hime dark glassmorphism color & geometry tokens.

use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

/// RGBA color represented in linear-friendly normalized `[0.0, 1.0]` floats.
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

/// Primary notch surface background (`#0A0B0E`).
pub const NOTCH_SURFACE: Color = Color::from_rgb_u8(0x0A, 0x0B, 0x0E);

/// Translucent glassmorphic panel surface (`#0A0B0E` at ~92% opacity).
pub const GLASS_SURFACE: Color = Color::from_rgba_u8(0x0A, 0x0B, 0x0E, 0xEB);

/// Elevated card / chip surface inside expanded notch (`#14171F` at 90% opacity).
pub const SURFACE_ELEVATED: Color = Color::from_rgba_u8(0x14, 0x17, 0x1F, 0xE6);

/// Subtle glassmorphic border stroke (`#FFFFFF` at 12% opacity).
pub const GLASS_BORDER: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x1F);

/// Primary text color (`#FFFFFF`).
pub const TEXT_PRIMARY: Color = Color::from_rgb_u8(0xFF, 0xFF, 0xFF);

/// Secondary / muted text color (`#9CA3AF`).
pub const TEXT_SECONDARY: Color = Color::from_rgb_u8(0x9C, 0xA3, 0xAF);

/// Primary Hime cyan accent (`#00D2FF`).
pub const ACCENT_CYAN: Color = Color::from_rgb_u8(0x00, 0xD2, 0xFF);

/// Soft cyan glow for eye halos and active indicators (`#00D2FF` at 30% opacity).
pub const ACCENT_GLOW: Color = Color::from_rgba_u8(0x00, 0xD2, 0xFF, 0x4D);

/// Warning / worried state color (`#F59E0B`).
pub const STATUS_WARNING: Color = Color::from_rgb_u8(0xF5, 0x9E, 0x0B);

/// Alert / waspada state color (`#EF4444`).
pub const STATUS_ALERT: Color = Color::from_rgb_u8(0xEF, 0x44, 0x44);

/// Offline / muted state color (`#6B7280`).
pub const STATUS_OFFLINE: Color = Color::from_rgb_u8(0x6B, 0x72, 0x80);

/// Success / happy state color (`#10B981`).
pub const STATUS_SUCCESS: Color = Color::from_rgb_u8(0x10, 0xB9, 0x81);

/// Fully transparent color for clearing DirectComposition swap chains.
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
        assert_eq!(Color::from_hex("#0A0B0E"), Some(NOTCH_SURFACE));
        assert_eq!(Color::from_hex("#FFFFFF"), Some(TEXT_PRIMARY));
        assert_eq!(Color::from_hex("#00D2FF"), Some(ACCENT_CYAN));
        assert_eq!(Color::from_hex("invalid"), None);
    }

    #[test]
    fn geometry_constants_match_spec() {
        assert_eq!(CORNER_RADIUS, 18.0);
        assert_eq!(SAFE_PADDING, 24.0);
    }
}
