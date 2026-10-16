//! File formats: wvd, txt, md (docx, rtf, html, odt, pdf planned). See README.md.

mod error;
mod format;
mod md;
mod txt;
mod wvd;

use std::fs;
use std::path::Path;

pub use error::IoError;
pub use format::Format;
use wv_core::Document;

/// Reads a document; the format is chosen by the file extension.
pub fn load_file(path: &Path) -> Result<Document, IoError> {
    let format = Format::from_path(path).ok_or_else(|| IoError::Unsupported(extension(path)))?;
    let bytes = fs::read(path)?;
    match format {
        Format::Wvd => wvd::decode(&bytes),
        Format::Txt => Ok(txt::decode(&bytes)),
        Format::Md => Ok(md::decode(&txt::decode_text(&bytes))),
    }
}

/// Writes a document atomically (temp file in the same folder, then rename).
pub fn save_file(path: &Path, doc: &Document) -> Result<(), IoError> {
    let format = Format::from_path(path).ok_or_else(|| IoError::Unsupported(extension(path)))?;
    let bytes = match format {
        Format::Wvd => wvd::encode(doc)?,
        Format::Txt => txt::encode(doc),
        Format::Md => md::encode(doc).into_bytes(),
    };
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = path.with_file_name(format!(".{name}.wvtmp"));
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })?;
    Ok(())
}

fn extension(path: &Path) -> String {
    path.extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_every_format_from_disk() {
        let dir = std::env::temp_dir().join(format!("wv-io-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let doc = Document::from_text("Привет, мир\nsecond");
        for name in ["a.wvd", "a.txt", "a.md"] {
            let path = dir.join(name);
            save_file(&path, &doc).unwrap();
            assert_eq!(load_file(&path).unwrap().text(), doc.text(), "{name}");
        }
        assert!(matches!(
            save_file(&dir.join("a.xyz"), &doc),
            Err(IoError::Unsupported(_))
        ));
        assert!(!dir.join(".a.wvd.wvtmp").exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
