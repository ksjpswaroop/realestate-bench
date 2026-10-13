//! Sidecar invocation — spawn the Python `realestate-extract` process,
//! hand it a JSON request, parse the JSON response.

use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

use crate::errors::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
pub struct SidecarRequest {
    pub doc_path: String,
    pub schema_id: String,
    /// Optional: when `schema_id` refers to a user-defined schema, the Rust
    /// side looks up the schema definition in SQLite and passes the JSON
    /// here. The sidecar uses it to build a dynamic Pydantic class and
    /// extract against it. The sidecar echoes the schema_id back so the
    /// caller can correlate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_schema_json: Option<String>,
    /// Optional: human-readable name of the user schema. Stored alongside
    /// the extraction for display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_schema_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SidecarResponse {
    pub extraction: Option<serde_json::Value>,
    pub confidences: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub mean_confidence: f64,
    pub latency_ms: i64,
    pub model_version: String,
    pub schema_id: Option<String>,
    pub schema_version: Option<String>,
    #[serde(default)]
    pub doc_meta: Option<serde_json::Value>,
    pub error: Option<SidecarError>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SidecarError {
    pub code: String,
    pub message: String,
}

/// Run the sidecar and return its JSON response.
///
/// This function clears PYTHONPATH in the spawned environment so the venv
/// site-packages take precedence over any caller-injected paths.
pub async fn run(app: &AppHandle, req: SidecarRequest) -> AppResult<SidecarResponse> {
    // Confirm the file exists before spawning the sidecar; the sidecar
    // would otherwise surface a NOT_FOUND error and we'd lose the chance
    // to give a better message.
    if !Path::new(&req.doc_path).exists() {
        return Err(AppError::NotFound(format!("file not found: {}", req.doc_path)));
    }

    let shell = app.shell();
    let sidecar = shell
        .sidecar("realestate-extract")
        .map_err(|e| AppError::Sidecar(format!("failed to find sidecar: {e}")))?;

    let (mut rx, mut child) = sidecar
        .args([std::ffi::OsStr::new("")])
        .env("PYTHONPATH", "")
        .env("NEEDLE_TELEMETRY", "0")
        .env("HF_HUB_DISABLE_PROGRESS_BARS", "1")
        .env("TRANSFORMERS_VERBOSITY", "error")
        .spawn()
        .map_err(|e| AppError::Sidecar(format!("failed to spawn sidecar: {e}")))?;

    // Write the request to the sidecar's stdin
    let req_bytes = serde_json::to_vec(&req)?;
    child.write(&req_bytes).map_err(io_to_sidecar)?;
    child.write(b"\n").map_err(io_to_sidecar)?;
    drop(child); // close stdin so the sidecar's read() returns

    // Collect stdout, stderr
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(line) => stdout_bytes.extend_from_slice(&line),
            CommandEvent::Stderr(line) => stderr_bytes.extend_from_slice(&line),
            CommandEvent::Terminated(payload) => {
                if let Some(code) = payload.code {
                    if code != 0 {
                        let stderr = String::from_utf8_lossy(&stderr_bytes).to_string();
                        return Err(AppError::Sidecar(format!(
                            "sidecar exited with code {code}; stderr: {stderr}"
                        )));
                    }
                }
            }
            _ => {}
        }
    }

    let stdout = String::from_utf8_lossy(&stdout_bytes).to_string();
    if stdout.trim().is_empty() {
        let stderr = String::from_utf8_lossy(&stderr_bytes).to_string();
        return Err(AppError::Sidecar(format!(
            "sidecar produced no output; stderr: {stderr}"
        )));
    }
    let response: SidecarResponse = serde_json::from_str(stdout.trim())
        .map_err(|e| AppError::Sidecar(format!("sidecar output not valid JSON: {e}; raw={stdout}")))?;
    Ok(response)
}

fn io_to_sidecar(e: tauri_plugin_shell::Error) -> AppError {
    AppError::Sidecar(format!("io error talking to sidecar: {e}"))
}
