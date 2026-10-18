use std::time::Duration;

use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_core::NamedStyle;
use wv_i18n::{Key, t};

use super::state::MainWindow;
use super::style_names::style_label;
use crate::theme::{Theme, radius, space};
use crate::ui_kit::block;

const WIDTH: f32 = 220.;

/// Left-hand styles pane: every built-in style written in its own look; a click applies it to
/// the paragraphs of the selection. Slides in/out like the navigation pane.
pub fn styles_panel(this: &MainWindow, cx: &mut Context<MainWindow>) -> impl IntoElement {
    let open = this.styles_open;
    let lang = this.lang;
    let current = this.editor.read(cx).core.named_style();
    let th = cx.global::<Theme>();
    let (muted, hover, subtle, accent, text) = (
        th.text_muted,
        th.surface_alt,
        th.accent_subtle,
        th.accent,
        th.text,
    );
    let rows: Vec<_> = NamedStyle::ALL
        .into_iter()
        .map(|style| {
            let size = style
                .size_pt()
                .map_or(14.0, |pt| f32::from(pt).clamp(13.0, 22.0));
            let selected = style == current;
            div()
                .id(("style-row", style as usize))
                .w_full()
                .px(space::sm())
                .py(px(6.))
                .rounded(radius::sm())
                .cursor_pointer()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_size(px(size))
                .text_color(if selected {
                    accent
                } else {
                    style.color().map_or(text, rgb)
                })
                .when(style.bold(), |d| d.font_weight(FontWeight::BOLD))
                .when(selected, |d| d.bg(subtle))
                .hover(move |s| s.bg(hover))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.edit(cx, move |c| c.set_named_style(style));
                    }),
                )
                .child(style_label(lang, style))
        })
        .collect();
    let content = block(cx)
        .w(px(WIDTH))
        .h_full()
        .flex_none()
        .p(space::md())
        .flex()
        .flex_col()
        .gap(space::sm())
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child(t(lang, Key::StylesTitle)),
        )
        .child(
            div()
                .text_xs()
                .text_color(muted)
                .child(t(lang, Key::StylesPaneHint)),
        )
        .child(
            div()
                .id("styles-list")
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .children(rows),
        );
    div()
        .h_full()
        .flex_none()
        .overflow_hidden()
        .child(content)
        .with_animation(
            SharedString::from(format!("styles-panel-{open}")),
            Animation::new(Duration::from_millis(260)).with_easing(ease_out_quint()),
            move |el, delta| {
                let k = if open { delta } else { 1.0 - delta };
                el.w(px(WIDTH * k))
            },
        )
}
