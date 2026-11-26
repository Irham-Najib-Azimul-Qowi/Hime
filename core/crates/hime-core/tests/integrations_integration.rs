use hime_core::integrations::{GitHubAdapter, GoogleDriveAdapter};
use hime_core::server::CoreServer;
use hime_ipc::framing::{Frame, MsgType};
use hime_ipc::protocol::{JsonRpcRequest, JsonRpcResponse};
use hime_storage::{Database, IntegrationAccountRecord};
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
async fn test_github_adapter_operations_and_approval_gate() {
    let storage = Arc::new(Database::open_in_memory().unwrap());
    let adapter = GitHubAdapter::new(storage.clone());

    // 1. Missing token failure
    let err = adapter.inspect_repository("nousresearch", "hermes", "unregistered-account");
    assert!(err.is_err());

    // 2. Register mock account
    let acc = IntegrationAccountRecord {
        id: "acc-gh-test".to_string(),
        service_name: "github".to_string(),
        account_label: "test-account".to_string(),
        auth_type: "oauth_pat".to_string(),
        token_vault_id: "vault-gh-tok".to_string(),
        scopes: "repo,read:user".to_string(),
        status: "active".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    storage.save_integration_account(&acc).unwrap();

    // 3. Inspect repository
    let repo_info = adapter.inspect_repository("owner", "hime", "test-account").unwrap();
    assert_eq!(repo_info.full_name, "owner/hime");
    assert_eq!(repo_info.default_branch, "main");

    // 4. List and create issues
    let issues = adapter.list_issues("owner", "hime", "test-account").unwrap();
    assert!(!issues.is_empty());

    let new_issue = adapter.create_issue("owner", "hime", "Test Bug Report", "Description", "test-account").unwrap();
    assert_eq!(new_issue.title, "Test Bug Report");
    assert_eq!(new_issue.state, "open");

    // 5. Open Pull Request
    let pr = adapter.open_pr("owner", "hime", "feat: new capability", "feature-branch", "main", "test-account").unwrap();
    assert_eq!(pr.head_branch, "feature-branch");

    // 6. Push without approval must be rejected
    assert!(adapter.check_push_approval(false).is_err());
    assert!(adapter.check_push_approval(true).is_ok());
}

#[tokio::test]
async fn test_google_drive_adapter_operations_and_sharing_approval() {
    let storage = Arc::new(Database::open_in_memory().unwrap());
    let adapter = GoogleDriveAdapter::new(storage.clone());
    let dir = TestDir::new("hime_drive_test");

    // Register mock account
    let acc = IntegrationAccountRecord {
        id: "acc-drive-test".to_string(),
        service_name: "google_drive".to_string(),
        account_label: "test-account".to_string(),
        auth_type: "oauth_loopback".to_string(),
        token_vault_id: "vault-drive-tok".to_string(),
        scopes: "drive.file".to_string(),
        status: "active".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    storage.save_integration_account(&acc).unwrap();

    // 1. List files
    let files = adapter.list_files(None, "test-account").unwrap();
    assert!(!files.is_empty());
    assert!(files.iter().any(|f| f.is_folder));

    // 2. Upload file
    let local_file = dir.path.join("local_sample.txt");
    std::fs::write(&local_file, "Sample text content").unwrap();

    let uploaded = adapter.upload_file(&local_file, "cloud_sample.txt", "text/plain", "test-account").unwrap();
    assert_eq!(uploaded.name, "cloud_sample.txt");

    // 3. Sharing requires explicit approval
    assert!(adapter.share_file(&uploaded.id, "collaborator@example.com", "editor", false).is_err());
    let shared = adapter.share_file(&uploaded.id, "collaborator@example.com", "editor", true).unwrap();
    assert!(shared.contains("shared with 'collaborator@example.com'"));
}

#[tokio::test]
async fn test_core_server_integrations_rpc_endpoints() {
    let server = CoreServer::new_in_memory("token-phase13-integrations".to_string()).unwrap();

    // Pre-seed test integration account
    let acc = IntegrationAccountRecord {
        id: "acc-gh-pre".to_string(),
        service_name: "github".to_string(),
        account_label: "test-account".to_string(),
        auth_type: "oauth_pat".to_string(),
        token_vault_id: "vault-gh-tok".to_string(),
        scopes: "repo".to_string(),
        status: "active".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    server.storage.save_integration_account(&acc).unwrap();

    let (client_stream, server_stream) = tokio::io::duplex(4096);
    tokio::spawn(async move {
        server.handle_connection(server_stream).await;
    });

    let (mut client_reader, mut client_writer) = tokio::io::split(client_stream);

    // 1. Inspect GitHub repo via RPC
    let req = JsonRpcRequest::new(
        "req-gh-1",
        "integrations.github.inspect",
        Some(serde_json::json!({
            "owner": "test-owner",
            "repo": "test-repo",
            "account_label": "test-account"
        })),
    );
    let payload = req.to_bytes().unwrap();
    Frame::new(MsgType::Request, payload).unwrap().write_to(&mut client_writer).await.unwrap();

    let resp_frame = Frame::read_from(&mut client_reader).await.unwrap();
    let resp = JsonRpcResponse::from_slice(&resp_frame.payload).unwrap();
    assert!(resp.result.is_some());
    assert_eq!(resp.result.unwrap()["full_name"], "test-owner/test-repo");

    // 2. List Integration accounts via RPC
    let req2 = JsonRpcRequest::new("req-acc-1", "integrations.accounts.list", None);
    let payload2 = req2.to_bytes().unwrap();
    Frame::new(MsgType::Request, payload2).unwrap().write_to(&mut client_writer).await.unwrap();

    let resp_frame2 = Frame::read_from(&mut client_reader).await.unwrap();
    let resp2 = JsonRpcResponse::from_slice(&resp_frame2.payload).unwrap();
    assert!(resp2.result.is_some());
    assert_eq!(resp2.result.unwrap().as_array().unwrap().len(), 1);
}
