use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::icon::{Icon, Weight, icon};
use crate::theme::{Theme, radius};

/// Square 32px button with an icon. `active` = toggled on (accent tint).
pub fn icon_button(id: impl Into<ElementId>, glyph: Icon, active: bool, cx: &App) -> Stateful<Div> {
    let t = cx.global::<Theme>();
    let (bg, fg, border) = if active {
        (t.accent_subtle, t.accent, t.accent)
    } else {
        (t.surface, t.text, t.border)
    };
    let (hover_bg, hover_border) = (t.surface_alt, t.border_strong);
    let weight = if active {
        Weight::Bold
    } else {
        Weight::Regular
    };
    div()
        .id(id)
        .size(px(32.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(radius::md())
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_color(fg)
        .cursor_pointer()
        .when(!active, move |s| {
            s.hover(move |s| s.bg(hover_bg).border_color(hover_border))
        })
        .child(icon(glyph, weight, px(18.), fg))
}
