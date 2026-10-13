//! SQLite storage layer.
//!
//! Schema (matches ARCHITECTURE §3.5):
//!   documents       — one row per unique file (keyed by sha256)
//!   extractions     — one row per (document, schema) extraction
//!   edits           — append-only log of user edits to extracted fields
//!   fts_extractions — FTS5 virtual table mirroring the JSON content
//!
//! Migrations use `PRAGMA user_version`; idempotent.

use std::path::Path;
use std::sync::Mutex;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};

/// Public row types. Used by `commands.rs` to return data to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRow {
    pub id: i64,
    pub path: String,
    pub sha256: String,
    pub file_name: String,
    pub file_size: i64,
    pub page_count: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRow {
    pub id: i64,
    pub document_id: i64,
    pub schema_id: String,
    pub schema_version: String,
    pub extraction_json: String, // JSON blob
    pub confidences_json: String, // JSON blob
    pub mean_confidence: f64,
    pub latency_ms: i64,
    pub model_version: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
    /// Path of the source document (joined in for the list view)
    pub document_path: String,
    pub document_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRow {
    pub id: i64,
    pub extraction_id: i64,
    pub field_path: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub edited_at: String,
}

// ---------------------------------------------------------------------------
// Connection (singleton, behind a Mutex — Tauri calls commands on multiple threads)
// ---------------------------------------------------------------------------

use std::sync::OnceLock;
static DB: OnceLock<Mutex<Connection>> = OnceLock::new();

pub fn init(db_path: &Path) -> AppResult<()> {
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    migrate(&conn)?;
    DB.set(Mutex::new(conn))
        .map_err(|_| AppError::Internal("db already initialized".into()))?;
    Ok(())
}

fn with_conn<F, T>(f: F) -> AppResult<T>
where
    F: FnOnce(&mut Connection) -> AppResult<T>,
{
    let mutex = DB.get().ok_or_else(|| AppError::Internal("db not initialized".into()))?;
    let mut conn = mutex.lock().map_err(|_| AppError::Internal("db lock poisoned".into()))?;
    f(&mut conn)
}

// ---------------------------------------------------------------------------
// Migrations
// ---------------------------------------------------------------------------

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS documents (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    path         TEXT NOT NULL,
    sha256       TEXT NOT NULL UNIQUE,
    file_name    TEXT NOT NULL,
    file_size    INTEGER NOT NULL,
    page_count   INTEGER,
    created_at   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS extractions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id     INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    schema_id       TEXT NOT NULL,
    schema_version  TEXT NOT NULL,
    extraction_json TEXT NOT NULL,
    confidences_json TEXT NOT NULL,
    mean_confidence REAL NOT NULL,
    latency_ms      INTEGER NOT NULL,
    model_version   TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    deleted_at      TEXT,
    UNIQUE (document_id, schema_id, schema_version)
);
CREATE INDEX IF NOT EXISTS idx_extractions_doc    ON extractions(document_id);
CREATE INDEX IF NOT EXISTS idx_extractions_schema  ON extractions(schema_id);
CREATE INDEX IF NOT EXISTS idx_extractions_created ON extractions(created_at DESC);

CREATE TABLE IF NOT EXISTS edits (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    extraction_id INTEGER NOT NULL REFERENCES extractions(id) ON DELETE CASCADE,
    field_path    TEXT NOT NULL,
    old_value     TEXT NOT NULL,
    new_value     TEXT NOT NULL,
    edited_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_edits_extraction ON edits(extraction_id);

CREATE VIRTUAL TABLE IF NOT EXISTS fts_extractions USING fts5(
    document_name,
    schema_id,
    extraction_text,
    tokenize = 'porter unicode61'
);
"#;

fn migrate(conn: &Connection) -> AppResult<()> {
    let user_version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if user_version == 0 {
        log::info!("running initial schema migration");
        conn.execute_batch(SCHEMA)?;
        conn.pragma_update(None, "user_version", 1)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Document operations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NewDocument<'a> {
    pub path: &'a str,
    pub sha256: &'a str,
    pub file_name: &'a str,
    pub file_size: i64,
    pub page_count: Option<i64>,
}

pub fn upsert_document(doc: NewDocument<'_>) -> AppResult<i64> {
    with_conn(|c| {
        // Try to insert; on conflict (sha256), return the existing id.
        c.execute(
            "INSERT OR IGNORE INTO documents (path, sha256, file_name, file_size, page_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                doc.path,
                doc.sha256,
                doc.file_name,
                doc.file_size,
                doc.page_count,
                Utc::now().to_rfc3339(),
            ],
        )?;
        let id: i64 = c.query_row(
            "SELECT id FROM documents WHERE sha256 = ?1",
            params![doc.sha256],
            |r| r.get(0),
        )?;
        Ok(id)
    })
}

// ---------------------------------------------------------------------------
// Extraction operations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NewExtraction<'a> {
    pub document_id: i64,
    pub schema_id: &'a str,
    pub schema_version: &'a str,
    pub extraction_json: &'a str,
    pub confidences_json: &'a str,
    pub mean_confidence: f64,
    pub latency_ms: i64,
    pub model_version: &'a str,
}

pub fn insert_extraction(e: NewExtraction<'_>) -> AppResult<i64> {
    with_conn(|c| {
        c.execute(
            "INSERT INTO extractions
             (document_id, schema_id, schema_version, extraction_json,
              confidences_json, mean_confidence, latency_ms, model_version, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                e.document_id,
                e.schema_id,
                e.schema_version,
                e.extraction_json,
                e.confidences_json,
                e.mean_confidence,
                e.latency_ms,
                e.model_version,
                Utc::now().to_rfc3339(),
            ],
        )?;
        let id: i64 = c.last_insert_rowid();

        // Mirror into FTS5
        let document_name: String = c.query_row(
            "SELECT file_name FROM documents WHERE id = ?1",
            params![e.document_id],
            |r| r.get(0),
        )?;
        let extraction_text = flatten_json_for_fts(e.extraction_json);
        c.execute(
            "INSERT INTO fts_extractions (rowid, document_name, schema_id, extraction_text)
             VALUES (?1, ?2, ?3, ?4)",
            params![id, document_name, e.schema_id, extraction_text],
        )?;
        Ok(id)
    })
}

/// Flatten a JSON object to a single text string for FTS5 indexing.
fn flatten_json_for_fts(json_str: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(json_str).unwrap_or(serde_json::Value::Null);
    let mut out = String::new();
    fn walk(v: &serde_json::Value, out: &mut String) {
        match v {
            serde_json::Value::String(s) => { out.push_str(s); out.push(' '); }
            serde_json::Value::Number(n) => { out.push_str(&n.to_string()); out.push(' '); }
            serde_json::Value::Bool(b) => { out.push_str(if *b { "true" } else { "false" }); out.push(' '); }
            serde_json::Value::Array(a) => { for x in a { walk(x, out); } }
            serde_json::Value::Object(m) => { for (_, val) in m { walk(val, out); } }
            serde_json::Value::Null => {}
        }
    }
    walk(&v, &mut out);
    out
}

pub fn list_extractions(
    schema_filter: Option<&str>,
    search: Option<&str>,
    limit: i64,
) -> AppResult<Vec<ExtractionRow>> {
    with_conn(|c| {
        // Build SQL conditionally; for safety, bind user input via params
        let mut sql = String::from(
            "SELECT e.id, e.document_id, e.schema_id, e.schema_version, e.extraction_json,
                    e.confidences_json, e.mean_confidence, e.latency_ms, e.model_version,
                    e.created_at, e.deleted_at, d.path, d.file_name
             FROM extractions e
             JOIN documents d ON d.id = e.document_id
             WHERE e.deleted_at IS NULL",
        );
        let mut binds: Vec<String> = vec![];

        if let Some(s) = schema_filter {
            sql.push_str(" AND e.schema_id = ?");
            binds.push(s.to_string());
        }
        if search.is_some() {
            sql.push_str(
                " AND e.id IN (SELECT rowid FROM fts_extractions WHERE fts_extractions MATCH ?)",
            );
        }
        sql.push_str(" ORDER BY e.created_at DESC LIMIT ?");
        binds.push(limit.to_string());

        let mut stmt = c.prepare(&sql)?;
        let params_fn: Vec<&dyn rusqlite::ToSql> = binds.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        let rows = stmt.query_map(
            rusqlite::params_from_iter(params_fn.iter().copied()),
            |r| Ok(map_extraction(r)),
        )?;
        let mut out = vec![];
        for row in rows {
            out.push(row?);
        }
        // We need to also bind the search term in a separate execution, since
        // the original logic mixed SQL building with ToSql binding. Re-execute
        // if search is present:
        if let Some(q) = search {
            let mut stmt = c.prepare(
                "SELECT e.id, e.document_id, e.schema_id, e.schema_version, e.extraction_json,
                        e.confidences_json, e.mean_confidence, e.latency_ms, e.model_version,
                        e.created_at, e.deleted_at, d.path, d.file_name
                 FROM extractions e
                 JOIN documents d ON d.id = e.document_id
                 WHERE e.deleted_at IS NULL
                   AND e.schema_id LIKE ?1
                   AND e.id IN (SELECT rowid FROM fts_extractions WHERE fts_extractions MATCH ?2)
                 ORDER BY e.created_at DESC LIMIT ?3",
            )?;
            let schema = schema_filter.unwrap_or("%");
            let mut rows = stmt.query_map(params![schema, q, limit], |r| Ok(map_extraction(r)))?;
            out.clear();
            for row in rows {
                out.push(row?);
            }
        }
        Ok(out)
    })
}

fn map_extraction(r: &rusqlite::Row<'_>) -> ExtractionRow {
    ExtractionRow {
        id: r.get(0).unwrap(),
        document_id: r.get(1).unwrap(),
        schema_id: r.get(2).unwrap(),
        schema_version: r.get(3).unwrap(),
        extraction_json: r.get(4).unwrap(),
        confidences_json: r.get(5).unwrap(),
        mean_confidence: r.get(6).unwrap(),
        latency_ms: r.get(7).unwrap(),
        model_version: r.get(8).unwrap(),
        created_at: r.get(9).unwrap(),
        deleted_at: r.get(10).ok(),
        document_path: r.get(11).unwrap(),
        document_name: r.get(12).unwrap(),
    }
}

pub fn get_extraction(id: i64) -> AppResult<ExtractionRow> {
    with_conn(|c| {
        let row: Option<ExtractionRow> = c
            .query_row(
                "SELECT e.id, e.document_id, e.schema_id, e.schema_version, e.extraction_json,
                        e.confidences_json, e.mean_confidence, e.latency_ms, e.model_version,
                        e.created_at, e.deleted_at, d.path, d.file_name
                 FROM extractions e
                 JOIN documents d ON d.id = e.document_id
                 WHERE e.id = ?1",
                params![id],
                |r| Ok(map_extraction(r)),
            )
            .optional()?;
        row.ok_or_else(|| AppError::NotFound(format!("extraction {id}")))
    })
}

pub fn soft_delete_extraction(id: i64) -> AppResult<()> {
    with_conn(|c| {
        let n = c.execute(
            "UPDATE extractions SET deleted_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id],
        )?;
        if n == 0 {
            return Err(AppError::NotFound(format!("extraction {id}")));
        }
        Ok(())
    })
}

pub fn record_edit(
    extraction_id: i64,
    field_path: &str,
    old_value: &serde_json::Value,
    new_value: &serde_json::Value,
) -> AppResult<i64> {
    with_conn(|c| {
        c.execute(
            "INSERT INTO edits (extraction_id, field_path, old_value, new_value, edited_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                extraction_id,
                field_path,
                serde_json::to_string(old_value)?,
                serde_json::to_string(new_value)?,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(c.last_insert_rowid())
    })
}

pub fn list_edits(extraction_id: i64) -> AppResult<Vec<EditRow>> {
    with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT id, extraction_id, field_path, old_value, new_value, edited_at
             FROM edits WHERE extraction_id = ?1 ORDER BY edited_at ASC",
        )?;
        let rows = stmt.query_map(params![extraction_id], |r| {
            let old: String = r.get(3)?;
            let new: String = r.get(4)?;
            Ok(EditRow {
                id: r.get(0)?,
                extraction_id: r.get(1)?,
                field_path: r.get(2)?,
                old_value: serde_json::from_str(&old).unwrap_or(serde_json::Value::Null),
                new_value: serde_json::from_str(&new).unwrap_or(serde_json::Value::Null),
                edited_at: r.get(5)?,
            })
        })?;
        let mut out = vec![];
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    })
}

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct DbInfo {
    pub path: String,
    pub document_count: i64,
    pub extraction_count: i64,
    pub edit_count: i64,
    pub size_bytes: i64,
}

pub fn db_info(path: &Path) -> AppResult<DbInfo> {
    with_conn(|c| {
        let doc_count: i64 =
            c.query_row("SELECT COUNT(*) FROM documents", [], |r| r.get(0))?;
        let ext_count: i64 =
            c.query_row("SELECT COUNT(*) FROM extractions", [], |r| r.get(0))?;
        let edit_count: i64 = c.query_row("SELECT COUNT(*) FROM edits", [], |r| r.get(0))?;
        let size_bytes = std::fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0);
        Ok(DbInfo {
            path: path.to_string_lossy().to_string(),
            document_count: doc_count,
            extraction_count: ext_count,
            edit_count: edit_count,
            size_bytes,
        })
    })
}

pub fn wipe() -> AppResult<()> {
    with_conn(|c| {
        c.execute("DELETE FROM edits", [])?;
        c.execute("DELETE FROM fts_extractions", [])?;
        c.execute("DELETE FROM extractions", [])?;
        c.execute("DELETE FROM documents", [])?;
        c.execute("VACUUM", [])?;
        Ok(())
    })
}
