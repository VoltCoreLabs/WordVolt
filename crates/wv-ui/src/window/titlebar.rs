use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_i18n::APP_NAME;

use super::state::MainWindow;
use super::tab_strip::tab_strip;
use crate::theme::space;
use crate::ui_kit::{CONTROLS_WIDTH, window_controls};

/// Custom top bar: the menus and the open documents as tabs on the left, the app name exactly in
/// the middle, the window buttons on the right (the two halves are equally wide, so the name stays
/// centred; empty parts drag the window). macOS keeps its native traffic lights, so a gap of the
/// same width is reserved there.
pub fn titlebar(this: &MainWindow, window: &Window, cx: &mut Context<MainWindow>) -> Div {
    let drag = || {
        div()
            .h_full()
            .window_control_area(WindowControlArea::Drag)
            .when(!cfg!(target_os = "windows"), |d| {
                d.on_mouse_down(MouseButton::Left, |ev, window, _| {
                    if ev.click_count == 2 {
                        window.zoom_window();
                    } else {
                        window.start_window_move();
                    }
                })
            })
    };
    let menus = div()
        .h_full()
        .flex_none()
        .flex()
        .items_center()
        .px(space::sm())
        .when(cfg!(target_os = "macos"), |d| d.pl(CONTROLS_WIDTH))
        .child(this.menu_bar.clone());
    let left = div()
        .h_full()
        .flex_1()
        .min_w_0()
        .flex()
        .items_center()
        .child(menus)
        .child(tab_strip(this, window, cx));
    let title = drag()
        .flex_none()
        .px(space::md())
        .flex()
        .items_center()
        .font_weight(FontWeight::SEMIBOLD)
        .child(APP_NAME);
    let right = div()
        .h_full()
        .flex_1()
        .min_w_0()
        .flex()
        .items_center()
        .child(drag().flex_1())
        .when(!cfg!(target_os = "macos"), |d| {
            d.child(window_controls(window, cx))
        });
    div()
        .h(px(40.))
        .flex()
        .items_center()
        .child(left)
        .child(title)
        .child(right)
}
