use std::path::PathBuf;

use gpui_kit::*;
use wv_core::Editor;
use wv_i18n::{APP_NAME, Key, Lang, t};

use super::color_picker::{ColorEvent, ColorPicker};
use super::combo::{Combo, ComboConfig, ComboEvent};
use super::link_editor::{LinkEditor, LinkEvent};
use crate::editor::EditorView;
use crate::theme::DEFAULT_FONT;
use crate::ui_kit::Icon;

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
    pub font_combo: Entity<Combo>,
    pub size_combo: Entity<Combo>,
    pub text_color: Entity<ColorPicker>,
    pub highlight: Entity<ColorPicker>,
    pub link_editor: Entity<LinkEditor>,
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
        let lang = Lang::default();
        let font_combo = cx.new(|cx| {
            let config = ComboConfig {
                id: "font",
                width: 168.0,
                default_label: Some(format!("{DEFAULT_FONT} ·")),
                preview_font: true,
                items: |cx| {
                    let mut fonts: Vec<String> = cx
                        .text_system()
                        .all_font_names()
                        .into_iter()
                        .filter(|n| !n.starts_with('.'))
                        .collect();
                    fonts.sort_by_key(|n| n.to_lowercase());
                    fonts.dedup();
                    fonts
                },
            };
            Combo::new(config, DEFAULT_FONT.to_string(), cx)
        });
        cx.subscribe(
            &font_combo,
            |this, _, ComboEvent::Picked(name): &ComboEvent, cx| {
                let name = name.clone();
                this.edit(cx, move |c| c.set_font_family(name));
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        let size_combo = cx.new(|cx| {
            let config = ComboConfig {
                id: "size",
                width: 64.0,
                default_label: None,
                preview_font: false,
                items: |_| {
                    [8, 9, 10, 11, 12, 14, 16, 18, 20, 22, 24, 26, 28, 36, 48, 72]
                        .iter()
                        .map(u16::to_string)
                        .collect()
                },
            };
            Combo::new(config, "12".to_string(), cx)
        });
        cx.subscribe(
            &size_combo,
            |this, _, ComboEvent::Picked(text): &ComboEvent, cx| {
                if let Some(size) = text
                    .as_deref()
                    .and_then(|t| t.parse::<u16>().ok())
                    .filter(|s| (1..=999).contains(s))
                {
                    this.edit(cx, move |c| c.set_size_pt(size));
                }
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        let text_color = cx.new(|cx| ColorPicker::new("text-color", Icon::TextColor, lang, cx));
        cx.subscribe(
            &text_color,
            |this, _, ColorEvent::Picked(color): &ColorEvent, cx| {
                let color = *color;
                this.edit(cx, move |c| c.set_color(color));
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        let highlight = cx.new(|cx| ColorPicker::new("highlight", Icon::Highlighter, lang, cx));
        cx.subscribe(
            &highlight,
            |this, _, ColorEvent::Picked(color): &ColorEvent, cx| {
                let color = *color;
                this.edit(cx, move |c| c.set_highlight(color));
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        let link_editor = cx.new(|cx| LinkEditor::new(lang, cx));
        cx.subscribe(
            &link_editor,
            |this, _, LinkEvent::Set(url): &LinkEvent, cx| {
                let url = url.clone();
                this.edit(cx, move |c| c.set_link(url));
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        Self {
            editor,
            font_combo,
            size_combo,
            text_color,
            highlight,
            link_editor,
            file_path: None,
            dirty: false,
            lang,
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

    /// File name without extension (suggested name in "Save as").
    pub fn file_stem(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_stem())
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
