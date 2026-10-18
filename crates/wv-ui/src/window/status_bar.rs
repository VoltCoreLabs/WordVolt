use gpui_kit::*;
use wv_i18n::{Key, Lang, page_of, t, words};

use super::ribbon::view::{STEP, zoom_to};
use super::state::MainWindow;
use crate::theme::{Theme, radius, space};
use crate::ui_kit::{Icon, Tip, Weight, block, icon};

/// A small clickable label of the status bar (zoom controls).
fn zoom_button(id: &'static str, label: String, w: f32, hover: Rgba) -> Stateful<Div> {
    div()
        .id(id)
        .h(px(22.))
        .min_w(px(w))
        .px(px(4.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(radius::sm())
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(label)
}

pub fn status_bar(
    lang: Lang,
    (page, pages): (usize, usize),
    word_count: usize,
    zoom: f32,
    multi_page: bool,
    spell_on: bool,
    cx: &mut Context<MainWindow>,
) -> Div {
    let th = cx.global::<Theme>();
    let (muted, text, hover, accent) = (th.text_muted, th.text, th.surface_alt, th.accent);
    let layout_toggle = div()
        .id("status-pages")
        .size(px(22.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(radius::sm())
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .tip(t(
            lang,
            if multi_page {
                Key::TipPagesOne
            } else {
                Key::TipPagesRow
            },
        ))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _, _, cx| {
                this.editor
                    .update(cx, |view, cx| view.toggle_multi_page(cx));
                cx.notify();
            }),
        )
        .child(icon(
            if multi_page {
                Icon::PagesRow
            } else {
                Icon::PageSingle
            },
            Weight::Regular,
            px(15.),
            text,
        ));
    let spell_toggle = div()
        .id("status-spell")
        .size(px(22.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(radius::sm())
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .tip(t(
            lang,
            if spell_on {
                Key::TipSpellOff
            } else {
                Key::TipSpellOn
            },
        ))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _, window, cx| this.toggle_spell(window, cx)),
        )
        .child(icon(
            Icon::TextAa,
            if spell_on {
                Weight::Bold
            } else {
                Weight::Regular
            },
            px(15.),
            if spell_on { accent } else { muted },
        ));
    let zoom_controls = div()
        .flex()
        .items_center()
        .gap(space::xs())
        .child(spell_toggle)
        .child(layout_toggle)
        .child(
            zoom_button("status-zoom-out", "−".into(), 22., hover)
                .tip(t(lang, Key::TipZoomOut))
                .on_click(zoom_to(cx, |z| z / STEP)),
        )
        .child(
            zoom_button(
                "status-zoom-reset",
                format!("{}%", (zoom * 100.0).round() as u32),
                44.,
                hover,
            )
            .tip(t(lang, Key::TipZoomReset))
            .on_click(zoom_to(cx, |_| 1.0)),
        )
        .child(
            zoom_button("status-zoom-in", "+".into(), 22., hover)
                .tip(t(lang, Key::TipZoomIn))
                .on_click(zoom_to(cx, |z| z * STEP)),
        );
    block(cx)
        .px(space::lg())
        .h(px(32.))
        .flex()
        .items_center()
        .justify_between()
        .text_xs()
        .text_color(muted)
        .child(format!(
            "{} · {}",
            page_of(lang, page, pages),
            words(lang, word_count)
        ))
        .child(div().text_color(text).child(zoom_controls))
}
