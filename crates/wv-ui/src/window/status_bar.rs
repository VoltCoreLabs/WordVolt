use gpui_kit::*;
use wv_i18n::{Key, Lang, page_of, t, words};

use super::ribbon::view::{STEP, zoom_to};
use super::state::MainWindow;
use crate::theme::{Theme, radius, space};
use crate::ui_kit::{Tip, block};

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
    cx: &mut Context<MainWindow>,
) -> Div {
    let th = cx.global::<Theme>();
    let (muted, text, hover) = (th.text_muted, th.text, th.surface_alt);
    let zoom_controls = div()
        .flex()
        .items_center()
        .gap(space::xs())
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
