mod cache;
mod commands;
mod database;
mod indexing;
mod library;
mod pdf;
mod storage;
mod sync;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::backend_status,
            commands::list_books,
            commands::import_pdf,
            commands::save_reading_state,
            commands::list_folders,
            commands::create_folder,
            commands::rename_folder,
            commands::move_folder,
            commands::delete_folder,
            commands::set_folder_color,
            commands::set_document_folder,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Librik desktop shell");
}
