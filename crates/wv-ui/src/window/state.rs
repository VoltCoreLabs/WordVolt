use super::widgets;
use super::widgets_page;
use std::path::PathBuf;

use gpui_kit::*;
use wv_core::Editor;
use wv_i18n::{APP_NAME, Key, Lang, t};

use super::color_picker::ColorPicker;
use super::combo::Combo;
use super::comment_editor::CommentEditor;
use super::doc_tabs::DocSlot;
use super::find_bar::FindBar;
use super::header_footer::HeaderFooterEditor;
use super::link_editor::LinkEditor;
use super::menu_bar::{CommandChosen, MenuBar};
use super::menu_button::MenuButton;
use super::page_setup::PageSetupEditor;
use super::paragraph_dialog::ParagraphEditor;
use super::table_picker::TablePicker;
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
    pub font_combo: Entity<Combo>,
    pub size_combo: Entity<Combo>,
    pub text_color: Entity<ColorPicker>,
    pub highlight: Entity<ColorPicker>,
    pub link_editor: Entity<LinkEditor>,
    pub comment_editor: Entity<CommentEditor>,
    pub case_menu: Entity<MenuButton>,
    pub spacing_menu: Entity<MenuButton>,
    pub style_combo: Entity<Combo>,
    pub find_bar: Entity<FindBar>,
    pub table_menu: Entity<TablePicker>,
    pub band_editor: Entity<HeaderFooterEditor>,
    pub table_edit_menu: Entity<MenuButton>,
    pub paper_menu: Entity<MenuButton>,
    pub margins_menu: Entity<MenuButton>,
    pub columns_menu: Entity<MenuButton>,
    pub page_setup: Entity<PageSetupEditor>,
    pub paragraph_dialog: Entity<ParagraphEditor>,
    pub image_dialog: Entity<super::image_dialog::ImageDialog>,
    /// Where the right-click menu was last opened (window coordinates).
    pub last_menu_at: Option<Point<Pixels>>,
    pub menu_bar: Entity<MenuBar>,
    /// `None` = document was never saved (shown as "New document").
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
    /// All open documents; `active` is the one shown (its state is in the fields above).
    pub docs: Vec<DocSlot>,
    pub active: usize,
    /// Word count of the document last counted: (editor, revision, words).
    pub words: (Option<EntityId>, u64, usize),
    /// Id of this running window (its recovery copies are marked with it).
    pub instance: u64,
    pub recovery_dir: PathBuf,
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
    /// The side panel lists the comments.
    pub comments_panel: bool,
    /// The "About" modal is shown.
    pub about_open: bool,
    pub styles_open: bool,
}

impl MainWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(EditorView::new);
        Self::observe_doc(&editor, cx);
        let lang = Lang::default();
        let font_combo = widgets::make_font_combo(lang, cx);
        let size_combo = widgets::make_size_combo(lang, cx);
        let text_color = widgets::make_text_color(lang, cx);
        let highlight = widgets::make_highlight(lang, cx);
        let link_editor = widgets::make_link_editor(lang, cx);
        let comment_editor = widgets::make_comment_editor(lang, cx);
        let case_menu = widgets::make_case_menu(lang, cx);
        let spacing_menu = widgets::make_spacing_menu(lang, cx);
        let style_combo = widgets::make_style_combo(lang, cx);
        let find_bar = cx.new(|cx| FindBar::new(editor.clone(), lang, cx));
        let table_menu = widgets_page::make_table_menu(lang, cx);
        let band_editor = widgets_page::make_band_editor(lang, cx);
        let paper_menu = widgets_page::make_paper_menu(lang, cx);
        let margins_menu = widgets_page::make_margins_menu(lang, cx);
        let columns_menu = widgets_page::make_columns_menu(lang, cx);
        let page_setup = widgets_page::make_page_setup(lang, cx);
        let image_dialog = widgets_page::make_image_dialog(lang, cx);
        let paragraph_dialog = widgets_page::make_paragraph_dialog(lang, cx);
        let menu_bar = cx.new(|_| MenuBar::new(lang));
        cx.subscribe_in(
            &menu_bar,
            window,
            |this, _, CommandChosen(c): &CommandChosen, window, cx| {
                this.run_command(*c, window, cx)
            },
        )
        .detach();
        let table_edit_menu = widgets_page::make_table_edit_menu(lang, cx);
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
            columns_menu,
            page_setup,
            paragraph_dialog,
            image_dialog,
            last_menu_at: None,
            menu_bar,
            font_combo,
            size_combo,
            text_color,
            highlight,
            link_editor,
            comment_editor,
            file_path: None,
            dirty: false,
            docs: vec![DocSlot::new(editor)],
            instance: wv_io::recovery::new_id(),
            recovery_dir: wv_io::recovery::default_dir(),
            active: 0,
            words: (None, 0, 0),
            context_menu: None,
            tab_menu: None,
            tab_scroll: ScrollHandle::new(),
            lang,
            applied_title: String::new(),
            tab: Tab::Home,
            ribbon_open: true,
            tab_pills: crate::ui_kit::PillState::new(Tab::ALL.len()),
            panel_open: false,
            comments_panel: false,
            about_open: false,
            styles_open: false,
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
