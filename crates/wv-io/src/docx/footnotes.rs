use wv_core::{Document, Paragraph};

use super::writer::{Lists, write_paragraph};

/// Size (pt) of the text of a footnote when it says nothing else.
const NOTE_SIZE_PT: u16 = 10;

pub(super) const FOOTNOTES_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes";

pub(super) const FOOTNOTES_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml";

/// word/footnotes.xml for the notes that have a mark in the text; `None` when there are none.
/// The first paragraph of a note starts with `w:footnoteRef` (the number Word draws).
pub(super) fn xml(doc: &Document) -> Option<String> {
    let numbers = doc.note_numbers();
    if numbers.is_empty() {
        return None;
    }
    let mut ids: Vec<(usize, u32)> = numbers.iter().map(|(id, n)| (*n, *id)).collect();
    ids.sort_unstable();
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:footnotes xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><w:footnote w:type=\"separator\" w:id=\"-1\"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type=\"continuationSeparator\" w:id=\"0\"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>",
    );
    for (_, id) in ids {
        out.push_str(&format!("<w:footnote w:id=\"{id}\">"));
        let mut first = true;
        for p in doc.paragraphs.iter().filter(|p| p.style.note == Some(id)) {
            out.push_str(&note_paragraph(p, first));
            first = false;
        }
        out.push_str("</w:footnote>");
    }
    out.push_str("</w:footnotes>");
    Some(out)
}

/// One paragraph of a note (links are left out: they would need relationships of this part).
fn note_paragraph(p: &Paragraph, with_ref: bool) -> String {
    let mut q = p.clone();
    q.style.note = None;
    for run in &mut q.runs {
        run.style.link = None;
        run.style.size_pt.get_or_insert(NOTE_SIZE_PT);
    }
    let mut xml = String::new();
    write_paragraph(&mut xml, &q, &mut Vec::new(), &mut Lists::default());
    if with_ref {
        let reference = "<w:r><w:rPr><w:rStyle w:val=\"FootnoteReference\"/></w:rPr><w:footnoteRef/></w:r><w:r><w:t xml:space=\"preserve\"> </w:t></w:r>";
        let at = xml
            .find("</w:pPr>")
            .map(|i| i + "</w:pPr>".len())
            .unwrap_or("<w:p>".len());
        xml.insert_str(at, reference);
    }
    xml
}

/// The first paragraph of a note read from a file starts with the space after Word's number.
pub(super) fn trim_start(p: &mut Paragraph) {
    if let Some(run) = p.runs.first_mut() {
        run.text = run.text.trim_start().to_string();
    }
    p.runs.retain(|r| !r.text.is_empty());
}
