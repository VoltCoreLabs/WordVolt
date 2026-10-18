use super::Editor;
use crate::position::{Position, Selection};

impl Editor {
    /// Ends the section at the caret with a break (the next one starts on a new page): the
    /// paragraph is split there and its first half carries the break. Both sections keep the page
    /// setup of the one that was split. One undo step.
    pub fn insert_section_break(&mut self) {
        let at = self.sel.head;
        let p = &self.doc.paragraphs[at.para];
        if p.style.cell.is_some() || p.style.note.is_some() || p.style.image.is_some() {
            return;
        }
        let page = self.page_here();
        let id = self.doc.next_section_id();
        let mut first = p.clone();
        let mut rest = first.split_off(at.offset);
        first.style.section = Some(id);
        // The text after the break is a paragraph of its own (it keeps the old section end).
        rest.style.page_break = false;
        let after = Selection::caret(Position::new(at.para + 1, 0));
        let mut settings = self.doc.settings();
        settings.sections.insert(id, page);
        self.edit_paragraphs_with(at.para, 1, vec![first, rest], after, Some(settings));
    }

    /// Takes the break that ends the section at the caret away: the section joins the next one
    /// (the page setup of the next one is used for both).
    pub fn remove_section_break(&mut self) {
        let Some((para, id)) = (self.sel.head.para..self.doc.paragraphs.len())
            .find_map(|i| self.doc.paragraphs[i].style.section.map(|id| (i, id)))
        else {
            return;
        };
        let mut p = self.doc.paragraphs[para].clone();
        p.style.section = None;
        let mut settings = self.doc.settings();
        settings.sections.remove(&id);
        let after = self.sel;
        self.edit_paragraphs_with(para, 1, vec![p], after, Some(settings));
    }

    /// How many sections the document has.
    pub fn section_count(&self) -> usize {
        1 + self
            .doc
            .paragraphs
            .iter()
            .filter(|p| p.style.section.is_some())
            .count()
    }
}
