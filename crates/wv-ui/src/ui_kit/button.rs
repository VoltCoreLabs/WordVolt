use gpui_kit::*;

use crate::theme::{Theme, radius, space};

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum ButtonKind {
    Default,
    Primary,
}

/// Filled button with a border, 32px high.
pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    kind: ButtonKind,
    cx: &App,
) -> Stateful<Div> {
    let t = cx.global::<Theme>();
    let (bg, hover, fg, border) = match kind {
        ButtonKind::Default => (t.surface, t.surface_alt, t.text, t.border),
        ButtonKind::Primary => (t.accent, t.accent_hover, t.on_accent, t.accent),
    };
    let border_strong = t.border_strong;
    div()
        .id(id)
        .h(px(32.))
        .px(space::md())
        .flex()
        .items_center()
        .justify_center()
        .rounded(radius::md())
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_color(fg)
        .text_sm()
        .cursor_pointer()
        .hover(move |s| s.bg(hover).border_color(border_strong))
        .child(label.into())
}
