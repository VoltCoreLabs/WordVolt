use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_core::{Align, ListKind};
use wv_i18n::{Key, t};

use super::menu_bar::MOD;
use super::state::MainWindow;
use crate::theme::{Theme, radius};
use crate::ui_kit::{Icon, Tip, icon_button, pop_in};

const WIDTH: f32 = 252.0;

type Action = fn(&mut MainWindow, &mut Window, &mut Context<MainWindow>);

impl MainWindow {
    /// Runs a context-menu command and puts the caret back into the page.
    fn menu_do(&mut self, f: Action, window: &mut Window, cx: &mut Context<Self>) {
        self.context_menu = None;
        f(self, window, cx);
        self.editor.update(cx, |view, cx| view.request_focus(cx));
        cx.notify();
    }
}

/// A command row: label on the left, shortcut on the right; greyed out when not `enabled`.
fn row(
    id: &'static str,
    label: &'static str,
    shortcut: Option<&'static str>,
    enabled: bool,
    action: Action,
    cx: &mut Context<MainWindow>,
) -> Stateful<Div> {
    let th = cx.global::<Theme>();
    let (hover, muted) = (th.surface_alt, th.text_muted);
    div()
        .id(id)
        .h(px(30.))
        .px(px(10.))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(24.))
        .rounded(radius::sm())
        .text_sm()
        .when(!enabled, |d| d.text_color(muted))
        .when(enabled, |d| {
            d.cursor_pointer()
                .hover(move |s| s.bg(hover))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| this.menu_do(action, window, cx)),
                )
        })
        .child(label)
        .child(
            div()
                .text_xs()
                .text_color(muted)
                .child(shortcut.map(|s| format!("{MOD}{s}")).unwrap_or_default()),
        )
}

fn separator(cx: &App) -> Div {
    let border = cx.global::<Theme>().border;
    div().h(px(1.)).my(px(4.)).bg(border)
}

/// One toggle of the mini toolbar at the top of the menu.
fn tool(
    id: &'static str,
    glyph: Icon,
    on: bool,
    tip: Key,
    lang: wv_i18n::Lang,
    action: Action,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    icon_button(id, glyph, on, cx)
        .tip(t(lang, tip))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, window, cx| this.menu_do(action, window, cx)),
        )
        .into_any_element()
}

/// The right-click menu at `at`: formatting toggles, clipboard, link, and what fits the place
/// (table rows and columns, picture size).
pub fn context_menu(
    this: &MainWindow,
    at: Point<Pixels>,
    cx: &mut Context<MainWindow>,
) -> impl IntoElement {
    let lang = this.lang;
    let th = cx.global::<Theme>();
    let (surface, border) = (th.surface, th.border);
    let (style, align, list, quote, selected, in_table, image, link) = {
        let core = &this.editor.read(cx).core;
        let caret = core.sel.head;
        let link = core.doc.paragraphs[caret.para]
            .style_after(caret.offset)
            .and_then(|s| s.link);
        (
            core.current_style(),
            core.align(),
            core.list_kind(),
            core.quote_active(),
            !core.selected_text().is_empty(),
            core.table_info().is_some(),
            core.image_at_caret().is_some(),
            link,
        )
    };

    let formats = div()
        .flex()
        .gap(px(4.))
        .child(tool(
            "ctx-bold",
            Icon::TextB,
            style.bold,
            Key::TipBold,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_bold()),
            cx,
        ))
        .child(tool(
            "ctx-italic",
            Icon::TextItalic,
            style.italic,
            Key::TipItalic,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_italic()),
            cx,
        ))
        .child(tool(
            "ctx-underline",
            Icon::TextUnderline,
            style.underline,
            Key::TipUnderline,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_underline()),
            cx,
        ))
        .child(tool(
            "ctx-strike",
            Icon::Strikethrough,
            style.strike,
            Key::TipStrike,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_strike()),
            cx,
        ))
        .child(tool(
            "ctx-code",
            Icon::Code,
            style.mono,
            Key::TipCode,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_mono()),
            cx,
        ))
        .child(tool(
            "ctx-spoiler",
            Icon::EyeSlash,
            style.spoiler,
            Key::TipSpoiler,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_spoiler()),
            cx,
        ));
    let paragraph = div()
        .flex()
        .gap(px(4.))
        .child(tool(
            "ctx-left",
            Icon::AlignLeft,
            align == Align::Left,
            Key::TipAlignLeft,
            lang,
            |t, _, cx| t.edit(cx, |c| c.set_align(Align::Left)),
            cx,
        ))
        .child(tool(
            "ctx-center",
            Icon::AlignCenter,
            align == Align::Center,
            Key::TipAlignCenter,
            lang,
            |t, _, cx| t.edit(cx, |c| c.set_align(Align::Center)),
            cx,
        ))
        .child(tool(
            "ctx-right",
            Icon::AlignRight,
            align == Align::Right,
            Key::TipAlignRight,
            lang,
            |t, _, cx| t.edit(cx, |c| c.set_align(Align::Right)),
            cx,
        ))
        .child(tool(
            "ctx-justify",
            Icon::AlignJustify,
            align == Align::Justify,
            Key::TipAlignJustify,
            lang,
            |t, _, cx| t.edit(cx, |c| c.set_align(Align::Justify)),
            cx,
        ))
        .child(tool(
            "ctx-bullets",
            Icon::ListBullets,
            list == Some(ListKind::Bullet),
            Key::TipBullets,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_list(ListKind::Bullet)),
            cx,
        ))
        .child(tool(
            "ctx-numbers",
            Icon::ListNumbers,
            list == Some(ListKind::Number),
            Key::TipNumbers,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_list(ListKind::Number)),
            cx,
        ))
        .child(tool(
            "ctx-quote",
            Icon::Quotes,
            quote,
            Key::TipQuote,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_quote()),
            cx,
        ));

    let cut: Action = |t, _, cx| {
        t.editor.update(cx, |view, cx| view.cut_selection(cx));
    };
    let copy: Action = |t, _, cx| {
        t.editor.update(cx, |view, cx| view.copy_selection(cx));
    };
    let paste: Action = |t, _, cx| {
        t.editor.update(cx, |view, cx| view.paste_clipboard(cx));
    };
    let mut items: Vec<AnyElement> = vec![
        row(
            "ctx-cut",
            t(lang, Key::CtxCut),
            Some("X"),
            selected,
            cut,
            cx,
        )
        .into_any_element(),
        row(
            "ctx-copy",
            t(lang, Key::CtxCopy),
            Some("C"),
            selected,
            copy,
            cx,
        )
        .into_any_element(),
        row(
            "ctx-paste",
            t(lang, Key::CtxPaste),
            Some("V"),
            true,
            paste,
            cx,
        )
        .into_any_element(),
        row(
            "ctx-all",
            t(lang, Key::CtxSelectAll),
            Some("A"),
            true,
            |t, _, cx| t.edit(cx, |c| c.select_all()),
            cx,
        )
        .into_any_element(),
        separator(cx).into_any_element(),
        row(
            "ctx-link",
            t(lang, Key::CtxLink),
            Some("K"),
            true,
            |t, window, cx| t.link_editor.update(cx, |e, cx| e.open(window, cx)),
            cx,
        )
        .into_any_element(),
    ];
    if link.is_some() {
        items.push(
            row(
                "ctx-unlink",
                t(lang, Key::RemoveLink),
                None,
                true,
                |t, _, cx| t.edit(cx, |c| c.set_link(None)),
                cx,
            )
            .into_any_element(),
        );
    }
    items.push(
        row(
            "ctx-clear",
            t(lang, Key::CtxClearFormat),
            Some("Shift+N"),
            true,
            |t, _, cx| t.edit(cx, |c| c.clear_formatting()),
            cx,
        )
        .into_any_element(),
    );
    if in_table {
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
    }
    if image {
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
    items.push(separator(cx).into_any_element());
    items.push(
        row(
            "ctx-find",
            t(lang, Key::MenuFind),
            Some("F"),
            true,
            |t, window, cx| t.find_bar.update(cx, |bar, cx| bar.open(false, window, cx)),
            cx,
        )
        .into_any_element(),
    );

    let panel = div()
        .id("context-menu")
        .occlude()
        .w(px(WIDTH))
        .p(px(6.))
        .flex()
        .flex_col()
        .gap(px(4.))
        .bg(surface)
        .border_1()
        .border_color(border)
        .rounded(radius::md())
        .shadow_lg()
        .child(formats)
        .child(paragraph)
        .child(separator(cx))
        .children(items);
    // A click anywhere else closes the menu (the page does not get it).
    deferred(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .occlude()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.context_menu = None;
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _, _, cx| {
                    this.context_menu = None;
                    cx.notify();
                }),
            )
            .child(
                anchored()
                    .position(at)
                    .anchor(Anchor::TopLeft)
                    .snap_to_window_with_margin(px(8.))
                    .child(pop_in(panel, "context-menu-pop".to_string())),
            ),
    )
    .with_priority(20)
}
