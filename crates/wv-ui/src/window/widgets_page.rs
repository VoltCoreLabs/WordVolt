use super::header_footer::{HeaderFooterEditor, HeaderFooterEvent};
use super::image_dialog::{ImageDialog, ImageDialogEvent};
use super::menu_button::{MenuButton, MenuEvent, MenuItem};
use super::page_setup::{PageSetupEditor, PageSetupEvent};
use super::paragraph_dialog::{ParagraphEditor, ParagraphEvent};
use super::state::MainWindow;
use super::table_picker::{TablePicker, TablePickerEvent};
use crate::ui_kit::Icon;
use gpui_kit::*;
use wv_core::{Margins, PageSpec};
use wv_i18n::{Key, Lang};

pub(super) fn make_table_menu(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<TablePicker> {
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
    table_menu
}

pub(super) fn make_band_editor(
    lang: Lang,
    cx: &mut Context<MainWindow>,
) -> Entity<HeaderFooterEditor> {
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
    band_editor
}

pub(super) fn make_paper_menu(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<MenuButton> {
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
    paper_menu
}

pub(super) fn make_margins_menu(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<MenuButton> {
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
    margins_menu
}

pub(super) fn make_columns_menu(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<MenuButton> {
    let columns_menu = cx.new(|cx| {
        let items = [Key::ColumnsOne, Key::ColumnsTwo, Key::ColumnsThree]
            .map(MenuItem::Tr)
            .to_vec();
        MenuButton::new("columns", Icon::Columns, Key::TipColumns, items, lang, cx)
    });
    cx.subscribe(
        &columns_menu,
        |this, _, MenuEvent::Picked(index): &MenuEvent, cx| {
            let n = (*index as u8 + 1).min(3);
            this.edit(cx, move |c| c.set_page(c.doc.page.with_columns(n)));
        },
    )
    .detach();
    columns_menu
}

pub(super) fn make_page_setup(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<PageSetupEditor> {
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
    page_setup
}

pub(super) fn make_paragraph_dialog(
    lang: Lang,
    cx: &mut Context<MainWindow>,
) -> Entity<ParagraphEditor> {
    let paragraph_dialog = cx.new(|cx| ParagraphEditor::new(lang, cx));
    cx.subscribe(
        &paragraph_dialog,
        |this, _, ParagraphEvent::Set(metrics): &ParagraphEvent, cx| {
            let metrics = metrics.clone();
            this.edit(cx, move |c| c.set_para_metrics(&metrics));
            this.editor.update(cx, |view, cx| view.request_focus(cx));
        },
    )
    .detach();
    paragraph_dialog
}

pub(super) fn make_table_edit_menu(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<MenuButton> {
    let table_edit_menu = cx.new(|cx| {
        let items = [
            Key::TblRowAbove,
            Key::TblRowBelow,
            Key::TblColLeft,
            Key::TblColRight,
            Key::TblDelRow,
            Key::TblDelCol,
            Key::TblDelTable,
            Key::CtxMerge,
            Key::CtxSplit,
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
                6 => c.delete_table(),
                7 => c.merge_cells(),
                _ => c.split_cell(),
            });
            this.editor.update(cx, |view, cx| view.request_focus(cx));
        },
    )
    .detach();
    table_edit_menu
}

pub(super) fn make_image_dialog(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<ImageDialog> {
    let image_dialog = cx.new(|cx| ImageDialog::new(lang, cx));
    cx.subscribe(
        &image_dialog,
        |this, _, ImageDialogEvent::Apply(s): &ImageDialogEvent, cx| {
            let s = *s;
            this.edit(cx, move |c| {
                c.set_image_geometry(s.width, s.height, s.float)
            });
            this.editor.update(cx, |view, cx| view.request_focus(cx));
        },
    )
    .detach();
    image_dialog
}
