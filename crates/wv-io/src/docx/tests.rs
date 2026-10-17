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
    e.doc.paragraphs[1].runs[0].style.strike = true;
    e.doc.paragraphs[1].runs[0].style.caps = true;
    e.doc.paragraphs[1].runs[0].style.link = Some("https://example.com/?a=1&b=<2>".into());
    e.doc.paragraphs[1].style.quote = true;
    e.doc.paragraphs[0].style.named = wv_core::NamedStyle::Heading2;
    e.doc.paragraphs[1].style.line_spacing = Some(1.5);
    e.doc.paragraphs[0].style.list = Some(wv_core::ListKind::Number);
    e.doc.paragraphs[0].style.indent = 2;
    e.doc.paragraphs[1].runs[0].style.highlight = Some(0x00FF00);
    e.doc.paragraphs[1].runs[0].style.script = wv_core::Script::Super;
    e.doc.paragraphs[1].runs[0].style.font = Some("Georgia \"Serif\" & Co".into());
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
    assert_eq!(doc.paragraphs[0].style.named, wv_core::NamedStyle::Heading1);
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

#[test]
fn reads_word_highlight_names_strike_and_vert_align() {
    let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
<w:p><w:r><w:rPr><w:strike/><w:highlight w:val="yellow"/><w:vertAlign w:val="subscript"/></w:rPr><w:t>x</w:t></w:r></w:p></w:body></w:document>"#;
    let doc = decode(&docx_with(xml)).unwrap();
    let style = &doc.paragraphs[0].runs[0].style;
    assert!(style.strike);
    assert_eq!(style.highlight, Some(0xFFFF00));
    assert_eq!(style.script, wv_core::Script::Sub);
}

#[test]
fn lists_indents_and_spacing_round_trip_with_separate_numbered_lists() {
    use wv_core::ListKind;
    let mut doc = Document::from_text("a\nb\nplain\nindented\nc\nd");
    let styles = [
        (Some(ListKind::Number), 0),
        (Some(ListKind::Number), 1),
        (None, 0),
        (None, 3),
        (Some(ListKind::Number), 0),
        (Some(ListKind::Bullet), 2),
    ];
    for (p, (list, indent)) in doc.paragraphs.iter_mut().zip(styles) {
        p.style.list = list;
        p.style.indent = indent;
    }
    doc.paragraphs[3].style.line_spacing = Some(2.0);
    let bytes = encode(&doc).unwrap();
    assert_eq!(decode(&bytes).unwrap(), doc);
}

#[test]
fn reads_word_numbering_and_indents() {
    let numbering = r#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:abstractNum w:abstractNumId="7"><w:lvl w:ilvl="0"><w:numFmt w:val="decimal"/></w:lvl></w:abstractNum>
<w:abstractNum w:abstractNumId="8"><w:lvl w:ilvl="0"><w:numFmt w:val="bullet"/></w:lvl></w:abstractNum>
<w:num w:numId="3"><w:abstractNumId w:val="7"/></w:num><w:num w:numId="4"><w:abstractNumId w:val="8"/></w:num></w:numbering>"#;
    let document = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="3"/></w:numPr></w:pPr><w:r><w:t>one</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="1"/><w:numId w:val="4"/></w:numPr></w:pPr><w:r><w:t>two</w:t></w:r></w:p>
<w:p><w:pPr><w:spacing w:line="360" w:lineRule="auto"/><w:ind w:left="1440"/></w:pPr><w:r><w:t>three</w:t></w:r></w:p>
<w:p><w:pPr><w:spacing w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:t>four</w:t></w:r></w:p></w:body></w:document>"#;
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("word/document.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.start_file("word/numbering.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(numbering.as_bytes()).unwrap();
    let doc = decode(&zip.finish().unwrap().into_inner()).unwrap();
    assert_eq!(
        doc.paragraphs[0].style.list,
        Some(wv_core::ListKind::Number)
    );
    assert_eq!(
        doc.paragraphs[1].style.list,
        Some(wv_core::ListKind::Bullet)
    );
    assert_eq!(doc.paragraphs[1].style.indent, 1);
    assert_eq!(doc.paragraphs[2].style.indent, 2);
    assert_eq!(doc.paragraphs[2].style.line_spacing, Some(1.5));
    assert_eq!(
        doc.paragraphs[3].style.line_spacing, None,
        "exact line heights are not a multiplier"
    );
}

#[test]
fn table_round_trips_and_merged_cells_become_empty_cells() {
    let mut e = Editor::default();
    e.insert_text("before");
    e.insert_table(2, 3);
    e.insert_text("a1");
    e.next_cell(true);
    e.insert_text("b1");
    let doc = decode(&encode(&e.doc).unwrap()).unwrap();
    assert_eq!(doc, e.doc);

    let xml = "<w:document xmlns:w=\"x\"><w:body><w:tbl><w:tr>\
        <w:tc><w:tcPr><w:gridSpan w:val=\"2\"/></w:tcPr><w:p><w:r><w:t>wide</w:t></w:r></w:p></w:tc>\
        <w:tc><w:p><w:r><w:t>c</w:t></w:r></w:p></w:tc></w:tr>\
        <w:tr><w:tc><w:p><w:r><w:t>1</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>";
    let doc = decode(&docx_with(xml)).unwrap();
    let cells: Vec<_> = doc
        .paragraphs
        .iter()
        .map(|p| {
            let c = p.style.cell.unwrap();
            (c.row, c.col, p.text())
        })
        .collect();
    let s = |t: &str| t.to_string();
    assert_eq!(
        cells,
        [
            (0, 0, s("wide")),
            (0, 1, s("")),
            (0, 2, s("c")),
            (1, 0, s("1")),
            (1, 1, s("")),
            (1, 2, s("")),
        ]
    );
}

#[test]
fn header_and_footer_round_trip_with_page_fields() {
    let mut doc = Document::from_text("body");
    doc.header = wv_core::Band::new("Отчёт <A&B>").aligned(Align::Right);
    doc.footer = wv_core::Band::new("Стр. {page} из {pages}");
    let back = decode(&encode(&doc).unwrap()).unwrap();
    assert_eq!(back.header, doc.header);
    assert_eq!(back.footer, doc.footer);
    assert_eq!(back.text(), "body");
}

#[test]
fn reads_word_style_complex_page_field_in_footer() {
    use std::io::{Cursor, Write};
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let o = SimpleFileOptions::default();
    let mut put = |name: &str, body: &str| {
        zip.start_file(name, o).unwrap();
        zip.write_all(body.as_bytes()).unwrap();
    };
    put(
        "word/document.xml",
        "<w:document xmlns:w=\"w\" xmlns:r=\"r\"><w:body><w:p><w:r><w:t>x</w:t></w:r></w:p><w:sectPr><w:footerReference w:type=\"default\" r:id=\"rId9\"/></w:sectPr></w:body></w:document>",
    );
    put(
        "word/_rels/document.xml.rels",
        "<Relationships><Relationship Id=\"rId9\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer\" Target=\"footer2.xml\"/></Relationships>",
    );
    put(
        "word/footer2.xml",
        "<w:ftr xmlns:w=\"w\"><w:p><w:r><w:t xml:space=\"preserve\">Page </w:t></w:r><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r><w:r><w:instrText xml:space=\"preserve\"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType=\"separate\"/></w:r><w:r><w:t>7</w:t></w:r><w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:p></w:ftr>",
    );
    let doc = decode(&zip.finish().unwrap().into_inner()).unwrap();
    assert_eq!(doc.footer.text, "Page {page}");
    assert!(doc.header.is_empty());
}

#[test]
fn pictures_round_trip() {
    // 1x1 transparent PNG.
    let png: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let mut e = Editor::default();
    e.insert_text("before");
    e.insert_image(wv_core::ImageFormat::Png, png, 320.0, 240.0);
    e.insert_text("after");
    e.doc.paragraphs[1].style.align = Align::Center;
    let back = decode(&encode(&e.doc).unwrap()).unwrap();
    assert_eq!(back, e.doc);
}

#[test]
fn page_size_and_margins_round_trip() {
    let mut doc = Document::from_text("x");
    doc.page = wv_core::PageSpec::a4()
        .with_paper(2)
        .with_landscape(true)
        .with_margins(wv_core::Margins::Wide);
    let back = decode(&encode(&doc).unwrap()).unwrap();
    for (a, b) in [
        (back.page.width, doc.page.width),
        (back.page.height, doc.page.height),
        (back.page.margin_left, doc.page.margin_left),
        (back.page.margin_top, doc.page.margin_top),
    ] {
        assert!((a - b).abs() < 0.1, "{a} vs {b}");
    }
    assert!(back.page.is_landscape());
}

#[test]
fn gutter_and_band_distances_round_trip() {
    let mut doc = Document::from_text("x");
    doc.page =
        wv_core::PageSpec::custom(600.0, 900.0, [90.0, 60.0, 75.0, 45.0], [30.0, 20.0, 25.0]);
    let back = decode(&encode(&doc).unwrap()).unwrap();
    assert_eq!(back.page, doc.page);
}

#[test]
fn first_page_and_even_page_bands_round_trip() {
    let mut doc = Document::from_text("x");
    doc.header = wv_core::Band::new("Main head");
    doc.footer = wv_core::Band::new("Стр. {page}");
    doc.variants = wv_core::BandVariants {
        different_first: true,
        odd_even: true,
        first: (wv_core::Band::new("Title page"), wv_core::Band::default()),
        even: (
            wv_core::Band::new("Even head").aligned(wv_core::Align::Right),
            wv_core::Band::new("{page}"),
        ),
    };
    let back = decode(&encode(&doc).unwrap()).unwrap();
    assert_eq!(back.header, doc.header);
    assert_eq!(back.footer, doc.footer);
    assert_eq!(back.variants, doc.variants);
}

#[test]
fn switches_off_means_variants_are_not_written() {
    let mut doc = Document::from_text("x");
    doc.variants.first = (wv_core::Band::new("hidden"), wv_core::Band::default());
    let back = decode(&encode(&doc).unwrap()).unwrap();
    assert!(!back.variants.different_first);
    assert!(back.variants.first.0.is_empty());
}
