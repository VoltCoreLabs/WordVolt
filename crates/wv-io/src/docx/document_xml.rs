use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use wv_core::{Align, CharStyle, Document, Paragraph};

use crate::IoError;

/// Reads word/document.xml. Matching is by local name, so any namespace prefix works.
pub fn parse(xml: &[u8]) -> Result<Document, IoError> {
    let mut reader = Reader::from_reader(xml);
    let mut state = State::default();
    let mut buf = Vec::new();
    loop {
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|e| IoError::Parse(format!("document.xml: {e}")))?;
        match event {
            Event::Eof => break,
            Event::Start(e) => state.start(&e, false),
            Event::Empty(e) => state.start(&e, true),
            Event::End(e) => state.end(e.local_name().as_ref()),
            Event::Text(t) if state.in_text => {
                let text = t
                    .unescape()
                    .map_err(|e| IoError::Parse(format!("document.xml: {e}")))?;
                state.text(&text);
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(state.finish())
}

#[derive(Default)]
struct State {
    paragraphs: Vec<Paragraph>,
    current: Option<Paragraph>,
    align: Align,
    heading: bool,
    run: CharStyle,
    in_paragraph_props: bool,
    in_run_props: bool,
    in_text: bool,
}

fn attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == name)
        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
}

/// `<w:b/>` is on; `<w:b w:val="0"/>` / `false` is off.
fn flag(e: &BytesStart) -> bool {
    !matches!(
        attr(e, b"val").as_deref(),
        Some("0" | "false" | "off" | "none")
    )
}

impl State {
    fn start(&mut self, e: &BytesStart, empty: bool) {
        let name = e.local_name();
        match name.as_ref() {
            b"p" => {
                self.begin_paragraph();
                if empty {
                    self.end_paragraph();
                }
            }
            b"pPr" => self.in_paragraph_props = true,
            b"jc" if self.in_paragraph_props => {
                self.align = match attr(e, b"val").as_deref() {
                    Some("center") => Align::Center,
                    Some("right" | "end") => Align::Right,
                    Some("both" | "distribute") => Align::Justify,
                    _ => Align::Left,
                };
            }
            b"pStyle" if self.in_paragraph_props => {
                let style = attr(e, b"val").unwrap_or_default();
                self.heading = style.starts_with("Heading") || style == "Title";
            }
            b"r" => self.run = CharStyle::default(),
            b"rPr" if !self.in_paragraph_props => self.in_run_props = true,
            b"b" if self.in_run_props => self.run.bold = flag(e),
            b"i" if self.in_run_props => self.run.italic = flag(e),
            b"u" if self.in_run_props => self.run.underline = flag(e),
            b"sz" if self.in_run_props => {
                self.run.size_pt = attr(e, b"val")
                    .and_then(|v| v.parse::<u16>().ok())
                    .map(|half| half.div_ceil(2));
            }
            b"color" if self.in_run_props => {
                self.run.color = attr(e, b"val")
                    .and_then(|v| u32::from_str_radix(&v, 16).ok().filter(|_| v.len() == 6));
            }
            b"t" => self.in_text = !empty,
            b"tab" if !self.in_run_props && !self.in_paragraph_props => self.text("\t"),
            b"br" | b"cr" if attr(e, b"type").as_deref() != Some("page") => self.line_break(),
            _ => {}
        }
    }

    fn end(&mut self, name: &[u8]) {
        match name {
            b"p" => self.end_paragraph(),
            b"pPr" => self.in_paragraph_props = false,
            b"rPr" => self.in_run_props = false,
            b"t" => self.in_text = false,
            _ => {}
        }
    }

    fn begin_paragraph(&mut self) {
        self.end_paragraph();
        self.current = Some(Paragraph::default());
        self.align = Align::default();
        self.heading = false;
    }

    fn end_paragraph(&mut self) {
        if let Some(mut p) = self.current.take() {
            p.style.align = self.align;
            self.paragraphs.push(p);
        }
    }

    /// A manual line break splits the paragraph (the model has no soft breaks yet).
    fn line_break(&mut self) {
        if self.current.is_some() {
            let (align, heading) = (self.align, self.heading);
            self.end_paragraph();
            self.current = Some(Paragraph::default());
            self.align = align;
            self.heading = heading;
        }
    }

    fn text(&mut self, text: &str) {
        let Some(p) = self.current.as_mut() else {
            return;
        };
        let mut style = self.run.clone();
        style.bold |= self.heading;
        let clean = text.replace(['\n', '\r'], " ");
        let end = p.len();
        p.insert(end, &clean, &style);
    }

    fn finish(mut self) -> Document {
        self.end_paragraph();
        if self.paragraphs.is_empty() {
            Document::default()
        } else {
            Document {
                paragraphs: self.paragraphs,
            }
        }
    }
}
