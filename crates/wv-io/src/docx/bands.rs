use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use wv_core::Band;

use super::package::{read_document_xml, read_part};

pub const HEADER_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
pub const FOOTER_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";

/// word/header1.xml or footer1.xml (`root` = "hdr" / "ftr"): one centred paragraph; `{page}` and
/// `{pages}` become PAGE / NUMPAGES fields.
pub fn part_xml(root: &str, band: &Band) -> String {
    let mut runs = String::new();
    let mut rest = band.text.as_str();
    while !rest.is_empty() {
        let next = ["{pages}", "{page}"]
            .iter()
            .filter_map(|t| rest.find(t).map(|i| (i, *t)))
            .min_by_key(|(i, t)| (*i, std::cmp::Reverse(t.len())));
        let (literal, field) = match next {
            Some((i, token)) => (&rest[..i], Some(token)),
            None => (rest, None),
        };
        if !literal.is_empty() {
            runs.push_str(&format!(
                "<w:r><w:t xml:space=\"preserve\">{}</w:t></w:r>",
                escape(literal)
            ));
        }
        let Some(token) = field else { break };
        let instr = if token == "{page}" {
            "PAGE"
        } else {
            "NUMPAGES"
        };
        runs.push_str(&format!(
            "<w:fldSimple w:instr=\" {instr} \"><w:r><w:t>1</w:t></w:r></w:fldSimple>"
        ));
        rest = &rest[rest.find(token).unwrap_or(0) + token.len()..];
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:{root} xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>{runs}</w:p></w:{root}>"
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == name)
        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
}

/// Header and footer of the document: the default header/footer of the last section that has one.
pub fn read(bytes: &[u8]) -> (Band, Band) {
    let rels = relationships(bytes);
    let (header_id, footer_id) = references(bytes);
    let load = |id: Option<String>, kind: &str| -> Band {
        let target = id
            .and_then(|id| {
                rels.get(&id)
                    .filter(|(t, _)| t == kind)
                    .map(|(_, v)| v.clone())
            })
            .or_else(|| {
                rels.values()
                    .find(|(t, _)| t == kind)
                    .map(|(_, target)| target.clone())
            });
        target
            .and_then(|t| read_part(bytes, &format!("word/{}", t.trim_start_matches('/'))))
            .map(|xml| Band::new(part_text(&xml)))
            .unwrap_or_default()
    };
    (load(header_id, HEADER_TYPE), load(footer_id, FOOTER_TYPE))
}

/// Relationship id -> (type, target) of word/_rels/document.xml.rels.
fn relationships(bytes: &[u8]) -> HashMap<String, (String, String)> {
    let mut map = HashMap::new();
    let Some(xml) = read_part(bytes, "word/_rels/document.xml.rels") else {
        return map;
    };
    let mut reader = Reader::from_reader(xml.as_slice());
    let mut buf = Vec::new();
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"Relationship" => {
                if let (Some(id), Some(kind), Some(target)) =
                    (attr(&e, b"Id"), attr(&e, b"Type"), attr(&e, b"Target"))
                {
                    map.insert(id, (kind, target));
                }
            }
            _ => {}
        }
        buf.clear();
    }
    map
}

/// Ids of the default `headerReference` / `footerReference` in word/document.xml.
fn references(bytes: &[u8]) -> (Option<String>, Option<String>) {
    let Ok(xml) = read_document_xml(bytes) else {
        return (None, None);
    };
    let mut reader = Reader::from_reader(xml.as_slice());
    let mut buf = Vec::new();
    let (mut header, mut footer) = (None, None);
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => {
                let default = attr(&e, b"type").is_none_or(|t| t == "default");
                match e.local_name().as_ref() {
                    b"headerReference" if default => header = attr(&e, b"id"),
                    b"footerReference" if default => footer = attr(&e, b"id"),
                    _ => {}
                }
            }
            _ => {}
        }
        buf.clear();
    }
    (header, footer)
}

/// Text of a header/footer part; PAGE and NUMPAGES fields become `{page}` / `{pages}` and the
/// cached field result is dropped. Paragraphs are joined with a space.
fn part_text(xml: &[u8]) -> String {
    let mut reader = Reader::from_reader(xml);
    let mut buf = Vec::new();
    let (mut out, mut instr) = (String::new(), String::new());
    let (mut in_t, mut in_instr, mut suppress) = (false, false, false);
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => match e.local_name().as_ref() {
                b"p" if !out.is_empty() && !out.ends_with(' ') => out.push(' '),
                b"t" => in_t = true,
                b"instrText" => in_instr = true,
                b"fldSimple" => {
                    out.push_str(&field_token(&attr(&e, b"instr").unwrap_or_default()));
                    suppress = true;
                }
                b"fldChar" => match attr(&e, b"fldCharType").as_deref() {
                    Some("begin") => instr.clear(),
                    Some("separate") => {
                        out.push_str(&field_token(&instr));
                        suppress = true;
                    }
                    Some("end") => suppress = false,
                    _ => {}
                },
                b"tab" => out.push(' '),
                _ => {}
            },
            Event::End(e) => match e.local_name().as_ref() {
                b"t" => in_t = false,
                b"instrText" => in_instr = false,
                b"fldSimple" => suppress = false,
                _ => {}
            },
            Event::Text(t) => {
                let text = t.unescape().map(|c| c.into_owned()).unwrap_or_default();
                if in_instr {
                    instr.push_str(&text);
                } else if in_t && !suppress {
                    out.push_str(&text);
                }
            }
            _ => {}
        }
        buf.clear();
    }
    out.trim().to_string()
}

fn field_token(instr: &str) -> String {
    let instr = instr.to_uppercase();
    if instr.contains("NUMPAGES") {
        "{pages}".into()
    } else if instr.contains("PAGE") {
        "{page}".into()
    } else {
        String::new()
    }
}
