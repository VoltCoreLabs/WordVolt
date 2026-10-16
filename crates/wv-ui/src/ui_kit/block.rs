use gpui_kit::*;

use crate::theme::{Theme, radius};

/// A white panel with a thin border on the window background.
pub fn block(cx: &App) -> Div {
    let t = cx.global::<Theme>();
    div()
        .bg(t.surface)
        .border_1()
        .border_color(t.border)
        .rounded(radius::lg())
}
