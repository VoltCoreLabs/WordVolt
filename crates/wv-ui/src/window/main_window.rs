use gpui_kit::*;

use super::ribbon::{ribbon, tab_bar};
use super::side_panel::side_panel;
pub use super::state::MainWindow;
use super::status_bar::status_bar;
use super::titlebar::titlebar;
use crate::actions::{InsertLink, NewDocument, OpenDocument, SaveDocument, SaveDocumentAs};
use crate::theme::{Theme, space, ui_font};
use crate::ui_kit::block;

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.window_title();
        if title != self.applied_title {
            window.set_window_title(&title);
            self.applied_title = title;
        }
        let words = self.editor.read(cx).core.doc.word_count();
        let pages = self.editor.update(cx, |view, _| view.page_info(window));
        let t = cx.global::<Theme>();
        let (bg, text) = (t.bg, t.text);
        let workspace = div()
            .flex_1()
            .flex()
            .gap(space::md())
            .overflow_hidden()
            .child(
                block(cx)
                    .flex_1()
                    .overflow_hidden()
                    .child(self.editor.clone()),
            )
            .child(side_panel(self, cx));
        div()
            .size_full()
            .on_action(
                cx.listener(|this, _: &NewDocument, window, cx| this.new_document(window, cx)),
            )
            .on_action(cx.listener(|this, _: &InsertLink, window, cx| {
                this.link_editor
                    .update(cx, |editor, cx| editor.open(window, cx));
            }))
            .on_action(
                cx.listener(|this, _: &OpenDocument, window, cx| this.open_document(window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &SaveDocument, window, cx| this.save_document(window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &SaveDocumentAs, window, cx| {
                    this.save_document_as(window, cx)
                }),
            )
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
                    .child(workspace)
                    .child(status_bar(self.lang, pages, words, cx)),
            )
    }
}
