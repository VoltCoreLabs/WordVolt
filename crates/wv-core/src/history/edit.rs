use crate::document::{Document, Paragraph};
use crate::position::Selection;

/// One reversible change: paragraphs `at..at+removed.len()` were replaced by `inserted`.
#[derive(Clone, Debug)]
pub struct Edit {
    pub at: usize,
    pub removed: Vec<Paragraph>,
    pub inserted: Vec<Paragraph>,
    pub before: Selection,
    pub after: Selection,
}

impl Edit {
    pub fn apply(&self, doc: &mut Document) {
        splice(doc, self.at, self.removed.len(), &self.inserted);
    }

    pub fn revert(&self, doc: &mut Document) {
        splice(doc, self.at, self.inserted.len(), &self.removed);
    }
}

fn splice(doc: &mut Document, at: usize, remove: usize, insert: &[Paragraph]) {
    doc.paragraphs
        .splice(at..at + remove, insert.iter().cloned());
    if doc.paragraphs.is_empty() {
        doc.paragraphs.push(Paragraph::default());
    }
}
