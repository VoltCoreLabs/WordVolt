use std::time::Duration;

use gpui_kit::*;
use wv_i18n::{Key, t};

use super::state::MainWindow;
use crate::theme::{Theme, space};
use crate::ui_kit::block;

const WIDTH: f32 = 260.;

/// Right-hand block (styles). Slides in/out with an ease-out animation.
pub fn side_panel(this: &MainWindow, cx: &App) -> impl IntoElement {
    let open = this.panel_open;
    let muted = cx.global::<Theme>().text_muted;
    let content = block(cx)
        .w(px(WIDTH))
        .h_full()
        .flex_none()
        .p(space::lg())
        .flex()
        .flex_col()
        .gap(space::sm())
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child(t(this.lang, Key::Styles)),
        )
        .child(
            div()
                .text_sm()
                .text_color(muted)
                .child(t(this.lang, Key::StylesHint)),
        );
    div()
        .h_full()
        .flex_none()
        .overflow_hidden()
        .child(content)
        .with_animation(
            SharedString::from(format!("side-panel-{open}")),
            Animation::new(Duration::from_millis(260)).with_easing(ease_out_quint()),
            move |el, delta| {
                let k = if open { delta } else { 1.0 - delta };
                el.w(px(WIDTH * k))
            },
        )
}
