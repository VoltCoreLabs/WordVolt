use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::state::MainWindow;
use super::tab_strip::tab_strip;
use crate::theme::space;
use crate::ui_kit::{CONTROLS_WIDTH, window_controls};

/// Custom top bar: the menus, the open documents as tabs (the empty part drags the window) and the
/// window buttons on the right. macOS keeps its native traffic lights, so a gap of the same
/// width is reserved there.
pub fn titlebar(this: &MainWindow, window: &Window, cx: &mut Context<MainWindow>) -> Div {
    let menus = div()
        .h_full()
        .flex_none()
        .flex()
        .items_center()
        .px(space::sm())
        .when(cfg!(target_os = "macos"), |d| d.pl(CONTROLS_WIDTH))
        .child(this.menu_bar.clone());
    let tabs = tab_strip(this, cx);
    div()
        .h(px(40.))
        .flex()
        .items_center()
        .child(menus)
        .child(tabs)
        .when(!cfg!(target_os = "macos"), |bar| {
            bar.child(window_controls(window, cx))
        })
}
