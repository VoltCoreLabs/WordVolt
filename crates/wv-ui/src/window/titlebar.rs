use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_i18n::APP_NAME;

use super::state::MainWindow;
use crate::theme::{Theme, space};
use crate::ui_kit::window_controls;

/// Custom top bar: app name + document title on the left (drag area), window buttons on the right.
/// macOS keeps its native traffic lights, so only a left gap is reserved there.
pub fn titlebar(this: &MainWindow, window: &Window, cx: &App) -> Div {
    let t = cx.global::<Theme>();
    let muted = t.text_muted;
    let left_pad = if cfg!(target_os = "macos") {
        px(72.)
    } else {
        space::sm()
    };
    let drag = div()
        .flex_1()
        .h_full()
        .flex()
        .items_center()
        .gap(space::sm())
        .pl(left_pad)
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
        .child(div().font_weight(FontWeight::SEMIBOLD).child(APP_NAME))
        .child(div().text_color(muted).child("—"))
        .child(div().text_color(muted).child(this.document_title()));
    div()
        .h(px(40.))
        .flex()
        .items_center()
        .child(drag)
        .when(!cfg!(target_os = "macos"), |bar| {
            bar.child(window_controls(window, cx))
        })
}
