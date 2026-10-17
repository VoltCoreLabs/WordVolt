use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use wv_core::{
    Align, Cell, CharStyle, Document, ImageRef, ListKind, NamedStyle, PageSpec, Paragraph, Script,
};

use super::read_util::{attr, flag, named_from_docx, named_highlight};
use crate::IoError;

/// Reads word/document.xml. Matching is by local name, so any namespace prefix works.
pub fn parse(
    xml: &[u8],
    links: &HashMap<String, String>,
    numbering: &HashMap<String, ListKind>,
) -> Result<(Document, Vec<String>), IoError> {
    let mut reader = Reader::from_reader(xml);
    let mut state = State {
        links: links.clone(),
        numbering: numbering.clone(),
        ..State::default()
    };
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
    let rels = std::mem::take(&mut state.pictures.rels);
    Ok((state.finish(), rels))
}

#[derive(Default)]
struct State {
    numbering: HashMap<String, ListKind>,
    list: Option<ListKind>,
    indent: u8,
    line_spacing: Option<f32>,
    links: HashMap<String, String>,
    link: Option<String>,
    paragraphs: Vec<Paragraph>,
    current: Option<Paragraph>,
    align: Align,
    named: NamedStyle,
    quote: bool,
    run: CharStyle,
    in_paragraph_props: bool,
    in_run_props: bool,
    in_text: bool,
    table: TableState,
    pictures: PictureState,
    page: PageSpec,
    /// Picture of the paragraph being read (index into `pictures.rels`).
    image: Option<ImageRef>,
}

/// Pictures found in the body: relationship ids in order of appearance (`ImageRef::id` indexes them).
#[derive(Default)]
struct PictureState {
    rels: Vec<String>,
    in_drawing: bool,
    embed: Option<String>,
    size: (f32, f32),
    /// The paragraph that follows a picture inside one `w:p`; dropped if it stays empty.
    after: bool,
}

/// Position inside the outermost table being read (nested tables are flattened into their cell).
#[derive(Default)]
struct TableState {
    depth: u32,
    next_id: u32,
    start: usize,
    row: Option<u16>,
    col: u16,
    span: u16,
    cell: Option<Cell>,
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
            b"tbl" if !empty => {
                self.table.depth += 1;
                if self.table.depth == 1 {
                    self.end_paragraph();
                    self.table.start = self.paragraphs.len();
                    self.table.row = None;
                }
            }
            b"tr" if self.table.depth == 1 => {
                self.table.row = Some(self.table.row.map_or(0, |r| r + 1));
                self.table.col = 0;
            }
            b"tc" if self.table.depth == 1 && !empty => {
                self.table.span = 1;
                self.table.cell = Some(Cell {
                    table: self.table.next_id,
                    row: self.table.row.unwrap_or(0),
                    col: self.table.col,
                });
            }
            b"gridSpan" if self.table.depth == 1 && self.table.cell.is_some() => {
                self.table.span = attr(e, b"val")
                    .and_then(|v| v.parse::<u16>().ok())
                    .map_or(1, |n| n.clamp(1, 63));
            }
            b"drawing" if !empty => {
                self.pictures.in_drawing = true;
                self.pictures.embed = None;
                self.pictures.size = (0.0, 0.0);
            }
            b"extent" if self.pictures.in_drawing && self.pictures.size.0 == 0.0 => {
                let px = |name: &[u8]| attr(e, name).map_or(0.0, |v| super::picture::emu_to_px(&v));
                self.pictures.size = (px(b"cx"), px(b"cy"));
            }
            b"blip" if self.pictures.in_drawing => self.pictures.embed = attr(e, b"embed"),
            b"pgSz" => {
                let px = |name: &[u8]| attr(e, name).and_then(|v| v.parse::<f32>().ok());
                if let (Some(w), Some(h)) = (px(b"w"), px(b"h"))
                    && w >= 1500.0
                    && h >= 1500.0
                {
                    (self.page.width, self.page.height) = (w / 15.0, h / 15.0);
                }
            }
            b"pgMar" => {
                let px = |name: &[u8], old: f32| {
                    attr(e, name)
                        .and_then(|v| v.parse::<f32>().ok())
                        .map_or(old, |v| (v / 15.0).max(0.0))
                };
                let p = &mut self.page;
                p.margin_top = px(b"top", p.margin_top);
                p.margin_right = px(b"right", p.margin_right);
                p.margin_bottom = px(b"bottom", p.margin_bottom);
                p.margin_left = px(b"left", p.margin_left);
            }
            b"pPr" => self.in_paragraph_props = true,
            b"ilvl" if self.in_paragraph_props => {
                self.indent = attr(e, b"val")
                    .and_then(|v| v.parse::<u8>().ok())
                    .unwrap_or(0)
                    .min(8);
            }
            b"numId" if self.in_paragraph_props => {
                self.list = attr(e, b"val").and_then(|id| self.numbering.get(&id).copied());
            }
            b"ind" if self.in_paragraph_props && self.list.is_none() => {
                let left = attr(e, b"left")
                    .or_else(|| attr(e, b"start"))
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(0.0);
                self.indent = (left / 720.0).round().clamp(0.0, 8.0) as u8;
            }
            b"spacing" if self.in_paragraph_props => {
                let auto = attr(e, b"lineRule").is_none_or(|rule| rule == "auto");
                if let (true, Some(line)) =
                    (auto, attr(e, b"line").and_then(|v| v.parse::<f32>().ok()))
                {
                    self.line_spacing = Some((line / 240.0).clamp(0.5, 5.0));
                }
            }
            b"hyperlink" if !empty => {
                self.link = attr(e, b"id").and_then(|id| self.links.get(&id).cloned())
            }
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
                self.named = named_from_docx(&style);
                self.quote = style == "Quote";
            }
            b"r" => self.run = CharStyle::default(),
            b"rPr" if !self.in_paragraph_props => self.in_run_props = true,
            b"rFonts" if self.in_run_props => {
                self.run.font = attr(e, b"ascii")
                    .or_else(|| attr(e, b"hAnsi"))
                    .filter(|f| !f.is_empty());
            }
            b"b" if self.in_run_props => self.run.bold = flag(e),
            b"i" if self.in_run_props => self.run.italic = flag(e),
            b"u" if self.in_run_props => self.run.underline = flag(e),
            b"caps" if self.in_run_props => self.run.caps = flag(e),
            b"strike" | b"dstrike" if self.in_run_props => self.run.strike = flag(e),
            b"highlight" if self.in_run_props => {
                self.run.highlight = attr(e, b"val").as_deref().and_then(named_highlight)
            }
            b"shd" if self.in_run_props => {
                if let Some(fill) = attr(e, b"fill").filter(|f| f.len() == 6) {
                    self.run.highlight = u32::from_str_radix(&fill, 16).ok();
                }
            }
            b"vertAlign" if self.in_run_props => {
                self.run.script = match attr(e, b"val").as_deref() {
                    Some("superscript") => Script::Super,
                    Some("subscript") => Script::Sub,
                    _ => Script::Normal,
                };
            }
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
            b"drawing" => self.end_drawing(),
            b"tc" if self.table.depth == 1 => self.end_cell(),
            b"tbl" if self.table.depth > 0 => {
                self.table.depth -= 1;
                if self.table.depth == 0 {
                    let rows = self.paragraphs.split_off(self.table.start);
                    let rows = super::table_read::normalize(rows, self.table.next_id);
                    self.paragraphs.extend(rows);
                    self.table.next_id += 1;
                    self.table.cell = None;
                }
            }
            b"pPr" => self.in_paragraph_props = false,
            b"hyperlink" => self.link = None,
            b"rPr" => self.in_run_props = false,
            b"t" => self.in_text = false,
            _ => {}
        }
    }

    fn begin_paragraph(&mut self) {
        self.end_paragraph();
        self.current = Some(Paragraph::default());
        self.align = Align::default();
        self.named = NamedStyle::Normal;
        self.quote = false;
        self.list = None;
        self.indent = 0;
        self.line_spacing = None;
    }

    fn end_paragraph(&mut self) {
        if let Some(mut p) = self.current.take() {
            if std::mem::take(&mut self.pictures.after) && p.is_empty() && self.image.is_none() {
                return;
            }
            p.style.align = self.align;
            p.style.quote = self.quote;
            p.style.named = self.named;
            p.style.list = self.list;
            p.style.indent = self.indent;
            p.style.line_spacing = self.line_spacing;
            p.style.cell = self.table.cell;
            p.style.image = self.image.take();
            self.paragraphs.push(p);
        }
    }

    /// A picture ends the text paragraph before it and becomes a paragraph of its own.
    fn end_drawing(&mut self) {
        let pictures = &mut self.pictures;
        pictures.in_drawing = false;
        let Some(rid) = pictures.embed.take() else {
            return;
        };
        let (mut width, mut height) = pictures.size;
        if width < 1.0 || height < 1.0 {
            (width, height) = (200.0, 150.0);
        }
        let id = pictures.rels.len() as u32;
        pictures.rels.push(rid);
        if self.current.as_ref().is_some_and(|p| !p.is_empty()) {
            self.line_break();
        }
        self.image = Some(ImageRef { id, width, height });
        let (align, named) = (self.align, self.named);
        self.end_paragraph();
        self.current = Some(Paragraph::default());
        self.align = align;
        self.named = named;
        self.pictures.after = true;
    }

    /// Closes a cell; a column span leaves empty cells for the columns it covered.
    fn end_cell(&mut self) {
        self.end_paragraph();
        if let Some(cell) = self.table.cell.take() {
            for extra in 1..self.table.span {
                let mut p = Paragraph::default();
                p.style.cell = Some(Cell {
                    col: cell.col + extra,
                    ..cell
                });
                self.paragraphs.push(p);
            }
            self.table.col += self.table.span.max(1);
        }
    }

    /// A manual line break splits the paragraph (the model has no soft breaks yet).
    fn line_break(&mut self) {
        if self.current.is_some() {
            let (align, named) = (self.align, self.named);
            self.end_paragraph();
            self.current = Some(Paragraph::default());
            self.align = align;
            self.named = named;
        }
    }

    fn text(&mut self, text: &str) {
        let Some(p) = self.current.as_mut() else {
            return;
        };
        let mut style = self.run.clone();
        style.link = self.link.clone();
        let clean = text.replace(['\n', '\r'], " ");
        let end = p.len();
        p.insert(end, &clean, &style);
    }

    fn finish(mut self) -> Document {
        self.end_paragraph();
        let mut doc = if self.paragraphs.is_empty() {
            Document::default()
        } else {
            Document {
                paragraphs: self.paragraphs,
                ..Document::default()
            }
        };
        doc.page = self.page.snapped();
        doc
    }
}
