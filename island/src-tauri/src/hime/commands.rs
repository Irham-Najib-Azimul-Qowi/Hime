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

#[tauri::command]
pub async fn hime_lyrics_get(
    title: String,
    artist: String,
    album: Option<String>,
    duration_sec: Option<u32>,
    allow_online: Option<bool>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.get_lyrics(
                    &title,
                    &artist,
                    album.as_deref(),
                    duration_sec,
                    allow_online.unwrap_or(true),
                )
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_lyrics_set_offset(
    cache_key: String,
    offset_ms: i32,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.set_lyrics_offset(&cache_key, offset_ms)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_lyrics_clear_cache() -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.clear_lyrics_cache()
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

// --- Tasks / Productivity ---

#[tauri::command]
pub async fn hime_list_tasks(board_id: Option<String>) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.list_tasks(board_id.as_deref())
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_create_task(
    title: String,
    priority: Option<String>,
    due_date: Option<i64>,
    labels: Option<String>,
    description: Option<String>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.create_task(
                    &title,
                    priority.as_deref().unwrap_or("medium"),
                    due_date,
                    labels.as_deref(),
                    description.as_deref(),
                )
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_update_task(
    task_id: String,
    title: Option<String>,
    priority: Option<String>,
    status: Option<String>,
    due_date: Option<i64>,
    labels: Option<String>,
    description: Option<String>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.update_task(
                    &task_id,
                    title.as_deref(),
                    priority.as_deref(),
                    status.as_deref(),
                    due_date,
                    labels.as_deref(),
                    description.as_deref(),
                )
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_delete_task(task_id: String) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.delete_task(&task_id)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_parse_task_natural(text: String) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.parse_task_natural(&text)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

// --- Translation ---

#[tauri::command]
pub async fn hime_translate_text(
    text: String,
    target_lang: Option<String>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.translate_text(&text, target_lang.as_deref().unwrap_or("id"))
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_translate_selection(
    target_lang: Option<String>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.translate_selection(target_lang.as_deref().unwrap_or("id"))
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_get_translate_opt_in() -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.get_translate_opt_in()
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_set_translate_opt_in(enabled: bool) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.set_translate_opt_in(enabled)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

// --- Stage U3: Routines, Commitments, Enforcement & Screen-Time ---

#[tauri::command]
pub async fn hime_list_routines(active_only: Option<bool>) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.list_routines(active_only.unwrap_or(false))
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_create_routine(routine: serde_json::Value) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.create_routine(routine)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_update_routine(routine: serde_json::Value) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.update_routine(routine)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_delete_routine(id: String) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.delete_routine(&id)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_start_routine_session(
    routine_id: String,
    scheduled_for: Option<i64>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.start_routine_session(&routine_id, scheduled_for)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_complete_routine_session(session_id: String) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.complete_routine_session(&session_id)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_skip_routine_session(
    session_id: String,
    reason: String,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.skip_routine_session(&session_id, &reason)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_snooze_routine_session(
    session_id: String,
    minutes: Option<u32>,
    reason: Option<String>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.snooze_routine_session(&session_id, minutes.unwrap_or(10), reason.as_deref())
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_list_commitments(active_only: Option<bool>) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.list_commitments(active_only.unwrap_or(false))
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_create_commitment(commitment: serde_json::Value) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.create_commitment(commitment)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_request_commitment_weakening(
    id: String,
    delay_seconds: Option<i64>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.request_commitment_weakening(&id, delay_seconds)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_trigger_emergency_pause(
    duration_min: i64,
    reason: String,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.trigger_emergency_pause(duration_min, &reason)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_get_emergency_pause_status() -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.get_emergency_pause_status()
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_get_enforcement_history(limit: Option<usize>) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.get_enforcement_history(limit.unwrap_or(50))
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_record_screentime(
    app_name: String,
    window_title: Option<String>,
    category: String,
    duration_secs: i64,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.record_screentime(&app_name, window_title.as_deref(), &category, duration_secs)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_list_screentime(limit: Option<usize>) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.list_screentime(limit.unwrap_or(50))
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_clear_screentime() -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.clear_screentime()
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_get_screentime_settings() -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.get_screentime_settings()
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_set_screentime_opt_in(opt_in: bool) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.set_screentime_opt_in(opt_in)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

// --- Proactive & Interaction Engine Commands ---

#[tauri::command]
pub async fn hime_get_proactive_settings() -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.get_proactive_settings()
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_set_proactive_settings(
    is_enabled: bool,
    cooldown_minutes: i64,
    mute_hours: Option<i64>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.set_proactive_settings(is_enabled, cooldown_minutes, mute_hours)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_list_proactive_suggestions(
    status: Option<String>,
    limit: Option<usize>,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.list_proactive_suggestions(status.as_deref(), limit.unwrap_or(20))
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_respond_proactive_suggestion(
    suggestion_id: String,
    action: String,
) -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.respond_proactive_suggestion(&suggestion_id, &action)
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn hime_evaluate_proactive_suggestions() -> Result<serde_json::Value, String> {
    match HimeBridgeService::global() {
        Some(service) => {
            tauri::async_runtime::spawn_blocking(move || {
                service.evaluate_proactive_suggestions()
            })
            .await
            .map_err(|e| format!("Task execution error: {}", e))?
        }
        None => Err("Hime Bridge service is not initialized".to_string()),
    }
}
