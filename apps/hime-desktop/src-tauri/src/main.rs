#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{command, LogicalSize, Size};
use tracing::info;

use hime_desktop::{
    cancel_conversation_logic, cancel_task_logic, create_conversation_logic, create_task_logic,
    execute_tool_logic, get_activity_events_logic, get_ai_models_logic, get_ai_status_logic,
    get_conversation_messages_logic, get_system_status_logic, get_tasks_logic,
    list_conversations_logic, list_credentials_logic, ping_core_logic, resolve_approval_logic,
    send_conversation_message_logic, storage_get_analysis_logic, storage_get_duplicates_logic,
    storage_get_operations_logic, storage_get_rules_logic, storage_restore_operation_logic,
    storage_scan_logic, storage_search_logic, storage_toggle_rule_logic, store_credential_logic,
    PingResult,
};
#[cfg(windows)]
use hime_ipc::transport::windows_pipe::DEFAULT_HIME_PIPE;

#[command]
async fn ping_core() -> Result<PingResult, String> {
    info!("Tauri command 'ping_core' invoked by frontend");
    #[cfg(windows)]
    {
        ping_core_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Windows Named Pipe IPC is only supported on Windows platform".to_string())
    }
}

#[command]
async fn execute_tool(action_type: String, path: String) -> Result<serde_json::Value, String> {
    info!(action = %action_type, path = %path, "Tauri command 'execute_tool' invoked by frontend");
    #[cfg(windows)]
    {
        execute_tool_logic(DEFAULT_HIME_PIPE, &action_type, &path).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn get_system_status() -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        get_system_status_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn get_activity_events(limit: Option<usize>) -> Result<Vec<serde_json::Value>, String> {
    let lim = limit.unwrap_or(30);
    #[cfg(windows)]
    {
        get_activity_events_logic(DEFAULT_HIME_PIPE, lim).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn create_task(
    name: String,
    description: String,
    priority: String,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        create_task_logic(DEFAULT_HIME_PIPE, &name, &description, &priority).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn get_tasks() -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        get_tasks_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn cancel_task(id: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        cancel_task_logic(DEFAULT_HIME_PIPE, &id).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn resolve_approval(
    request_id: String,
    decision: String,
) -> Result<serde_json::Value, String> {
    info!(
        request_id = %request_id,
        decision = %decision,
        "Tauri command 'resolve_approval' invoked by frontend"
    );
    #[cfg(windows)]
    {
        resolve_approval_logic(DEFAULT_HIME_PIPE, &request_id, &decision).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn window_set_always_on_top(
    window: tauri::Window,
    always_on_top: bool,
) -> Result<(), String> {
    window
        .set_always_on_top(always_on_top)
        .map_err(|e| e.to_string())
}

#[command]
async fn window_set_size_mode(window: tauri::Window, mode: String) -> Result<(), String> {
    match mode.as_str() {
        "compact" => {
            // Dynamic Island Floating HUD compact pill mode (460x88)
            window
                .set_size(Size::Logical(LogicalSize {
                    width: 460.0,
                    height: 88.0,
                }))
                .map_err(|e| e.to_string())?;
        }
        "expanded" => {
            // Main application desktop window (980x700)
            window
                .set_size(Size::Logical(LogicalSize {
                    width: 980.0,
                    height: 700.0,
                }))
                .map_err(|e| e.to_string())?;
        }
        _ => return Err(format!("Unknown mode: {}", mode)),
    }
    Ok(())
}

#[command]
async fn window_minimize(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[command]
async fn window_maximize(window: tauri::Window) -> Result<(), String> {
    if window.is_maximized().map_err(|e| e.to_string())? {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

#[command]
async fn window_close(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[command]
async fn ai_get_status() -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        get_ai_status_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn ai_get_models() -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        get_ai_models_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn credentials_store(provider: String, secret: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        store_credential_logic(DEFAULT_HIME_PIPE, &provider, &secret).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn credentials_list() -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        list_credentials_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn conversation_create(title: String, model: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        create_conversation_logic(DEFAULT_HIME_PIPE, &title, &model).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn conversation_list() -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        list_conversations_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn conversation_get_messages(conversation_id: String) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        get_conversation_messages_logic(DEFAULT_HIME_PIPE, &conversation_id).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn conversation_send_message(
    conversation_id: String,
    prompt: String,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        send_conversation_message_logic(DEFAULT_HIME_PIPE, &conversation_id, &prompt).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn conversation_cancel() -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        cancel_conversation_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn storage_get_analysis(top_n: Option<usize>) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        storage_get_analysis_logic(DEFAULT_HIME_PIPE, top_n).await
    }
    #[cfg(not(windows))]
    {
        let _ = top_n;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn storage_get_duplicates() -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        storage_get_duplicates_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn storage_get_operations(limit: Option<usize>) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        storage_get_operations_logic(DEFAULT_HIME_PIPE, limit).await
    }
    #[cfg(not(windows))]
    {
        let _ = limit;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn storage_restore_operation(original_op_id: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        storage_restore_operation_logic(DEFAULT_HIME_PIPE, &original_op_id).await
    }
    #[cfg(not(windows))]
    {
        let _ = original_op_id;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn storage_get_rules() -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        storage_get_rules_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn storage_toggle_rule(id: String, is_active: bool) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        storage_toggle_rule_logic(DEFAULT_HIME_PIPE, &id, is_active).await
    }
    #[cfg(not(windows))]
    {
        let _ = (id, is_active);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn storage_search(query: String, limit: Option<usize>) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        storage_search_logic(DEFAULT_HIME_PIPE, &query, limit).await
    }
    #[cfg(not(windows))]
    {
        let _ = (query, limit);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn storage_scan(root_path: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        storage_scan_logic(DEFAULT_HIME_PIPE, &root_path).await
    }
    #[cfg(not(windows))]
    {
        let _ = root_path;
        Err("Unsupported on this platform".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ping_core,
            execute_tool,
            get_system_status,
            get_activity_events,
            create_task,
            get_tasks,
            cancel_task,
            resolve_approval,
            ai_get_status,
            ai_get_models,
            credentials_store,
            credentials_list,
            conversation_create,
            conversation_list,
            conversation_get_messages,
            conversation_send_message,
            conversation_cancel,
            storage_get_analysis,
            storage_get_duplicates,
            storage_get_operations,
            storage_restore_operation,
            storage_get_rules,
            storage_toggle_rule,
            storage_search,
            storage_scan,
            window_set_always_on_top,
            window_set_size_mode,
            window_minimize,
            window_maximize,
            window_close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Hime Desktop application");
}
