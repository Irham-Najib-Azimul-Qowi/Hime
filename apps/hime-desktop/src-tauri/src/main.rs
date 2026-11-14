#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{command, LogicalSize, Size};
use tracing::info;

use hime_desktop::{
    browser_close_logic, browser_download_logic, browser_downloads_list_logic, browser_extract_logic,
    browser_history_list_logic, browser_open_logic, cancel_conversation_logic, cancel_task_logic,
    create_conversation_logic, create_task_logic, developer_command_run_logic, developer_git_commit_logic,
    developer_git_diff_logic, developer_git_status_logic, developer_templates_list_logic,
    developer_templates_render_logic, developer_workspaces_list_logic, developer_workspaces_register_logic,
    devices_list_logic, devices_pairing_start_logic, devices_revoke_logic, devices_storage_get_logic,
    devices_transfers_list_logic, documents_artcraft_discover_logic, documents_create_report_logic,
    documents_jobs_list_logic, documents_merge_pdfs_logic, execute_tool_logic,
    get_activity_events_logic, get_ai_models_logic, get_ai_status_logic,
    get_conversation_messages_logic, get_system_status_logic, get_tasks_logic,
    integrations_accounts_list_logic, integrations_drive_files_list_logic,
    integrations_drive_files_share_logic, integrations_drive_files_upload_logic,
    integrations_github_inspect_logic, integrations_github_issues_create_logic,
    integrations_github_issues_list_logic, integrations_github_pr_create_logic,
    kanban_board_get_logic, kanban_columns_list_logic, list_conversations_logic,
    list_credentials_logic, ping_core_logic, productivity_accountability_checkin_logic,
    productivity_accountability_get_logic, productivity_accountability_set_mode_logic,
    productivity_schedules_list_logic, productivity_schedules_plan_logic,
    productivity_tasks_create_logic, productivity_tasks_list_logic, productivity_tasks_move_logic,
    productivity_tasks_parse_natural_logic, resolve_approval_logic, send_conversation_message_logic,
    storage_get_analysis_logic, storage_get_duplicates_logic, storage_get_operations_logic,
    storage_get_rules_logic, storage_restore_operation_logic, storage_scan_logic,
    storage_search_logic, storage_toggle_rule_logic, store_credential_logic, PingResult,
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

#[command]
async fn devices_list() -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        devices_list_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn devices_pairing_start(ttl_secs: Option<u64>) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        devices_pairing_start_logic(DEFAULT_HIME_PIPE, ttl_secs).await
    }
    #[cfg(not(windows))]
    {
        let _ = ttl_secs;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn devices_revoke(device_id: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        devices_revoke_logic(DEFAULT_HIME_PIPE, &device_id).await
    }
    #[cfg(not(windows))]
    {
        let _ = device_id;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn devices_storage_get(device_id: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        devices_storage_get_logic(DEFAULT_HIME_PIPE, &device_id).await
    }
    #[cfg(not(windows))]
    {
        let _ = device_id;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn devices_transfers_list(device_id: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        devices_transfers_list_logic(DEFAULT_HIME_PIPE, device_id.as_deref()).await
    }
    #[cfg(not(windows))]
    {
        let _ = device_id;
        Err("Unsupported on this platform".to_string())
    }
}

// =========================================================================
// Phase 12 Commands
// =========================================================================

#[command]
async fn kanban_board_get() -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        kanban_board_get_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn kanban_columns_list(board_id: String) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        kanban_columns_list_logic(DEFAULT_HIME_PIPE, &board_id).await
    }
    #[cfg(not(windows))]
    {
        let _ = board_id;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn productivity_tasks_create(
    board_id: String,
    column_id: String,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    due_date: Option<i64>,
    tags: Option<String>,
    subtasks: Option<String>,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        productivity_tasks_create_logic(
            DEFAULT_HIME_PIPE,
            &board_id,
            &column_id,
            &title,
            description.as_deref(),
            priority.as_deref(),
            due_date,
            tags.as_deref(),
            subtasks.as_deref(),
        )
        .await
    }
    #[cfg(not(windows))]
    {
        let _ = (board_id, column_id, title, description, priority, due_date, tags, subtasks);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn productivity_tasks_list(
    board_id: Option<String>,
    column_id: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        productivity_tasks_list_logic(
            DEFAULT_HIME_PIPE,
            board_id.as_deref(),
            column_id.as_deref(),
        )
        .await
    }
    #[cfg(not(windows))]
    {
        let _ = (board_id, column_id);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn productivity_tasks_move(id: String, column_id: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        productivity_tasks_move_logic(DEFAULT_HIME_PIPE, &id, &column_id).await
    }
    #[cfg(not(windows))]
    {
        let _ = (id, column_id);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn productivity_tasks_parse_natural(text: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        productivity_tasks_parse_natural_logic(DEFAULT_HIME_PIPE, &text).await
    }
    #[cfg(not(windows))]
    {
        let _ = text;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn productivity_schedules_plan(
    title: String,
    subject: String,
    start_time: i64,
    duration_minutes: i64,
    priority: Option<String>,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        productivity_schedules_plan_logic(
            DEFAULT_HIME_PIPE,
            &title,
            &subject,
            start_time,
            duration_minutes,
            priority.as_deref(),
        )
        .await
    }
    #[cfg(not(windows))]
    {
        let _ = (title, subject, start_time, duration_minutes, priority);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn productivity_schedules_list(is_completed: Option<bool>) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        productivity_schedules_list_logic(DEFAULT_HIME_PIPE, is_completed).await
    }
    #[cfg(not(windows))]
    {
        let _ = is_completed;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn productivity_accountability_get() -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        productivity_accountability_get_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn productivity_accountability_set_mode(
    mode: String,
    opt_in_screen_time: Option<bool>,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        productivity_accountability_set_mode_logic(
            DEFAULT_HIME_PIPE,
            &mode,
            opt_in_screen_time,
        )
        .await
    }
    #[cfg(not(windows))]
    {
        let _ = (mode, opt_in_screen_time);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn productivity_accountability_checkin() -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        productivity_accountability_checkin_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn documents_create_report(
    template_id: String,
    title: String,
    author: String,
    institution: Option<String>,
    abstract_text: Option<String>,
    sections: Vec<serde_json::Value>,
    target_format: Option<String>,
    output_path: Option<String>,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        documents_create_report_logic(
            DEFAULT_HIME_PIPE,
            &template_id,
            &title,
            &author,
            institution.as_deref(),
            abstract_text.as_deref(),
            sections,
            target_format.as_deref(),
            output_path.as_deref(),
        )
        .await
    }
    #[cfg(not(windows))]
    {
        let _ = (template_id, title, author, institution, abstract_text, sections, target_format, output_path);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn documents_merge_pdfs(
    input_paths: Vec<String>,
    output_path: Option<String>,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        documents_merge_pdfs_logic(DEFAULT_HIME_PIPE, input_paths, output_path.as_deref()).await
    }
    #[cfg(not(windows))]
    {
        let _ = (input_paths, output_path);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn documents_jobs_list(limit: Option<usize>) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        documents_jobs_list_logic(DEFAULT_HIME_PIPE, limit).await
    }
    #[cfg(not(windows))]
    {
        let _ = limit;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn documents_artcraft_discover() -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        documents_artcraft_discover_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

// ---------------------------------------------------------------------------
// Phase 13: Browser, Integrations & Developer Workspace Commands
// ---------------------------------------------------------------------------

#[command]
async fn browser_open(url: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        browser_open_logic(DEFAULT_HIME_PIPE, &url).await
    }
    #[cfg(not(windows))]
    {
        let _ = url;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn browser_extract(session_id: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        browser_extract_logic(DEFAULT_HIME_PIPE, &session_id).await
    }
    #[cfg(not(windows))]
    {
        let _ = session_id;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn browser_download(url: String, target_path: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        browser_download_logic(DEFAULT_HIME_PIPE, &url, &target_path).await
    }
    #[cfg(not(windows))]
    {
        let _ = (url, target_path);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn browser_close(session_id: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        browser_close_logic(DEFAULT_HIME_PIPE, &session_id).await
    }
    #[cfg(not(windows))]
    {
        let _ = session_id;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn browser_history_list(limit: Option<usize>) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        browser_history_list_logic(DEFAULT_HIME_PIPE, limit).await
    }
    #[cfg(not(windows))]
    {
        let _ = limit;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn browser_downloads_list(limit: Option<usize>) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        browser_downloads_list_logic(DEFAULT_HIME_PIPE, limit).await
    }
    #[cfg(not(windows))]
    {
        let _ = limit;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn integrations_github_inspect(owner: String, repo: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        integrations_github_inspect_logic(DEFAULT_HIME_PIPE, &owner, &repo).await
    }
    #[cfg(not(windows))]
    {
        let _ = (owner, repo);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn integrations_github_issues_list(owner: String, repo: String, state: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        integrations_github_issues_list_logic(DEFAULT_HIME_PIPE, &owner, &repo, state.as_deref()).await
    }
    #[cfg(not(windows))]
    {
        let _ = (owner, repo, state);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn integrations_github_issues_create(owner: String, repo: String, title: String, body: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        integrations_github_issues_create_logic(DEFAULT_HIME_PIPE, &owner, &repo, &title, &body).await
    }
    #[cfg(not(windows))]
    {
        let _ = (owner, repo, title, body);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn integrations_github_pr_create(owner: String, repo: String, title: String, head: String, base: String, body: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        integrations_github_pr_create_logic(DEFAULT_HIME_PIPE, &owner, &repo, &title, &head, &base, &body).await
    }
    #[cfg(not(windows))]
    {
        let _ = (owner, repo, title, head, base, body);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn integrations_drive_files_list(parent_id: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        integrations_drive_files_list_logic(DEFAULT_HIME_PIPE, parent_id.as_deref()).await
    }
    #[cfg(not(windows))]
    {
        let _ = parent_id;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn integrations_drive_files_upload(name: String, local_path: String, mime_type: Option<String>) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        integrations_drive_files_upload_logic(DEFAULT_HIME_PIPE, &name, &local_path, mime_type.as_deref()).await
    }
    #[cfg(not(windows))]
    {
        let _ = (name, local_path, mime_type);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn integrations_drive_files_share(file_id: String, email: String, role: Option<String>) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        integrations_drive_files_share_logic(DEFAULT_HIME_PIPE, &file_id, &email, role.as_deref()).await
    }
    #[cfg(not(windows))]
    {
        let _ = (file_id, email, role);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn integrations_accounts_list() -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        integrations_accounts_list_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn developer_workspaces_register(name: String, root_path: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        developer_workspaces_register_logic(DEFAULT_HIME_PIPE, &name, &root_path).await
    }
    #[cfg(not(windows))]
    {
        let _ = (name, root_path);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn developer_workspaces_list() -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        developer_workspaces_list_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn developer_command_run(workspace_id: String, command: String, args: Vec<String>) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        developer_command_run_logic(DEFAULT_HIME_PIPE, &workspace_id, &command, args).await
    }
    #[cfg(not(windows))]
    {
        let _ = (workspace_id, command, args);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn developer_git_status(workspace_id: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        developer_git_status_logic(DEFAULT_HIME_PIPE, &workspace_id).await
    }
    #[cfg(not(windows))]
    {
        let _ = workspace_id;
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn developer_git_diff(workspace_id: String, staged_only: Option<bool>) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        developer_git_diff_logic(DEFAULT_HIME_PIPE, &workspace_id, staged_only).await
    }
    #[cfg(not(windows))]
    {
        let _ = (workspace_id, staged_only);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn developer_git_commit(workspace_id: String, message: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        developer_git_commit_logic(DEFAULT_HIME_PIPE, &workspace_id, &message).await
    }
    #[cfg(not(windows))]
    {
        let _ = (workspace_id, message);
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn developer_templates_list() -> Result<Vec<serde_json::Value>, String> {
    #[cfg(windows)]
    {
        developer_templates_list_logic(DEFAULT_HIME_PIPE).await
    }
    #[cfg(not(windows))]
    {
        Err("Unsupported on this platform".to_string())
    }
}

#[command]
async fn developer_templates_render(template_id: String, project_name: String, author: String) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        developer_templates_render_logic(DEFAULT_HIME_PIPE, &template_id, &project_name, &author).await
    }
    #[cfg(not(windows))]
    {
        let _ = (template_id, project_name, author);
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
            devices_list,
            devices_pairing_start,
            devices_revoke,
            devices_storage_get,
            devices_transfers_list,
            kanban_board_get,
            kanban_columns_list,
            productivity_tasks_create,
            productivity_tasks_list,
            productivity_tasks_move,
            productivity_tasks_parse_natural,
            productivity_schedules_plan,
            productivity_schedules_list,
            productivity_accountability_get,
            productivity_accountability_set_mode,
            productivity_accountability_checkin,
            documents_create_report,
            documents_merge_pdfs,
            documents_jobs_list,
            documents_artcraft_discover,
            browser_open,
            browser_extract,
            browser_download,
            browser_close,
            browser_history_list,
            browser_downloads_list,
            integrations_github_inspect,
            integrations_github_issues_list,
            integrations_github_issues_create,
            integrations_github_pr_create,
            integrations_drive_files_list,
            integrations_drive_files_upload,
            integrations_drive_files_share,
            integrations_accounts_list,
            developer_workspaces_register,
            developer_workspaces_list,
            developer_command_run,
            developer_git_status,
            developer_git_diff,
            developer_git_commit,
            developer_templates_list,
            developer_templates_render,
            window_set_always_on_top,
            window_set_size_mode,
            window_minimize,
            window_maximize,
            window_close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Hime Desktop application");
}
