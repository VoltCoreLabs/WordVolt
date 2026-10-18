use super::context_menu::{Action, row, separator};
use super::context_parts::shade_row;
use super::state::MainWindow;
use gpui_kit::*;
use wv_core::Wrap;
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

/// Picture wrapping, size and fit.
pub(super) fn image_items(
    this: &MainWindow,
    lang: Lang,
    cx: &mut Context<MainWindow>,
    items: &mut Vec<AnyElement>,
) {
    items.push(separator(cx).into_any_element());
    let current = this
        .editor
        .read(cx)
        .core
        .image_at_caret()
        .map(|i| i.float.map(|f| f.wrap));
    let wraps: [(&'static str, Key, Option<Wrap>, Action); 6] = [
        ("ctx-wrap-inline", Key::WrapInline, None, |t, _, cx| {
            t.set_picture_wrap(None, cx)
        }),
        (
            "ctx-wrap-square",
            Key::WrapSquare,
            Some(Wrap::Square),
            |t, _, cx| t.set_picture_wrap(Some(Wrap::Square), cx),
        ),
        (
            "ctx-wrap-tight",
            Key::WrapTight,
            Some(Wrap::Tight),
            |t, _, cx| t.set_picture_wrap(Some(Wrap::Tight), cx),
        ),
        (
            "ctx-wrap-tb",
            Key::WrapTopBottom,
            Some(Wrap::TopBottom),
            |t, _, cx| t.set_picture_wrap(Some(Wrap::TopBottom), cx),
        ),
        (
            "ctx-wrap-front",
            Key::WrapFront,
            Some(Wrap::Front),
            |t, _, cx| t.set_picture_wrap(Some(Wrap::Front), cx),
        ),
        (
            "ctx-wrap-behind",
            Key::WrapBehind,
            Some(Wrap::Behind),
            |t, _, cx| t.set_picture_wrap(Some(Wrap::Behind), cx),
        ),
    ];
    for (id, key, wrap, action) in wraps {
        let mark = (current == Some(wrap)).then_some("✓");
        items.push(row(id, t(lang, key), mark, true, action, cx).into_any_element());
    }
    items.push(separator(cx).into_any_element());
    let (cropped, cropping, upright) = {
        let view = this.editor.read(cx);
        let image = view.core.image_at_caret();
        (
            image.is_some_and(|i| !i.crop.is_none()),
            view.cropping,
            // A turned picture is not cropped (the edges would not line up with the handles).
            image.is_none_or(|i| i.angle.rem_euclid(360.0) < 0.01),
        )
    };
    let crop_key = if cropping {
        Key::CtxCropDone
    } else {
        Key::CtxCrop
    };
    let pictures: [(&'static str, Key, bool, Action); 8] = [
        ("ctx-rotate-right", Key::CtxRotateRight, true, |t, _, cx| {
            t.edit(cx, |c| c.turn_image(90.0))
        }),
        ("ctx-rotate-left", Key::CtxRotateLeft, true, |t, _, cx| {
            t.edit(cx, |c| c.turn_image(-90.0))
        }),
        ("ctx-crop", crop_key, upright, |t, _, cx| {
            t.editor.update(cx, |view, cx| {
                view.cropping = !view.cropping;
                cx.notify();
            })
        }),
        ("ctx-crop-reset", Key::CtxCropReset, cropped, |t, _, cx| {
            t.edit(cx, |c| c.reset_image_crop())
        }),
        ("ctx-pic-smaller", Key::CtxPicSmaller, true, |t, _, cx| {
            t.edit(cx, |c| c.scale_image(0.8))
        }),
        ("ctx-pic-larger", Key::CtxPicLarger, true, |t, _, cx| {
            t.edit(cx, |c| c.scale_image(1.25))
        }),
        ("ctx-pic-fit", Key::CtxPicFit, true, |t, _, cx| {
            t.edit(cx, |c| c.set_image_width(f32::MAX))
        }),
        (
            "ctx-pic-settings",
            Key::CtxImageSettings,
            true,
            |t, window, cx| t.open_image_dialog(window, cx),
        ),
    ];
    for (id, key, enabled, action) in pictures {
        items.push(row(id, t(lang, key), None, enabled, action, cx).into_any_element());
    }
}
