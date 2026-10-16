use gpui_kit::*;

use super::ribbon::ribbon;
use super::status_bar::status_bar;
use crate::theme::{Theme, space};
use crate::ui_kit::block;

/// Root view: window background with blocks on top.
pub struct MainWindow;

impl Render for MainWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        let (bg, page, border, text) = (t.bg, t.page, t.border, t.text);
        div()
            .size_full()
            .bg(bg)
            .p(space::md())
            .flex()
            .flex_col()
            .gap(space::md())
            .text_color(text)
            .child(ribbon(cx))
            .child(
                block(cx)
                    .flex_1()
                    .flex()
                    .justify_center()
                    .overflow_hidden()
                    .p(space::xl())
                    .child(
                        div()
                            .w(px(620.))
                            .h_full()
                            .bg(page)
                            .border_1()
                            .border_color(border)
                            .p(px(48.))
                            .child("WordVolt"),
                    ),
            )
            .child(status_bar(cx))
    }
}
