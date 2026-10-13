//! Integration test: run the sidecar exactly the way Tauri will.
//! This is a real binary that we already proved works via pytest.
//! Here we spawn it with the same env and parse the same JSON.

use std::process::Stdio;

#[test]
fn sidecar_extraction_smoke() {
    // The Tauri sidecar binary lives in src-tauri/bin/ next to the manifest
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let sidecar = manifest_dir.join("bin/realestate-extract-aarch64-apple-darwin");
    assert!(sidecar.exists(), "sidecar binary missing: {sidecar:?}");

    let sample = manifest_dir.join("../tests/sample_invoice.txt");
    assert!(sample.exists(), "sample missing: {sample:?}");

    let req = serde_json::json!({
        "doc_path": sample.to_string_lossy(),
        "schema_id": "invoice",
    });

    let mut child = std::process::Command::new(&sidecar)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("PYTHONPATH", "")
        .env("NEEDLE_TELEMETRY", "0")
        .env("HF_HUB_DISABLE_PROGRESS_BARS", "1")
        .env("TRANSFORMERS_VERBOSITY", "error")
        .spawn()
        .expect("spawn sidecar");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(serde_json::to_string(&req).unwrap().as_bytes())
        .unwrap();

    let out = child.wait_with_output().expect("wait sidecar");
    assert!(out.status.success(), "sidecar failed: stderr={}", String::from_utf8_lossy(&out.stderr));

    let resp: serde_json::Value = serde_json::from_slice(&out.stdout).expect("sidecar returned invalid JSON");
    assert!(resp["extraction"].is_object(), "extraction missing: {resp}");
    assert!(resp["mean_confidence"].is_number());
    assert!(resp["extraction"]["total"].is_number());
    println!("✓ sidecar invocation: total={:?} conf={:?} latency={}ms",
        resp["extraction"]["total"],
        resp["mean_confidence"],
        resp["latency_ms"],
    );
}
