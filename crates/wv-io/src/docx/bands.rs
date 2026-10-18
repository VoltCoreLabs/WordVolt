use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use wv_core::{Align, BAND_SIZE_PT, Band, BandVariants, CharStyle, Run};

pub use super::band_write::part_xml;
use super::package::{read_document_xml, read_part};

pub const HEADER_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
pub const FOOTER_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";

fn attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == name)
        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
}

/// Header and footer of the document (the default ones of the last section that has them), plus
/// the first-page and even-page variants.
pub fn read(bytes: &[u8]) -> (Band, Band, BandVariants) {
    let rels = relationships(bytes);
    let refs = references(bytes);
    let load = |id: &Option<String>, kind: &str| -> Band {
        let target = id
            .as_ref()
            .and_then(|id| {
                rels.get(id)
                    .filter(|(t, _)| t == kind)
                    .map(|(_, v)| v.clone())
            })
            // Only the default slot falls back to any part of this kind.
            .or(None);
        target
            .and_then(|t| read_part(bytes, &format!("word/{}", t.trim_start_matches('/'))))
            .map(|xml| {
                let (runs, align) = part_text(&xml);
                Band::from_runs(runs).aligned(align)
            })
            .unwrap_or_default()
    };
    let any = |kind: &str| {
        rels.iter()
            .find(|(_, (t, _))| t == kind)
            .map(|(id, _)| id.clone())
    };
    let default_header = refs.header[0].clone().or_else(|| any(HEADER_TYPE));
    let default_footer = refs.footer[0].clone().or_else(|| any(FOOTER_TYPE));
    let variants = BandVariants {
        different_first: refs.title_page,
        odd_even: even_and_odd(bytes),
        first: (
            load(&refs.header[1], HEADER_TYPE),
            load(&refs.footer[1], FOOTER_TYPE),
        ),
        even: (
            load(&refs.header[2], HEADER_TYPE),
            load(&refs.footer[2], FOOTER_TYPE),
        ),
    };
    (
        load(&default_header, HEADER_TYPE),
        load(&default_footer, FOOTER_TYPE),
        variants,
    )
}

/// `<w:evenAndOddHeaders/>` in word/settings.xml.
fn even_and_odd(bytes: &[u8]) -> bool {
    let Some(xml) = read_part(bytes, "word/settings.xml") else {
        return false;
    };
    let mut reader = Reader::from_reader(xml.as_slice());
    let mut buf = Vec::new();
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e)
                if e.local_name().as_ref() == b"evenAndOddHeaders" =>
            {
                return !matches!(attr(&e, b"val").as_deref(), Some("0" | "false" | "off"));
            }
            _ => {}
        }
        buf.clear();
    }
    false
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

/// Relationship ids of the header/footer references of word/document.xml, by slot
/// (0 = default, 1 = first page, 2 = even pages), and whether the first page is special.
#[derive(Default)]
struct Refs {
    header: [Option<String>; 3],
    footer: [Option<String>; 3],
    title_page: bool,
}

fn references(bytes: &[u8]) -> Refs {
    let Ok(xml) = read_document_xml(bytes) else {
        return Refs::default();
    };
    let mut reader = Reader::from_reader(xml.as_slice());
    let mut buf = Vec::new();
    let mut refs = Refs::default();
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => {
                let slot = match attr(&e, b"type").as_deref() {
                    None | Some("default") => Some(0),
                    Some("first") => Some(1),
                    Some("even") => Some(2),
                    _ => None,
                };
                match (e.local_name().as_ref(), slot) {
                    (b"headerReference", Some(i)) => refs.header[i] = attr(&e, b"id"),
                    (b"footerReference", Some(i)) => refs.footer[i] = attr(&e, b"id"),
                    (b"titlePg", _) => {
                        refs.title_page =
                            !matches!(attr(&e, b"val").as_deref(), Some("0" | "false" | "off"));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        buf.clear();
    }
    refs
}

/// Collected formatted text of a part.
#[derive(Default)]
struct Collected {
    runs: Vec<Run>,
}

impl Collected {
    fn push(&mut self, text: &str, style: &CharStyle) {
        if text.is_empty() {
            return;
        }
        match self.runs.last_mut() {
            Some(last) if last.style == *style => last.text.push_str(text),
            _ => self.runs.push(Run::new(text, style.clone())),
        }
    }

    fn ends_with_space(&self) -> bool {
        self.runs.last().is_none_or(|r| r.text.ends_with(' '))
    }

    /// The runs without leading and trailing blanks.
    fn finish(mut self) -> Vec<Run> {
        if let Some(first) = self.runs.first_mut() {
            first.text = first.text.trim_start().to_string();
        }
        if let Some(last) = self.runs.last_mut() {
            last.text = last.text.trim_end().to_string();
        }
        self.runs.retain(|r| !r.text.is_empty());
        self.runs
    }
}

/// Formatted text of a header/footer part; PAGE and NUMPAGES fields become `{page}` / `{pages}`
/// (in the formatting of the field) and the cached field result is dropped. Paragraphs are joined
/// with a space.
fn part_text(xml: &[u8]) -> (Vec<Run>, Align) {
    let mut align = None;
    let mut reader = Reader::from_reader(xml);
    let mut buf = Vec::new();
    let mut out = Collected::default();
    let (mut cur, mut instr) = (CharStyle::default(), String::new());
    // A field's token waits for the end of its run, which carries the formatting.
    let mut pending: Option<String> = None;
    let (mut in_t, mut in_instr, mut suppress, mut in_ppr) = (false, false, false, false);
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => match e.local_name().as_ref() {
                b"p" if !out.ends_with_space() => out.push(" ", &CharStyle::default()),
                b"r" => cur = CharStyle::default(),
                b"pPr" => in_ppr = true,
                b"t" => in_t = true,
                b"instrText" => in_instr = true,
                b"fldSimple" => {
                    pending = Some(field_token(&attr(&e, b"instr").unwrap_or_default()));
                    suppress = true;
                }
                b"fldChar" => match attr(&e, b"fldCharType").as_deref() {
                    Some("begin") => instr.clear(),
                    Some("separate") => {
                        pending = Some(field_token(&instr));
                        suppress = true;
                    }
                    Some("end") => suppress = false,
                    _ => {}
                },
                b"tab" => out.push(" ", &cur),
                b"b" if !in_ppr => cur.bold = flag(&e),
                b"i" if !in_ppr => cur.italic = flag(&e),
                b"strike" if !in_ppr => cur.strike = flag(&e),
                b"u" if !in_ppr => cur.underline = attr(&e, b"val").as_deref() != Some("none"),
                b"sz" if !in_ppr => {
                    let points = attr(&e, b"val")
                        .and_then(|v| v.parse::<u16>().ok())
                        .map(|half| (half / 2).clamp(6, 72));
                    cur.size_pt = points.filter(|p| *p != BAND_SIZE_PT);
                }
                b"color" if !in_ppr => {
                    cur.color = attr(&e, b"val").and_then(|v| u32::from_str_radix(&v, 16).ok());
                }
                b"jc" if align.is_none() => {
                    align = Some(match attr(&e, b"val").as_deref() {
                        Some("right" | "end") => Align::Right,
                        Some("left" | "start" | "both") => Align::Left,
                        _ => Align::Center,
                    });
                }
                _ => {}
            },
            Event::End(e) => match e.local_name().as_ref() {
                b"t" => in_t = false,
                b"pPr" => in_ppr = false,
                b"instrText" => in_instr = false,
                b"fldSimple" => suppress = false,
                b"r" => {
                    if let Some(token) = pending.take() {
                        out.push(&token, &cur);
                    }
                }
                _ => {}
            },
            Event::Text(t) => {
                let text = t.unescape().map(|c| c.into_owned()).unwrap_or_default();
                if in_instr {
                    instr.push_str(&text);
                } else if in_t && !suppress {
                    out.push(&text, &cur);
                }
            }
            _ => {}
        }
        buf.clear();
    }
    (out.finish(), align.unwrap_or(Align::Left))
}

/// `<w:b/>` is on, `<w:b w:val="0"/>` off.
fn flag(e: &BytesStart) -> bool {
    !matches!(attr(e, b"val").as_deref(), Some("0" | "false" | "off"))
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
