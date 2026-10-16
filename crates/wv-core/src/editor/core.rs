use crate::document::{Document, Paragraph};
use crate::history::{Edit, History};
use crate::position::{Position, Selection};
use crate::style::CharStyle;

/// All document changes go through here so that undo/redo always works.
pub struct Editor {
    pub doc: Document,
    pub sel: Selection,
    /// Style for the next typed text when the caret is collapsed (e.g. after pressing Bold).
    pub pending_style: Option<CharStyle>,
    pub(super) history: History,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new(Document::default())
    }
}

impl Editor {
    pub fn new(doc: Document) -> Self {
        Self {
            doc,
            sel: Selection::default(),
            pending_style: None,
            history: History::default(),
        }
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn undo(&mut self) -> bool {
        let Some(edit) = self.history.pop_undo() else {
            return false;
        };
        edit.revert(&mut self.doc);
        self.sel = edit.before;
        self.pending_style = None;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(edit) = self.history.pop_redo() else {
            return false;
        };
        edit.apply(&mut self.doc);
        self.sel = edit.after;
        self.pending_style = None;
        true
    }

    /// Replaces paragraphs `at..at+count` with `new` and records the change.
    pub(super) fn edit_paragraphs(
        &mut self,
        at: usize,
        count: usize,
        new: Vec<Paragraph>,
        after: Selection,
    ) {
        let edit = Edit {
            at,
            removed: self.doc.paragraphs[at..at + count].to_vec(),
            inserted: new,
            before: self.sel,
            after,
        };
        edit.apply(&mut self.doc);
        self.sel = after;
        self.history.push(edit);
    }

    /// Moves the caret; with `extend` the anchor stays (selecting).
    pub fn set_head(&mut self, head: Position, extend: bool) {
        self.sel.head = head;
        if !extend {
            self.sel.anchor = head;
        }
        self.pending_style = None;
    }
}
