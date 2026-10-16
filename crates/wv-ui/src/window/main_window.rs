use gpui_kit::*;

use super::ribbon::{ribbon, tab_bar};
use super::side_panel::side_panel;
pub use super::state::MainWindow;
use super::status_bar::status_bar;
use super::titlebar::titlebar;
use crate::theme::{Theme, space, ui_font};
use crate::ui_kit::block;

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.window_title();
        if title != self.applied_title {
            window.set_window_title(&title);
            self.applied_title = title;
        }
        let t = cx.global::<Theme>();
        let (bg, page, border, text) = (t.bg, t.page, t.border, t.text);
        let sheet = div()
            .w(px(620.))
            .h_full()
            .bg(page)
            .border_1()
            .border_color(border)
            .p(px(48.))
            .child("WordVolt");
        div()
            .size_full()
            .bg(bg)
            .font(ui_font())
            .flex()
            .flex_col()
            .text_color(text)
            .child(titlebar(self, window, cx))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(space::sm())
                    .px(space::md())
                    .pb(space::md())
                    .overflow_hidden()
                    .child(tab_bar(self, cx))
                    .child(ribbon(self, cx))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .gap(space::md())
                            .overflow_hidden()
                            .child(
                                block(cx)
                                    .flex_1()
                                    .flex()
                                    .justify_center()
                                    .overflow_hidden()
                                    .p(space::xl())
                                    .child(sheet),
                            )
                            .child(side_panel(self, cx)),
                    )
                    .child(status_bar(cx)),
            )
    }
}
