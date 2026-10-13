//! Filesystem paths used by the app (per FR-011: app data dir).

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::errors::{AppError, AppResult};

/// SQLite database location: `<app_data>/app.db`.
/// On macOS this resolves to `~/Library/Application Support/com.ksjpswaroop.realestate-bench/`.
pub fn db_path(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("could not resolve app_data_dir: {e}")))?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("app.db"))
}

/// Per-document cache directory (for OCR results, thumbnails).
pub fn cache_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| AppError::Internal(format!("could not resolve app_cache_dir: {e}")))?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
