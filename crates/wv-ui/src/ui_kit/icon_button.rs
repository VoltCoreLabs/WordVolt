use gpui_kit::*;

use super::hover::{HoverLook, hover_layer, track_hover};
use super::icon::{Icon, Weight, icon};
use crate::theme::{Theme, radius};

/// Square 32px button with an icon. `active` = toggled on (accent tint). Hover fades in smoothly.
pub fn icon_button(id: impl Into<ElementId>, glyph: Icon, active: bool, cx: &App) -> Stateful<Div> {
    let id: ElementId = id.into();
    let t = cx.global::<Theme>();
    let (bg, fg, border) = if active {
        (t.accent_subtle, t.accent, t.accent)
    } else {
        (t.surface, t.text, t.border)
    };
    let weight = if active {
        Weight::Bold
    } else {
        Weight::Regular
    };
    let mut button = div()
        .id(id.clone())
        .relative()
        .size(px(32.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(radius::md())
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_color(fg)
        .cursor_pointer();
    if !active {
        let look = HoverLook {
            bg: t.surface_alt,
            border: Some(t.border_strong),
            radius: radius::md(),
            bleed: px(1.),
        };
        button = track_hover(button, &id).child(hover_layer(&id, look, None, cx));
    }
    button.child(icon(glyph, weight, px(18.), fg))
}
