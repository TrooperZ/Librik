use std::{fs, path::Path, time::Duration};

use rusqlite::{params, Connection, OptionalExtension, Row};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::library::{Document, FileRecord, FolderRow, ReadingState};

const SCHEMA_VERSION: u32 = 6;
const SELECT_DOCUMENTS: &str = "
    SELECT id, title, creator, doc_type, status, source, progress_percent, pages,
           file_size_bytes, file_path, original_path, description, tags, added_at,
           modified_at, fingerprint, current_page, accent, folder_id
    FROM documents";

const CREATE_DOCUMENTS: &str = "
    CREATE TABLE documents (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        creator TEXT NOT NULL,
        doc_type TEXT NOT NULL,
        status TEXT NOT NULL,
        source TEXT NOT NULL,
        folder_id TEXT,
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
    backup_before_migration(&connection, &path)?;
    migrate(&mut connection)?;
    Ok(connection)
}

fn backup_before_migration(connection: &Connection, path: &Path) -> Result<(), String> {
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if version >= SCHEMA_VERSION || !table_exists(connection).map_err(|error| error.to_string())? {
        return Ok(());
    }

    let backup = path.with_file_name(format!("librik.pre-v{SCHEMA_VERSION}.sqlite3"));
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

    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    if version < 2 {
        if table_exists(&transaction).map_err(|error| error.to_string())? {
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
    }
    if version < 3 {
        transaction
            .execute_batch(
                "CREATE TABLE folders (
                    name TEXT PRIMARY KEY COLLATE NOCASE,
                    created_at INTEGER NOT NULL
                 );
                 INSERT OR IGNORE INTO folders (name, created_at)
                 SELECT DISTINCT source, CAST(strftime('%s', 'now') AS INTEGER)
                 FROM documents WHERE TRIM(source) <> '';
                 INSERT OR IGNORE INTO folders (name, created_at)
                 VALUES ('Inbox', CAST(strftime('%s', 'now') AS INTEGER));",
            )
            .map_err(|error| error.to_string())?;
    }
    if version < 4 {
        transaction
            .execute_batch(
                "ALTER TABLE folders RENAME TO folders_v3;
                 CREATE TABLE folders (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    parent_id TEXT REFERENCES folders(id) ON DELETE CASCADE,
                    color TEXT,
                    created_at INTEGER NOT NULL
                 );
                 CREATE UNIQUE INDEX folders_unique_name
                 ON folders(COALESCE(parent_id, ''), name COLLATE NOCASE);
                 INSERT INTO folders (id, name, parent_id, color, created_at)
                 SELECT lower(hex(randomblob(16))), name, NULL, NULL, created_at
                 FROM folders_v3;",
            )
            .map_err(|error| error.to_string())?;
        if version >= 2 {
            transaction
                .execute_batch("ALTER TABLE documents ADD COLUMN folder_id TEXT;")
                .map_err(|error| error.to_string())?;
        }
        transaction
            .execute_batch(
                "UPDATE documents
                 SET folder_id = (
                    SELECT id FROM folders
                    WHERE folders.name = documents.source COLLATE NOCASE
                    AND folders.parent_id IS NULL
                    LIMIT 1
                 );
                 DROP TABLE folders_v3;",
            )
            .map_err(|error| error.to_string())?;
    }
    if version < 5 {
        transaction
            .execute_batch(
                "ALTER TABLE folders ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
                 UPDATE folders
                 SET sort_order = (
                    SELECT COUNT(*) FROM folders AS sibling
                    WHERE sibling.parent_id IS folders.parent_id
                    AND (
                        lower(sibling.name) < lower(folders.name)
                        OR (lower(sibling.name) = lower(folders.name) AND sibling.id < folders.id)
                    )
                 );",
            )
            .map_err(|error| error.to_string())?;
    }
    if version < 6 {
        transaction
            .execute_batch(
                "UPDATE folders AS child
                 SET name = substr(child.name, 1, 48) || ' (' || substr(child.id, 1, 8) || ')'
                 WHERE child.parent_id IN (
                    SELECT id FROM folders WHERE parent_id IS NULL AND name = 'Inbox' COLLATE NOCASE
                 )
                 AND EXISTS (
                    SELECT 1 FROM folders AS root
                    WHERE root.parent_id IS NULL AND root.name = child.name COLLATE NOCASE
                 );
                 UPDATE folders
                 SET parent_id = NULL,
                     sort_order = sort_order + COALESCE(
                        (SELECT MAX(root.sort_order) + 1 FROM folders AS root WHERE root.parent_id IS NULL),
                        0
                     )
                 WHERE parent_id IN (
                    SELECT id FROM folders WHERE parent_id IS NULL AND name = 'Inbox' COLLATE NOCASE
                 );
                 UPDATE documents
                 SET folder_id = NULL, source = ''
                 WHERE folder_id IN (
                    SELECT id FROM folders WHERE parent_id IS NULL AND name = 'Inbox' COLLATE NOCASE
                 );
                 DELETE FROM folders WHERE parent_id IS NULL AND name = 'Inbox' COLLATE NOCASE;",
            )
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
                id, title, creator, doc_type, status, source, folder_id,
                progress_percent, pages, file_size_bytes, file_path, original_path,
                description, tags, added_at, modified_at, fingerprint, current_page, accent
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                       ?13, ?14, ?15, ?16, ?17, ?18, ?19)
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
                document.folder_id,
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

pub fn list_folders(connection: &Connection) -> Result<Vec<FolderRow>, String> {
    let mut statement = connection
        .prepare(
            "SELECT folders.id, folders.name, folders.parent_id, folders.color,
                    folders.sort_order, COUNT(documents.id)
             FROM folders
             LEFT JOIN documents ON documents.folder_id = folders.id
             GROUP BY folders.id
             ORDER BY folders.sort_order, folders.name COLLATE NOCASE, folders.id",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(FolderRow {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                color: row.get(3)?,
                sort_order: row.get(4)?,
                document_count: row.get(5)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())
}

pub fn create_folder(
    connection: &Connection,
    name: &str,
    parent_id: Option<&str>,
) -> Result<FolderRow, String> {
    let name = validate_folder_name(name)?;
    if let Some(parent_id) = parent_id {
        require_folder(connection, parent_id)?;
    }
    let id = Uuid::new_v4().to_string();
    let sort_order = connection
        .query_row(
            "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM folders WHERE parent_id IS ?1",
            params![parent_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO folders (id, name, parent_id, color, created_at, sort_order)
             VALUES (?1, ?2, ?3, NULL, CAST(strftime('%s', 'now') AS INTEGER), ?4)",
            params![id, name, parent_id, sort_order],
        )
        .map_err(|error| {
            if error.to_string().contains("UNIQUE constraint failed") {
                "a folder with that name already exists".to_string()
            } else {
                error.to_string()
            }
        })?;
    Ok(FolderRow {
        id,
        name,
        parent_id: parent_id.map(str::to_string),
        color: None,
        sort_order,
        document_count: 0,
    })
}

pub fn rename_folder(connection: &Connection, id: &str, name: &str) -> Result<(), String> {
    require_editable_folder(connection, id)?;
    let name = validate_folder_name(name)?;
    connection
        .execute(
            "UPDATE folders SET name = ?1 WHERE id = ?2",
            params![name, id],
        )
        .map_err(folder_write_error)?;
    connection
        .execute(
            "UPDATE documents SET source = ?1 WHERE folder_id = ?2",
            params![name, id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn move_folder(
    connection: &mut Connection,
    id: &str,
    parent_id: Option<&str>,
    before_id: Option<&str>,
) -> Result<(), String> {
    require_editable_folder(connection, id)?;
    if let Some(parent_id) = parent_id {
        require_folder(connection, parent_id)?;
        let creates_cycle: bool = connection
            .query_row(
                "WITH RECURSIVE subtree(id) AS (
                    SELECT ?1 UNION ALL
                    SELECT folders.id FROM folders JOIN subtree ON folders.parent_id = subtree.id
                 )
                 SELECT EXISTS(SELECT 1 FROM subtree WHERE id = ?2)",
                params![id, parent_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if creates_cycle {
            return Err("a folder cannot be moved into itself or its subfolders".into());
        }
    }
    let old_parent_id: Option<String> = connection
        .query_row("SELECT parent_id FROM folders WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .map_err(|error| error.to_string())?;
    let mut destination_ids = ordered_folder_ids(connection, parent_id)?;
    destination_ids.retain(|folder_id| folder_id != id);
    let insert_at = match before_id {
        Some(before_id) => destination_ids
            .iter()
            .position(|folder_id| folder_id == before_id)
            .ok_or_else(|| "invalid folder reorder target".to_string())?,
        None => destination_ids.len(),
    };
    destination_ids.insert(insert_at, id.to_string());

    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "UPDATE folders SET parent_id = ?1 WHERE id = ?2",
            params![parent_id, id],
        )
        .map_err(folder_write_error)?;
    write_folder_order(&transaction, &destination_ids)?;
    if old_parent_id.as_deref() != parent_id {
        let source_ids = ordered_folder_ids(&transaction, old_parent_id.as_deref())?;
        write_folder_order(&transaction, &source_ids)?;
    }
    transaction.commit().map_err(|error| error.to_string())
}

fn ordered_folder_ids(
    connection: &Connection,
    parent_id: Option<&str>,
) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id FROM folders WHERE parent_id IS ?1
             ORDER BY sort_order, name COLLATE NOCASE, id",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![parent_id], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())
}

fn write_folder_order(connection: &Connection, ids: &[String]) -> Result<(), String> {
    for (sort_order, id) in ids.iter().enumerate() {
        connection
            .execute(
                "UPDATE folders SET sort_order = ?1 WHERE id = ?2",
                params![sort_order as i64, id],
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn set_folder_color(
    connection: &Connection,
    id: &str,
    color: Option<&str>,
) -> Result<(), String> {
    require_folder(connection, id)?;
    let color = color.map(validate_folder_color).transpose()?;
    connection
        .execute(
            "UPDATE folders SET color = ?1 WHERE id = ?2",
            params![color, id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn delete_folder(connection: &mut Connection, id: &str) -> Result<(), String> {
    require_editable_folder(connection, id)?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "WITH RECURSIVE subtree(id) AS (
                SELECT ?1 UNION ALL
                SELECT folders.id FROM folders JOIN subtree ON folders.parent_id = subtree.id
             )
             UPDATE documents SET folder_id = NULL, source = ''
             WHERE folder_id IN (SELECT id FROM subtree)",
            [id],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute("DELETE FROM folders WHERE id = ?1", [id])
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())
}

pub fn set_document_folder(
    connection: &Connection,
    id: &str,
    folder_id: Option<&str>,
) -> Result<Document, String> {
    let folder_name = folder_id
        .map(|folder_id| require_folder(connection, folder_id))
        .transpose()?
        .unwrap_or_default();
    let updated = connection
        .execute(
            "UPDATE documents SET source = ?1, folder_id = ?2 WHERE id = ?3",
            params![folder_name, folder_id, id],
        )
        .map_err(|error| error.to_string())?;
    if updated != 1 {
        return Err("document not found".into());
    }
    connection
        .query_row(
            &format!("{SELECT_DOCUMENTS} WHERE id = ?1"),
            [id],
            row_to_document,
        )
        .map_err(|error| error.to_string())
}

fn require_folder(connection: &Connection, id: &str) -> Result<String, String> {
    connection
        .query_row("SELECT name FROM folders WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "folder not found".into())
}

fn require_editable_folder(connection: &Connection, id: &str) -> Result<(), String> {
    require_folder(connection, id).map(|_| ())
}

fn folder_write_error(error: rusqlite::Error) -> String {
    if error.to_string().contains("UNIQUE constraint failed") {
        "a folder with that name already exists here".into()
    } else {
        error.to_string()
    }
}

fn validate_folder_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty()
        || name.len() > 64
        || name
            .chars()
            .any(|character| character.is_control() || character == '/' || character == '\\')
    {
        return Err("folder names must be 1-64 characters without slashes".into());
    }
    Ok(name.to_string())
}

fn validate_folder_color(color: &str) -> Result<String, String> {
    if color.len() == 7
        && color.starts_with('#')
        && color[1..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        Ok(color.to_ascii_lowercase())
    } else {
        Err("invalid folder color".into())
    }
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
        folder_id: row.get(18)?,
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
    fn removing_inbox_preserves_user_folders() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(&format!(
                "PRAGMA foreign_keys = ON;
                 {CREATE_DOCUMENTS}
                 CREATE TABLE folders (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    parent_id TEXT REFERENCES folders(id) ON DELETE CASCADE,
                    color TEXT,
                    created_at INTEGER NOT NULL,
                    sort_order INTEGER NOT NULL DEFAULT 0
                 );
                 CREATE UNIQUE INDEX folders_unique_name
                 ON folders(COALESCE(parent_id, ''), name COLLATE NOCASE);
                 INSERT INTO folders VALUES ('inbox', 'Inbox', NULL, NULL, 1, 0);
                 INSERT INTO folders VALUES ('project', 'Project', NULL, '#db4b9b', 2, 1);
                 INSERT INTO folders VALUES ('chapter', 'Chapter', 'project', NULL, 3, 0);
                 INSERT INTO folders VALUES ('loose', 'Loose', 'inbox', NULL, 4, 0);
                 PRAGMA user_version = 5;"
            ))
            .unwrap();

        migrate(&mut connection).unwrap();
        let folders = list_folders(&connection).unwrap();
        assert!(!folders.iter().any(|folder| folder.name == "Inbox"));
        assert_eq!(
            folders
                .iter()
                .find(|folder| folder.id == "chapter")
                .unwrap()
                .parent_id
                .as_deref(),
            Some("project")
        );
        assert_eq!(
            folders
                .iter()
                .find(|folder| folder.id == "loose")
                .unwrap()
                .parent_id,
            None
        );
    }

    #[test]
    fn legacy_library_migrates_and_survives_reopen() {
        let path = std::env::temp_dir().join(format!("librik-{}.sqlite3", uuid::Uuid::new_v4()));
        {
            let mut connection = Connection::open(&path).unwrap();
            connection
                .execute_batch("PRAGMA foreign_keys = ON;")
                .unwrap();
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
            assert!(list_folders(&connection).unwrap().is_empty());
            assert_eq!(list(&connection).unwrap()[0].folder_id, None);
        }

        let mut connection = Connection::open(&path).unwrap();
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();
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
        let research = create_folder(&connection, "Research", None).unwrap();
        let chapter = create_folder(&connection, "Chapter 1", Some(&research.id)).unwrap();
        assert!(move_folder(&mut connection, &research.id, Some(&chapter.id), None).is_err());
        move_folder(&mut connection, &chapter.id, None, Some(&research.id)).unwrap();
        assert_eq!(
            list_folders(&connection)
                .unwrap()
                .iter()
                .find(|folder| folder.id == chapter.id)
                .unwrap()
                .parent_id,
            None
        );
        let root_ids = list_folders(&connection)
            .unwrap()
            .into_iter()
            .filter(|folder| folder.parent_id.is_none())
            .map(|folder| folder.id)
            .collect::<Vec<_>>();
        assert!(
            root_ids.iter().position(|id| id == &chapter.id).unwrap()
                < root_ids.iter().position(|id| id == &research.id).unwrap()
        );
        move_folder(&mut connection, &chapter.id, Some(&research.id), None).unwrap();
        rename_folder(&connection, &chapter.id, "Introduction").unwrap();
        set_folder_color(&connection, &research.id, Some("#ff8800")).unwrap();
        let moved = set_document_folder(&connection, "one", Some(&research.id)).unwrap();
        assert_eq!(moved.source, "Research");
        assert_eq!(moved.folder_id.as_deref(), Some(research.id.as_str()));
        let folders = list_folders(&connection).unwrap();
        assert_eq!(
            folders
                .iter()
                .find(|folder| folder.id == chapter.id)
                .unwrap()
                .name,
            "Introduction"
        );
        assert_eq!(
            folders
                .iter()
                .find(|folder| folder.id == research.id)
                .unwrap()
                .color
                .as_deref(),
            Some("#ff8800")
        );
        delete_folder(&mut connection, &research.id).unwrap();
        let moved_back = list(&connection).unwrap();
        assert_eq!(moved_back[0].source, "");
        assert_eq!(moved_back[0].folder_id, None);
        assert!(!list_folders(&connection)
            .unwrap()
            .iter()
            .any(|folder| folder.id == research.id || folder.id == chapter.id));
        drop(connection);
        fs::remove_file(path).unwrap();
    }
}
