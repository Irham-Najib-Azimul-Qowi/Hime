//! Hime Tauri Commands Surface
//!
//! Exposes IPC commands (`hime_*`) invoked by the frontend React components.

use super::services::{ApprovalItem, HimeBridgeService, HimeCoreStatus};

#[tauri::command]
pub fn hime_core_status() -> HimeCoreStatus {
    match HimeBridgeService::global() {
        Some(service) => service.get_status(),
        None => HimeCoreStatus {
            connection_state: super::services::CoreConnectionState::Offline,
            is_online: false,
            core_version: None,
            uptime_secs: 0,
            active_approvals_count: 0,
            last_ping_ms: None,
        },
    }
}

#[tauri::command]
pub fn hime_ping() -> Result<u64, String> {
    match HimeBridgeService::global() {
        Some(service) => service.ping_core(),
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub fn hime_approval_list() -> Vec<ApprovalItem> {
    match HimeBridgeService::global() {
        Some(service) => service.list_approvals(),
        None => Vec::new(),
    }
}

#[tauri::command]
pub fn hime_approval_resolve(request_id: String, decision: String) -> Result<String, String> {
    match HimeBridgeService::global() {
        Some(service) => service.resolve_approval(&request_id, &decision),
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub fn hime_dev_trigger_approval(
    action_type: String,
    path: String,
    description: String,
) -> Result<ApprovalItem, String> {
    match HimeBridgeService::global() {
        Some(service) => service.dev_trigger_test_approval(&action_type, &path, &description),
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_send_chat(
    prompt: String,
    model: Option<String>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.send_chat(&prompt, model.as_deref())
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_store_credential(
    provider: String,
    secret: String,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.store_credential(&provider, &secret)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_poll_new_downloads() -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.poll_new_downloads()
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_inspect_download(download_path: String) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.inspect_download(&download_path)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_replace_and_archive_download(
    new_download: String,
    existing_file: String,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.replace_and_archive_download(&new_download, &existing_file)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_create_academic_report(
    title: String,
    markdown_body: String,
    destination_path: Option<String>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.create_academic_report(&title, &markdown_body, destination_path.as_deref())
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_merge_pdfs(
    source_paths: Vec<String>,
    destination_path: String,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.merge_pdfs(&source_paths, &destination_path)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}
