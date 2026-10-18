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
    doc.paragraphs[4].style.first_line = 18;
    doc.paragraphs[2].style.page_break = true;
    doc.paragraphs[3].style.left_extra = 7;
    doc.paragraphs[4].style.tabs = vec![36, 144];
    doc.paragraphs[4].style.space_before = Some(12);
    doc.paragraphs[4].style.space_after = Some(0);
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
fn table_round_trips_and_a_column_span_stays_one_cell() {
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
    assert_eq!(doc.paragraphs[0].style.cell.unwrap().colspan, 2);
    assert_eq!(
        cells,
        [
            (0, 0, s("wide")),
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
    assert_eq!(doc.footer.text(), "Page {page}");
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

#[test]
fn formatted_band_words_round_trip() {
    let mut doc = Document::from_text("x");
    let mut head = wv_core::Band::new("Head bold {page} end");
    head.restyle(5, 9, |s| {
        s.bold = true;
        s.italic = true;
        s.underline = true;
        s.size_pt = Some(14);
        s.color = Some(0x336699);
    });
    head.restyle(10, 16, |s| s.bold = true);
    doc.header = head;
    doc.footer = wv_core::Band::new("Стр. {page}");
    let back = decode(&encode(&doc).unwrap()).unwrap();
    assert_eq!(back.header, doc.header);
    assert_eq!(back.footer, doc.footer);
}

#[test]
fn table_look_and_shading_round_trip() {
    let mut e = wv_core::Editor::new(Document::from_text("x"));
    e.insert_table(2, 3);
    let table = e.table_info().unwrap().table;
    // Narrower than the text, indented, with a tall first row.
    e.set_table_look(
        table,
        wv_core::TableLook {
            widths: vec![0.2, 0.2, 0.3],
            indent: 0.1,
            rows: vec![60.0, 0.0],
        },
    );
    e.set_cell_shade(Some(0x99CCFF));
    let doc = e.doc.clone();
    let back = decode(&encode(&doc).unwrap()).unwrap();
    let look = &back.table_looks[&0];
    for (a, b) in look.widths.iter().zip([0.2, 0.2, 0.3]) {
        assert!((a - b).abs() < 0.01, "{look:?}");
    }
    assert!((look.indent - 0.1).abs() < 0.01 && (look.rows[0] - 60.0).abs() < 1.0);
    let shaded: Vec<_> = back
        .paragraphs
        .iter()
        .filter_map(|p| p.style.cell)
        .filter(|c| c.shade == Some(0x99CCFF))
        .map(|c| (c.row, c.col))
        .collect();
    assert_eq!(shaded, vec![(0, 0)]);
}

#[test]
fn merged_cells_round_trip() {
    let mut e = wv_core::Editor::new(Document::from_text("x"));
    e.insert_table(3, 3);
    let first = e.table_info().unwrap().first;
    // 2x2 block at the top left, then a two-column cell in the last row.
    e.select_range(
        wv_core::Position::new(first, 0),
        wv_core::Position::new(first + 4, 0),
    );
    e.merge_cells();
    let last_row = e
        .doc
        .paragraphs
        .iter()
        .position(|p| p.style.cell.is_some_and(|c| (c.row, c.col) == (2, 0)))
        .unwrap();
    e.select_range(
        wv_core::Position::new(last_row, 0),
        wv_core::Position::new(last_row + 1, 0),
    );
    e.merge_cells();
    let doc = e.doc.clone();
    let back = decode(&encode(&doc).unwrap()).unwrap();
    let cells = |d: &Document| -> Vec<(u16, u16, u8, u8)> {
        d.paragraphs
            .iter()
            .filter_map(|p| p.style.cell)
            .map(|c| (c.row, c.col, c.rowspan, c.colspan))
            .collect()
    };
    assert_eq!(cells(&back), cells(&doc));
}

#[test]
fn deep_headings_round_trip() {
    let mut doc = Document::from_text("a\nb\nc");
    doc.paragraphs[0].style.named = wv_core::NamedStyle::Heading4;
    doc.paragraphs[1].style.named = wv_core::NamedStyle::Heading6;
    doc.paragraphs[2].style.named = wv_core::NamedStyle::Heading9;
    let back = decode(&encode(&doc).unwrap()).unwrap();
    assert_eq!(back, doc);
}

#[test]
fn reads_word_page_breaks() {
    let bytes = docx_with(
        r#"<w:p><w:r><w:t>one</w:t></w:r><w:r><w:br w:type="page"/></w:r><w:r><w:t>two</w:t></w:r></w:p><w:p><w:pPr><w:pageBreakBefore/></w:pPr><w:r><w:t>three</w:t></w:r></w:p><w:p><w:r><w:t>four</w:t></w:r></w:p>"#,
    );
    let doc = decode(&bytes).unwrap();
    let got: Vec<(String, bool)> = doc
        .paragraphs
        .iter()
        .map(|p| (p.text(), p.style.page_break))
        .collect();
    assert_eq!(
        got,
        [
            ("one".to_string(), false),
            ("two".to_string(), true),
            ("three".to_string(), true),
            ("four".to_string(), false),
        ]
    );
}

#[test]
fn columns_round_trip() {
    let mut doc = Document::from_text("a");
    doc.page = doc.page.with_columns(3);
    assert_eq!(decode(&encode(&doc).unwrap()).unwrap().page.columns, 3);
}

#[test]
fn floating_pictures_round_trip_with_their_wrapping() {
    let png: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    for wrap in [
        wv_core::Wrap::Square,
        wv_core::Wrap::Tight,
        wv_core::Wrap::TopBottom,
        wv_core::Wrap::Front,
        wv_core::Wrap::Behind,
    ] {
        let mut e = Editor::default();
        e.insert_text("before");
        e.insert_image(wv_core::ImageFormat::Png, png.clone(), 320.0, 240.0);
        e.sel = wv_core::Selection::caret(wv_core::Position::new(1, 0));
        e.set_image_geometry(
            200.0,
            100.0,
            Some(wv_core::Float {
                x: 48.0,
                y: 96.0,
                wrap,
            }),
        );
        let back = decode(&encode(&e.doc).unwrap()).unwrap();
        let image = back.paragraphs[1].style.image.unwrap();
        let float = image.float.expect("the picture stays floating");
        assert_eq!(float.wrap, wrap);
        assert!((float.x - 48.0).abs() < 0.1 && (float.y - 96.0).abs() < 0.1);
        assert!((image.width - 200.0).abs() < 0.1 && (image.height - 100.0).abs() < 0.1);
    }
}

#[test]
fn rotated_and_cropped_pictures_round_trip() {
    let png: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    for angle in [0.0_f32, 90.0, 37.5, 270.0] {
        for float in [
            None,
            Some(wv_core::Float {
                x: 10.0,
                y: 20.0,
                wrap: wv_core::Wrap::Square,
            }),
        ] {
            let mut e = Editor::default();
            e.insert_image(wv_core::ImageFormat::Png, png.clone(), 320.0, 240.0);
            e.sel = wv_core::Selection::caret(wv_core::Position::new(0, 0));
            e.set_image_geometry(320.0, 240.0, float);
            e.drag_image_crop(
                wv_core::Crop {
                    left: 0.1,
                    top: 0.2,
                    right: 0.0,
                    bottom: 0.25,
                },
                288.0,
                132.0,
                float,
                true,
            );
            e.turn_image(angle);
            let back = decode(&encode(&e.doc).unwrap()).unwrap();
            let (a, b) = (
                back.paragraphs[0].style.image.unwrap(),
                e.doc.paragraphs[0].style.image.unwrap(),
            );
            assert!((a.angle - b.angle).abs() < 0.01, "angle {angle}");
            assert!((a.width - b.width).abs() < 0.1 && (a.height - b.height).abs() < 0.1);
            for (x, y) in [
                (a.crop.left, b.crop.left),
                (a.crop.top, b.crop.top),
                (a.crop.right, b.crop.right),
                (a.crop.bottom, b.crop.bottom),
            ] {
                assert!((x - y).abs() < 0.001, "angle {angle}: {a:?} vs {b:?}");
            }
            assert_eq!(a.float.map(|f| f.wrap), b.float.map(|f| f.wrap));
        }
    }
}

#[test]
fn footnotes_round_trip_through_footnotes_xml() {
    let mut e = Editor::new(Document::from_text("Text with notes\nsecond paragraph"));
    e.sel = wv_core::Selection::caret(wv_core::Position::new(0, 4));
    e.insert_footnote();
    e.insert_text("First note");
    e.sel = wv_core::Selection::caret(wv_core::Position::new(1, 6));
    e.insert_footnote();
    e.insert_text("Second note");
    let bytes = encode(&e.doc).unwrap();
    let back = decode(&bytes).unwrap();
    assert_eq!(back.paragraphs[0].text(), e.doc.paragraphs[0].text());
    assert_eq!(back.paragraphs[1].text(), e.doc.paragraphs[1].text());
    // The notes follow the text, in order, with their text and no number in it.
    let notes: Vec<String> = back
        .paragraphs
        .iter()
        .filter(|p| p.style.note.is_some())
        .map(|p| p.text())
        .collect();
    assert_eq!(notes, ["First note", "Second note"]);
    assert_eq!(back.note_numbers().len(), 2);
    // Word sees the reference and the note parts.
    let zip_has = |name: &str| {
        let mut z = zip::ZipArchive::new(Cursor::new(bytes.clone())).unwrap();
        z.by_name(name).is_ok()
    };
    assert!(zip_has("word/footnotes.xml"));
}

#[test]
fn reads_word_footnotes() {
    let document = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
<w:p><w:r><w:t>Body</w:t></w:r><w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteReference w:id="2"/></w:r><w:r><w:t> text</w:t></w:r></w:p></w:body></w:document>"#;
    let notes = r#"<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>
<w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>
<w:footnote w:id="2"><w:p><w:pPr><w:pStyle w:val="FootnoteText"/></w:pPr><w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteRef/></w:r><w:r><w:t xml:space="preserve"> The note.</w:t></w:r></w:p></w:footnote></w:footnotes>"#;
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("word/document.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.start_file("word/footnotes.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(notes.as_bytes()).unwrap();
    let doc = decode(&zip.finish().unwrap().into_inner()).unwrap();
    assert_eq!(
        doc.paragraphs.len(),
        2,
        "the text and one note, no separators"
    );
    assert_eq!(doc.paragraphs[0].text(), "Body\u{FFFC} text");
    assert_eq!(doc.paragraphs[1].text(), "The note.");
    assert_eq!(doc.paragraphs[1].style.note, Some(2));
    assert_eq!(doc.note_numbers().get(&2), Some(&1));
}
