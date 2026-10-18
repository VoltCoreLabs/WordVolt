use gpui_kit::*;
use wv_i18n::{Key, t};

use super::context_menu::{Action, row, separator};
use super::context_parts::popup;
use super::state::MainWindow;
use crate::theme::{Theme, radius};

const WIDTH: f32 = 252.0;

/// The right-click menu of a tab (the tab was made active by the click): closing, saving and what
/// can be done with the file.
pub fn tab_menu(
    this: &MainWindow,
    at: Point<Pixels>,
    cx: &mut Context<MainWindow>,
) -> impl IntoElement {
    let lang = this.lang;
    let th = cx.global::<Theme>();
    let (surface, border) = (th.surface, th.border);
    let count = this.docs.len();
    let has_right = this.active + 1 < count;
    let has_file = this.file_path.is_some();

    let close: Action = |t, window, cx| t.close_tab(t.active, window, cx);
    let others: Action = |t, window, cx| t.close_other_tabs(window, cx);
    let right: Action = |t, window, cx| t.close_tabs_to_the_right(window, cx);
    let duplicate: Action = |t, _, cx| t.duplicate_tab(cx);
    let save: Action = |t, window, cx| t.save_document(window, cx);
    let save_as: Action = |t, window, cx| t.save_document_as(window, cx);
    let pdf: Action = |t, window, cx| t.export_pdf(window, cx);
    let reveal: Action = |t, _, cx| {
        if let Some(path) = &t.file_path {
            cx.reveal_path(path);
        }
    };
    let copy_path: Action = |t, _, cx| {
        if let Some(path) = &t.file_path {
            cx.write_to_clipboard(ClipboardItem::new_string(path.display().to_string()));
        }
    };
    let items: Vec<AnyElement> = vec![
        row(
            "tab-close",
            t(lang, Key::TabClose),
            Some("W"),
            true,
            close,
            cx,
        )
        .into_any_element(),
        row(
            "tab-close-others",
            t(lang, Key::TabCloseOthers),
            None,
            count > 1,
            others,
            cx,
        )
        .into_any_element(),
        row(
            "tab-close-right",
            t(lang, Key::TabCloseRight),
            None,
            has_right,
            right,
            cx,
        )
        .into_any_element(),
        separator(cx).into_any_element(),
        row("tab-save", t(lang, Key::Save), Some("S"), true, save, cx).into_any_element(),
        row(
            "tab-save-as",
            t(lang, Key::MenuSaveAs),
            Some("Shift+S"),
            true,
            save_as,
            cx,
        )
        .into_any_element(),
        row("tab-pdf", t(lang, Key::TipExportPdf), None, true, pdf, cx).into_any_element(),
        separator(cx).into_any_element(),
        row(
            "tab-duplicate",
            t(lang, Key::TabDuplicate),
            None,
            true,
            duplicate,
            cx,
        )
        .into_any_element(),
        row(
            "tab-reveal",
            t(lang, Key::TabReveal),
            None,
            has_file,
            reveal,
            cx,
        )
        .into_any_element(),
        row(
            "tab-copy-path",
            t(lang, Key::TabCopyPath),
            None,
            has_file,
            copy_path,
            cx,
        )
        .into_any_element(),
    ];
    let panel = div()
        .id("tab-menu")
        .occlude()
        .w(px(WIDTH))
        .p(px(6.))
        .flex()
        .flex_col()
        .bg(surface)
        .border_1()
        .border_color(border)
        .rounded(radius::md())
        .shadow_lg()
        .children(items);
    popup(at, panel, "tab-menu-pop", |this| this.tab_menu = None, cx)
}
