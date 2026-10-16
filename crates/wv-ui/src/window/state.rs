use gpui_kit::*;
use wv_core::Editor;
use wv_i18n::{APP_NAME, Key, Lang, t};

use crate::editor::EditorView;

/// Ribbon tabs.
#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Home,
    Insert,
    Layout,
    View,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Home, Tab::Insert, Tab::Layout, Tab::View];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Home => "Home",
            Tab::Insert => "Insert",
            Tab::Layout => "Layout",
            Tab::View => "View",
        }
    }
}

/// UI state of the main window. The document itself lives in `editor` (wv-core inside).
pub struct MainWindow {
    pub editor: Entity<EditorView>,
    /// `None` = document was never saved (shown as "New document").
    pub file_name: Option<String>,
    pub dirty: bool,
    pub lang: Lang,
    pub applied_title: String,
    pub tab: Tab,
    pub ribbon_open: bool,
    pub panel_open: bool,
}

impl MainWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let editor = cx.new(EditorView::new);
        cx.observe(&editor, |this, editor, cx| {
            this.dirty = editor.read(cx).core.revision() != 0;
            cx.notify();
        })
        .detach();
        Self {
            editor,
            file_name: None,
            dirty: false,
            lang: Lang::default(),
            applied_title: String::new(),
            tab: Tab::Home,
            ribbon_open: true,
            panel_open: false,
        }
    }

    /// Runs an editing action on the document (used by ribbon buttons).
    pub fn edit(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut Editor)) {
        self.editor.update(cx, |view, cx| view.apply(cx, f));
    }

    /// File name, or "New document" if there is none; `•` marks unsaved changes.
    pub fn document_title(&self) -> String {
        let name = self
            .file_name
            .clone()
            .unwrap_or_else(|| t(self.lang, Key::UntitledDocument).to_string());
        if self.dirty {
            format!("{name} •")
        } else {
            name
        }
    }

    /// Title for the OS (taskbar, alt-tab).
    pub fn window_title(&self) -> String {
        format!("{APP_NAME} — {}", self.document_title())
    }
}
