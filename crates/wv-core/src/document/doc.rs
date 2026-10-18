use std::collections::BTreeMap;

use super::{Band, BandVariants, ImageData, PageSpec, Paragraph, TableLook};
use crate::position::Position;
use crate::style::CharStyle;

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
}

/// The character that stands for a footnote mark in the text (the number is drawn instead).
pub const NOTE_MARK: char = '\u{FFFC}';

impl Document {
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
        }
    }

    pub fn set_settings(&mut self, settings: &DocSettings) {
        self.header = settings.header.clone();
        self.footer = settings.footer.clone();
        self.variants = settings.variants.clone();
        self.page = settings.page;
        self.table_looks = settings.table_looks.clone();
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
