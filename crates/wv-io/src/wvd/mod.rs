//! Native format: JSON with a version header.

mod schema;

use wv_core::Document;

use crate::IoError;

pub fn decode(bytes: &[u8]) -> Result<Document, IoError> {
    let file: schema::File =
        serde_json::from_slice(bytes).map_err(|e| IoError::Parse(e.to_string()))?;
    file.into_document()
}

pub fn encode(doc: &Document) -> Result<Vec<u8>, IoError> {
    serde_json::to_vec_pretty(&schema::File::from_document(doc))
        .map_err(|e| IoError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use wv_core::{Align, Editor};

    use super::*;

    #[test]
    fn round_trip_keeps_formatting() {
        let mut e = Editor::default();
        e.insert_text("hello world\nsecond");
        e.sel.anchor = wv_core::Position::new(0, 0);
        e.sel.head = wv_core::Position::new(0, 5);
        e.toggle_bold();
        e.toggle_underline();
        e.sel.anchor = wv_core::Position::new(1, 0);
        e.sel.head = wv_core::Position::new(1, 3);
        e.set_align(Align::Center);
        let back = decode(&encode(&e.doc).unwrap()).unwrap();
        assert_eq!(back, e.doc);
    }

    #[test]
    fn rejects_garbage_and_future_versions() {
        assert!(matches!(decode(b"not json"), Err(IoError::Parse(_))));
        assert!(matches!(
            decode(br#"{"format":"other","version":1,"paragraphs":[]}"#),
            Err(IoError::Parse(_))
        ));
        assert!(matches!(
            decode(br#"{"format":"wvd","version":99,"paragraphs":[]}"#),
            Err(IoError::Parse(_))
        ));
    }

    #[test]
    fn empty_file_gives_one_empty_paragraph() {
        let doc = decode(br#"{"format":"wvd","version":1,"paragraphs":[]}"#).unwrap();
        assert_eq!(doc, Document::default());
    }
}
