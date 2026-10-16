use wv_core::{Align, CharStyle, Document, Paragraph, Run};

const NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// word/document.xml for the document: A4, 2.5 cm margins.
pub fn document_xml(doc: &Document) -> String {
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:document xmlns:w=\"{NS}\"><w:body>"
    );
    for paragraph in &doc.paragraphs {
        write_paragraph(&mut xml, paragraph);
    }
    xml.push_str(
        "<w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/><w:pgMar w:top=\"1418\" w:right=\"1418\" w:bottom=\"1418\" w:left=\"1418\" w:header=\"709\" w:footer=\"709\" w:gutter=\"0\"/></w:sectPr></w:body></w:document>",
    );
    xml
}

fn write_paragraph(xml: &mut String, p: &Paragraph) {
    xml.push_str("<w:p>");
    let jc = match p.style.align {
        Align::Left => None,
        Align::Center => Some("center"),
        Align::Right => Some("right"),
        Align::Justify => Some("both"),
    };
    if let Some(jc) = jc {
        xml.push_str(&format!("<w:pPr><w:jc w:val=\"{jc}\"/></w:pPr>"));
    }
    for run in &p.runs {
        write_run(xml, run);
    }
    xml.push_str("</w:p>");
}

fn write_run(xml: &mut String, run: &Run) {
    xml.push_str("<w:r>");
    write_properties(xml, &run.style);
    for (i, piece) in run.text.split('\t').enumerate() {
        if i > 0 {
            xml.push_str("<w:tab/>");
        }
        if !piece.is_empty() {
            xml.push_str(&format!(
                "<w:t xml:space=\"preserve\">{}</w:t>",
                escape(piece)
            ));
        }
    }
    xml.push_str("</w:r>");
}

/// Child order follows the schema: b, i, color, sz, u.
fn write_properties(xml: &mut String, s: &CharStyle) {
    if *s == CharStyle::default() {
        return;
    }
    xml.push_str("<w:rPr>");
    if let Some(font) = &s.font {
        let name = escape_attr(font);
        xml.push_str(&format!(
            "<w:rFonts w:ascii=\"{name}\" w:hAnsi=\"{name}\" w:cs=\"{name}\"/>"
        ));
    }
    if s.bold {
        xml.push_str("<w:b/>");
    }
    if s.italic {
        xml.push_str("<w:i/>");
    }
    if let Some(color) = s.color {
        xml.push_str(&format!("<w:color w:val=\"{:06X}\"/>", color & 0xFF_FFFF));
    }
    if let Some(size) = s.size_pt {
        xml.push_str(&format!("<w:sz w:val=\"{}\"/>", u32::from(size) * 2));
    }
    if s.underline {
        xml.push_str("<w:u w:val=\"single\"/>");
    }
    xml.push_str("</w:rPr>");
}

/// XML-escapes text and drops characters XML 1.0 forbids.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            c => out.push(c),
        }
    }
    out
}

fn escape_attr(text: &str) -> String {
    escape(text).replace('"', "&quot;")
}
