use std::path::PathBuf;

use gpui_kit::*;
use wv_core::Editor;
use wv_i18n::{APP_NAME, Key, Lang, t};

use super::color_picker::{ColorEvent, ColorPicker};
use super::combo::{Combo, ComboConfig, ComboEvent, ItemLook};
use super::doc_tabs::DocSlot;
use super::find_bar::FindBar;
use super::header_footer::{HeaderFooterEditor, HeaderFooterEvent};
use super::link_editor::{LinkEditor, LinkEvent};
use super::menu_bar::{CommandChosen, MenuBar};
use super::menu_button::{MenuButton, MenuEvent, MenuItem};
use super::page_setup::{PageSetupEditor, PageSetupEvent};
use super::style_names::{style_from_label, style_label};
use super::table_picker::{TablePicker, TablePickerEvent};
use crate::editor::EditorView;
use crate::theme::DEFAULT_FONT;
use crate::ui_kit::Icon;
use wv_core::{CaseKind, Margins, NamedStyle, PageSpec};

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

/// Drop-down preview of a paragraph style: roughly its size (capped to keep rows compact), weight and colour.
fn style_look(label: &str) -> ItemLook {
    let Some(style) = style_from_label(Lang::default(), label) else {
        return ItemLook::default();
    };
    ItemLook {
        size_px: style
            .size_pt()
            .map(|pt| (f32::from(pt) * 96.0 / 72.0).min(24.0)),
        bold: style.bold(),
        color: style.color(),
        ..ItemLook::default()
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
    pub find_bar: Entity<FindBar>,
    pub table_menu: Entity<TablePicker>,
    pub band_editor: Entity<HeaderFooterEditor>,
    pub table_edit_menu: Entity<MenuButton>,
    pub paper_menu: Entity<MenuButton>,
    pub margins_menu: Entity<MenuButton>,
    pub page_setup: Entity<PageSetupEditor>,
    pub menu_bar: Entity<MenuBar>,
    /// `None` = document was never saved (shown as "New document").
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
    /// All open documents; `active` is the one shown (its state is in the fields above).
    pub docs: Vec<DocSlot>,
    pub active: usize,
    /// Where the right-click menu is open (window coordinates).
    pub context_menu: Option<Point<Pixels>>,
    /// The tab menu (window coordinates); it belongs to the active tab.
    pub tab_menu: Option<Point<Pixels>>,
    /// Horizontal scroll of the tab strip.
    pub tab_scroll: ScrollHandle,
    pub lang: Lang,
    pub applied_title: String,
    pub tab: Tab,
    pub ribbon_open: bool,
    pub tab_pills: crate::ui_kit::PillState,
    pub panel_open: bool,
}

impl MainWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(EditorView::new);
        Self::observe_doc(&editor, cx);
        let lang = Lang::default();
        let font_combo = cx.new(|cx| {
            let config = ComboConfig {
                id: "font",
                width: 168.0,
                default_label: Some(format!("{DEFAULT_FONT} ·")),
                look: Some(|name| ItemLook {
                    family: Some(name.to_string()),
                    ..ItemLook::default()
                }),
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
                look: None,
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
                width: 210.0,
                default_label: None,
                look: Some(style_look),
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
        let find_bar = cx.new(|cx| FindBar::new(editor.clone(), lang, cx));
        let table_menu = cx.new(|cx| TablePicker::new(lang, cx));
        cx.subscribe(
            &table_menu,
            |this, _, TablePickerEvent::Insert(rows, cols): &TablePickerEvent, cx| {
                let (rows, cols) = (*rows, *cols);
                this.edit(cx, move |c| c.insert_table(rows, cols));
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        let band_editor = cx.new(|cx| HeaderFooterEditor::new(lang, cx));
        cx.subscribe(
            &band_editor,
            |this, _, HeaderFooterEvent::Set(header, footer, variants): &HeaderFooterEvent, cx| {
                let (header, footer, variants) = (header.clone(), footer.clone(), variants.clone());
                this.edit(cx, move |c| c.set_bands(header, footer, variants));
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        let paper_menu = cx.new(|cx| {
            let items = PageSpec::SIZES
                .iter()
                .map(|(name, ..)| MenuItem::Text(name))
                .collect();
            MenuButton::new("paper", Icon::Printer, Key::TipPaperSize, items, lang, cx)
        });
        cx.subscribe(
            &paper_menu,
            |this, _, MenuEvent::Picked(index): &MenuEvent, cx| {
                let index = *index;
                this.edit(cx, move |c| c.set_page(c.doc.page.with_paper(index)));
            },
        )
        .detach();
        let margins_menu = cx.new(|cx| {
            let items = [
                Key::MarginNormal,
                Key::MarginNarrow,
                Key::MarginModerate,
                Key::MarginWide,
            ]
            .map(MenuItem::Tr)
            .to_vec();
            MenuButton::new(
                "margins",
                Icon::PageMargins,
                Key::TipMargins,
                items,
                lang,
                cx,
            )
        });
        cx.subscribe(
            &margins_menu,
            |this, _, MenuEvent::Picked(index): &MenuEvent, cx| {
                let preset = [
                    Margins::Normal,
                    Margins::Narrow,
                    Margins::Moderate,
                    Margins::Wide,
                ][(*index).min(3)];
                this.edit(cx, move |c| c.set_page(c.doc.page.with_margins(preset)));
            },
        )
        .detach();
        let page_setup = cx.new(|cx| PageSetupEditor::new(lang, cx));
        cx.subscribe(
            &page_setup,
            |this, _, PageSetupEvent::Set(page): &PageSetupEvent, cx| {
                let page = *page;
                this.edit(cx, move |c| c.set_page(page));
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        let menu_bar = cx.new(|_| MenuBar::new(lang));
        cx.subscribe_in(
            &menu_bar,
            window,
            |this, _, CommandChosen(c): &CommandChosen, window, cx| {
                this.run_command(*c, window, cx)
            },
        )
        .detach();
        let table_edit_menu = cx.new(|cx| {
            let items = [
                Key::TblRowAbove,
                Key::TblRowBelow,
                Key::TblColLeft,
                Key::TblColRight,
                Key::TblDelRow,
                Key::TblDelCol,
                Key::TblDelTable,
            ]
            .map(MenuItem::Tr)
            .to_vec();
            MenuButton::new("table-edit", Icon::Rows, Key::TipTableEdit, items, lang, cx)
        });
        cx.subscribe(
            &table_edit_menu,
            |this, _, MenuEvent::Picked(index): &MenuEvent, cx| {
                let index = *index;
                this.edit(cx, move |c| match index {
                    0 => c.insert_row(false),
                    1 => c.insert_row(true),
                    2 => c.insert_col(false),
                    3 => c.insert_col(true),
                    4 => c.delete_row(),
                    5 => c.delete_col(),
                    _ => c.delete_table(),
                });
                this.editor.update(cx, |view, cx| view.request_focus(cx));
            },
        )
        .detach();
        Self {
            editor: editor.clone(),
            case_menu,
            spacing_menu,
            style_combo,
            find_bar,
            table_menu,
            band_editor,
            table_edit_menu,
            paper_menu,
            margins_menu,
            page_setup,
            menu_bar,
            font_combo,
            size_combo,
            text_color,
            highlight,
            link_editor,
            file_path: None,
            dirty: false,
            docs: vec![DocSlot {
                editor,
                file_path: None,
                dirty: false,
            }],
            active: 0,
            context_menu: None,
            tab_menu: None,
            tab_scroll: ScrollHandle::new(),
            lang,
            applied_title: String::new(),
            tab: Tab::Home,
            ribbon_open: true,
            tab_pills: crate::ui_kit::PillState::new(Tab::ALL.len()),
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
