use std::path::Path;

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::{
    database,
    library::{BookRow, ReadingState},
    pdf, storage,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendStatus {
    app_name: &'static str,
    backend: &'static str,
    platform_targets: [&'static str; 3],
    database: &'static str,
    indexing: &'static str,
    sync: &'static str,
    message: &'static str,
}

#[tauri::command]
pub fn backend_status() -> BackendStatus {
    BackendStatus {
        app_name: "Librik",
        backend: "rust-tauri-v2",
        platform_targets: ["macOS", "Windows", "Linux"],
        database: "sqlite",
        indexing: "planned",
        sync: "planned",
        message: "Frontend-to-Rust command boundary is connected.",
    }
}

#[tauri::command]
pub fn list_books(app: AppHandle) -> Result<Vec<BookRow>, String> {
    let connection = database::open(&app)?;
    storage::reconcile(&app, &connection)?;
    database::list(&connection).map(|documents| {
        documents
            .into_iter()
            .map(|mut document| {
                if !Path::new(&document.file_path).is_file() {
                    document.status = "Missing file".into();
                }
                BookRow::from(document)
            })
            .collect()
    })
}

#[tauri::command]
pub async fn import_pdf(app: AppHandle) -> Result<Option<BookRow>, String> {
    let selected = app
        .dialog()
        .file()
        .add_filter("PDF", &["pdf"])
        .blocking_pick_file();
    let Some(selected) = selected else {
        return Ok(None);
    };
    let path = selected.into_path().map_err(|error| error.to_string())?;
    let mut document = pdf::inspect(&path)?;
    let connection = database::open(&app)?;
    document.file_path = storage::store(&app, &path, &document.fingerprint)?
        .to_string_lossy()
        .into_owned();
    database::import(&connection, &document)
        .map(BookRow::from)
        .map(Some)
}

#[tauri::command]
pub fn save_reading_state(app: AppHandle, state: ReadingState) -> Result<(), String> {
    database::save_reading_state(&database::open(&app)?, &state)
}
