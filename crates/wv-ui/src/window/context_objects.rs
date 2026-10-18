use super::context_menu::{Action, row, separator};
use super::context_parts::shade_row;
use super::state::MainWindow;
use gpui_kit::*;
use wv_i18n::Lang;
use wv_i18n::{Key, t};

/// Table rows and columns.
pub(super) fn table_items(
    this: &MainWindow,
    lang: Lang,
    cx: &mut Context<MainWindow>,
    items: &mut Vec<AnyElement>,
) {
    items.push(separator(cx).into_any_element());
    let table: [(&'static str, Key, Action); 7] = [
        ("ctx-row-up", Key::TblRowAbove, |t, _, cx| {
            t.edit(cx, |c| c.insert_row(false))
        }),
        ("ctx-row-down", Key::TblRowBelow, |t, _, cx| {
            t.edit(cx, |c| c.insert_row(true))
        }),
        ("ctx-col-left", Key::TblColLeft, |t, _, cx| {
            t.edit(cx, |c| c.insert_col(false))
        }),
        ("ctx-col-right", Key::TblColRight, |t, _, cx| {
            t.edit(cx, |c| c.insert_col(true))
        }),
        ("ctx-del-row", Key::TblDelRow, |t, _, cx| {
            t.edit(cx, |c| c.delete_row())
        }),
        ("ctx-del-col", Key::TblDelCol, |t, _, cx| {
            t.edit(cx, |c| c.delete_col())
        }),
        ("ctx-del-table", Key::TblDelTable, |t, _, cx| {
            t.edit(cx, |c| c.delete_table())
        }),
    ];
    for (id, key, action) in table {
        items.push(row(id, t(lang, key), None, true, action, cx).into_any_element());
    }
    let (can_merge, can_split) = {
        let core = &this.editor.read(cx).core;
        (core.can_merge(), core.can_split())
    };
    let merge: Action = |t, _, cx| t.edit(cx, |c| c.merge_cells());
    let split: Action = |t, _, cx| t.edit(cx, |c| c.split_cell());
    items.push(
        row(
            "ctx-merge",
            t(lang, Key::CtxMerge),
            None,
            can_merge,
            merge,
            cx,
        )
        .into_any_element(),
    );
    items.push(
        row(
            "ctx-split",
            t(lang, Key::CtxSplit),
            None,
            can_split,
            split,
            cx,
        )
        .into_any_element(),
    );
    items.push(shade_row(lang, cx).into_any_element());
}

/// Picture size and fit.
pub(super) fn image_items(lang: Lang, cx: &mut Context<MainWindow>, items: &mut Vec<AnyElement>) {
    items.push(separator(cx).into_any_element());
    let pictures: [(&'static str, Key, Action); 3] = [
        ("ctx-pic-smaller", Key::CtxPicSmaller, |t, _, cx| {
            t.edit(cx, |c| c.scale_image(0.8))
        }),
        ("ctx-pic-larger", Key::CtxPicLarger, |t, _, cx| {
            t.edit(cx, |c| c.scale_image(1.25))
        }),
        ("ctx-pic-fit", Key::CtxPicFit, |t, _, cx| {
            t.edit(cx, |c| c.set_image_width(f32::MAX))
        }),
    ];
    for (id, key, action) in pictures {
        items.push(row(id, t(lang, key), None, true, action, cx).into_any_element());
    }
}
