use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use chrono::{DateTime, Utc};
use lopdf::{decode_text_string, Document as PdfDocument};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::library::Document;

pub fn inspect(path: &Path) -> Result<Document, String> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("pdf"))
    {
        return Err("Please select a PDF file.".into());
    }

    let path = path.canonicalize().map_err(|error| error.to_string())?;
    let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
    if !metadata.is_file() {
        return Err("Please select a PDF file.".into());
    }

    let document = PdfDocument::load(&path)
        .map_err(|error| format!("Could not read PDF metadata: {error}"))?;
    let fallback_title = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled PDF")
        .to_string();
    let now = Utc::now().timestamp();
    let modified_at = metadata
        .modified()
        .ok()
        .map(DateTime::<Utc>::from)
        .map(|date| date.timestamp())
        .unwrap_or(now);

    Ok(Document {
        id: Uuid::new_v4().to_string(),
        title: info(&document, b"Title").unwrap_or(fallback_title),
        creator: info(&document, b"Author").unwrap_or_else(|| "Unknown Author".into()),
        doc_type: "PDF".into(),
        status: "Queued".into(),
        source: "Inbox".into(),
        progress_percent: 0,
        pages: document.get_pages().len() as u32,
        file_size_bytes: metadata.len(),
        file_path: path.to_string_lossy().into_owned(),
        original_path: Some(path.to_string_lossy().into_owned()),
        description: info(&document, b"Subject").unwrap_or_else(|| "Imported PDF.".into()),
        tags: vec!["inbox".into()],
        added_at: now,
        modified_at,
        fingerprint: fingerprint(&path)?,
        current_page: 1,
        accent: "#2563eb".into(),
    })
}

fn info(document: &PdfDocument, key: &[u8]) -> Option<String> {
    let id = document.trailer.get(b"Info").ok()?.as_reference().ok()?;
    let value = document.get_dictionary(id).ok()?.get(key).ok()?;
    decode_text_string(value)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn fingerprint(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Object};

    #[test]
    fn imported_metadata_comes_from_a_real_pdf() {
        let path = std::env::temp_dir().join(format!("librik-{}.pdf", Uuid::new_v4()));
        let mut pdf = PdfDocument::with_version("1.5");
        let pages_id = pdf.new_object_id();
        let page_id = pdf.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        });
        pdf.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => vec![page_id.into()],
                "Count" => 1,
            }),
        );
        let catalog_id = pdf.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        let info_id = pdf.add_object(dictionary! {
            "Title" => Object::string_literal("Foundation Test"),
            "Author" => Object::string_literal("Librik"),
        });
        pdf.trailer.set("Root", catalog_id);
        pdf.trailer.set("Info", info_id);
        pdf.save(&path).unwrap();

        let document = inspect(&path).unwrap();
        assert_eq!(document.title, "Foundation Test");
        assert_eq!(document.creator, "Librik");
        assert_eq!(document.pages, 1);
        assert!(document.fingerprint.starts_with("sha256:"));

        fs::remove_file(path).unwrap();
    }
}
