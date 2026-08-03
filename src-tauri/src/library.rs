use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub creator: String,
    pub doc_type: String,
    pub status: String,
    pub source: String,
    pub progress_percent: u8,
    pub pages: u32,
    pub file_size_bytes: u64,
    pub file_path: String,
    pub original_path: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub added_at: i64,
    pub modified_at: i64,
    pub fingerprint: String,
    pub current_page: u32,
    pub accent: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookRow {
    pub id: String,
    pub title: String,
    pub creator: String,
    #[serde(rename = "type")]
    pub doc_type: String,
    pub status: String,
    pub source: String,
    pub progress: String,
    pub pages: u32,
    pub file_size: String,
    pub file_path: String,
    pub description: String,
    pub tags: Vec<String>,
    pub added_at: String,
    pub modified_at: String,
    pub fingerprint: String,
    pub current_page: u32,
    pub accent: String,
}

impl From<Document> for BookRow {
    fn from(document: Document) -> Self {
        Self {
            id: document.id,
            title: document.title,
            creator: document.creator,
            doc_type: document.doc_type,
            status: document.status,
            source: document.source,
            progress: format!("{}%", document.progress_percent),
            pages: document.pages,
            file_size: format_size(document.file_size_bytes),
            file_path: document.file_path,
            description: document.description,
            tags: document.tags,
            added_at: format_date(document.added_at),
            modified_at: format_date(document.modified_at),
            fingerprint: document.fingerprint,
            current_page: document.current_page,
            accent: document.accent,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileRecord {
    pub id: String,
    pub file_path: String,
    pub original_path: Option<String>,
    pub fingerprint: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingState {
    pub id: String,
    pub status: String,
    pub progress: String,
    pub tags: Vec<String>,
    pub current_page: u32,
}

fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    if bytes >= MB as u64 {
        format!("{:.1} MB", bytes as f64 / MB)
    } else if bytes >= KB as u64 {
        format!("{:.1} KB", bytes as f64 / KB)
    } else {
        format!("{bytes} B")
    }
}

fn format_date(timestamp: i64) -> String {
    Local
        .timestamp_opt(timestamp, 0)
        .single()
        .map(|date| date.format("%b %-d, %Y").to_string())
        .unwrap_or_else(|| "Unknown".into())
}
