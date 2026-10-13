//! App-wide error type. Implements `serde::Serialize` so Tauri can return
//! it as JSON to the frontend.

use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("sidecar failed: {0}")]
    Sidecar(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = s.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", &self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            AppError::Io(_) => "io",
            AppError::Sqlite(_) => "sqlite",
            AppError::Json(_) => "json",
            AppError::Tauri(_) => "tauri",
            AppError::Sidecar(_) => "sidecar",
            AppError::NotFound(_) => "not_found",
            AppError::InvalidRequest(_) => "invalid_request",
            AppError::Internal(_) => "internal",
            AppError::Zip(_) => "zip",
        }
    }
}
