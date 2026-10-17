use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_i18n::{Key, t};

use super::state::MainWindow;
use crate::theme::{Theme, radius, space};
use crate::ui_kit::{Icon, Tip, Weight, icon};

const TAB_HEIGHT: f32 = 28.0;
const MAX_TAB: f32 = 190.0;
/// Narrowest tab: just the document icon.
const MIN_TAB: f32 = 34.0;
/// Below this the name is dropped (icon only, like in Chrome).
const TEXT_MIN: f32 = 72.0;
/// Below this only the active tab keeps its close button, and below the next one nobody does
/// (Ctrl+W and the middle mouse button still close).
const CLOSE_ALL_MIN: f32 = 120.0;
const CLOSE_ACTIVE_MIN: f32 = 96.0;
/// Width of the menus on the left of the bar and of the app name in the middle.
const MENUS_WIDTH: f32 = 236.0;
const TITLE_WIDTH: f32 = 110.0;

/// Open documents as tabs (like Figma's file tabs); the "+" opens another file in a new tab. The
/// more tabs there are, the narrower they get: first the name is shortened with "...", then it
/// goes and only the document icons remain.
pub fn tab_strip(this: &MainWindow, window: &Window, cx: &mut Context<MainWindow>) -> Div {
    let th = cx.global::<Theme>();
    let (surface, border, hover, text, muted, accent) = (
        th.surface,
        th.border,
        th.surface_alt,
        th.text,
        th.text_muted,
        th.accent,
    );
    let lang = this.lang;
    let count = this.docs.len();
    // The tabs share the left half of the bar.
    let half = (f32::from(window.bounds().size.width) - TITLE_WIDTH) / 2.0;
    let available = half - MENUS_WIDTH - TAB_HEIGHT - 8.0;
    let width = (available / count as f32).clamp(MIN_TAB, MAX_TAB);
    let tabs: Vec<_> = (0..count)
        .map(|i| {
            let (label, dirty) = this.tab_info(i);
            let active = i == this.active;
            let show_text = width >= TEXT_MIN;
            let show_close =
                show_text && (width >= CLOSE_ALL_MIN || (active && width >= CLOSE_ACTIVE_MIN));
            let tone = if active { text } else { muted };
            div()
                .id(("doc-tab", i))
                .h(px(TAB_HEIGHT))
                .w(px(width))
                .flex_none()
                .px(if show_text { space::sm() } else { px(0.) })
                .flex()
                .items_center()
                .justify_center()
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
                .text_color(tone)
                .cursor_pointer()
                .when(!active, |d| d.hover(move |s| s.bg(hover)))
                .tip(label.clone())
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| this.switch_to(i, cx)),
                )
                .on_mouse_down(
                    MouseButton::Middle,
                    cx.listener(move |this, _, window, cx| this.close_tab(i, window, cx)),
                )
                .child(
                    icon(
                        Icon::FileText,
                        if active {
                            Weight::Bold
                        } else {
                            Weight::Regular
                        },
                        px(14.),
                        // Without a name, an unsaved document shows it in the icon colour.
                        if dirty && !show_text { accent } else { tone },
                    )
                    .flex_none(),
                )
                .when(show_text, |d| {
                    d.child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(label.clone()),
                    )
                    .when(dirty, |d| d.child(div().text_color(muted).child("•")))
                })
                .when(show_close, |d| {
                    d.child(
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
        .gap(px(2.))
        .overflow_hidden()
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
