use crate::document::{Band, Document, Paragraph};
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
    revision: u64,
    saved_revision: u64,
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
            revision: 0,
            saved_revision: 0,
        }
    }

    /// Grows with every change (also undo/redo); 0 = untouched document.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// True if the document changed since it was created/loaded or last marked saved.
    pub fn is_modified(&self) -> bool {
        self.revision != self.saved_revision
    }

    pub fn mark_saved(&mut self) {
        self.saved_revision = self.revision;
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Sets the header and footer text (not part of undo history; the document counts as changed).
    pub fn set_header_footer(&mut self, header: Band, footer: Band) {
        if self.doc.header != header || self.doc.footer != footer {
            self.doc.header = header;
            self.doc.footer = footer;
            self.revision += 1;
        }
    }

    /// Adds `{page}` to the footer, or removes it when it is already there.
    pub fn toggle_page_number(&mut self) {
        let mut footer = self.doc.footer.clone();
        footer.text = if footer.has_page_number() {
            footer.text.replace("{page}", "").trim().to_string()
        } else if footer.is_empty() {
            "{page}".to_string()
        } else {
            format!("{} {{page}}", footer.text)
        };
        self.set_header_footer(self.doc.header.clone(), footer);
    }

    pub fn undo(&mut self) -> bool {
        let Some(edit) = self.history.pop_undo() else {
            return false;
        };
        edit.revert(&mut self.doc);
        self.revision += 1;
        self.sel = edit.before;
        self.pending_style = None;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(edit) = self.history.pop_redo() else {
            return false;
        };
        edit.apply(&mut self.doc);
        self.revision += 1;
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
        self.revision += 1;
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
