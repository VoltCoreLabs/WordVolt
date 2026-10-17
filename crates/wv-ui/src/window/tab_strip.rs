use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_i18n::{Key, t};

use super::state::MainWindow;
use crate::theme::{Theme, radius, space};
use crate::ui_kit::Tip;

const TAB_HEIGHT: f32 = 28.0;

/// Open documents as tabs (like Figma's file tabs); the "+" opens another file in a new tab.
pub fn tab_strip(this: &MainWindow, cx: &mut Context<MainWindow>) -> Div {
    let th = cx.global::<Theme>();
    let (surface, border, hover, text, muted) = (
        th.surface,
        th.border,
        th.surface_alt,
        th.text,
        th.text_muted,
    );
    let lang = this.lang;
    let single = this.docs.len() == 1;
    let tabs: Vec<_> = (0..this.docs.len())
        .map(|i| {
            let (label, dirty) = this.tab_info(i);
            let active = i == this.active;
            div()
                .id(("doc-tab", i))
                .h(px(TAB_HEIGHT))
                .min_w(px(96.))
                .max_w(px(200.))
                .px(space::sm())
                .flex()
                .flex_none()
                .items_center()
                .gap(space::xs())
                .rounded(radius::md())
                .border_1()
                .border_color(if active { border } else { surface.opacity(0.0) })
                .bg(if active {
                    surface
                } else {
                    surface.opacity(0.0)
                })
                .text_sm()
                .text_color(if active { text } else { muted })
                .cursor_pointer()
                .when(!active, |d| d.hover(move |s| s.bg(hover)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| this.switch_to(i, cx)),
                )
                .child(
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(label),
                )
                .when(dirty, |d| d.child(div().text_color(muted).child("•")))
                .child(
                    div()
                        .id(("doc-close", i))
                        .size(px(18.))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(radius::sm())
                        .text_color(muted)
                        .hover(move |s| s.bg(hover).text_color(text))
                        .tip(t(lang, Key::TipCloseTab))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.close_tab(i, window, cx)
                            }),
                        )
                        .child("×"),
                )
        })
        .collect();
    let plus = div()
        .id("doc-plus")
        .size(px(TAB_HEIGHT))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(radius::md())
        .text_color(muted)
        .cursor_pointer()
        .hover(move |s| s.bg(hover).text_color(text))
        .tip(t(lang, Key::TipNewTab))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _, window, cx| this.open_document(window, cx)),
        )
        .child("+");
    div()
        .h_full()
        .min_w_0()
        .flex_1()
        .flex()
        .items_center()
        .gap(px(4.))
        .overflow_hidden()
        // A single document needs no strip: keep the title look (name only, no close button).
        .when(single, |d| d.pl(space::sm()))
        .children(tabs)
        .child(plus)
        .child(
            // Empty space drags the window.
            div()
                .h_full()
                .flex_1()
                .window_control_area(WindowControlArea::Drag)
                .when(!cfg!(target_os = "windows"), |d| {
                    d.on_mouse_down(MouseButton::Left, |ev, window, _| {
                        if ev.click_count == 2 {
                            window.zoom_window();
                        } else {
                            window.start_window_move();
                        }
                    })
                }),
        )
}
