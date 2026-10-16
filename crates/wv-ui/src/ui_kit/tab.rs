use gpui_kit::prelude::FluentBuilder;
use std::time::Duration;

use gpui_kit::*;

use crate::theme::{Theme, radius, space};

/// Ribbon tab. The accent underline slides in when the tab becomes active.
pub fn tab(label: &'static str, active: bool, cx: &App) -> Stateful<Div> {
    let t = cx.global::<Theme>();
    let (fg, accent, hover) = (
        if active { t.text } else { t.text_muted },
        t.accent,
        t.surface,
    );
    let indicator = if active {
        div()
            .h(px(3.))
            .rounded(px(2.))
            .bg(accent)
            .with_animation(
                SharedString::from(format!("tab-indicator-{label}")),
                Animation::new(Duration::from_millis(220)).with_easing(ease_out_quint()),
                |bar, delta| bar.w(relative(delta)),
            )
            .into_any_element()
    } else {
        div().h(px(3.)).into_any_element()
    };
    div()
        .id(SharedString::from(format!("tab-{label}")))
        .flex()
        .flex_col()
        .gap(px(2.))
        .px(space::md())
        .pt(space::xs())
        .rounded_t(radius::sm())
        .text_color(fg)
        .text_sm()
        .when(active, |s| s.font_weight(FontWeight::SEMIBOLD))
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(div().py(space::xs()).child(label))
        .child(indicator)
}
