use hime_core::bus::EventBus;
use hime_core::device::pairing::PairingError;
use hime_core::device::DeviceManager;
use hime_core::server::ApprovalRecord;
use hime_core::storage::operations::SafeFileOperationsEngine;
use hime_ipc::protocol::JsonRpcRequest;
use hime_storage::Database;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;

struct TestContext {
    db: Arc<Database>,
    device_manager: Arc<DeviceManager>,
    approvals: Arc<Mutex<HashMap<String, ApprovalRecord>>>,
    test_dir: std::path::PathBuf,
}

impl Drop for TestContext {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.test_dir);
    }
}

fn setup_test_context() -> TestContext {
    let db = Arc::new(Database::open_in_memory().expect("in-memory db"));
    let temp_dir = std::env::temp_dir().join(format!(
        "hime_device_test_{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap();

    let event_bus = EventBus::new(64);
    let approvals = Arc::new(Mutex::new(HashMap::new()));
    let ops_engine = Arc::new(SafeFileOperationsEngine::new(db.clone(), &temp_dir).unwrap());

    let device_manager = Arc::new(
        DeviceManager::new(
            db.clone(),
            ops_engine,
            event_bus,
            approvals.clone(),
            &temp_dir,
            "127.0.0.1",
            20130,
        )
        .unwrap(),
    );

    TestContext {
        db,
        device_manager,
        approvals,
        test_dir: temp_dir,
    }
}

#[tokio::test]
async fn test_device_pairing_session_and_anti_replay() {
    let ctx = setup_test_context();
    let pairing = ctx.device_manager.pairing.clone();

    // 1. Initiate pairing session (QR payload)
    let qr = pairing.create_pairing_session(300).expect("session created");
    assert_eq!(qr.server_address, "127.0.0.1");
    assert_eq!(qr.server_port, 20130);
    assert!(!qr.pairing_token.is_empty());
    assert!(!qr.confirmation_code.is_empty());

    // 2. Negative test: wrong confirmation code
    let wrong_code_res = pairing.complete_pairing(
        &qr.pairing_token,
        "000000",
        "dev-pixel-01",
        "Pixel 8 Pro",
        "android",
        "SHA256:DEVICE-CLIENT-CERT-FINGERPRINT",
    );
    assert!(matches!(wrong_code_res, Err(PairingError::CodeMismatch)));

    // 3. Positive test: valid confirmation code
    let dev = pairing
        .complete_pairing(
            &qr.pairing_token,
            &qr.confirmation_code,
            "dev-pixel-01",
            "Pixel 8 Pro",
            "android",
            "SHA256:DEVICE-CLIENT-CERT-FINGERPRINT",
        )
        .expect("pairing successful");

    assert_eq!(dev.id, "dev-pixel-01");
    assert_eq!(dev.device_name, "Pixel 8 Pro");
    assert_eq!(dev.status, "active");

    // 4. Anti-replay test: using the consumed token again must fail
    let replay_res = pairing.complete_pairing(
        &qr.pairing_token,
        &qr.confirmation_code,
        "dev-pixel-01",
        "Pixel 8 Pro",
        "android",
        "SHA256:DEVICE-CLIENT-CERT-FINGERPRINT",
    );
    assert!(matches!(replay_res, Err(PairingError::InvalidToken)));
}

#[tokio::test]
async fn test_device_active_verification_and_revocation() {
    let ctx = setup_test_context();
    let pairing = ctx.device_manager.pairing.clone();

    // Pair a device
    let qr = pairing.create_pairing_session(300).unwrap();
    let dev = pairing
        .complete_pairing(
            &qr.pairing_token,
            &qr.confirmation_code,
            "dev-galaxy-01",
            "Galaxy S24",
            "android",
            "SHA256:VALID-FINGERPRINT-ABC",
        )
        .unwrap();

    // Verify with valid fingerprint
    let is_valid = pairing
        .verify_device_active(&dev.id, "SHA256:VALID-FINGERPRINT-ABC")
        .expect("verify ok");
    assert!(is_valid);

    // Verify with invalid fingerprint -> false
    let is_invalid = pairing
        .verify_device_active(&dev.id, "SHA256:ATTACKER-FINGERPRINT-XYZ")
        .expect("verify ok");
    assert!(!is_invalid);

    // Revoke device
    let revoked = ctx.db.revoke_device(&dev.id).expect("revoke ok");
    assert!(revoked);

    // After revocation, verification must return DeviceRevoked error
    let revoked_res = pairing.verify_device_active(&dev.id, "SHA256:VALID-FINGERPRINT-ABC");
    assert!(matches!(revoked_res, Err(PairingError::DeviceRevoked)));
}

#[tokio::test]
async fn test_remote_approval_resolution_lifecycle() {
    let ctx = setup_test_context();
    let gw = ctx.device_manager.gateway.clone();

    // Pair device first
    let qr = ctx.device_manager.pairing.create_pairing_session(300).unwrap();
    ctx.device_manager
        .pairing
        .complete_pairing(
            &qr.pairing_token,
            &qr.confirmation_code,
            "dev-mobile-remote",
            "Mobile Companion",
            "android",
            "SHA256:CLIENT-FP-001",
        )
        .unwrap();

    // Desktop creates a pending approval ticket
    let ticket_id = "ticket-storage-cleanup-01";
    {
        let mut map = ctx.approvals.lock().await;
        map.insert(
            ticket_id.to_string(),
            ApprovalRecord {
                request_id: ticket_id.to_string(),
                action_type: "storage.delete_file".to_string(),
                path: "C:/Users/User/Downloads/large_dump.zip".to_string(),
                description: "Delete obsolete 4GB archive".to_string(),
                status: "pending".to_string(),
                created_at: 1000,
                expires_at: 5000,
                consumed: false,
            },
        );
    }

    // 1. Mobile requests list of pending approvals
    let list_req = JsonRpcRequest::new(
        "req-1",
        "device.approvals.list",
        Some(serde_json::json!({ "device_id": "dev-mobile-remote" })),
    );
    let list_resp = gw.dispatch_device_rpc(list_req, "SHA256:CLIENT-FP-001").await;
    assert!(list_resp.error.is_none());
    let pending_items = list_resp.result.unwrap();
    let pending_arr = pending_items.as_array().unwrap();
    assert_eq!(pending_arr.len(), 1);
    assert_eq!(pending_arr[0]["request_id"], ticket_id);

    // 2. Mobile submits Approval Decision
    let resolve_req = JsonRpcRequest::new(
        "req-2",
        "device.approvals.resolve",
        Some(serde_json::json!({
            "device_id": "dev-mobile-remote",
            "ticket_id": ticket_id,
            "decision": "approve"
        })),
    );
    let resolve_resp = gw.dispatch_device_rpc(resolve_req, "SHA256:CLIENT-FP-001").await;
    assert!(resolve_resp.error.is_none());
    let res_val = resolve_resp.result.unwrap();
    assert_eq!(res_val["status"], "approved");
    assert_eq!(res_val["resolved_by_device"], "dev-mobile-remote");

    // 3. Ticket is now consumed in Core
    {
        let map = ctx.approvals.lock().await;
        let record = map.get(ticket_id).unwrap();
        assert!(record.consumed);
        assert_eq!(record.status, "approved");
    }

    // 4. Resolving again must fail (already consumed)
    let duplicate_resolve_req = JsonRpcRequest::new(
        "req-3",
        "device.approvals.resolve",
        Some(serde_json::json!({
            "device_id": "dev-mobile-remote",
            "ticket_id": ticket_id,
            "decision": "approve"
        })),
    );
    let dup_resp = gw.dispatch_device_rpc(duplicate_resolve_req, "SHA256:CLIENT-FP-001").await;
    assert!(dup_resp.error.is_some());
    assert_eq!(dup_resp.error.unwrap().message, "Approval ticket already consumed");
}

#[tokio::test]
async fn test_device_status_and_storage_reporting() {
    let ctx = setup_test_context();
    let gw = ctx.device_manager.gateway.clone();

    // Pair device
    let qr = ctx.device_manager.pairing.create_pairing_session(300).unwrap();
    ctx.device_manager
        .pairing
        .complete_pairing(
            &qr.pairing_token,
            &qr.confirmation_code,
            "dev-status-01",
            "Pixel 7",
            "android",
            "SHA256:STATUS-CERT-FP",
        )
        .unwrap();

    // Device reports status
    let report_req = JsonRpcRequest::new(
        "req-status",
        "device.status.report",
        Some(serde_json::json!({
            "device_id": "dev-status-01",
            "battery_percent": 88,
            "is_charging": false,
            "storage_total_bytes": 128_000_000_000i64,
            "storage_free_bytes": 45_000_000_000i64,
            "security_posture": {
                "is_device_secure": true,
                "storage_encrypted": true,
                "security_patch_level": "2026-11-01",
                "unknown_sources_enabled": false,
                "os_version": "Android 15"
            },
            "timestamp": 1731500000i64
        })),
    );

    let report_resp = gw.dispatch_device_rpc(report_req, "SHA256:STATUS-CERT-FP").await;
    assert!(report_resp.error.is_none());

    // Verify storage summary was persisted in DB
    let summary = ctx
        .db
        .get_device_storage_summary("dev-status-01")
        .expect("db query ok")
        .expect("summary exists");

    assert_eq!(summary.device_id, "dev-status-01");
    assert_eq!(summary.total_bytes, 128_000_000_000);
    assert_eq!(summary.free_bytes, 45_000_000_000);
}

#[tokio::test]
async fn test_chunked_file_transfer_with_sha256_verification() {
    let ctx = setup_test_context();
    let gw = ctx.device_manager.gateway.clone();

    // Pair device
    let qr = ctx.device_manager.pairing.create_pairing_session(300).unwrap();
    ctx.device_manager
        .pairing
        .complete_pairing(
            &qr.pairing_token,
            &qr.confirmation_code,
            "dev-transfer-01",
            "Transfer Test Phone",
            "android",
            "SHA256:TRANSFER-FP",
        )
        .unwrap();

    // Prepare test payload: "Hello Hime Secure Mobile Transfer System!"
    let content = b"Hello Hime Secure Mobile Transfer System! Part 1 and Part 2.";
    let chunk1 = &content[..25];
    let chunk2 = &content[25..];

    use sha2::{Digest, Sha256};
    let overall_sha = format!("{:x}", Sha256::digest(content));
    let chunk1_sha = format!("{:x}", Sha256::digest(chunk1));
    let chunk2_sha = format!("{:x}", Sha256::digest(chunk2));

    // Base64 encode helper
    fn to_b64(bytes: &[u8]) -> String {
        const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        let mut i = 0;
        while i < bytes.len() {
            let b0 = bytes[i] as usize;
            let b1 = if i + 1 < bytes.len() { bytes[i + 1] as usize } else { 0 };
            let b2 = if i + 2 < bytes.len() { bytes[i + 2] as usize } else { 0 };

            out.push(TABLE[b0 >> 2] as char);
            out.push(TABLE[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
            if i + 1 < bytes.len() {
                out.push(TABLE[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
            } else {
                out.push('=');
            }
            if i + 2 < bytes.len() {
                out.push(TABLE[b2 & 0x3f] as char);
            } else {
                out.push('=');
            }
            i += 3;
        }
        out
    }

    let transfer_id = "xfer-001";
    let init_req = JsonRpcRequest::new(
        "req-init",
        "device.transfer.init",
        Some(serde_json::json!({
            "device_id": "dev-transfer-01",
            "transfer_id": transfer_id,
            "file_name": "mobile_note.txt",
            "file_size": content.len() as i64,
            "sha256": overall_sha
        })),
    );
    let init_resp = gw.dispatch_device_rpc(init_req, "SHA256:TRANSFER-FP").await;
    assert!(init_resp.error.is_none());

    // Upload chunk 1
    let chunk1_req = JsonRpcRequest::new(
        "req-chunk-1",
        "device.transfer.chunk",
        Some(serde_json::json!({
            "device_id": "dev-transfer-01",
            "transfer_id": transfer_id,
            "data_base64": to_b64(chunk1),
            "chunk_sha256": chunk1_sha
        })),
    );
    let chunk1_resp = gw.dispatch_device_rpc(chunk1_req, "SHA256:TRANSFER-FP").await;
    assert!(chunk1_resp.error.is_none());

    // Upload chunk 2
    let chunk2_req = JsonRpcRequest::new(
        "req-chunk-2",
        "device.transfer.chunk",
        Some(serde_json::json!({
            "device_id": "dev-transfer-01",
            "transfer_id": transfer_id,
            "data_base64": to_b64(chunk2),
            "chunk_sha256": chunk2_sha
        })),
    );
    let chunk2_resp = gw.dispatch_device_rpc(chunk2_req, "SHA256:TRANSFER-FP").await;
    assert!(chunk2_resp.error.is_none());

    // Finalize transfer to destination path
    let dest_file = ctx.test_dir.join("received_mobile_note.txt");
    let fin_req = JsonRpcRequest::new(
        "req-finalize",
        "device.transfer.finalize",
        Some(serde_json::json!({
            "device_id": "dev-transfer-01",
            "transfer_id": transfer_id,
            "destination_path": dest_file.to_str().unwrap()
        })),
    );
    let fin_resp = gw.dispatch_device_rpc(fin_req, "SHA256:TRANSFER-FP").await;
    assert!(fin_resp.error.is_none());

    // Verify written file
    assert!(dest_file.exists());
    let read_back = fs::read(&dest_file).unwrap();
    assert_eq!(read_back, content);
}

#[tokio::test]
async fn test_core_server_device_rpc_endpoints() {
    let server = hime_core::server::CoreServer::new_in_memory("test-token".to_string()).unwrap();

    // 1. devices.pairing.start
    let req_pair_start = JsonRpcRequest::new(
        "rpc-1",
        "devices.pairing.start",
        Some(serde_json::json!({ "ttl_secs": 600 })),
    );
    let resp_pair_start = server.dispatch_rpc(req_pair_start).await;
    assert!(resp_pair_start.error.is_none());
    let session = resp_pair_start.result.unwrap();
    let token = session["pairing_token"].as_str().unwrap();
    let code = session["confirmation_code"].as_str().unwrap();

    // 2. devices.pairing.confirm
    let req_confirm = JsonRpcRequest::new(
        "rpc-2",
        "devices.pairing.confirm",
        Some(serde_json::json!({
            "token": token,
            "confirmation_code": code,
            "device_id": "dev-core-server-phone",
            "device_name": "Test Pixel Phone",
            "device_type": "android",
            "client_cert_fingerprint": "SHA256:CORE-PHONE-FP"
        })),
    );
    let resp_confirm = server.dispatch_rpc(req_confirm).await;
    assert!(resp_confirm.error.is_none());
    let dev = resp_confirm.result.unwrap();
    assert_eq!(dev["id"], "dev-core-server-phone");

    // 3. devices.list
    let req_list = JsonRpcRequest::new("rpc-3", "devices.list", None);
    let resp_list = server.dispatch_rpc(req_list).await;
    assert!(resp_list.error.is_none());
    let list_arr = resp_list.result.unwrap();
    assert_eq!(list_arr.as_array().unwrap().len(), 1);

    // 4. devices.revoke
    let req_revoke = JsonRpcRequest::new(
        "rpc-4",
        "devices.revoke",
        Some(serde_json::json!({ "device_id": "dev-core-server-phone" })),
    );
    let resp_revoke = server.dispatch_rpc(req_revoke).await;
    assert!(resp_revoke.error.is_none());
    assert_eq!(resp_revoke.result.unwrap()["revoked"], true);
}

#[tokio::test]
async fn test_concurrent_approval_resolution_race_safety() {
    let ctx = setup_test_context();
    let gw = ctx.device_manager.gateway.clone();

    // Pair device
    let qr = ctx.device_manager.pairing.create_pairing_session(300).unwrap();
    ctx.device_manager
        .pairing
        .complete_pairing(
            &qr.pairing_token,
            &qr.confirmation_code,
            "dev-race-phone",
            "Race Phone",
            "android",
            "SHA256:RACE-FP",
        )
        .unwrap();

    let ticket_id = "ticket-race-01";
    {
        let mut map = ctx.approvals.lock().await;
        map.insert(
            ticket_id.to_string(),
            ApprovalRecord {
                request_id: ticket_id.to_string(),
                action_type: "storage.delete_file".to_string(),
                path: "test/path.dat".to_string(),
                description: "Critical race delete".to_string(),
                status: "pending".to_string(),
                created_at: 1000,
                expires_at: 5000,
                consumed: false,
            },
        );
    }

    // Launch 2 concurrent resolution requests: one approving, one denying
    let gw1 = gw.clone();
    let gw2 = gw.clone();

    let handle1 = tokio::spawn(async move {
        let req = JsonRpcRequest::new(
            "r1",
            "device.approvals.resolve",
            Some(serde_json::json!({
                "device_id": "dev-race-phone",
                "ticket_id": "ticket-race-01",
                "decision": "approve"
            })),
        );
        gw1.dispatch_device_rpc(req, "SHA256:RACE-FP").await
    });

    let handle2 = tokio::spawn(async move {
        let req = JsonRpcRequest::new(
            "r2",
            "device.approvals.resolve",
            Some(serde_json::json!({
                "device_id": "dev-race-phone",
                "ticket_id": "ticket-race-01",
                "decision": "deny"
            })),
        );
        gw2.dispatch_device_rpc(req, "SHA256:RACE-FP").await
    });

    let (res1, res2) = tokio::join!(handle1, handle2);
    let resp1 = res1.unwrap();
    let resp2 = res2.unwrap();

    // Exactly one must succeed and one must fail with "Approval ticket already consumed"
    let success_count = (resp1.error.is_none() as usize) + (resp2.error.is_none() as usize);
    assert_eq!(success_count, 1, "Exactly one approval attempt must succeed");

    let err_count = (resp1.error.is_some() as usize) + (resp2.error.is_some() as usize);
    assert_eq!(err_count, 1, "Exactly one approval attempt must fail");
}
