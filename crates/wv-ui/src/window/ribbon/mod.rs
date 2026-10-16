//! Ribbon: tab bar plus the command groups of the active tab.

mod home;
mod insert;
mod placeholder;
mod tabs;

use std::time::Duration;

use gpui_kit::*;

use super::state::{MainWindow, Tab};
use crate::theme::space;

pub use tabs::tab_bar;

const HEIGHT: f32 = 104.;

/// Command groups of the active tab. Fades in on tab switch; collapses with animation.
pub fn ribbon(this: &MainWindow, cx: &mut Context<MainWindow>) -> impl IntoElement {
    let (open, tab) = (this.ribbon_open, this.tab);
    let groups = match tab {
        Tab::Home => home::groups(this, cx),
        Tab::Insert => insert::groups(cx),
        Tab::Layout | Tab::View => placeholder::groups(tab, cx),
    };
    let row = div()
        .flex()
        .gap(space::md())
        .children(groups)
        .with_animation(
            SharedString::from(format!("ribbon-fade-{}", tab.label())),
            Animation::new(Duration::from_millis(200)).with_easing(ease_out_quint()),
            |el, delta| el.opacity(delta),
        );
    div()
        .overflow_hidden()
        .flex_none()
        .child(row)
        .with_animation(
            SharedString::from(format!("ribbon-height-{open}")),
            Animation::new(Duration::from_millis(240)).with_easing(ease_out_quint()),
            move |el, delta| {
                let k = if open { delta } else { 1.0 - delta };
                el.h(px(HEIGHT * k))
            },
        )
}
