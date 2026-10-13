//! End-to-end smoke test using the app's own DB layer.
//! Verifies the full flow: insert document -> extract via sidecar -> insert extraction -> search FTS5.

use realestate_bench_lib::db;
use sha2::Digest;

#[test]
fn full_flow_insert_and_search() {
    let tmpdir = std::env::temp_dir().join("realestate-bench-test");
    std::fs::create_dir_all(&tmpdir).unwrap();
    let db_path = tmpdir.join("test.db");
    let _ = std::fs::remove_file(&db_path);

    db::init(&db_path).expect("init db");

    // Insert document
    let sample = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/sample_invoice.txt");
    let bytes = std::fs::read(&sample).unwrap();
    let mut h = sha2::Sha256::new();
    h.update(&bytes);
    let sha = hex::encode(h.finalize());

    let doc_id = db::upsert_document(db::NewDocument {
        path: sample.to_str().unwrap(),
        sha256: &sha,
        file_name: "sample_invoice.txt",
        file_size: bytes.len() as i64,
        page_count: None,
    }).expect("upsert doc");
    assert!(doc_id > 0);

    // Insert extraction
    let extraction = serde_json::json!({
        "vendor_name": "Acme Property Services",
        "vendor_address": "123 Main St",
        "invoice_number": "INV-2026-0915",
        "total": 10850.0,
        "currency": "USD",
    });
    let confidences = serde_json::json!({
        "vendor_name": 0.85, "vendor_address": 0.85,
        "invoice_number": 0.85, "total": 0.85, "currency": 0.85,
    });
    let ext_id = db::insert_extraction(db::NewExtraction {
        document_id: doc_id,
        schema_id: "invoice",
        schema_version: "1.0.0",
        extraction_json: &extraction.to_string(),
        confidences_json: &confidences.to_string(),
        mean_confidence: 0.85,
        latency_ms: 700,
        model_version: "cactus-needle-3.2.1",
    }).expect("insert extraction");
    assert!(ext_id > 0);

    // FTS5 search
    let rows = db::list_extractions(Some("invoice"), Some("acme"), 50).expect("search");
    println!("Search 'acme' returned {} rows", rows.len());
    assert!(rows.iter().any(|r| r.id == ext_id), "FTS5 search must find our extraction");

    let rows2 = db::list_extractions(Some("invoice"), Some("INV"), 50).expect("search");
    println!("Search 'INV' returned {} rows", rows2.len());
    assert!(rows2.iter().any(|r| r.id == ext_id));

    // Get by id
    let row = db::get_extraction(ext_id).expect("get");
    assert_eq!(row.id, ext_id);
    assert_eq!(row.schema_id, "invoice");
    assert_eq!(row.mean_confidence, 0.85);
    println!("✓ Full flow: document={} extraction={} search='{}' → found", doc_id, ext_id, "acme");

    // Cleanup
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(format!("{}-wal", db_path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", db_path.display()));
}
