use gpui_kit::*;
use wv_i18n::{Key, t};

use super::state::MainWindow;
use crate::theme::{Theme, radius, space};
use crate::ui_kit::{ButtonKind, block, button};

const REPO: &str = "https://github.com/VoltCoreLabs/WordVolt";

/// The "About" modal: a dimmed backdrop (click closes) with a card about the project.
pub fn about_dialog(this: &MainWindow, cx: &mut Context<MainWindow>) -> Option<AnyElement> {
    if !this.about_open {
        return None;
    }
    let lang = this.lang;
    let th = cx.global::<Theme>();
    let muted = th.text_muted;
    let line = |label: Key, value: String| {
        div()
            .flex()
            .gap(space::md())
            .child(
                div()
                    .w(px(110.))
                    .flex_none()
                    .text_color(muted)
                    .child(t(lang, label)),
            )
            .child(div().flex_1().child(value))
    };
    let card = block(cx)
        .id("about-card")
        .occlude()
        .w(px(440.))
        .p(space::lg())
        .flex()
        .flex_col()
        .gap(space::md())
        .rounded(radius::md())
        .shadow_lg()
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::SEMIBOLD)
                .child("WordVolt"),
        )
        .child(
            div()
                .text_sm()
                .text_color(muted)
                .child(t(lang, Key::AboutDesc)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(space::sm())
                .text_sm()
                .child(line(
                    Key::AboutVersion,
                    env!("CARGO_PKG_VERSION").to_string(),
                ))
                .child(line(
                    Key::AboutStack,
                    t(lang, Key::AboutStackValue).to_string(),
                ))
                .child(line(
                    Key::AboutFormats,
                    t(lang, Key::AboutFormatsValue).to_string(),
                ))
                .child(line(Key::AboutRepo, REPO.to_string())),
        )
        .child(
            div().flex().justify_end().child(
                button(
                    "about-close",
                    t(lang, Key::AboutClose),
                    ButtonKind::Primary,
                    cx,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.about_open = false;
                    cx.notify();
                })),
            ),
        );
    Some(
        div()
            .id("about-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(gpui::hsla(0., 0., 0., 0.35))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.about_open = false;
                    cx.notify();
                }),
            )
            .child(card)
            .into_any_element(),
    )
}
