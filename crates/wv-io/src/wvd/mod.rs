//! Native format: JSON with a version header.

mod convert;
mod dto;
mod schema;
mod variants;

use wv_core::Document;

use crate::IoError;

pub fn decode(bytes: &[u8]) -> Result<Document, IoError> {
    let file: dto::File =
        serde_json::from_slice(bytes).map_err(|e| IoError::Parse(e.to_string()))?;
    file.into_document()
}

pub fn encode(doc: &Document) -> Result<Vec<u8>, IoError> {
    serde_json::to_vec_pretty(&dto::File::from_document(doc))
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
        e.doc.paragraphs[1].style.cell = Some(wv_core::Cell::at(3, 1, 2));
        e.doc.paragraphs[0].style.list = Some(wv_core::ListKind::Number);
        e.doc.paragraphs[0].style.indent = 2;
        e.doc.paragraphs[0].style.line_spacing = Some(1.5);
        e.doc.paragraphs[0].style.first_line = 27;
        e.doc.paragraphs[1].style.page_break = true;
        e.doc.paragraphs[0].style.left_extra = 5;
        e.doc.paragraphs[0].style.tabs = vec![36, 108];
        e.doc.paragraphs[0].style.space_before = Some(6);
        e.doc.paragraphs[0].style.space_after = Some(0);
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
        doc.header = wv_core::Band::new("Шапка").aligned(Align::Left);
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

    #[test]
    fn floating_pictures_round_trip() {
        use wv_core::{Float, ImageFormat, Wrap};
        let mut e = Editor::default();
        e.insert_image(ImageFormat::Png, vec![1, 2, 3], 100.0, 50.0);
        let float = Float {
            x: 12.5,
            y: 30.0,
            wrap: Wrap::Behind,
        };
        e.sel = wv_core::Selection::caret(wv_core::Position::new(0, 0));
        e.set_image_geometry(80.0, 40.0, Some(float));
        let doc = decode(&encode(&e.doc).unwrap()).unwrap();
        assert_eq!(doc, e.doc);
        assert_eq!(doc.paragraphs[0].style.image.unwrap().float, Some(float));
    }

    #[test]
    fn rotated_and_cropped_pictures_round_trip() {
        use wv_core::{Crop, ImageFormat};
        let mut e = Editor::default();
        e.insert_image(ImageFormat::Png, vec![1, 2, 3], 100.0, 50.0);
        e.sel = wv_core::Selection::caret(wv_core::Position::new(0, 0));
        let crop = Crop {
            left: 0.1,
            top: 0.0,
            right: 0.2,
            bottom: 0.05,
        };
        e.drag_image_crop(crop, 70.0, 47.5, None, true);
        e.turn_image(123.0);
        let doc = decode(&encode(&e.doc).unwrap()).unwrap();
        assert_eq!(doc, e.doc);
        assert_eq!(doc.paragraphs[0].style.image.unwrap().angle, 123.0);
    }

    #[test]
    fn footnotes_round_trip() {
        let mut e = Editor::new(Document::from_text("Text with a note"));
        e.sel = wv_core::Selection::caret(wv_core::Position::new(0, 4));
        e.insert_footnote();
        e.insert_text("The note");
        let doc = decode(&encode(&e.doc).unwrap()).unwrap();
        assert_eq!(doc, e.doc);
        assert_eq!(doc.note_numbers().len(), 1);
    }

    #[test]
    fn page_setup_round_trips() {
        let mut doc = Document::from_text("x");
        doc.page = wv_core::PageSpec::a4()
            .with_paper(1)
            .with_margins(wv_core::Margins::Narrow)
            .with_columns(2);
        assert_eq!(decode(&encode(&doc).unwrap()).unwrap(), doc);
    }

    #[test]
    fn gutter_and_band_distances_round_trip() {
        let mut doc = Document::from_text("x");
        doc.page =
            wv_core::PageSpec::custom(600.0, 900.0, [90.0, 60.0, 75.0, 45.0], [30.0, 20.0, 25.0]);
        assert_eq!(decode(&encode(&doc).unwrap()).unwrap(), doc);
    }

    #[test]
    fn band_variants_round_trip() {
        let mut doc = Document::from_text("x");
        doc.variants = wv_core::BandVariants {
            different_first: true,
            odd_even: true,
            first: (wv_core::Band::new("Титул"), wv_core::Band::default()),
            even: (
                wv_core::Band::new("Чётная").aligned(Align::Right),
                wv_core::Band::new("{page}"),
            ),
        };
        assert_eq!(decode(&encode(&doc).unwrap()).unwrap(), doc);
    }

    #[test]
    fn formatted_band_words_round_trip() {
        let mut doc = Document::from_text("x");
        let mut head = wv_core::Band::new("Head bold end");
        head.restyle(5, 9, |s| {
            s.bold = true;
            s.size_pt = Some(14);
            s.color = Some(0x336699);
        });
        doc.header = head.clone();
        doc.variants.first = (head, wv_core::Band::default());
        doc.variants.different_first = true;
        let back = decode(&encode(&doc).unwrap()).unwrap();
        assert_eq!(back, doc);
        assert_eq!(back.header.runs().len(), 3);
    }

    #[test]
    fn table_look_and_shading_round_trip() {
        let mut e = wv_core::Editor::new(Document::from_text("x"));
        e.insert_table(2, 2);
        let table = e.table_info().unwrap().table;
        let look = wv_core::TableLook {
            widths: vec![0.3, 0.5],
            indent: 0.1,
            rows: vec![0.0, 40.0],
        };
        e.set_table_look(table, look.clone());
        e.set_cell_shade(Some(0xFFEE99));
        let doc = e.doc.clone();
        let back = decode(&encode(&doc).unwrap()).unwrap();
        assert_eq!(back, doc);
        assert_eq!(back.table_looks[&table], look);
    }

    #[test]
    fn merged_cells_round_trip() {
        let mut e = wv_core::Editor::new(Document::from_text("x"));
        e.insert_table(3, 3);
        let first = e.table_info().unwrap().first;
        e.select_range(
            wv_core::Position::new(first, 0),
            wv_core::Position::new(first + 4, 0),
        );
        e.merge_cells();
        let doc = e.doc.clone();
        let back = decode(&encode(&doc).unwrap()).unwrap();
        assert_eq!(back, doc);
        let merged = back.paragraphs[first].style.cell.unwrap();
        assert_eq!((merged.rowspan, merged.colspan), (2, 2));
    }
}
