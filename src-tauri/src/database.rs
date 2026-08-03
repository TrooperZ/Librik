use std::{fs, path::Path, time::Duration};

use rusqlite::{params, Connection, OptionalExtension, Row};
use tauri::{AppHandle, Manager};

use crate::library::{Document, FileRecord, ReadingState};

const SCHEMA_VERSION: u32 = 2;
const SELECT_DOCUMENTS: &str = "
    SELECT id, title, creator, doc_type, status, source, progress_percent, pages,
           file_size_bytes, file_path, original_path, description, tags, added_at,
           modified_at, fingerprint, current_page, accent
    FROM documents";

const CREATE_DOCUMENTS: &str = "
    CREATE TABLE documents (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        creator TEXT NOT NULL,
        doc_type TEXT NOT NULL,
        status TEXT NOT NULL,
        source TEXT NOT NULL,
        progress_percent INTEGER NOT NULL CHECK(progress_percent BETWEEN 0 AND 100),
        pages INTEGER NOT NULL CHECK(pages > 0),
        file_size_bytes INTEGER NOT NULL CHECK(file_size_bytes >= 0),
        file_path TEXT NOT NULL,
        original_path TEXT,
        description TEXT NOT NULL,
        tags TEXT NOT NULL,
        added_at INTEGER NOT NULL,
        modified_at INTEGER NOT NULL,
        fingerprint TEXT NOT NULL UNIQUE,
        current_page INTEGER NOT NULL CHECK(current_page > 0),
        accent TEXT NOT NULL
    );";

pub fn open(app: &AppHandle) -> Result<Connection, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let path = directory.join("librik.sqlite3");
    let mut connection = Connection::open(&path).map_err(|error| error.to_string())?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|error| error.to_string())?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| error.to_string())?;
    backup_legacy(&connection, &path)?;
    migrate(&mut connection)?;
    Ok(connection)
}

fn backup_legacy(connection: &Connection, path: &Path) -> Result<(), String> {
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if version >= SCHEMA_VERSION || !table_exists(connection).map_err(|error| error.to_string())? {
        return Ok(());
    }

    let backup = path.with_file_name("librik.pre-v2.sqlite3");
    if !backup.exists() {
        connection
            .execute_batch("PRAGMA wal_checkpoint(FULL);")
            .map_err(|error| error.to_string())?;
        fs::copy(path, backup).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn migrate(connection: &mut Connection) -> Result<(), String> {
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    if version > SCHEMA_VERSION {
        return Err(format!(
            "Library schema {version} is newer than this Librik build supports"
        ));
    }

    let has_legacy_table = table_exists(connection).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    if has_legacy_table {
        transaction
            .execute_batch(&format!(
                "ALTER TABLE documents RENAME TO documents_legacy;
                 {CREATE_DOCUMENTS}
                 INSERT INTO documents (
                    id, title, creator, doc_type, status, source, progress_percent,
                    pages, file_size_bytes, file_path, original_path, description,
                    tags, added_at, modified_at, fingerprint, current_page, accent
                 )
                 SELECT id, title, creator, doc_type, status, source,
                    CASE
                        WHEN CAST(REPLACE(progress, '%', '') AS INTEGER) BETWEEN 0 AND 100
                        THEN CAST(REPLACE(progress, '%', '') AS INTEGER)
                        ELSE 0
                    END,
                    pages, 0, file_path, file_path, description, tags,
                    CAST(strftime('%s', 'now') AS INTEGER),
                    CAST(strftime('%s', 'now') AS INTEGER),
                    fingerprint, current_page, accent
                 FROM documents_legacy;
                 DROP TABLE documents_legacy;"
            ))
            .map_err(|error| error.to_string())?;
    } else {
        transaction
            .execute_batch(CREATE_DOCUMENTS)
            .map_err(|error| error.to_string())?;
    }
    transaction
        .pragma_update(None, "user_version", SCHEMA_VERSION)
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())
}

fn table_exists(connection: &Connection) -> rusqlite::Result<bool> {
    connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'documents'",
            [],
            |_| Ok(true),
        )
        .optional()
        .map(|value| value.unwrap_or(false))
}

pub fn import(connection: &Connection, document: &Document) -> Result<Document, String> {
    let tags = serde_json::to_string(&document.tags).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO documents (
                id, title, creator, doc_type, status, source, progress_percent,
                pages, file_size_bytes, file_path, original_path, description,
                tags, added_at, modified_at, fingerprint, current_page, accent
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                       ?13, ?14, ?15, ?16, ?17, ?18)
             ON CONFLICT(fingerprint) DO UPDATE SET
                file_size_bytes = excluded.file_size_bytes,
                file_path = excluded.file_path,
                original_path = excluded.original_path,
                modified_at = excluded.modified_at",
            params![
                document.id,
                document.title,
                document.creator,
                document.doc_type,
                document.status,
                document.source,
                document.progress_percent,
                document.pages,
                document.file_size_bytes,
                document.file_path,
                document.original_path,
                document.description,
                tags,
                document.added_at,
                document.modified_at,
                document.fingerprint,
                document.current_page,
                document.accent,
            ],
        )
        .map_err(|error| error.to_string())?;

    connection
        .query_row(
            &format!("{SELECT_DOCUMENTS} WHERE fingerprint = ?1"),
            [&document.fingerprint],
            row_to_document,
        )
        .map_err(|error| error.to_string())
}

pub fn list(connection: &Connection) -> Result<Vec<Document>, String> {
    let mut statement = connection
        .prepare(&format!(
            "{SELECT_DOCUMENTS} ORDER BY added_at DESC, rowid DESC"
        ))
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], row_to_document)
        .map_err(|error| error.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())
}

pub fn file_records(connection: &Connection) -> Result<Vec<FileRecord>, String> {
    let mut statement = connection
        .prepare("SELECT id, file_path, original_path, fingerprint FROM documents")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                file_path: row.get(1)?,
                original_path: row.get(2)?,
                fingerprint: row.get(3)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())
}

pub fn update_file_location(
    connection: &Connection,
    id: &str,
    file_path: &Path,
    file_size_bytes: u64,
    modified_at: i64,
) -> Result<(), String> {
    connection
        .execute(
            "UPDATE documents
             SET file_path = ?1, file_size_bytes = ?2, modified_at = ?3
             WHERE id = ?4",
            params![
                file_path.to_string_lossy(),
                file_size_bytes,
                modified_at,
                id
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn save_reading_state(connection: &Connection, state: &ReadingState) -> Result<(), String> {
    let progress = state
        .progress
        .trim_end_matches('%')
        .parse::<u8>()
        .map_err(|_| "progress must be between 0% and 100%".to_string())?;
    if progress > 100 || state.current_page == 0 {
        return Err("invalid reading progress".into());
    }
    if state.status.trim().is_empty()
        || state.status.len() > 32
        || state
            .tags
            .iter()
            .any(|tag| tag.trim().is_empty() || tag.len() > 64)
    {
        return Err("invalid reading state".into());
    }

    let tags = serde_json::to_string(&state.tags).map_err(|error| error.to_string())?;
    let updated = connection
        .execute(
            "UPDATE documents
             SET status = ?1, progress_percent = ?2, tags = ?3,
                 current_page = MIN(?4, pages)
             WHERE id = ?5",
            params![state.status, progress, tags, state.current_page, state.id],
        )
        .map_err(|error| error.to_string())?;
    if updated == 1 {
        Ok(())
    } else {
        Err("document not found".into())
    }
}

fn row_to_document(row: &Row<'_>) -> rusqlite::Result<Document> {
    let raw_tags: String = row.get(12)?;
    let tags = serde_json::from_str(&raw_tags).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(12, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(Document {
        id: row.get(0)?,
        title: row.get(1)?,
        creator: row.get(2)?,
        doc_type: row.get(3)?,
        status: row.get(4)?,
        source: row.get(5)?,
        progress_percent: row.get(6)?,
        pages: row.get(7)?,
        file_size_bytes: row.get(8)?,
        file_path: row.get(9)?,
        original_path: row.get(10)?,
        description: row.get(11)?,
        tags,
        added_at: row.get(13)?,
        modified_at: row.get(14)?,
        fingerprint: row.get(15)?,
        current_page: row.get(16)?,
        accent: row.get(17)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_library_migrates_and_survives_reopen() {
        let path = std::env::temp_dir().join(format!("librik-{}.sqlite3", uuid::Uuid::new_v4()));
        {
            let mut connection = Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE documents (
                        id TEXT PRIMARY KEY, title TEXT NOT NULL, creator TEXT NOT NULL,
                        doc_type TEXT NOT NULL, status TEXT NOT NULL, source TEXT NOT NULL,
                        progress TEXT NOT NULL, pages INTEGER NOT NULL, file_size TEXT NOT NULL,
                        file_path TEXT NOT NULL, description TEXT NOT NULL, tags TEXT NOT NULL,
                        added_at TEXT NOT NULL, modified_at TEXT NOT NULL,
                        fingerprint TEXT NOT NULL UNIQUE, current_page INTEGER NOT NULL,
                        accent TEXT NOT NULL
                    );
                    INSERT INTO documents VALUES (
                        'one', 'A PDF', 'Author', 'PDF', 'Queued', 'Inbox', '25%', 12,
                        '1.0 MB', '/tmp/a.pdf', 'Imported PDF.', '[\"inbox\"]',
                        'Aug 2, 2026', 'Aug 2, 2026', 'sha256:abc', 3, '#2563eb'
                    );",
                )
                .unwrap();
            migrate(&mut connection).unwrap();
            let version: u32 = connection
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .unwrap();
            assert_eq!(version, SCHEMA_VERSION);
            assert_eq!(list(&connection).unwrap()[0].progress_percent, 25);
        }

        let mut connection = Connection::open(&path).unwrap();
        migrate(&mut connection).unwrap();
        save_reading_state(
            &connection,
            &ReadingState {
                id: "one".into(),
                status: "Reading".into(),
                progress: "50%".into(),
                tags: vec!["inbox".into(), "reading".into()],
                current_page: 6,
            },
        )
        .unwrap();
        let documents = list(&connection).unwrap();
        assert_eq!(documents[0].progress_percent, 50);
        assert_eq!(documents[0].tags, ["inbox", "reading"]);
        drop(connection);
        fs::remove_file(path).unwrap();
    }
}
