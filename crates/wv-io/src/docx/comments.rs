use quick_xml::Reader;
use quick_xml::events::Event;
use wv_core::{Comment, Document};

use super::read_util::attr;
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
            "<w:comment w:id=\"{}\" w:author=\"{}\"{}>",
            c.id,
            escape_attr(&c.author),
            if c.date.is_empty() {
                String::new()
            } else {
                format!(" w:date=\"{}\"", escape_attr(&c.date))
            }
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

/// Reads word/comments.xml: id, author, date and text (paragraphs joined by line breaks).
pub(super) fn read(xml: &[u8]) -> Vec<Comment> {
    let mut reader = Reader::from_reader(xml);
    let mut out: Vec<Comment> = Vec::new();
    let mut current: Option<Comment> = None;
    let mut in_text = false;
    let mut buf = Vec::new();
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Eof => break,
            Event::Start(e) => match e.local_name().as_ref() {
                b"comment" => {
                    current = attr(&e, b"id")
                        .and_then(|v| v.parse().ok())
                        .map(|id| Comment {
                            id,
                            author: attr(&e, b"author").unwrap_or_default(),
                            text: String::new(),
                            date: attr(&e, b"date").unwrap_or_default(),
                        });
                }
                b"t" => in_text = true,
                _ => {}
            },
            Event::Text(t) if in_text => {
                if let (Some(c), Ok(text)) = (current.as_mut(), t.unescape()) {
                    c.text.push_str(&text);
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                b"t" => in_text = false,
                b"p" => {
                    if let Some(c) = current.as_mut() {
                        c.text.push('\n');
                    }
                }
                b"comment" => out.extend(current.take().map(|mut c| {
                    c.text.truncate(c.text.trim_end().len());
                    c
                })),
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }
    out
}
