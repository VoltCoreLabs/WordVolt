use super::state::MainWindow;
use crate::theme::{Theme, radius};
use crate::ui_kit::pop_in;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_i18n::{Key, t};

/// Shows `panel` at `at` over everything; a click anywhere else runs `close` (the page does not
/// get that click).
pub(super) fn popup(
    at: Point<Pixels>,
    panel: Stateful<Div>,
    key: &'static str,
    close: fn(&mut MainWindow),
    cx: &mut Context<MainWindow>,
) -> impl IntoElement {
    deferred(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .occlude()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    close(this);
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, _, _, cx| {
                    close(this);
                    cx.notify();
                }),
            )
            .child(
                anchored()
                    .position(at)
                    .anchor(Anchor::TopLeft)
                    .snap_to_window_with_margin(px(8.))
                    .child(pop_in(panel, key.to_string())),
            ),
    )
    .with_priority(20)
}

/// Light backgrounds offered for table cells (document content, not theme colours).
const SHADES: [u32; 10] = [
    0xFFF2CC, 0xFCE5CD, 0xF4CCCC, 0xD9EAD3, 0xCFE2F3, 0xD9D2E9, 0xEEEEEE, 0xFFE599, 0xB6D7A8,
    0x9FC5E8,
];

/// "Cell background": a row of swatches, the first one clears the background.
pub(super) fn shade_row(lang: wv_i18n::Lang, cx: &mut Context<MainWindow>) -> Div {
    let th = cx.global::<Theme>();
    let (border, muted, surface) = (th.border, th.text_muted, th.surface);
    let swatches = std::iter::once(None)
        .chain(SHADES.map(Some))
        .enumerate()
        .map(|(i, color)| {
            div()
                .id(("ctx-shade", i))
                .size(px(18.))
                .flex_none()
                .rounded(radius::sm())
                .border_1()
                .border_color(border)
                .bg(color.map_or(surface, rgb))
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .text_xs()
                .text_color(muted)
                .child(if color.is_none() { "×" } else { "" })
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.context_menu = None;
                        this.edit(cx, move |c| c.set_cell_shade(color));
                        this.editor.update(cx, |view, cx| view.request_focus(cx));
                    }),
                )
        });
    div()
        .px(px(10.))
        .py(px(6.))
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(div().text_sm().child(t(lang, Key::CtxShade)))
        .child(div().flex().gap(px(2.)).children(swatches))
}

/// A command row built at run time (a spelling correction): `act` runs when it is clicked.
pub(super) fn choice_row(
    id: (&'static str, usize),
    label: String,
    color: Option<Rgba>,
    cx: &mut Context<MainWindow>,
    act: impl Fn(&mut MainWindow, &mut Window, &mut Context<MainWindow>) + 'static,
) -> Stateful<Div> {
    let hover = cx.global::<Theme>().surface_alt;
    div()
        .id(id)
        .h(px(30.))
        .px(px(10.))
        .flex()
        .items_center()
        .rounded(radius::sm())
        .text_sm()
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .when_some(color, |d, c| {
            d.text_color(c).font_weight(FontWeight::SEMIBOLD)
        })
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, window, cx| {
                this.context_menu = None;
                act(this, window, cx);
                this.editor.update(cx, |view, cx| view.request_focus(cx));
                cx.notify();
            }),
        )
        .child(label)
}
