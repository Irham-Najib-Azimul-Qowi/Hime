use hime_core::developer::{
    DeveloperCommandRunner, GitOperations, SecretScanner, SoftwareEngineeringTemplates,
    WorkspaceManager,
};
use hime_core::server::CoreServer;
use hime_ipc::framing::{Frame, MsgType};
use hime_ipc::protocol::{JsonRpcRequest, JsonRpcResponse};
use hime_storage::Database;
use std::sync::Arc;

struct TestDir {
    path: std::path::PathBuf,
}
impl TestDir {
    fn new(prefix: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "{}_{:x}",
            prefix,
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let _ = std::fs::create_dir_all(&path);
        Self { path }
    }
}
impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[tokio::test]
async fn test_workspace_manager_registration_and_boundary_defense() {
    let storage = Arc::new(Database::open_in_memory().unwrap());
    let mgr = WorkspaceManager::new(storage.clone());
    let dir = TestDir::new("hime_dev_ws_test");

    // 1. Register workspace
    let ws = mgr.register_workspace("Test Project", &dir.path, None).unwrap();
    assert_eq!(ws.name, "Test Project");

    // 2. Allowed file path inside workspace
    let safe_file = dir.path.join("src").join("main.rs");
    let validated = mgr.validate_file_access(&dir.path, &safe_file).unwrap();
    assert!(validated.starts_with(&dir.path));

    // 3. Path traversal outside workspace must fail
    let outside_file = dir.path.join("..").join("secret_data.txt");
    assert!(mgr.validate_file_access(&dir.path, &outside_file).is_err());
}

#[tokio::test]
async fn test_command_runner_allowlist_and_injection_defense() {
    let storage = Arc::new(Database::open_in_memory().unwrap());
    let runner = DeveloperCommandRunner::new(storage.clone());

    // 1. Disallowed binary
    assert!(runner.validate_command("powershell", &[], false).is_err());
    assert!(runner.validate_command("cmd.exe", &[], false).is_err());
    assert!(runner.validate_command("curl", &[], false).is_err());

    // 2. Allowed binary
    assert!(runner.validate_command("git", &["status".to_string()], false).is_ok());
    assert!(runner.validate_command("cargo", &["check".to_string()], false).is_ok());

    // 3. Shell injection metacharacters blocked
    assert!(runner.validate_command("git", &["status; rm -rf /".to_string()], false).is_err());
    assert!(runner.validate_command("cargo", &["test".to_string(), "&&".to_string(), "echo".to_string()], false).is_err());
    assert!(runner.validate_command("git", &["log | grep secret".to_string()], false).is_err());

    // 4. Destructive command requires explicit approval
    assert!(runner.validate_command("git", &["reset".to_string(), "--hard".to_string(), "HEAD".to_string()], false).is_err());
    assert!(runner.validate_command("git", &["reset".to_string(), "--hard".to_string(), "HEAD".to_string()], true).is_ok());

    // 5. Git status inspection
    let status_res = GitOperations::status(std::path::Path::new("."));
    assert!(status_res.is_ok());
    let status = status_res.unwrap();
    assert!(!status.current_branch.is_empty());
}

#[tokio::test]
async fn test_secret_scanner_and_se_templates() {
    // 1. Scanner finds AWS and GH tokens
    let text = "AWS_SECRET_KEY=AKIA1234567890ABCDEF\nGH_PAT=ghp_123456789012345678901234567890123456";
    let findings = SecretScanner::scan_text("test_config.env", text);
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].secret_type, "AWS Access Key");
    assert_eq!(findings[1].secret_type, "GitHub Personal Access Token");

    // 2. SE Templates available and rendered
    let templates = SoftwareEngineeringTemplates::list_available();
    assert_eq!(templates.len(), 4);
    assert!(templates.iter().any(|t| t.id == "srs-skpl"));
    assert!(templates.iter().any(|t| t.id == "sad-dppl"));

    let rendered_srs = SoftwareEngineeringTemplates::render_template("srs-skpl", "Hime Platform", "Satoshi").unwrap();
    assert!(rendered_srs.contains("SPESIFIKASI KEBUTUHAN PERANGKAT LUNAK (SKPL)"));
    assert!(rendered_srs.contains("Hime Platform"));
}

#[tokio::test]
async fn test_core_server_developer_rpc_endpoints() {
    let server = CoreServer::new_in_memory("token-phase13-developer".to_string()).unwrap();
    let dir = TestDir::new("hime_dev_rpc_test");

    let (client_stream, server_stream) = tokio::io::duplex(4096);
    tokio::spawn(async move {
        server.handle_connection(server_stream).await;
    });

    let (mut client_reader, mut client_writer) = tokio::io::split(client_stream);

    // 1. Register workspace via RPC
    let req = JsonRpcRequest::new(
        "req-dev-1",
        "developer.workspaces.register",
        Some(serde_json::json!({
            "name": "Integration Test WS",
            "path": dir.path.to_string_lossy().to_string(),
        })),
    );
    let payload = req.to_bytes().unwrap();
    Frame::new(MsgType::Request, payload).unwrap().write_to(&mut client_writer).await.unwrap();

    let resp_frame = Frame::read_from(&mut client_reader).await.unwrap();
    let resp = JsonRpcResponse::from_slice(&resp_frame.payload).unwrap();
    assert!(resp.result.is_some());
    assert_eq!(resp.result.unwrap()["name"], "Integration Test WS");

    // 2. List templates via RPC
    let req2 = JsonRpcRequest::new("req-dev-2", "developer.templates.list", None);
    let payload2 = req2.to_bytes().unwrap();
    Frame::new(MsgType::Request, payload2).unwrap().write_to(&mut client_writer).await.unwrap();

    let resp_frame2 = Frame::read_from(&mut client_reader).await.unwrap();
    let resp2 = JsonRpcResponse::from_slice(&resp_frame2.payload).unwrap();
    assert!(resp2.result.is_some());
    assert_eq!(resp2.result.unwrap().as_array().unwrap().len(), 4);
}
