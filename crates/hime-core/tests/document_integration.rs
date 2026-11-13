use hime_core::documents::artcraft::ArtcraftAdapter;
use hime_core::documents::operations::DocumentOperations;
use hime_core::documents::service::DocumentService;
use hime_core::documents::templates::TemplateRegistry;
use hime_core::server::CoreServer;
use hime_ipc::protocol::JsonRpcRequest;
use hime_storage::Database;
use std::fs;
use std::sync::Arc;

struct DocumentTestContext {
    db: Arc<Database>,
    service: DocumentService,
    temp_dir: std::path::PathBuf,
}

impl Drop for DocumentTestContext {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

fn setup_document_test_context() -> DocumentTestContext {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let temp_dir = std::env::temp_dir().join(format!(
        "hime_doc_test_{:x}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap();
    let service = DocumentService::new(db.clone(), &temp_dir).unwrap();

    DocumentTestContext { db, service, temp_dir }
}

#[test]
fn test_academic_id_template_specifications() {
    let spec = TemplateRegistry::academic_id();
    assert_eq!(spec.font_family, "Times New Roman");
    assert_eq!(spec.font_size_pt, 12.0);
    assert_eq!(spec.line_spacing, 1.5);
    assert_eq!(spec.margin_left_cm, 4.0);
    assert_eq!(spec.margin_right_cm, 3.0);
    assert_eq!(spec.margin_top_cm, 3.0);
    assert_eq!(spec.margin_bottom_cm, 3.0);
    assert!(spec.requires_table_of_contents);
}

#[tokio::test]
async fn test_create_academic_report_and_verification() {
    let ctx = setup_document_test_context();
    let output_file = ctx.temp_dir.join("skripsi_bab1.html");

    let markdown_content = r#"
# BAB I PENDAHULUAN
## 1.1 Latar Belakang
Perkembangan kecerdasan buatan telah mengubah paradigma interaksi manusia dengan komputer.
## 1.2 Rumusan Masalah
Bagaimana merancang arsitektur personal AI environment yang aman dan deterministik?
"#;

    let res = ctx
        .service
        .create_report_job(
            "job-doc-01",
            "Sistem Operasi Personal Berbasis AI",
            markdown_content,
            "Akademik-ID",
            &output_file,
        )
        .await
        .expect("report created");

    assert!(output_file.exists());
    assert!(res.file_size_bytes > 0);
    assert!(res.verification.is_verified);
    assert_eq!(res.verification.font_family, "Times New Roman");
    assert!(res.verification.has_table_of_contents);

    // Verify written file contains styling
    let written = fs::read_to_string(&output_file).unwrap();
    assert!(written.contains("font-family: 'Times New Roman'"));
    assert!(written.contains("margin: 3cm 3cm 3cm 4cm"));
    assert!(written.contains("DAFTAR ISI"));

    // Verify job was persisted in database
    let jobs = ctx.db.list_document_jobs().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, "completed");
}

#[test]
fn test_merge_and_compress_documents() {
    let ctx = setup_document_test_context();
    let doc1 = ctx.temp_dir.join("part1.txt");
    let doc2 = ctx.temp_dir.join("part2.txt");
    let merged_file = ctx.temp_dir.join("merged.pdf");

    fs::write(&doc1, b"Bab 1 Content Data\n\n\n").unwrap();
    fs::write(&doc2, b"Bab 2 Content Data\n\n\n").unwrap();

    // 1. Merge
    let merge_res = DocumentOperations::merge_documents(&[doc1.clone(), doc2], &merged_file).unwrap();
    assert!(merged_file.exists());
    assert_eq!(merge_res.verification.page_count, 2);

    // 2. Compress
    let compressed_file = ctx.temp_dir.join("compressed.txt");
    let compress_res = DocumentOperations::compress_document(&doc1, &compressed_file).unwrap();
    assert!(compressed_file.exists());
    assert!(compress_res.file_size_bytes <= fs::metadata(&doc1).unwrap().len());
}

#[test]
fn test_artcraft_discovery_adapter() {
    let disc = ArtcraftAdapter::discover();
    assert!(disc.supported_formats.contains(&"docx".to_string()));
    assert!(disc.supported_formats.contains(&"pdf".to_string()));
    assert!(!disc.limitations.is_empty());
}

#[tokio::test]
async fn test_document_core_server_rpc_endpoints() {
    let server = CoreServer::new_in_memory("test-token".to_string()).unwrap();

    // Test artcraft discover via RPC
    let req_art = JsonRpcRequest::new("rpc-art", "documents.artcraft.discover", None);
    let resp_art = server.dispatch_rpc(req_art).await;
    assert!(resp_art.error.is_none());
    let disc = resp_art.result.unwrap();
    assert!(disc["supported_formats"].as_array().unwrap().len() >= 4);

    // Test document jobs list
    let req_jobs = JsonRpcRequest::new("rpc-jobs", "documents.jobs.list", None);
    let resp_jobs = server.dispatch_rpc(req_jobs).await;
    assert!(resp_jobs.error.is_none());
    assert!(resp_jobs.result.unwrap().as_array().is_some());
}
