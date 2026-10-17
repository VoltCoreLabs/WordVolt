use gpui_kit::*;
use wv_i18n::{Lang, page_of, words};

use crate::theme::{Theme, space};
use crate::ui_kit::block;

pub fn status_bar(
    lang: Lang,
    (page, pages): (usize, usize),
    word_count: usize,
    zoom: f32,
    cx: &App,
) -> Div {
    let muted = cx.global::<Theme>().text_muted;
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
        .child(format!("{}%", (zoom * 100.0).round() as u32))
}
