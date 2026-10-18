use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_i18n::{Key, t};

use super::context_objects::{image_items, table_items};
use super::context_parts::popup;
use super::context_sections::{format_tools, paragraph_tools, spelling_rows};
use super::menu_bar::MOD;
use super::state::MainWindow;
use crate::theme::{Theme, radius};
use crate::ui_kit::{Icon, Tip, icon_button};

pub(super) const WIDTH: f32 = 252.0;

pub(super) type Action = fn(&mut MainWindow, &mut Window, &mut Context<MainWindow>);

impl MainWindow {
    /// Runs a context-menu command and puts the caret back into the page.
    pub(super) fn menu_do(&mut self, f: Action, window: &mut Window, cx: &mut Context<Self>) {
        self.last_menu_at = self.context_menu.take().or(self.last_menu_at);
        self.tab_menu = None;
        f(self, window, cx);
        // The size dialog keeps the focus for its fields.
        if !self.image_dialog.read(cx).is_open() {
            self.editor.update(cx, |view, cx| view.request_focus(cx));
        }
        cx.notify();
    }
}

/// A command row: label on the left, shortcut on the right; greyed out when not `enabled`.
pub(super) fn row(
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
            div().text_xs().text_color(muted).child(
                shortcut
                    .map(|s| {
                        if s == "✓" {
                            s.to_string()
                        } else {
                            format!("{MOD}{s}")
                        }
                    })
                    .unwrap_or_default(),
            ),
        )
}

pub(super) fn separator(cx: &App) -> Div {
    let border = cx.global::<Theme>().border;
    div().h(px(1.)).my(px(4.)).bg(border)
}

/// One toggle of the mini toolbar at the top of the menu.
pub(super) fn tool(
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

    let spell_rows = spelling_rows(this, lang, cx);

    let formats = format_tools(&style, lang, cx);
    let paragraph = paragraph_tools(align, list, quote, lang, cx);

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
        table_items(this, lang, cx, &mut items);
    }
    if image {
        image_items(this, lang, cx, &mut items);
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
        .children(spell_rows)
        .child(formats)
        .child(paragraph)
        .child(separator(cx))
        .children(items);
    popup(
        at,
        panel,
        "context-menu-pop",
        |this| this.context_menu = None,
        cx,
    )
}
