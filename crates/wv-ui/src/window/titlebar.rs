use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_i18n::APP_NAME;

use super::state::MainWindow;
use crate::theme::{Theme, space};
use crate::ui_kit::{CONTROLS_WIDTH, window_controls};

/// Custom top bar: app name + document title centred (drag area), window buttons on the right.
/// macOS keeps its native traffic lights, so only a gap of the same width is reserved there.
pub fn titlebar(this: &MainWindow, window: &Window, cx: &App) -> Div {
    let t = cx.global::<Theme>();
    let muted = t.text_muted;
    // Left spacer is as wide as the window buttons, so the title stays truly centred.
    let drag_area = || {
        div()
            .h_full()
            .flex()
            .items_center()
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
    let spacer = drag_area().w(CONTROLS_WIDTH).flex_none();
    let title = drag_area()
        .flex_1()
        .justify_center()
        .gap(space::sm())
        .child(div().font_weight(FontWeight::SEMIBOLD).child(APP_NAME))
        .child(div().text_color(muted).child("—"))
        .child(div().text_color(muted).child(this.document_title()));
    div()
        .h(px(40.))
        .flex()
        .items_center()
        .child(spacer)
        .child(title)
        .when(!cfg!(target_os = "macos"), |bar| {
            bar.child(window_controls(window, cx))
        })
}
