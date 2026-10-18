use std::collections::BTreeMap;

use super::{Band, BandVariants, ImageData, PageSpec, Paragraph, TableLook};
use crate::position::Position;
use crate::style::CharStyle;

/// A comment on a stretch of text (the text carries `CharStyle::comment = id`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Comment {
    pub id: u32,
    pub author: String,
    pub text: String,
    /// `YYYY-MM-DDTHH:MM:SSZ`; empty when unknown.
    pub date: String,
}

/// Who made a tracked change and when (the text carries `CharStyle::change`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Revision {
    pub id: u32,
    pub author: String,
    /// `YYYY-MM-DDTHH:MM:SSZ`; empty when unknown.
    pub date: String,
}

/// A document: a list of paragraphs (never empty).
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub paragraphs: Vec<Paragraph>,
    pub header: Band,
    pub footer: Band,
    /// Headers and footers of the first and even pages.
    pub variants: BandVariants,
    /// Pictures used by paragraphs (`ParaStyle::image`).
    pub images: Vec<ImageData>,
    pub page: PageSpec,
    /// Sizes of tables by table id (absent = equal columns, rows as tall as their text).
    pub table_looks: BTreeMap<u32, TableLook>,
    /// Page setup of the sections that end with a break, by `ParaStyle::section` id.
    pub sections: BTreeMap<u32, PageSpec>,
    /// Comments by `CharStyle::comment` id; one without anchored text is not shown.
    pub comments: Vec<Comment>,
    /// Authors and dates of tracked changes by `Change::id`.
    pub revisions: Vec<Revision>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            paragraphs: vec![Paragraph::default()],
            header: Band::default(),
            footer: Band::default(),
            variants: BandVariants::default(),
            images: Vec::new(),
            page: PageSpec::default(),
            table_looks: BTreeMap::new(),
            sections: BTreeMap::new(),
            comments: Vec::new(),
            revisions: Vec::new(),
        }
    }
}

/// Document-wide settings that are not paragraphs; changing them is one undo step.
#[derive(Clone, Debug, PartialEq)]
pub struct DocSettings {
    pub header: Band,
    pub footer: Band,
    pub variants: BandVariants,
    pub page: PageSpec,
    pub table_looks: BTreeMap<u32, TableLook>,
    pub sections: BTreeMap<u32, PageSpec>,
}

/// The character that stands for a footnote mark in the text (the number is drawn instead).
pub const NOTE_MARK: char = '\u{FFFC}';

impl Document {
    /// An id no comment has.
    pub fn next_comment_id(&self) -> u32 {
        let anchored = self
            .paragraphs
            .iter()
            .flat_map(|p| p.runs.iter().filter_map(|r| r.style.comment));
        let known = self.comments.iter().map(|c| c.id);
        anchored.chain(known).max().map_or(1, |m| m + 1)
    }

    /// An id no tracked change has.
    pub fn next_revision_id(&self) -> u32 {
        let used = self
            .paragraphs
            .iter()
            .flat_map(|p| p.runs.iter().filter_map(|r| r.style.change.map(|c| c.id)));
        let known = self.revisions.iter().map(|r| r.id);
        used.chain(known).max().map_or(1, |m| m + 1)
    }

    /// The tracked changes that still have text, as (id, deleted), in document order.
    pub fn changes(&self) -> Vec<(u32, bool)> {
        let mut out: Vec<(u32, bool)> = Vec::new();
        for c in self
            .paragraphs
            .iter()
            .flat_map(|p| p.runs.iter().filter_map(|r| r.style.change))
        {
            if !out.contains(&(c.id, c.deleted)) {
                out.push((c.id, c.deleted));
            }
        }
        out
    }

    /// Comments that have text anchored to them, in the order of their anchors in the document.
    pub fn visible_comments(&self) -> Vec<&Comment> {
        let mut ids: Vec<u32> = Vec::new();
        for p in &self.paragraphs {
            for id in p.runs.iter().filter_map(|r| r.style.comment) {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        ids.iter()
            .filter_map(|id| self.comments.iter().find(|c| c.id == *id))
            .collect()
    }

    /// Where the text of comment `id` starts and ends (the first and last run that carry it).
    pub fn comment_range(&self, id: u32) -> Option<(Position, Position)> {
        let mut start = None;
        let mut end = None;
        for (i, p) in self.paragraphs.iter().enumerate() {
            let mut offset = 0;
            for run in &p.runs {
                let from = offset;
                offset += run.text.len();
                if run.style.comment == Some(id) {
                    start.get_or_insert(Position::new(i, from));
                    end = Some(Position::new(i, offset));
                }
            }
        }
        Some((start?, end?))
    }

    /// Footnote numbers: the marks in the text, counted in order.
    pub fn note_numbers(&self) -> std::collections::HashMap<u32, usize> {
        let mut out = std::collections::HashMap::new();
        for p in self.paragraphs.iter().filter(|p| p.style.note.is_none()) {
            for id in p.runs.iter().filter_map(|r| r.style.note) {
                let next = out.len() + 1;
                out.entry(id).or_insert(next);
            }
        }
        out
    }

    /// An id no footnote has.
    pub fn next_note_id(&self) -> u32 {
        let in_text = self
            .paragraphs
            .iter()
            .flat_map(|p| p.runs.iter().filter_map(|r| r.style.note));
        let notes = self.paragraphs.iter().filter_map(|p| p.style.note);
        in_text.chain(notes).max().map_or(1, |m| m + 1)
    }

    /// Index of the first paragraph of footnote `id`.
    pub fn note_paragraph(&self, id: u32) -> Option<usize> {
        self.paragraphs
            .iter()
            .position(|p| p.style.note == Some(id))
    }

    /// Position just after the mark of footnote `id` in the text.
    pub fn note_mark_position(&self, id: u32) -> Option<crate::position::Position> {
        for (i, p) in self.paragraphs.iter().enumerate() {
            let mut offset = 0;
            for run in &p.runs {
                offset += run.text.len();
                if run.style.note == Some(id) && p.style.note.is_none() {
                    return Some(crate::position::Position::new(i, offset));
                }
            }
        }
        None
    }
    pub fn settings(&self) -> DocSettings {
        DocSettings {
            header: self.header.clone(),
            footer: self.footer.clone(),
            variants: self.variants.clone(),
            page: self.page,
            table_looks: self.table_looks.clone(),
            sections: self.sections.clone(),
        }
    }

    pub fn set_settings(&mut self, settings: &DocSettings) {
        self.header = settings.header.clone();
        self.footer = settings.footer.clone();
        self.variants = settings.variants.clone();
        self.page = settings.page;
        self.table_looks = settings.table_looks.clone();
        self.sections = settings.sections.clone();
    }

    /// The section break (id) that ends the section paragraph `para` is in; `None` in the last one.
    pub fn section_end(&self, para: usize) -> Option<u32> {
        self.paragraphs[para..]
            .iter()
            .find_map(|p| p.style.section)
            .filter(|id| self.sections.contains_key(id))
    }

    /// Page setup of the section paragraph `para` is in.
    pub fn page_of(&self, para: usize) -> PageSpec {
        self.section_end(para)
            .and_then(|id| self.sections.get(&id).copied())
            .unwrap_or(self.page)
    }

    /// An id no section break has.
    pub fn next_section_id(&self) -> u32 {
        let used = self.paragraphs.iter().filter_map(|p| p.style.section);
        let known = self.sections.keys().copied();
        used.chain(known).max().map_or(1, |m| m + 1)
    }

    /// The (header, footer) shown on page `page` (0-based).
    pub fn bands_for(&self, page: usize) -> (&Band, &Band) {
        self.variants.pick(page, &self.header, &self.footer)
    }

    pub fn from_text(text: &str) -> Self {
        let paragraphs: Vec<_> = text.split('\n').map(Paragraph::plain).collect();
        Self {
            paragraphs,
            ..Self::default()
        }
    }

    /// The document with footnotes written into the text, for formats that have none: a mark
    /// becomes `[n]` and a note starts with `[n] `. `None` when there are no footnotes.
    pub fn with_inline_notes(&self) -> Option<Document> {
        let numbers = self.note_numbers();
        if numbers.is_empty() && self.paragraphs.iter().all(|p| p.style.note.is_none()) {
            return None;
        }
        let mut out = self.clone();
        let mut previous = None;
        for p in &mut out.paragraphs {
            for run in &mut p.runs {
                if let Some(n) = run.style.note.and_then(|id| numbers.get(&id)) {
                    run.text = format!("[{n}]");
                    run.style.note = None;
                }
            }
            if let Some(id) = p.style.note.take() {
                if previous != Some(id) {
                    let label = numbers
                        .get(&id)
                        .map_or(String::from("[?] "), |n| format!("[{n}] "));
                    p.insert_runs(
                        0,
                        vec![crate::document::Run::new(label, Default::default())],
                    );
                }
                previous = Some(id);
            } else {
                previous = None;
            }
        }
        Some(out)
    }

    /// Plain text, paragraphs joined by `\n`.
    pub fn text(&self) -> String {
        self.paragraphs
            .iter()
            .map(Paragraph::text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn word_count(&self) -> usize {
        self.paragraphs
            .iter()
            .map(|p| p.text().split_whitespace().count())
            .sum()
    }

    pub fn end(&self) -> Position {
        let last = self.paragraphs.len() - 1;
        Position::new(last, self.paragraphs[last].len())
    }

    /// True if every character in `start..end` satisfies `pred` (empty range: false).
    pub fn all_chars(
        &self,
        start: Position,
        end: Position,
        pred: impl Fn(&CharStyle) -> bool,
    ) -> bool {
        if start == end {
            return false;
        }
        (start.para..=end.para).all(|i| {
            let p = &self.paragraphs[i];
            let from = if i == start.para { start.offset } else { 0 };
            let to = if i == end.para { end.offset } else { p.len() };
            p.all_chars(from, to, &pred)
        })
    }
}
