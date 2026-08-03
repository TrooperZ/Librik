use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::{database, pdf};

pub fn store(app: &AppHandle, source: &Path, fingerprint: &str) -> Result<PathBuf, String> {
    let vault = vault_directory(app)?;
    store_in(&vault, source, fingerprint)
}

// ponytail: this synchronous startup scan is enough until large libraries make it measurable.
pub fn reconcile(app: &AppHandle, connection: &Connection) -> Result<(), String> {
    let vault = vault_directory(app)?;
    for record in database::file_records(connection)? {
        let current = PathBuf::from(&record.file_path);
        if current.starts_with(&vault) && current.is_file() {
            continue;
        }

        let source = if current.is_file() {
            Some(current)
        } else {
            record
                .original_path
                .map(PathBuf::from)
                .filter(|path| path.is_file())
        };
        let Some(source) = source else {
            continue;
        };

        let managed = store_in(&vault, &source, &record.fingerprint)?;
        let metadata = fs::metadata(&managed).map_err(|error| error.to_string())?;
        let modified_at = metadata
            .modified()
            .ok()
            .map(DateTime::<Utc>::from)
            .map(|date| date.timestamp())
            .unwrap_or_else(|| Utc::now().timestamp());
        database::update_file_location(
            connection,
            &record.id,
            &managed,
            metadata.len(),
            modified_at,
        )?;
    }
    Ok(())
}

fn vault_directory(app: &AppHandle) -> Result<PathBuf, String> {
    let vault = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("vault");
    fs::create_dir_all(&vault).map_err(|error| error.to_string())?;
    Ok(vault)
}

fn store_in(vault: &Path, source: &Path, fingerprint: &str) -> Result<PathBuf, String> {
    let hash = fingerprint
        .strip_prefix("sha256:")
        .filter(|hash| hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| "invalid PDF fingerprint".to_string())?;
    fs::create_dir_all(vault).map_err(|error| error.to_string())?;
    let target = vault.join(format!("{hash}.pdf"));

    if target.exists() {
        return if pdf::fingerprint(&target)? == fingerprint {
            Ok(target)
        } else {
            Err(format!(
                "Managed file {} does not match its fingerprint",
                target.display()
            ))
        };
    }

    let temporary = vault.join(format!(".{hash}.{}.part", Uuid::new_v4()));
    let result = (|| {
        fs::copy(source, &temporary).map_err(|error| error.to_string())?;
        File::open(&temporary)
            .and_then(|file| file.sync_all())
            .map_err(|error| error.to_string())?;
        if pdf::fingerprint(&temporary)? != fingerprint {
            return Err("PDF changed while it was being imported".into());
        }
        fs::rename(&temporary, &target).map_err(|error| error.to_string())?;
        Ok(target.clone())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_copy_is_verified_and_deduplicated() {
        let root = std::env::temp_dir().join(format!("librik-vault-{}", Uuid::new_v4()));
        let source = root.join("source.pdf");
        let vault = root.join("vault");
        fs::create_dir_all(&root).unwrap();
        fs::write(&source, b"%PDF-1.4\nsmoke test\n%%EOF\n").unwrap();
        let fingerprint = pdf::fingerprint(&source).unwrap();

        let first = store_in(&vault, &source, &fingerprint).unwrap();
        let second = store_in(&vault, &source, &fingerprint).unwrap();
        assert_eq!(first, second);
        assert_eq!(fs::read(first).unwrap(), fs::read(source).unwrap());
        assert_eq!(fs::read_dir(&vault).unwrap().count(), 1);

        fs::remove_dir_all(root).unwrap();
    }
}
