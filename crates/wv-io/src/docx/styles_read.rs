use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::Event;
use wv_core::{Align, UserStyle};

use super::read_util::{attr, named_from_docx};

/// One `w:style` as written, before `w:basedOn` is followed.
#[derive(Default)]
struct Raw {
    name: String,
    based_on: Option<String>,
    style: UserStyle,
    /// Properties this style sets itself (the rest comes from the style it is based on).
    set: Set,
}

#[derive(Default)]
struct Set {
    font: bool,
    size: bool,
    color: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    align: bool,
    line: bool,
    before: bool,
    after: bool,
}

/// Reads the paragraph styles of word/styles.xml that are not built-in ones (Normal, Title,
/// headings), by style id, with `w:basedOn` followed up to a built-in style.
pub(super) fn read(xml: &[u8]) -> HashMap<String, UserStyle> {
    let raws = parse(xml);
    raws.keys()
        .filter(|id| !is_builtin(id))
        .map(|id| (id.clone(), flatten(id, &raws, 0)))
        .collect()
}

fn is_builtin(id: &str) -> bool {
    // The styles the app itself writes (see `parts::STYLES`) are not user styles.
    named_from_docx(id) != wv_core::NamedStyle::Normal
        || ["normal", "quote", "footnotetext"].contains(&id.to_lowercase().as_str())
}

fn flatten(id: &str, raws: &HashMap<String, Raw>, depth: u8) -> UserStyle {
    let Some(raw) = raws.get(id) else {
        return UserStyle::default();
    };
    let mut out = match raw.based_on.as_deref() {
        Some(parent) if raws.contains_key(parent) && !is_builtin(parent) && depth < 10 => {
            flatten(parent, raws, depth + 1)
        }
        Some(parent) => UserStyle {
            base: named_from_docx(parent),
            ..UserStyle::default()
        },
        None => UserStyle::default(),
    };
    let (s, own) = (&raw.style, &raw.set);
    out.name = raw.name.clone();
    if own.font {
        out.font = s.font.clone();
    }
    if own.size {
        out.size_pt = s.size_pt;
    }
    if own.color {
        out.color = s.color;
    }
    if own.bold {
        out.bold = s.bold;
    }
    if own.italic {
        out.italic = s.italic;
    }
    if own.underline {
        out.underline = s.underline;
    }
    if own.align {
        out.align = s.align;
    }
    if own.line {
        out.line_spacing = s.line_spacing;
    }
    if own.before {
        out.space_before = s.space_before;
    }
    if own.after {
        out.space_after = s.space_after;
    }
    out
}

fn parse(xml: &[u8]) -> HashMap<String, Raw> {
    let mut reader = Reader::from_reader(xml);
    let mut out: HashMap<String, Raw> = HashMap::new();
    let mut current: Option<(String, Raw)> = None;
    let mut buf = Vec::new();
    while let Ok(event) = reader.read_event_into(&mut buf) {
        let (e, empty) = match &event {
            Event::Eof => break,
            Event::Start(e) => (e, false),
            Event::Empty(e) => (e, true),
            Event::End(e) => {
                if e.local_name().as_ref() == b"style"
                    && let Some((id, raw)) = current.take()
                {
                    out.insert(id, raw);
                }
                buf.clear();
                continue;
            }
            _ => {
                buf.clear();
                continue;
            }
        };
        let name = e.local_name();
        if name.as_ref() == b"style" {
            let is_paragraph = attr(e, b"type").is_none_or(|t| t == "paragraph");
            current = match (is_paragraph, attr(e, b"styleId")) {
                (true, Some(id)) if !empty => Some((id, Raw::default())),
                _ => None,
            };
        } else if let Some((_, raw)) = current.as_mut() {
            property(raw, name.as_ref(), e);
        }
        buf.clear();
    }
    out
}

fn property(raw: &mut Raw, name: &[u8], e: &quick_xml::events::BytesStart) {
    let val = attr(e, b"val");
    let on = || val.as_deref().is_none_or(|v| !matches!(v, "0" | "false"));
    let twips = |key: &[u8]| attr(e, key).and_then(|v| v.parse::<f32>().ok());
    let (s, set) = (&mut raw.style, &mut raw.set);
    match name {
        b"name" => raw.name = val.unwrap_or_default(),
        b"basedOn" => raw.based_on = val,
        b"b" => (s.bold, set.bold) = (on(), true),
        b"i" => (s.italic, set.italic) = (on(), true),
        b"u" => (s.underline, set.underline) = (val.as_deref() != Some("none"), true),
        b"color" => {
            s.color = val
                .as_deref()
                .filter(|v| v.len() == 6)
                .and_then(|v| u32::from_str_radix(v, 16).ok());
            set.color = true;
        }
        b"sz" => {
            s.size_pt = val
                .and_then(|v| v.parse::<u16>().ok())
                .map(|half| half.div_ceil(2));
            set.size = true;
        }
        b"rFonts" => {
            s.font = attr(e, b"ascii").or_else(|| attr(e, b"hAnsi"));
            set.font = true;
        }
        b"jc" => {
            s.align = Some(match val.as_deref() {
                Some("center") => Align::Center,
                Some("right" | "end") => Align::Right,
                Some("both" | "distribute") => Align::Justify,
                _ => Align::Left,
            });
            set.align = true;
        }
        b"spacing" => {
            if let Some(v) = twips(b"before") {
                (s.space_before, set.before) =
                    (Some((v / 20.0).round().clamp(0.0, 255.0) as u8), true);
            }
            if let Some(v) = twips(b"after") {
                (s.space_after, set.after) =
                    (Some((v / 20.0).round().clamp(0.0, 255.0) as u8), true);
            }
            let auto = attr(e, b"lineRule").is_none_or(|rule| rule == "auto");
            if let (true, Some(line)) = (auto, twips(b"line")) {
                (s.line_spacing, set.line) = (Some((line / 240.0).clamp(0.5, 5.0)), true);
            }
        }
        _ => {}
    }
}
