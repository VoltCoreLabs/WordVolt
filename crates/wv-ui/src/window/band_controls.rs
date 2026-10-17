use gpui_kit::*;

use crate::theme::{Theme, radius};

/// A labelled check box row (the caller adds `on_click`).
pub fn check_box(
    id: &'static str,
    label: impl Into<SharedString>,
    on: bool,
    cx: &App,
) -> Stateful<Div> {
    let th = cx.global::<Theme>();
    let (accent, border, surface) = (th.accent, th.border_strong, th.surface);
    let mark = div()
        .size(px(16.))
        .flex_none()
        .rounded(radius::sm())
        .border_1()
        .border_color(if on { accent } else { border })
        .bg(if on { accent } else { surface })
        .flex()
        .items_center()
        .justify_center()
        .text_color(surface)
        .text_xs()
        .child(if on { "✓" } else { "" });
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(8.))
        .py(px(2.))
        .cursor_pointer()
        .text_sm()
        .child(mark)
        .child(label.into())
}
