use wv_core::Document;

use super::writer::escape_attr;

pub(super) const COMMENTS_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";

pub(super) const COMMENTS_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";

/// word/comments.xml for the comments that are anchored in the text; `None` when there are none.
pub(super) fn xml(doc: &Document) -> Option<String> {
    let shown = doc.visible_comments();
    if shown.is_empty() {
        return None;
    }
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:comments xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
    );
    for c in shown {
        out.push_str(&format!(
            "<w:comment w:id=\"{}\" w:author=\"{}\">",
            c.id,
            escape_attr(&c.author)
        ));
        for line in c.text.split('\n') {
            out.push_str(&format!(
                "<w:p><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
                super::writer::escape(line)
            ));
        }
        out.push_str("</w:comment>");
    }
    out.push_str("</w:comments>");
    Some(out)
}

/// Adds the comments part to the content types and the relationships of word/document.xml.
pub(super) fn register(content_types: &mut String, rels: &mut String) {
    *content_types = content_types.replace(
        "</Types>",
        &format!(
            "<Override PartName=\"/word/comments.xml\" ContentType=\"{COMMENTS_CONTENT_TYPE}\"/></Types>"
        ),
    );
    *rels = rels.replace(
        "</Relationships>",
        &format!(
            "<Relationship Id=\"rIdCm1\" Type=\"{COMMENTS_TYPE}\" Target=\"comments.xml\"/></Relationships>"
        ),
    );
}
