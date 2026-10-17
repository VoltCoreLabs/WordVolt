use gpui_kit::*;

use super::block::block;
use crate::theme::{Theme, space};

/// Ribbon group: a block with a row of controls and a caption below.
pub fn block_group(caption: &'static str, controls: Vec<AnyElement>, cx: &App) -> Div {
    let muted = cx.global::<Theme>().text_muted;
    block(cx)
        .p(space::sm())
        .flex()
        .flex_col()
        .gap(space::sm())
        .child(div().flex().gap(space::sm()).children(controls))
        .child(
            div()
                .w_full()
                .text_center()
                .text_xs()
                .text_color(muted)
                .child(caption),
        )
}
