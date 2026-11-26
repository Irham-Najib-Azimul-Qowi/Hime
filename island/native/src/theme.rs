//! Apple Liquid Glass adaptive monochrome & semantic color tokens for Hime native shell.
//!
//! Follows Apple Human Interface Guidelines (Liquid Glass + Color) & AntiSlop contrast rules:
//! - Adaptive polarity (`ThemePolarity::LightGlass` over dark backdrops, `DarkGlass` over light backdrops)
//! - `ColorThemeMode::{Auto, Dark, Light}`
//! - Monochrome label hierarchy (`TEXT_PRIMARY`..`TEXT_QUATERNARY` >= 4.5:1 contrast)
//! - Subtle hairline dividers & optical 1px rims (zero Windows 7 glossy diagonal gradients)
//! - Single restrained system accent (`SYSTEM_BLUE`) + semantic live indicators (`STATUS_LIVE_*`)

use serde::{Deserialize, Serialize};
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

/// User-selectable color theme mode (`auto` adapts dynamically to the backdrop behind each surface).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorThemeMode {
    #[default]
    Auto,
    Dark,
    Light,
}

impl ColorThemeMode {
    /// Parses `"auto"`, `"dark"`, or `"light"` case-insensitively, defaulting to `Auto`.
    pub fn from_str_lossy(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "dark" => Self::Dark,
            "light" => Self::Light,
            _ => Self::Auto,
        }
    }

    /// Canonical lowercase serialization token.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

/// Resolved Liquid Glass polarity for a surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ThemePolarity {
    /// Dark smoked charcoal glass (`#141418`) with white typography (over light/white backdrops).
    #[default]
    DarkGlass,
    /// Light frosted white glass (`#F3F5F8`) with near-black `#111214` typography (over dark/black backdrops).
    LightGlass,
}

/// Luminance threshold below which `ColorThemeMode::Auto` flips to `LightGlass`
/// so dark/black desktops receive crisp frosted-white Apple glass.
pub const AUTO_LUMINANCE_THRESHOLD: f32 = 0.45;

/// Complete adaptive monochrome + semantic palette resolved for a [`ThemePolarity`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdaptivePalette {
    pub polarity: ThemePolarity,
    pub glass_regular: Color,
    pub glass_clear: Color,
    pub glass_elevated: Color,
    pub control_fill: Color,
    pub control_hover: Color,
    pub control_active: Color,
    pub hairline_border: Color,
    pub inner_rim: Color,
    pub divider: Color,
    pub slider_track: Color,
    pub slider_fill: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_quaternary: Color,
    pub accent: Color,
    pub accent_tint: Color,
    pub status_green: Color,
    pub status_red: Color,
    pub status_amber: Color,
    pub status_muted: Color,
    pub shadow_color: Color,
}

impl AdaptivePalette {
    /// Dark smoked Liquid Glass palette (crisp white labels, used over bright/light backdrops or `Dark` mode).
    pub const fn dark_glass() -> Self {
        Self {
            polarity: ThemePolarity::DarkGlass,
            glass_regular: GLASS_REGULAR,
            glass_clear: GLASS_CLEAR,
            glass_elevated: GLASS_ELEVATED,
            control_fill: GLASS_CONTROL,
            control_hover: GLASS_CONTROL_HOVER,
            control_active: GLASS_CONTROL_ACTIVE,
            hairline_border: GLASS_HAIRLINE,
            inner_rim: Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x38),
            divider: DIVIDER_HAIRLINE,
            slider_track: SLIDER_TRACK,
            slider_fill: SLIDER_FILL,
            text_primary: TEXT_PRIMARY,
            text_secondary: Color::from_rgba_u8(0xEB, 0xEB, 0xF5, 0xB8),
            text_tertiary: Color::from_rgba_u8(0xEB, 0xEB, 0xF5, 0x80),
            text_quaternary: TEXT_QUATERNARY,
            accent: SYSTEM_BLUE,
            accent_tint: SYSTEM_BLUE_TINT,
            status_green: STATUS_LIVE_GREEN,
            status_red: STATUS_LIVE_RED,
            status_amber: STATUS_LIVE_AMBER,
            status_muted: STATUS_MUTED,
            shadow_color: GLASS_SHADOW,
        }
    }

    /// Light frosted white Liquid Glass palette (near-black `#111214` labels, used over black/dark backdrops or `Light` mode).
    pub const fn light_glass() -> Self {
        Self {
            polarity: ThemePolarity::LightGlass,
            glass_regular: LIGHT_GLASS_REGULAR,
            glass_clear: LIGHT_GLASS_CLEAR,
            glass_elevated: LIGHT_GLASS_ELEVATED,
            control_fill: Color::from_rgba_u8(0x00, 0x00, 0x00, 0x10),
            control_hover: Color::from_rgba_u8(0x00, 0x00, 0x00, 0x1B),
            control_active: Color::from_rgba_u8(0x00, 0x00, 0x00, 0x26),
            hairline_border: LIGHT_GLASS_HAIRLINE,
            inner_rim: Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x8C),
            divider: LIGHT_DIVIDER_HAIRLINE,
            slider_track: Color::from_rgba_u8(0x00, 0x00, 0x00, 0x1E),
            slider_fill: LIGHT_TEXT_PRIMARY,
            text_primary: LIGHT_TEXT_PRIMARY,
            text_secondary: LIGHT_TEXT_SECONDARY,
            text_tertiary: LIGHT_TEXT_TERTIARY,
            text_quaternary: Color::from_rgba_u8(0x11, 0x12, 0x14, 0x45),
            accent: SYSTEM_BLUE_LIGHT,
            accent_tint: Color::from_rgba_u8(0x00, 0x7A, 0xFF, 0x29),
            status_green: Color::from_rgb_u8(0x1E, 0x9E, 0x44),
            status_red: Color::from_rgb_u8(0xD9, 0x2B, 0x26),
            status_amber: Color::from_rgb_u8(0xC9, 0x6F, 0x00),
            status_muted: Color::from_rgb_u8(0x6C, 0x6C, 0x70),
            shadow_color: Color::from_rgba_u8(0x00, 0x00, 0x00, 0x38),
        }
    }

    /// Returns the palette matching `polarity`.
    pub const fn for_polarity(polarity: ThemePolarity) -> Self {
        match polarity {
            ThemePolarity::DarkGlass => Self::dark_glass(),
            ThemePolarity::LightGlass => Self::light_glass(),
        }
    }

    /// Resolves polarity & palette from `mode` and optional sampled `backdrop_luminance` (`0.0..=1.0`).
    pub fn resolve(mode: ColorThemeMode, backdrop_luminance: Option<f32>) -> Self {
        let polarity = match mode {
            ColorThemeMode::Dark => ThemePolarity::DarkGlass,
            ColorThemeMode::Light => ThemePolarity::LightGlass,
            ColorThemeMode::Auto => match backdrop_luminance {
                Some(lum) if lum < AUTO_LUMINANCE_THRESHOLD => ThemePolarity::LightGlass,
                _ => ThemePolarity::DarkGlass,
            },
        };
        Self::for_polarity(polarity)
    }
}

// ---------------------------------------------------------------------------
// Apple Liquid Glass Materials (Dark Smoked & Light Frosted)
// ---------------------------------------------------------------------------

/// `Glass.regular` (Dark): obsidian smoked glass (~78% alpha).
pub const GLASS_REGULAR: Color = Color::from_rgba_u8(0x14, 0x14, 0x18, 0xC7);

/// `Glass.clear` (Dark): lighter translucent smoked glass (~64% alpha).
pub const GLASS_CLEAR: Color = Color::from_rgba_u8(0x10, 0x10, 0x14, 0xA3);

/// Elevated inner card / grouped section surface (Dark, ~82% alpha).
pub const GLASS_ELEVATED: Color = Color::from_rgba_u8(0x20, 0x20, 0x24, 0xD1);

/// `Glass.regular` (Light): frosted white glass (~82% alpha).
pub const LIGHT_GLASS_REGULAR: Color = Color::from_rgba_u8(0xF3, 0xF5, 0xF8, 0xD1);

/// `Glass.clear` (Light): lighter frosted white glass (~68% alpha).
pub const LIGHT_GLASS_CLEAR: Color = Color::from_rgba_u8(0xF7, 0xF8, 0xFB, 0xAD);

/// Elevated inner card surface (Light, ~86% alpha).
pub const LIGHT_GLASS_ELEVATED: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0xDB);

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

/// Crisp 1px Liquid Glass hairline border (Dark, `rgba(255, 255, 255, 0.18)`).
pub const GLASS_HAIRLINE: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x2E);

/// Crisp 1px Liquid Glass hairline border (Light, `rgba(0, 0, 0, 0.14)`).
pub const LIGHT_GLASS_HAIRLINE: Color = Color::from_rgba_u8(0x00, 0x00, 0x00, 0x24);

/// Soft outer drop shadow base (`rgba(0, 0, 0, 0.30)`).
pub const GLASS_SHADOW: Color = Color::from_rgba_u8(0x00, 0x00, 0x00, 0x4D);

/// Subtle section & toolbar hairline divider (Dark).
pub const DIVIDER_HAIRLINE: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x1F);

/// Subtle section & toolbar hairline divider (Light).
pub const LIGHT_DIVIDER_HAIRLINE: Color = Color::from_rgba_u8(0x00, 0x00, 0x00, 0x1A);

/// Subtle slider track background (Dark).
pub const SLIDER_TRACK: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0x28);

/// Crisp monochrome slider active fill (Dark).
pub const SLIDER_FILL: Color = Color::from_rgba_u8(0xFF, 0xFF, 0xFF, 0xEB);

// ---------------------------------------------------------------------------
// Monochrome Apple HIG Label Hierarchy
// ---------------------------------------------------------------------------

/// Primary label color (Dark Glass: `#FFFFFF`).
pub const TEXT_PRIMARY: Color = Color::from_rgb_u8(0xFF, 0xFF, 0xFF);

/// Secondary label color (Dark Glass, >= 4.5:1 contrast).
pub const TEXT_SECONDARY: Color = Color::from_rgba_u8(0xEB, 0xEB, 0xF5, 0xB8);

/// Tertiary label color (Dark Glass).
pub const TEXT_TERTIARY: Color = Color::from_rgba_u8(0xEB, 0xEB, 0xF5, 0x80);

/// Quaternary label color (Dark Glass).
pub const TEXT_QUATERNARY: Color = Color::from_rgba_u8(0xEB, 0xEB, 0xF5, 0x40);

/// Primary label color (Light Glass: near-black `#111214`).
pub const LIGHT_TEXT_PRIMARY: Color = Color::from_rgb_u8(0x11, 0x12, 0x14);

/// Secondary label color (Light Glass, >= 4.5:1 contrast on `#F3F5F8`).
pub const LIGHT_TEXT_SECONDARY: Color = Color::from_rgba_u8(0x11, 0x12, 0x14, 0xB8);

/// Tertiary label color (Light Glass).
pub const LIGHT_TEXT_TERTIARY: Color = Color::from_rgba_u8(0x11, 0x12, 0x14, 0x85);

// ---------------------------------------------------------------------------
// Single System Accent + Semantic Live Indicators
// ---------------------------------------------------------------------------

/// Apple dark-mode `systemBlue` (`#0A84FF`) — reserved for active states & selected tabs.
pub const SYSTEM_BLUE: Color = Color::from_rgb_u8(0x0A, 0x84, 0xFF);

/// Apple light-mode `systemBlue` (`#007AFF`) — high contrast on Light Frosted Glass.
pub const SYSTEM_BLUE_LIGHT: Color = Color::from_rgb_u8(0x00, 0x7A, 0xFF);

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
// Backwards-Compatible Token Aliases
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

/// System accent alias (`#0A84FF`).
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
        assert_eq!(Color::from_hex("#111214"), Some(LIGHT_TEXT_PRIMARY));
        assert_eq!(Color::from_hex("#30D158"), Some(STATUS_LIVE_GREEN));
        assert_eq!(Color::from_hex("#FF453A"), Some(STATUS_LIVE_RED));
        assert_eq!(Color::from_hex("invalid"), None);
    }

    #[test]
    fn adaptive_palette_flips_with_backdrop_luminance_and_mode() {
        // Dark/black backdrop under Auto -> Light Frosted White Glass with dark text.
        let on_black = AdaptivePalette::resolve(ColorThemeMode::Auto, Some(0.05));
        assert_eq!(on_black.polarity, ThemePolarity::LightGlass);
        assert_eq!(on_black.text_primary, LIGHT_TEXT_PRIMARY);

        // Bright/white backdrop under Auto -> Dark Smoked Glass with white text.
        let on_white = AdaptivePalette::resolve(ColorThemeMode::Auto, Some(0.92));
        assert_eq!(on_white.polarity, ThemePolarity::DarkGlass);
        assert_eq!(on_white.text_primary, TEXT_PRIMARY);

        // Explicit overrides win regardless of backdrop luminance.
        assert_eq!(
            AdaptivePalette::resolve(ColorThemeMode::Dark, Some(0.02)).polarity,
            ThemePolarity::DarkGlass
        );
        assert_eq!(
            AdaptivePalette::resolve(ColorThemeMode::Light, Some(0.95)).polarity,
            ThemePolarity::LightGlass
        );
        assert_eq!(ColorThemeMode::from_str_lossy("Light"), ColorThemeMode::Light);
        assert_eq!(ColorThemeMode::from_str_lossy("DARK"), ColorThemeMode::Dark);
        assert_eq!(ColorThemeMode::from_str_lossy("auto"), ColorThemeMode::Auto);
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
