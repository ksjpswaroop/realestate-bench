//! RealEstate-Bench — Tauri core library.
//!
//! This is the heart of the desktop app. It owns:
//!   - SQLite database (FTS5-enabled, WAL mode)
//!   - The cactus-needle sidecar process
//!   - IPC commands exposed to the React frontend
//!
//! See ARCHITECTURE §3 for the full design.

mod commands;
pub mod db;
mod errors;
mod extract;
mod paths;
mod user_schemas;

pub use errors::{AppError, AppResult};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize SQLite at the OS-appropriate app data dir
            let db_path = paths::db_path(&app.handle())?;
            log::info!("RealEstate-Bench starting up; db = {}", db_path.display());
            db::init(&db_path)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::extract_document,
            commands::list_schemas,
            commands::get_schema_fields,
            commands::list_extractions,
            commands::get_extraction,
            commands::delete_extraction,
            commands::record_edit,
            commands::export_excel,
            commands::export_json,
            commands::export_csv,
            commands::export_db,
            commands::import_db,
            commands::wipe_db,
            commands::db_info,
            commands::search_extractions,
            // Phase 2: user-defined schemas + tags
            commands::list_user_schemas,
            commands::get_user_schema,
            commands::create_user_schema,
            commands::update_user_schema,
            commands::delete_user_schema,
            commands::add_extraction_tag,
            commands::remove_extraction_tag,
            commands::get_extraction_tags,
            commands::list_all_tags,
            commands::filter_extractions_by_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
