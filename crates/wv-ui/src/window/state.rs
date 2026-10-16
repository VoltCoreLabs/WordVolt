use std::path::PathBuf;

use gpui_kit::*;
use wv_core::Editor;
use wv_i18n::{APP_NAME, Key, Lang, t};

use super::color_picker::{ColorEvent, ColorPicker};
use super::combo::{Combo, ComboConfig, ComboEvent};
use super::link_editor::{LinkEditor, LinkEvent};
use super::menu_button::{MenuButton, MenuEvent, MenuItem};
use super::style_names::{style_from_label, style_label};
use crate::editor::EditorView;
use crate::theme::DEFAULT_FONT;
use crate::ui_kit::Icon;
use wv_core::{CaseKind, NamedStyle};

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

/// Line spacing choices: label and multiplier.
const SPACINGS: [(&str, f32); 6] = [
    ("1.0", 1.0),
    ("1.15", 1.15),
    ("1.5", 1.5),
    ("2.0", 2.0),
    ("2.5", 2.5),
    ("3.0", 3.0),
];

/// UI state of the main window. The document itself lives in `editor` (wv-core inside).
pub struct MainWindow {
    pub editor: Entity<EditorView>,
    pub font_combo: Entity<Combo>,
    pub size_combo: Entity<Combo>,
    pub text_color: Entity<ColorPicker>,
    pub highlight: Entity<ColorPicker>,
    pub link_editor: Entity<LinkEditor>,
    pub case_menu: Entity<MenuButton>,
    pub spacing_menu: Entity<MenuButton>,
    pub style_combo: Entity<Combo>,
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
        let text_color = cx
            .new(|cx| ColorPicker::new("text-color", Icon::TextColor, Key::TipTextColor, lang, cx));
        cx.subscribe(
            &text_color,
            |this, _, ColorEvent::Picked(color): &ColorEvent, cx| {
                let color = *color;
                this.edit(cx, move |c| c.set_color(color));
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        let highlight = cx.new(|cx| {
            ColorPicker::new("highlight", Icon::Highlighter, Key::TipHighlight, lang, cx)
        });
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
        let case_menu = cx.new(|cx| {
            let items = vec![
                MenuItem::Tr(Key::CaseUpper),
                MenuItem::Tr(Key::CaseLower),
                MenuItem::Tr(Key::CaseTitle),
                MenuItem::Tr(Key::CaseSentence),
                MenuItem::Tr(Key::CaseDisplay),
            ];
            MenuButton::new("case", Icon::TextAa, Key::TipCase, items, lang, cx)
        });
        cx.subscribe(
            &case_menu,
            |this, _, MenuEvent::Picked(index): &MenuEvent, cx| {
                let index = *index;
                this.edit(cx, move |c| match index {
                    0 => c.change_case(CaseKind::Upper),
                    1 => c.change_case(CaseKind::Lower),
                    2 => c.change_case(CaseKind::Title),
                    3 => c.change_case(CaseKind::Sentence),
                    _ => c.toggle_caps(),
                });
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        let spacing_menu = cx.new(|cx| {
            let items = SPACINGS
                .iter()
                .map(|(label, _)| MenuItem::Text(label))
                .collect();
            MenuButton::new(
                "spacing",
                Icon::LineSpacing,
                Key::TipLineSpacing,
                items,
                lang,
                cx,
            )
        });
        cx.subscribe(
            &spacing_menu,
            |this, _, MenuEvent::Picked(index): &MenuEvent, cx| {
                if let Some((_, value)) = SPACINGS.get(*index) {
                    let value = *value;
                    this.edit(cx, move |c| c.set_line_spacing(Some(value)));
                }
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        let style_combo = cx.new(|cx| {
            let config = ComboConfig {
                id: "style",
                width: 160.0,
                default_label: None,
                preview_font: false,
                items: |_| {
                    NamedStyle::ALL
                        .iter()
                        .map(|s| style_label(Lang::default(), *s).to_string())
                        .collect()
                },
            };
            Combo::new(
                config,
                style_label(lang, NamedStyle::Normal).to_string(),
                cx,
            )
        });
        cx.subscribe(
            &style_combo,
            move |this, _, ComboEvent::Picked(label): &ComboEvent, cx| {
                if let Some(style) = label.as_deref().and_then(|l| style_from_label(lang, l)) {
                    this.edit(cx, move |c| c.set_named_style(style));
                }
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        Self {
            editor,
            case_menu,
            spacing_menu,
            style_combo,
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
