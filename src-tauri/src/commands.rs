use serde::Serialize;

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
        database: "planned",
        indexing: "planned",
        sync: "planned",
        message: "Frontend-to-Rust command boundary is connected.",
    }
}
