//! Hime Integration Root Module for Bloom Shell
//!
//! Exposes the IPC bridge, background process supervisor, and Tauri commands.

pub mod client;
pub mod commands;
pub mod services;

use std::sync::Arc;
use tauri::AppHandle;

pub use commands::*;
pub use services::{ApprovalItem, CoreConnectionState, HimeBridgeService, HimeCoreStatus};

/// Initializes the Hime Bridge service and starts the background monitoring loop.
pub fn init(app: &AppHandle) -> Arc<HimeBridgeService> {
    let service = HimeBridgeService::new(app.clone());
    service.clone().start_supervisor_loop();
    service
}
