use std::io::{Cursor, Write};

use wv_core::{Align, Document, Editor, Position};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use super::*;

fn docx_with(document_xml: &str) -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("word/document.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(document_xml.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

#[test]
fn round_trip_keeps_text_styles_alignment_size_and_colour() {
    let mut e = Editor::default();
    e.insert_text("Привет <мир> & co\tTab\nsecond & last");
    e.sel.anchor = Position::new(0, 0);
    e.sel.head = Position::new(0, 6);
    e.toggle_bold();
    e.toggle_italic();
    e.sel.anchor = Position::new(1, 0);
    e.sel.head = Position::new(1, 6);
    e.toggle_underline();
    e.set_align(Align::Right);
    e.doc.paragraphs[1].runs[0].style.size_pt = Some(18);
    e.doc.paragraphs[1].runs[0].style.color = Some(0x2563EB);
    let back = decode(&encode(&e.doc).unwrap()).unwrap();
    assert_eq!(back, e.doc);
}

#[test]
fn empty_paragraphs_survive() {
    let doc = Document::from_text("a\n\nb");
    assert_eq!(decode(&encode(&doc).unwrap()).unwrap(), doc);
}

#[test]
fn reads_a_word_style_document() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:jc w:val="center"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>
<w:p><w:r><w:rPr><w:b w:val="0"/><w:i/><w:sz w:val="28"/><w:color w:val="FF0000"/></w:rPr><w:t xml:space="preserve">plain </w:t></w:r><w:del><w:r><w:delText>removed</w:delText></w:r></w:del><w:r><w:rPr><w:u w:val="single"/></w:rPr><w:t>under</w:t></w:r></w:p>
<w:p/>
<w:p><w:r><w:t>one</w:t><w:br/><w:t>two</w:t></w:r></w:p>
<w:sectPr/></w:body></w:document>"#;
    let doc = decode(&docx_with(xml)).unwrap();
    assert_eq!(doc.text(), "Title\nplain under\n\none\ntwo");
    assert_eq!(doc.paragraphs[0].style.align, Align::Center);
    assert!(
        doc.paragraphs[0].runs[0].style.bold,
        "Heading style reads as bold"
    );
    let second = &doc.paragraphs[1].runs;
    assert!(!second[0].style.bold && second[0].style.italic);
    assert_eq!(second[0].style.size_pt, Some(14));
    assert_eq!(second[0].style.color, Some(0xFF0000));
    assert!(second[1].style.underline);
}

#[test]
fn rejects_non_docx_and_missing_document_part() {
    assert!(matches!(decode(b"not a zip"), Err(IoError::Parse(_))));
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("other.txt", SimpleFileOptions::default())
        .unwrap();
    let bytes = zip.finish().unwrap().into_inner();
    assert!(matches!(decode(&bytes), Err(IoError::Parse(_))));
}

#[test]
fn drops_characters_xml_forbids() {
    let mut doc = Document::from_text("a");
    doc.paragraphs[0].runs[0].text = "a\u{1}b".into();
    assert_eq!(decode(&encode(&doc).unwrap()).unwrap().text(), "ab");
}
