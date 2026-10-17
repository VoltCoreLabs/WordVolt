use std::path::{Path, PathBuf};

use gpui_kit::*;
use wv_core::Document;
use wv_i18n::{Key, Lang, t};

use super::state::MainWindow;
use crate::editor::EditorView;

/// An open document. The active one lives in `MainWindow`'s own fields (`editor`, `file_path`,
/// `dirty`); its slot is brought up to date whenever the tab is left.
pub struct DocSlot {
    pub editor: Entity<EditorView>,
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
}

/// File name, or "New document".
pub fn label_of(path: Option<&Path>, lang: Lang) -> String {
    path.and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| t(lang, Key::UntitledDocument).to_string())
}

impl MainWindow {
    /// Keeps `dirty` of the document whose editor this is up to date.
    pub(super) fn observe_doc(editor: &Entity<EditorView>, cx: &mut Context<Self>) {
        cx.observe(editor, |this, editor, cx| {
            let dirty = editor.read(cx).core.is_modified();
            if editor.entity_id() == this.editor.entity_id() {
                this.dirty = dirty;
                if let Some(at) = editor.update(cx, |view, _| view.context_request.take()) {
                    this.context_menu = Some(at);
                }
            } else if let Some(slot) = this
                .docs
                .iter_mut()
                .find(|d| d.editor.entity_id() == editor.entity_id())
            {
                slot.dirty = dirty;
            }
            cx.notify();
        })
        .detach();
    }

    /// Writes the active document's state back into its slot.
    fn park_active(&mut self) {
        if let Some(slot) = self.docs.get_mut(self.active) {
            slot.editor = self.editor.clone();
            slot.file_path = self.file_path.clone();
            slot.dirty = self.dirty;
        }
    }

    /// Makes document `index` the active one.
    fn activate(&mut self, index: usize, cx: &mut Context<Self>) {
        let slot = &self.docs[index];
        self.editor = slot.editor.clone();
        self.file_path = slot.file_path.clone();
        self.dirty = slot.dirty;
        self.active = index;
        self.tab_scroll.scroll_to_item(index);
        let editor = self.editor.clone();
        self.find_bar
            .update(cx, |bar, cx| bar.set_editor(editor, cx));
        self.editor.update(cx, |view, cx| view.request_focus(cx));
        cx.notify();
    }

    pub fn switch_to(&mut self, index: usize, cx: &mut Context<Self>) {
        if index != self.active && index < self.docs.len() {
            self.park_active();
            self.activate(index, cx);
        }
    }

    /// A new, empty document in a new tab.
    pub fn new_tab(&mut self, cx: &mut Context<Self>) {
        self.park_active();
        let editor = cx.new(EditorView::new);
        Self::observe_doc(&editor, cx);
        self.docs.push(DocSlot {
            editor,
            file_path: None,
            dirty: false,
        });
        self.activate(self.docs.len() - 1, cx);
    }

    /// The active document is an untouched new one (opening a file replaces it).
    fn is_pristine(&self, cx: &App) -> bool {
        let view = self.editor.read(cx);
        self.file_path.is_none() && !self.dirty && view.core.doc == Document::default()
    }

    /// Opens `doc` (read from `path`): the tab of an already open file is shown instead; an
    /// untouched new document is replaced; otherwise a new tab is made.
    pub(crate) fn install(&mut self, doc: Document, path: Option<PathBuf>, cx: &mut Context<Self>) {
        if let Some(path) = &path {
            let open = (0..self.docs.len()).find(|&i| {
                let known = if i == self.active {
                    &self.file_path
                } else {
                    &self.docs[i].file_path
                };
                known.as_ref() == Some(path)
            });
            if let Some(i) = open {
                self.switch_to(i, cx);
                return;
            }
        }
        if !self.is_pristine(cx) {
            self.new_tab(cx);
        }
        self.file_path = path;
        self.editor.update(cx, |view, cx| view.load(doc, cx));
        cx.notify();
    }

    /// Closes the active tab (the document was dealt with); the last one is replaced by an empty
    /// document.
    fn remove_active(&mut self, cx: &mut Context<Self>) {
        self.docs.remove(self.active);
        if self.docs.is_empty() {
            let editor = cx.new(EditorView::new);
            Self::observe_doc(&editor, cx);
            self.docs.push(DocSlot {
                editor,
                file_path: None,
                dirty: false,
            });
        }
        let next = self.active.min(self.docs.len() - 1);
        self.activate(next, cx);
    }

    /// Close button of tab `index`: asks about unsaved changes first.
    pub fn close_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.switch_to(index, cx);
        self.confirm_then(window, cx, Box::new(|this, _, cx| this.remove_active(cx)));
    }

    /// Label and unsaved mark of tab `index`.
    pub fn tab_info(&self, index: usize) -> (String, bool) {
        if index == self.active {
            (label_of(self.file_path.as_deref(), self.lang), self.dirty)
        } else {
            let slot = &self.docs[index];
            (label_of(slot.file_path.as_deref(), self.lang), slot.dirty)
        }
    }

    /// First tab at or after `start` with unsaved changes.
    fn unsaved_from(&self, start: usize) -> Option<usize> {
        (start..self.docs.len()).find(|&i| self.tab_info(i).1)
    }

    /// Window close request: every document with unsaved changes is asked about in turn (returns
    /// `false` while that goes on; the window closes once all are dealt with, and stays open if
    /// the user cancels).
    pub fn should_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.unsaved_from(0).is_none() {
            return true;
        }
        self.close_scan(0, window, cx);
        false
    }

    fn close_scan(&mut self, start: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.unsaved_from(start) else {
            window.remove_window();
            return;
        };
        cx.spawn_in(window, async move |this, cx| {
            this.update_in(cx, |this, window, cx| {
                this.switch_to(index, cx);
                // Saved or discarded: on to the next one.
                this.confirm_then(
                    window,
                    cx,
                    Box::new(move |this, window, cx| this.close_scan(index + 1, window, cx)),
                );
            })
            .ok();
        })
        .detach();
    }
}
