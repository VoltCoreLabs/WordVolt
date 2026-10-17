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
        e.doc.paragraphs[0].runs[0].style.font = Some("Georgia".into());
        e.doc.paragraphs[0].runs[0].style.strike = true;
        e.doc.paragraphs[0].runs[0].style.caps = true;
        e.doc.paragraphs[0].runs[0].style.mono = true;
        e.doc.paragraphs[0].runs[0].style.spoiler = true;
        e.doc.paragraphs[0].runs[0].style.link = Some("https://example.com/?a=1&b=2".into());
        e.doc.paragraphs[1].style.quote = true;
        e.doc.paragraphs[0].style.named = wv_core::NamedStyle::Heading2;
        e.doc.paragraphs[1].style.cell = Some(wv_core::Cell {
            table: 3,
            row: 1,
            col: 2,
        });
        e.doc.paragraphs[0].style.list = Some(wv_core::ListKind::Number);
        e.doc.paragraphs[0].style.indent = 2;
        e.doc.paragraphs[0].style.line_spacing = Some(1.5);
        e.doc.paragraphs[0].runs[0].style.highlight = Some(0xFFFF00);
        e.doc.paragraphs[0].runs[0].style.script = wv_core::Script::Sub;
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

    #[test]
    fn header_and_footer_round_trip() {
        let mut doc = Document::from_text("x");
        doc.header = wv_core::Band::new("Шапка");
        doc.footer = wv_core::Band::new("{page}");
        assert_eq!(decode(&encode(&doc).unwrap()).unwrap(), doc);
    }

    #[test]
    fn pictures_round_trip() {
        use wv_core::{ImageFormat, ImageRef};
        let mut e = Editor::default();
        e.insert_image(ImageFormat::Jpeg, vec![9, 8, 7, 0, 255], 100.0, 50.0);
        let doc = decode(&encode(&e.doc).unwrap()).unwrap();
        assert_eq!(doc, e.doc);
        let image: ImageRef = doc.paragraphs[0].style.image.unwrap();
        assert_eq!((image.width, image.height), (100.0, 50.0));
    }
}
