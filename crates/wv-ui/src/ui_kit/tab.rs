use gpui_kit::prelude::FluentBuilder;
use std::time::Duration;

use gpui_kit::*;

use super::hover::{HoverLook, hover_layer, track_hover};
use crate::theme::{Theme, space};

/// Pill tab of the ribbon switcher (put several in a `pill_tabs` container). The active pill fades
/// in as a raised white capsule.
pub fn tab(label: &'static str, active: bool, cx: &App) -> Stateful<Div> {
    let t = cx.global::<Theme>();
    let (fg, hover, surface, border) = (
        if active { t.text } else { t.text_muted },
        t.surface,
        t.surface,
        t.border_strong,
    );
    let id = ElementId::from(SharedString::from(format!("tab-{label}")));
    let look = HoverLook {
        bg: hover,
        border: None,
        radius: px(999.),
        bleed: px(0.),
    };
    let capsule = if active {
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .rounded(px(999.))
            .border_1()
            .border_color(border)
            .bg(surface)
            .shadow_sm()
            .with_animation(
                SharedString::from(format!("tab-pill-{label}")),
                Animation::new(Duration::from_millis(200)).with_easing(ease_out_quint()),
                move |pill, delta| pill.opacity(delta),
            )
            .into_any_element()
    } else {
        div().into_any_element()
    };
    let tab = div()
        .id(id.clone())
        .relative()
        .px(space::lg())
        .py(px(6.))
        .rounded(px(999.))
        .text_color(fg)
        .text_sm()
        .when(active, |s| s.font_weight(FontWeight::SEMIBOLD))
        .cursor_pointer();
    track_hover(tab, &id)
        .child(hover_layer(&id, look, None, cx))
        .child(capsule)
        .child(div().relative().child(label))
}

/// Capsule that holds the pill tabs.
pub fn pill_tabs(tabs: impl IntoIterator<Item = Stateful<Div>>, cx: &App) -> Div {
    let t = cx.global::<Theme>();
    div()
        .flex()
        .items_center()
        .gap(px(2.))
        .p(px(3.))
        .rounded(px(999.))
        .bg(t.border)
        .children(tabs)
}
