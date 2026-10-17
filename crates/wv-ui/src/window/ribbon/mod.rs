//! Ribbon: tab bar plus the command groups of the active tab.

mod home;
mod insert;
mod layout;
mod placeholder;
mod tabs;

use std::time::Duration;

use gpui_kit::*;

use super::state::{MainWindow, Tab};
use crate::theme::space;

pub use tabs::tab_bar;

/// Upper bound of the ribbon height while open (it grows with the number of wrapped rows).
const MAX_HEIGHT: f32 = 480.;

/// Command groups of the active tab. Fades in on tab switch; collapses with animation.
pub fn ribbon(this: &MainWindow, cx: &mut Context<MainWindow>) -> impl IntoElement {
    let (open, tab) = (this.ribbon_open, this.tab);
    let groups = match tab {
        Tab::Home => home::groups(this, cx),
        Tab::Insert => insert::groups(this, cx),
        Tab::Layout => layout::groups(this, cx),
        Tab::View => placeholder::groups(tab, this.lang, cx),
    };
    let row = div()
        .flex()
        .flex_wrap()
        .gap(space::md())
        .children(groups)
        .with_animation(
            SharedString::from(format!("ribbon-fade-{}", tab.label(this.lang))),
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
                el.max_h(px(MAX_HEIGHT * k))
            },
        )
}
