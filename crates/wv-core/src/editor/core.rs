use crate::document::{Band, BandVariants, DocSettings, Document, PageSpec, Paragraph};
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
    /// Text being composed by an input method.
    pub(super) marked: Option<super::ime_ops::Marked>,
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
            marked: None,
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

    /// Counts as changed since the last save (a recovered document that was never saved).
    pub fn mark_modified(&mut self) {
        self.revision += 1;
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

    /// Sets the header and footer (one undo step).
    pub fn set_header_footer(&mut self, header: Band, footer: Band) {
        let mut settings = self.doc.settings();
        (settings.header, settings.footer) = (header, footer);
        self.change_settings(settings);
    }

    /// Sets the main header and footer together with the first/even-page variants (one undo step).
    pub fn set_bands(&mut self, header: Band, footer: Band, variants: BandVariants) {
        let mut settings = self.doc.settings();
        (settings.header, settings.footer, settings.variants) = (header, footer, variants);
        self.change_settings(settings);
    }

    /// Sets page size and margins (one undo step).
    pub fn set_page(&mut self, page: PageSpec) {
        let mut settings = self.doc.settings();
        settings.page = page;
        self.change_settings(settings);
    }

    pub(super) fn change_settings(&mut self, settings: DocSettings) {
        let before = self.doc.settings();
        if before == settings {
            return;
        }
        let edit = Edit {
            at: 0,
            removed: vec![],
            inserted: vec![],
            before: self.sel,
            after: self.sel,
            settings: Some((before, settings)),
        };
        edit.apply(&mut self.doc);
        self.revision += 1;
        self.history.push(edit);
    }

    /// Adds `{page}` to the footer, or removes it when it is already there.
    pub fn toggle_page_number(&mut self) {
        let mut footer = self.doc.footer.clone();
        let text = footer.text();
        let new = if footer.has_page_number() {
            text.replace("{page}", "").trim().to_string()
        } else if footer.is_empty() {
            "{page}".to_string()
        } else {
            format!("{text} {{page}}")
        };
        footer.set_text(&new);
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
        self.edit_paragraphs_with(at, count, new, after, None);
    }

    /// Like `edit_paragraphs`; `settings` (the document settings afterwards) change in the same
    /// undo step.
    pub(super) fn edit_paragraphs_with(
        &mut self,
        at: usize,
        count: usize,
        new: Vec<Paragraph>,
        after: Selection,
        settings: Option<DocSettings>,
    ) {
        let settings = settings.map(|s| (self.doc.settings(), s));
        let edit = Edit {
            at,
            removed: self.doc.paragraphs[at..at + count].to_vec(),
            inserted: new,
            before: self.sel,
            after,
            settings,
        };
        edit.apply(&mut self.doc);
        self.revision += 1;
        self.sel = after;
        self.history.push(edit);
    }

    /// Changes the settings of the latest undo step (it must be a settings change) and the
    /// document, so a drag stays one step.
    pub(super) fn amend_last_settings(&mut self, f: impl FnOnce(&mut DocSettings)) {
        if let Some(edit) = self.history.last_mut()
            && let Some((_, after)) = edit.settings.as_mut()
        {
            f(after);
            self.doc.set_settings(after);
            self.revision += 1;
        }
    }

    /// Replaces paragraph `at` in the document and in the latest undo step (which must be an edit
    /// of exactly that paragraph), so a drag stays one step.
    pub(super) fn amend_last_edit(&mut self, at: usize, para: Paragraph) {
        if let Some(edit) = self.history.last_mut()
            && edit.at == at
            && edit.inserted.len() == 1
        {
            edit.inserted[0] = para.clone();
            self.doc.paragraphs[at] = para;
            self.revision += 1;
        }
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
