use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::Event;
use wv_core::{
    Align, Cell, CharStyle, Document, ImageRef, ListKind, NamedStyle, PageSpec, Paragraph,
};

use crate::IoError;

/// Reads word/document.xml. Matching is by local name, so any namespace prefix works.
pub fn parse(
    xml: &[u8],
    links: &HashMap<String, String>,
    numbering: &HashMap<String, ListKind>,
    styles: &HashMap<String, wv_core::UserStyle>,
) -> Result<(Document, Vec<String>), IoError> {
    let mut reader = Reader::from_reader(xml);
    let mut state = State {
        links: links.clone(),
        numbering: numbering.clone(),
        user_styles: styles.clone(),
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
            Event::Text(t) if state.pictures.in_offset => {
                let text = t
                    .unescape()
                    .map_err(|e| IoError::Parse(format!("document.xml: {e}")))?;
                state.picture_offset(&text);
            }
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
    /// Inside `w:footnote` with this id (its paragraphs belong to the note).
    note: Option<u32>,
    /// Inside a separator note: its paragraphs are not read.
    skip_note: bool,
    numbering: HashMap<String, ListKind>,
    list: Option<ListKind>,
    indent: u8,
    line_spacing: Option<f32>,
    first_line: u8,
    page_break: bool,
    left_extra: u8,
    tabs: Vec<u16>,
    space_before: Option<u8>,
    space_after: Option<u8>,
    links: HashMap<String, String>,
    link: Option<String>,
    comment: Option<u32>,
    /// The bookmark being read (our id, the file's id) and the bookmarks found.
    bookmark: Option<(u32, String)>,
    bookmarks: Vec<wv_core::Bookmark>,
    /// User styles of styles.xml by style id; the ones used get an id in `doc_styles`.
    user_styles: HashMap<String, wv_core::UserStyle>,
    custom_ids: HashMap<String, u32>,
    doc_styles: std::collections::BTreeMap<u32, wv_core::UserStyle>,
    custom: Option<u32>,
    /// Page setup before a `w:sectPr` inside a paragraph (its values are read into `page`).
    saved_page: Option<PageSpec>,
    /// The section the paragraph being read ends.
    pending_section: Option<PageSpec>,
    sections: std::collections::BTreeMap<u32, PageSpec>,
    /// Inside `w:ins` / `w:del`.
    change: Option<wv_core::Change>,
    revisions: Vec<wv_core::Revision>,
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
    pictures: picture_state::PictureState,
    page: PageSpec,
    /// What the file says about the size of the tables read so far, by table id.
    table_raw: std::collections::BTreeMap<u32, super::table_read::RawTable>,
    /// Picture of the paragraph being read (index into `pictures.rels`).
    image: Option<ImageRef>,
}

/// Pictures found in the body: relationship ids in order of appearance (`ImageRef::id` indexes them).
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
    /// The cell being read continues the one above (`w:vMerge` without `restart`).
    continues: bool,
    /// Where the cell's paragraphs start in `State::paragraphs`.
    cell_start: usize,
    /// `w:gridCol` widths, indent and row heights (twips) of the table being read.
    raw: super::table_read::RawTable,
}

impl State {
    fn end(&mut self, name: &[u8]) {
        match name {
            b"p" => self.end_paragraph(),
            b"footnote" => {
                self.note = None;
                self.skip_note = false;
            }
            b"drawing" => self.end_drawing(),
            b"posOffset" => self.pictures.in_offset = false,
            b"positionH" | b"positionV" => self.pictures.axis = None,
            b"tc" if self.table.depth == 1 => self.end_cell(),
            b"tbl" if self.table.depth > 0 => {
                self.table.depth -= 1;
                if self.table.depth == 0 {
                    let rows = self.paragraphs.split_off(self.table.start);
                    let rows = super::table_read::normalize(rows, self.table.next_id);
                    self.table_raw
                        .insert(self.table.next_id, std::mem::take(&mut self.table.raw));
                    self.paragraphs.extend(rows);
                    self.table.next_id += 1;
                    self.table.cell = None;
                }
            }
            b"pPr" => self.in_paragraph_props = false,
            b"hyperlink" => self.link = None,
            b"sectPr" if self.saved_page.is_some() => {
                let restored = self.saved_page.take().unwrap_or_default();
                self.pending_section = Some(std::mem::replace(&mut self.page, restored));
            }
            b"rPr" => self.in_run_props = false,
            b"t" | b"delText" => self.in_text = false,
            b"ins" | b"del" if !self.in_paragraph_props && !self.in_run_props => self.change = None,
            _ => {}
        }
    }

    /// The id of the user style with the file's style id `style` (the style is added to the
    /// document the first time); sets the built-in style it is based on.
    fn user_style_id(&mut self, style: &str) -> Option<u32> {
        let user = self.user_styles.get(style)?.clone();
        self.named = user.base;
        let next = self.doc_styles.len() as u32 + 1;
        let id = *self.custom_ids.entry(style.to_string()).or_insert(next);
        self.doc_styles.entry(id).or_insert(user);
        Some(id)
    }

    fn begin_paragraph(&mut self) {
        self.end_paragraph();
        self.current = Some(Paragraph::default());
        self.align = Align::default();
        self.named = NamedStyle::Normal;
        self.custom = None;
        self.quote = false;
        self.list = None;
        self.indent = 0;
        self.line_spacing = None;
        self.first_line = 0;
        self.page_break = false;
        self.left_extra = 0;
        self.tabs.clear();
        self.space_before = None;
        self.space_after = None;
    }

    fn end_paragraph(&mut self) {
        if let Some(mut p) = self.current.take() {
            if self.skip_note {
                return;
            }
            p.style.note = self.note;
            if std::mem::take(&mut self.pictures.after) && p.is_empty() && self.image.is_none() {
                return;
            }
            p.style.align = self.align;
            p.style.quote = self.quote;
            p.style.named = self.named;
            p.style.custom = self.custom;
            p.style.list = self.list;
            p.style.indent = self.indent;
            p.style.line_spacing = self.line_spacing;
            p.style.first_line = self.first_line;
            p.style.page_break = self.page_break;
            p.style.left_extra = self.left_extra;
            p.style.tabs = std::mem::take(&mut self.tabs);
            p.style.tabs.sort_unstable();
            p.style.tabs.dedup();
            p.style.space_before = self.space_before;
            p.style.space_after = self.space_after;
            p.style.cell = self.table.cell;
            p.style.image = self.image.take();
            if let Some(spec) = self.pending_section.take() {
                let id = self.sections.len() as u32 + 1;
                self.sections.insert(id, spec);
                p.style.section = Some(id);
            }
            self.paragraphs.push(p);
        }
    }

    /// Closes a cell. One that continues the cell above (`w:vMerge`) is dropped and makes that
    /// cell taller; a column span just widens the cell (the covered columns have no cells).
    fn end_cell(&mut self) {
        self.end_paragraph();
        if let Some(cell) = self.table.cell.take() {
            if self.table.continues {
                self.paragraphs.truncate(self.table.cell_start);
                let start = self.table.start;
                let above = (cell.row.saturating_sub(1), cell.col);
                // The cell above in this column, wherever it started.
                let origin = self.paragraphs[start..]
                    .iter()
                    .filter_map(|p| p.style.cell)
                    .find(|c| c.col == above.1 && c.row_end() == cell.row)
                    .map(|c| (c.row, c.col));
                if let Some(key) = origin {
                    for p in self.paragraphs[start..].iter_mut() {
                        if let Some(c) = p.style.cell.as_mut()
                            && (c.row, c.col) == key
                        {
                            c.rowspan = c.rowspan.saturating_add(1);
                        }
                    }
                }
            }
            self.table.col += self.table.span.max(1);
        }
    }

    /// A manual line break splits the paragraph (the model has no soft breaks yet).
    /// A line break starts a new paragraph (`page`: one that begins a new page).
    fn line_break(&mut self, page: bool) {
        if self.current.is_some() {
            let (align, named) = (self.align, self.named);
            self.end_paragraph();
            self.current = Some(Paragraph::default());
            self.align = align;
            self.named = named;
            self.page_break = page;
        }
    }

    /// A `w:footnoteReference`: the mark of footnote `id` goes into the text.
    fn note_mark(&mut self, id: u32) {
        let saved = std::mem::take(&mut self.run);
        self.run.note = Some(id);
        self.text(&wv_core::NOTE_MARK.to_string());
        self.run = saved;
    }

    fn text(&mut self, text: &str) {
        let Some(p) = self.current.as_mut() else {
            return;
        };
        let mut style = self.run.clone();
        style.link = self.link.clone();
        style.comment = self.comment;
        style.bookmark = self.bookmark.as_ref().map(|(id, _)| *id);
        style.change = self.change;
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
        doc.revisions = self.revisions;
        doc.bookmarks = self.bookmarks;
        doc.styles = self.doc_styles;
        doc.page = self.page.snapped();
        doc.sections = self
            .sections
            .into_iter()
            .map(|(id, spec)| (id, spec.snapped()))
            .collect();
        // Table sizes are fractions of the text width, known now that the page is.
        let page = &doc.page;
        let text_width = (page.width - page.margin_left - page.margin_right - page.gutter) * 15.0;
        for (table, raw) in &self.table_raw {
            let dims = doc
                .paragraphs
                .iter()
                .filter_map(|p| p.style.cell)
                .filter(|c| c.table == *table)
                .fold((0, 0), |(r, c), cell| {
                    (
                        r.max(usize::from(cell.row_end())),
                        c.max(usize::from(cell.col_end())),
                    )
                });
            if let Some(look) = super::table_read::look_of(raw, text_width, dims.0, dims.1) {
                doc.table_looks.insert(*table, look);
            }
        }
        doc
    }
}

mod picture_state;
mod start;
