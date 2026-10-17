use super::{Band, ImageData, PageSpec, Paragraph};
use crate::position::Position;
use crate::style::CharStyle;

/// A document: a list of paragraphs (never empty).
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub paragraphs: Vec<Paragraph>,
    pub header: Band,
    pub footer: Band,
    /// Pictures used by paragraphs (`ParaStyle::image`).
    pub images: Vec<ImageData>,
    pub page: PageSpec,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            paragraphs: vec![Paragraph::default()],
            header: Band::default(),
            footer: Band::default(),
            images: Vec::new(),
            page: PageSpec::default(),
        }
    }
}

/// Document-wide settings that are not paragraphs; changing them is one undo step.
#[derive(Clone, Debug, PartialEq)]
pub struct DocSettings {
    pub header: Band,
    pub footer: Band,
    pub page: PageSpec,
}

impl Document {
    pub fn settings(&self) -> DocSettings {
        DocSettings {
            header: self.header.clone(),
            footer: self.footer.clone(),
            page: self.page,
        }
    }

    pub fn set_settings(&mut self, settings: &DocSettings) {
        self.header = settings.header.clone();
        self.footer = settings.footer.clone();
        self.page = settings.page;
    }

    pub fn from_text(text: &str) -> Self {
        let paragraphs: Vec<_> = text.split('\n').map(Paragraph::plain).collect();
        Self {
            paragraphs,
            ..Self::default()
        }
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
