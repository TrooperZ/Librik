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
        .invoke_handler(tauri::generate_handler![commands::backend_status])
        .run(tauri::generate_context!())
        .expect("failed to run Librik desktop shell");
}
