use hime_core::storage::{
    BackupService, DownloadIntelligence, DownloadStatus, DuplicateDetector, IndexerService,
    SafeFileOperationsEngine, SmartOrganizer,
};
use hime_core::CoreServer;
use hime_ipc::protocol::JsonRpcRequest;
use hime_storage::{Database, FileRuleRecord};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct TestSandbox {
    root: PathBuf,
    hime_dir: PathBuf,
    db: Arc<Database>,
}

impl TestSandbox {
    fn new(name: &str) -> Self {
        let base_tmp = std::env::temp_dir().join(format!("hime_sandbox_{}_{:x}", name, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let root = base_tmp.join("user_files");
        let hime_dir = base_tmp.join(".hime");

        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&hime_dir).unwrap();

        let db = Arc::new(Database::open_in_memory().unwrap());

        Self {
            root,
            hime_dir,
            db,
        }
    }

    fn create_file(&self, rel_path: &str, content: &[u8]) -> PathBuf {
        let full_path = self.root.join(rel_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = File::create(&full_path).unwrap();
        f.write_all(content).unwrap();
        full_path
    }
}

impl Drop for TestSandbox {
    fn drop(&mut self) {
        if let Some(parent) = self.root.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }
}

#[tokio::test]
async fn test_storage_e2e_indexing_and_analysis_on_synthetic_sandbox() {
    let sandbox = TestSandbox::new("indexing_analysis");

    // Populate sandbox with files
    sandbox.create_file("Documents/report1.pdf", b"PDF report content one");
    sandbox.create_file("Documents/report2.pdf", b"PDF report content two");
    sandbox.create_file("Projects/main.rs", b"fn main() { println!(\"hello\"); }");
    sandbox.create_file("Downloads/setup.exe", b"Installer binary data mock");
    sandbox.create_file("Downloads/temp.tmp", b"temporary cache data");

    let indexer = IndexerService::new(sandbox.db.clone());
    let scanned = indexer.scan_directory(&sandbox.root).unwrap();
    assert!(scanned >= 5, "Expected at least 5 files/dirs scanned, got {}", scanned);

    let count = sandbox.db.count_indexed_files().unwrap();
    assert!(count >= 5);

    // Search
    let search_results = sandbox.db.search_indexed_files("report", 10).unwrap();
    assert_eq!(search_results.len(), 2);

    // Analysis
    let analyzer = hime_core::storage::AnalyzerService::new(sandbox.db.clone());
    let report = analyzer.generate_report(10).unwrap();
    assert!(report.base_report.total_files >= 5);
    assert!(report.base_report.total_bytes > 0);

    // Cleanable suggestions must detect .tmp file
    assert!(report.cleanable_suggestions.iter().any(|s| s.path.ends_with(".tmp")));
}

#[tokio::test]
async fn test_storage_e2e_duplicate_and_version_detection() {
    let sandbox = TestSandbox::new("duplicates_versions");

    let identical_bytes = b"Exact identical document content for testing SHA-256";
    sandbox.create_file("Documents/original.docx", identical_bytes);
    sandbox.create_file("Downloads/original_copy.docx", identical_bytes);

    // Version candidate (different content, name variation)
    sandbox.create_file("Documents/laporan.docx", b"Laporan Draft Versi 1");
    sandbox.create_file("Documents/laporan (1).docx", b"Laporan Draft Versi 2 Revisi Tambahan");

    let indexer = IndexerService::new(sandbox.db.clone());
    indexer.scan_directory(&sandbox.root).unwrap();

    let detector = DuplicateDetector::new(sandbox.db.clone());

    // 1. Exact duplicates
    let dups = detector.find_exact_duplicates().unwrap();
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0].files.len(), 2);

    // 2. Version candidates
    let versions = detector.find_version_candidates().unwrap();
    assert!(!versions.is_empty(), "Expected at least 1 version candidate pair");
    assert!(versions[0].candidate_file.file_name.contains("(1)"));
}

#[tokio::test]
async fn test_storage_e2e_safe_file_move_atomic_recycle_and_journal() {
    let sandbox = TestSandbox::new("safe_move");

    let src = sandbox.create_file("Documents/proposal.txt", b"Confidential business plan");
    let dst = sandbox.root.join("Archive/proposal_archived.txt");

    let ops = SafeFileOperationsEngine::new(sandbox.db.clone(), &sandbox.hime_dir).unwrap();

    let res = ops.safe_move("op-move-101", &src, &dst, "agent_hermes", None).unwrap();
    assert_eq!(res.status, "committed");

    // Destination must exist and have correct content
    assert!(dst.exists());
    let dst_content = fs::read_to_string(&dst).unwrap();
    assert_eq!(dst_content, "Confidential business plan");

    // Original source file must no longer be at original path
    assert!(!src.exists());

    // Source must be safely preserved in Hime Recycle
    let recycle_path = PathBuf::from(res.backup_path.unwrap());
    assert!(recycle_path.exists());

    // Journal must be committed
    let op_rec = sandbox.db.get_file_operation("op-move-101").unwrap().unwrap();
    assert_eq!(op_rec.status, "committed");

    // Test One-Click Restore:
    let restore_res = ops.restore_file("op-res-101", "op-move-101", "user_desktop").unwrap();
    assert_eq!(restore_res.status, "committed");

    // Restored file must be back at original path
    assert!(src.exists());
    assert_eq!(fs::read_to_string(&src).unwrap(), "Confidential business plan");
}

#[tokio::test]
async fn test_storage_e2e_safe_delete_to_recycle_and_restore() {
    let sandbox = TestSandbox::new("safe_delete");

    let file_to_delete = sandbox.create_file("Documents/obsolete.txt", b"Old data that needs deletion");

    let ops = SafeFileOperationsEngine::new(sandbox.db.clone(), &sandbox.hime_dir).unwrap();

    let res = ops.safe_delete("del-op-001", &file_to_delete, "user", Some("ticket-appr-99")).unwrap();
    assert_eq!(res.status, "committed");

    // Original file must not exist
    assert!(!file_to_delete.exists());

    // File must be in Hime Recycle
    let backup_p = PathBuf::from(res.backup_path.unwrap());
    assert!(backup_p.exists());

    // Restore it
    let restore_res = ops.restore_file("res-op-001", "del-op-001", "user").unwrap();
    assert_eq!(restore_res.status, "committed");
    assert!(file_to_delete.exists());
    assert_eq!(fs::read_to_string(&file_to_delete).unwrap(), "Old data that needs deletion");
}

#[tokio::test]
async fn test_storage_e2e_crash_recovery_rolls_back_uncommitted_staging() {
    let sandbox = TestSandbox::new("crash_recovery");

    let ops = SafeFileOperationsEngine::new(sandbox.db.clone(), &sandbox.hime_dir).unwrap();

    // Create a dangling staged operation and a dummy staging file
    let staging_file = ops.staging_dir().join("op-crash-001_file.staging");
    fs::write(&staging_file, b"uncommitted temporary data").unwrap();

    let dummy_op = hime_storage::FileOperationRecord {
        id: "op-crash-001".to_string(),
        op_type: "move".to_string(),
        source_path: "C:\\mock\\file.txt".to_string(),
        destination_path: Some("C:\\mock\\dest.txt".to_string()),
        backup_path: None,
        source_hash: Some("mockhash".to_string()),
        dest_hash: None,
        actor: "agent".to_string(),
        approval_id: None,
        status: "staged".to_string(),
        started_at: 1000,
        completed_at: None,
        error_message: None,
    };
    sandbox.db.record_file_operation(&dummy_op).unwrap();

    // Verify it is uncommitted
    assert_eq!(sandbox.db.get_uncommitted_file_operations().unwrap().len(), 1);

    // Run recovery
    let recovered = ops.recover_uncommitted_operations().unwrap();
    assert_eq!(recovered, 1);

    // Staging file must be cleaned
    assert!(!staging_file.exists());

    // Status in DB must be rolled_back
    let updated_op = sandbox.db.get_file_operation("op-crash-001").unwrap().unwrap();
    assert_eq!(updated_op.status, "rolled_back");
}

#[tokio::test]
async fn test_storage_e2e_security_path_validation_and_malicious_rejection() {
    let sandbox = TestSandbox::new("security_paths");

    let ops = SafeFileOperationsEngine::new(sandbox.db.clone(), &sandbox.hime_dir).unwrap();

    // 1. Traversal rejection
    let traversal = Path::new("C:\\Users\\Satoshi\\Documents\\..\\..\\Windows\\System32");
    assert!(ops.validate_path(traversal).is_err());

    // 2. Windows reserved name
    let con_path = sandbox.root.join("CON.txt");
    assert!(ops.validate_path(&con_path).is_err());

    let nul_path = sandbox.root.join("NUL");
    assert!(ops.validate_path(&nul_path).is_err());

    // 3. System folder denial
    let sys_path = Path::new("C:\\Windows\\System32\\cmd.exe");
    assert!(ops.validate_path(sys_path).is_err());
}

#[tokio::test]
async fn test_storage_e2e_smart_organizer_rules_matching_and_proposal() {
    let sandbox = TestSandbox::new("smart_organizer");

    let doc = sandbox.create_file("Unsorted/kuliah_ai_tugas1.pdf", b"Tugas Makalah AI");
    let target_dir = sandbox.root.join("Kuliah/AI");

    // Create active rule
    let rule = FileRuleRecord {
        id: "rule-kuliah-ai".to_string(),
        name: "Kuliah AI Rule".to_string(),
        pattern: "*.pdf".to_string(),
        target_dir: target_dir.to_string_lossy().to_string(),
        naming_template: Some("makalah_{name}.{ext}".to_string()),
        action: "propose".to_string(),
        is_active: true,
        created_at: 1000,
        updated_at: 1000,
    };
    sandbox.db.create_file_rule(&rule).unwrap();

    let indexer = IndexerService::new(sandbox.db.clone());
    indexer.scan_directory(&sandbox.root).unwrap();

    let ops = Arc::new(SafeFileOperationsEngine::new(sandbox.db.clone(), &sandbox.hime_dir).unwrap());
    let organizer = SmartOrganizer::new(sandbox.db.clone(), ops.clone());

    let proposals = organizer.evaluate_proposals(&doc.parent().unwrap().to_string_lossy()).unwrap();
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].rule_id, "rule-kuliah-ai");
    assert!(proposals[0].suggested_destination.contains("makalah_kuliah_ai_tugas1.pdf"));

    // Execute proposal
    let exec_res = organizer.execute_proposal(&proposals[0], "user", None).unwrap();
    assert_eq!(exec_res.status, "committed");

    let organized_path = PathBuf::from(&proposals[0].suggested_destination);
    assert!(organized_path.exists());
    assert!(!doc.exists());
}

#[tokio::test]
async fn test_storage_e2e_download_intelligence_workflow() {
    let sandbox = TestSandbox::new("download_intelligence");

    let ops = Arc::new(SafeFileOperationsEngine::new(sandbox.db.clone(), &sandbox.hime_dir).unwrap());
    let download_intel = DownloadIntelligence::new(sandbox.db.clone(), ops.clone());

    // 1. In-progress download
    let incomplete = sandbox.create_file("Downloads/large_video.mp4.crdownload", b"incomplete chunks");
    let r1 = download_intel.inspect_download(&incomplete).unwrap();
    assert!(matches!(r1.status, DownloadStatus::InProgress));

    // 2. Exact duplicate download
    let content = b"Identical file content";
    let existing = sandbox.create_file("Documents/contract.pdf", content);
    let indexer = IndexerService::new(sandbox.db.clone());
    indexer.scan_directory(&sandbox.root).unwrap();

    let dup_dl = sandbox.create_file("Downloads/contract_download.pdf", content);
    let r2 = download_intel.inspect_download(&dup_dl).unwrap();
    match r2.status {
        DownloadStatus::ExactDuplicate { existing_path } => {
            assert_eq!(PathBuf::from(&existing_path), existing);
            assert!(r2.notification_prompt.is_some());
        }
        _ => panic!("Expected ExactDuplicate, got {:?}", r2.status),
    }

    // 3. Replace and archive old
    let new_dl = sandbox.create_file("Downloads/contract_v2.pdf", b"New contract version with revisions");
    let rep_res = download_intel.replace_and_archive_old(&new_dl, &existing, "user").unwrap();
    assert_eq!(rep_res.status, "committed");

    // Existing file should now contain new content
    assert_eq!(fs::read_to_string(&existing).unwrap(), "New contract version with revisions");
}

#[tokio::test]
async fn test_storage_e2e_backup_snapshot_and_manifest_verification() {
    let sandbox = TestSandbox::new("backup_verification");

    sandbox.create_file("SourceData/file1.txt", b"First text file");
    sandbox.create_file("SourceData/sub/file2.txt", b"Second nested text file");

    let backup_svc = BackupService::new(sandbox.db.clone());
    let src_dir = sandbox.root.join("SourceData");
    let dst_dir = sandbox.root.join("Backups/Snapshot1");

    let backup_rec = backup_svc.create_backup("bak-test-1", &src_dir, &dst_dir).unwrap();
    assert_eq!(backup_rec.file_count, 2);
    assert!(backup_rec.verified_at.is_some());

    // Verify backup integrity
    assert!(backup_svc.verify_backup("bak-test-1").unwrap());

    // Tamper with backup file
    let corrupted_file = dst_dir.join("file1.txt");
    fs::write(&corrupted_file, b"HACKED CORRUPTED CONTENT").unwrap();

    // Verify backup must detect tampering
    assert!(!backup_svc.verify_backup("bak-test-1").unwrap());
}

#[tokio::test]
async fn test_storage_e2e_core_server_rpc_endpoints() {
    let server = CoreServer::new_in_memory("test-auth-token".to_string()).unwrap();

    // storage.rules.create
    let rule_json = serde_json::json!({
        "id": "rule-rpc-01",
        "name": "Images to Gallery",
        "pattern": "*.png",
        "target_dir": "C:\\Pictures",
        "naming_template": "{name}.{ext}",
        "action": "propose",
        "is_active": true,
        "created_at": 1000,
        "updated_at": 1000
    });

    let req_create = JsonRpcRequest::new(
        "rpc-1",
        "storage.rules.create",
        Some(serde_json::json!({ "rule": rule_json })),
    );
    let res_create = server.dispatch_rpc(req_create).await;
    assert!(res_create.error.is_none());

    // storage.rules.list
    let req_list = JsonRpcRequest::new("rpc-2", "storage.rules.list", None);
    let res_list = server.dispatch_rpc(req_list).await;
    assert!(res_list.error.is_none());
    let rules = res_list.result.unwrap();
    assert_eq!(rules.as_array().unwrap().len(), 1);

    // storage.analysis
    let req_analysis = JsonRpcRequest::new(
        "rpc-3",
        "storage.analysis",
        Some(serde_json::json!({ "top_n": 5 })),
    );
    let res_analysis = server.dispatch_rpc(req_analysis).await;
    assert!(res_analysis.error.is_none());

    // storage.duplicates
    let req_dups = JsonRpcRequest::new("rpc-4", "storage.duplicates", None);
    let res_dups = server.dispatch_rpc(req_dups).await;
    assert!(res_dups.error.is_none());

    // storage.operations.list
    let req_ops = JsonRpcRequest::new(
        "rpc-5",
        "storage.operations.list",
        Some(serde_json::json!({ "limit": 10 })),
    );
    let res_ops = server.dispatch_rpc(req_ops).await;
    assert!(res_ops.error.is_none());
}
