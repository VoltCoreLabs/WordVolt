use gpui_kit::*;

use super::hover::{HoverLook, hover_layer, track_hover};
use crate::theme::{Theme, radius, space};

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum ButtonKind {
    Default,
    Primary,
}

/// Filled button with a border, 32px high. Hover fades in smoothly.
pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    kind: ButtonKind,
    cx: &App,
) -> Stateful<Div> {
    let id: ElementId = id.into();
    let t = cx.global::<Theme>();
    let (bg, hover, fg, border) = match kind {
        ButtonKind::Default => (t.surface, t.surface_alt, t.text, t.border),
        ButtonKind::Primary => (t.accent, t.accent_hover, t.on_accent, t.accent),
    };
    let look = HoverLook {
        bg: hover,
        border: Some(if kind == ButtonKind::Primary {
            hover
        } else {
            t.border_strong
        }),
        radius: radius::md(),
        bleed: px(1.),
    };
    let button = div()
        .id(id.clone())
        .relative()
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
        .cursor_pointer();
    track_hover(button, &id)
        .child(hover_layer(&id, look, None, cx))
        .child(label.into())
}
