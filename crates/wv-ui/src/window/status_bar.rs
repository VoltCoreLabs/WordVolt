use gpui_kit::*;

use crate::theme::{Theme, space};
use crate::ui_kit::block;

pub fn status_bar(words: usize, cx: &App) -> Div {
    let muted = cx.global::<Theme>().text_muted;
    block(cx)
        .px(space::lg())
        .h(px(32.))
        .flex()
        .items_center()
        .justify_between()
        .text_xs()
        .text_color(muted)
        .child(format!("Page 1 of 1 · {words} words"))
        .child("100%")
}
