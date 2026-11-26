//! Hime Island Native UI Shell (`hime-island-native`).
//!
//! Direct2D + DirectWrite + DirectComposition rendering foundation for
//! ADR-017 Phase 3, replacing WebView2 with a pure-Rust GPU-composited shell.

pub mod ai_drawer;
pub mod animation;
pub mod d2d;
pub mod dock;
pub mod hotkey;
pub mod ipc_client;
pub mod module_mgr;
pub mod notch;
pub mod os_shell;
pub mod overlay;
pub mod settings_ui;
pub mod shelf;
pub mod telemetry;
pub mod theme;

pub use animation::SpringValue;
pub use d2d::{HimeExpression, NativeSurface, Rect, TextAlign};
pub use theme::Color;
