use crate::document::{DocSettings, Document, Paragraph};
use crate::position::Selection;

/// One reversible change: paragraphs `at..at+removed.len()` were replaced by `inserted`.
#[derive(Clone, Debug)]
pub struct Edit {
    pub at: usize,
    pub removed: Vec<Paragraph>,
    pub inserted: Vec<Paragraph>,
    pub before: Selection,
    pub after: Selection,
    /// Header/footer/page setup before and after, for edits that change them.
    pub settings: Option<(DocSettings, DocSettings)>,
}

impl Edit {
    pub fn apply(&self, doc: &mut Document) {
        splice(doc, self.at, self.removed.len(), &self.inserted);
        if let Some((_, after)) = &self.settings {
            doc.set_settings(after);
        }
    }

    pub fn revert(&self, doc: &mut Document) {
        splice(doc, self.at, self.inserted.len(), &self.removed);
        if let Some((before, _)) = &self.settings {
            doc.set_settings(before);
        }
    }
}

fn splice(doc: &mut Document, at: usize, remove: usize, insert: &[Paragraph]) {
    doc.paragraphs
        .splice(at..at + remove, insert.iter().cloned());
    if doc.paragraphs.is_empty() {
        doc.paragraphs.push(Paragraph::default());
    }
}
