use std::path::PathBuf;

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

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            Tab::Home => t(lang, Key::TabHome),
            Tab::Insert => t(lang, Key::TabInsert),
            Tab::Layout => t(lang, Key::TabLayout),
            Tab::View => t(lang, Key::TabView),
        }
    }
}

/// UI state of the main window. The document itself lives in `editor` (wv-core inside).
pub struct MainWindow {
    pub editor: Entity<EditorView>,
    /// `None` = document was never saved (shown as "New document").
    pub file_path: Option<PathBuf>,
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
            this.dirty = editor.read(cx).core.is_modified();
            cx.notify();
        })
        .detach();
        Self {
            editor,
            file_path: None,
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

    /// File name, or "New document" if there is none.
    pub fn file_label(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| t(self.lang, Key::UntitledDocument).to_string())
    }

    /// Title shown in the title bar; `•` marks unsaved changes.
    pub fn document_title(&self) -> String {
        let name = self.file_label();
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
