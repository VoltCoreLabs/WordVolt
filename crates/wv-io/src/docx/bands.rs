use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use wv_core::{Align, BAND_SIZE_PT, Band, BandStyle, BandVariants};

use super::package::{read_document_xml, read_part};

pub const HEADER_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
pub const FOOTER_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";

/// word/header1.xml or footer1.xml (`root` = "hdr" / "ftr"): one centred paragraph; `{page}` and
/// `{pages}` become PAGE / NUMPAGES fields.
pub fn part_xml(root: &str, band: &Band) -> String {
    let rpr = run_props(&band.style);
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
                "<w:r>{rpr}<w:t xml:space=\"preserve\">{}</w:t></w:r>",
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
            "<w:fldSimple w:instr=\" {instr} \"><w:r>{rpr}<w:t>1</w:t></w:r></w:fldSimple>"
        ));
        rest = &rest[rest.find(token).unwrap_or(0) + token.len()..];
    }
    let jc = match band.align {
        Align::Left | Align::Justify => "left",
        Align::Center => "center",
        Align::Right => "right",
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:{root} xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:p><w:pPr><w:jc w:val=\"{jc}\"/></w:pPr>{runs}</w:p></w:{root}>"
    )
}

/// `<w:rPr>` of the band's formatting (the size is always written: Word's default is 11 pt).
fn run_props(style: &BandStyle) -> String {
    let mut xml = String::from("<w:rPr>");
    if style.bold {
        xml.push_str("<w:b/>");
    }
    if style.italic {
        xml.push_str("<w:i/>");
    }
    if style.underline {
        xml.push_str("<w:u w:val=\"single\"/>");
    }
    if let Some(color) = style.color {
        xml.push_str(&format!("<w:color w:val=\"{color:06X}\"/>"));
    }
    let half_points = u32::from(style.size()) * 2;
    xml.push_str(&format!(
        "<w:sz w:val=\"{half_points}\"/><w:szCs w:val=\"{half_points}\"/></w:rPr>"
    ));
    xml
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
                let (text, align, style) = part_text(&xml);
                Band::new(text).aligned(align).styled(style)
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

/// Text of a header/footer part; PAGE and NUMPAGES fields become `{page}` / `{pages}` and the
/// cached field result is dropped. Paragraphs are joined with a space.
fn part_text(xml: &[u8]) -> (String, Align, BandStyle) {
    let mut align = None;
    let mut style = BandStyle::default();
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
                b"b" => style.bold = flag(&e),
                b"i" => style.italic = flag(&e),
                b"u" => style.underline = attr(&e, b"val").as_deref() != Some("none"),
                b"sz" => {
                    let points = attr(&e, b"val")
                        .and_then(|v| v.parse::<u16>().ok())
                        .map(|half| (half / 2).clamp(6, 72));
                    style.size_pt = points.filter(|p| *p != BAND_SIZE_PT);
                }
                b"color" => {
                    style.color = attr(&e, b"val").and_then(|v| u32::from_str_radix(&v, 16).ok());
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
    (out.trim().to_string(), align.unwrap_or(Align::Left), style)
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
