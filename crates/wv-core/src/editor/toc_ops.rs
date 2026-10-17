use super::Editor;
use crate::document::Paragraph;
use crate::position::{Position, Selection};
use crate::style::{CharStyle, NamedStyle};

/// One line of a table of contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TocEntry {
    /// 1..=3 (Heading 1..3).
    pub level: u8,
    pub text: String,
    pub page: usize,
}

/// A heading of the document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heading {
    pub para: usize,
    pub level: u8,
    pub text: String,
}

fn level(style: NamedStyle) -> Option<u8> {
    match style {
        NamedStyle::Heading1 => Some(1),
        NamedStyle::Heading2 => Some(2),
        NamedStyle::Heading3 => Some(3),
        _ => None,
    }
}

impl Editor {
    /// Headings (Heading 1-3, outside tables and the table of contents) in document order.
    pub fn headings(&self) -> Vec<Heading> {
        self.doc
            .paragraphs
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.style.toc && p.style.cell.is_none() && !p.is_empty())
            .filter_map(|(para, p)| {
                level(p.style.named).map(|level| Heading {
                    para,
                    level,
                    text: p.text(),
                })
            })
            .collect()
    }

    /// Range of the existing table of contents (`first..=last`), if there is one.
    pub fn toc_range(&self) -> Option<(usize, usize)> {
        let paras = &self.doc.paragraphs;
        let first = paras.iter().position(|p| p.style.toc)?;
        let last = first + paras[first..].iter().take_while(|p| p.style.toc).count() - 1;
        Some((first, last))
    }

    /// The heading a contents entry (paragraph `para`) points to: the one at the same place in
    /// the list, else the first heading with the same text. `None` for other paragraphs.
    pub fn toc_target(&self, para: usize) -> Option<usize> {
        let (first, last) = self.toc_range()?;
        if para <= first || para > last {
            return None;
        }
        let text = self.doc.paragraphs[para].text();
        let label = text.split('\t').next()?;
        let headings = self.headings();
        headings
            .get(para - first - 1)
            .filter(|h| h.text == label)
            .or_else(|| headings.iter().find(|h| h.text == label))
            .map(|h| h.para)
    }

    /// Inserts the table of contents after the caret's paragraph, or replaces the existing one
    /// (one undo step). Entries are indented by level and end with a tab and the page number (the layout right-aligns it with dot leaders).
    pub fn set_toc(&mut self, title: &str, entries: &[TocEntry]) {
        let mut paras = Vec::with_capacity(entries.len() + 1);
        let mut head = Paragraph::plain(title);
        head.style.toc = true;
        head.style.named = NamedStyle::Subtitle;
        paras.push(head);
        for e in entries {
            let mut p = Paragraph::default();
            p.insert(0, &format!("{}\t{}", e.text, e.page), &CharStyle::default());
            p.style.toc = true;
            p.style.indent = e.level.saturating_sub(1);
            paras.push(p);
        }
        let (at, removed) = match self.toc_range() {
            Some((first, last)) => (first, last - first + 1),
            None => (self.sel.head.para + 1, 0),
        };
        // Typing must be possible after the contents.
        if at + removed >= self.doc.paragraphs.len() {
            paras.push(Paragraph::default());
        }
        let caret = Position::new(at + paras.len() - 1, 0);
        self.edit_paragraphs(at, removed, paras, Selection::caret(caret));
    }
}
