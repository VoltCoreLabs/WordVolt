use gpui_kit::*;

use super::context_menu::context_menu;
use super::ribbon::{ribbon, tab_bar};
use super::side_panel::side_panel;
pub use super::state::MainWindow;
use super::status_bar::status_bar;
use super::styles_panel::styles_panel;
use super::tab_menu::tab_menu;
use super::titlebar::titlebar;
use crate::actions::{
    CloseTab, FindInDocument, FindNext, FindPrevious, InsertLink, NewDocument, NewTab, NextTab,
    OpenDocument, OpenPalette, PrevTab, PrintDocument, ReplaceInDocument, SaveDocument,
    SaveDocumentAs,
};
use crate::theme::{Theme, space, ui_font};
use crate::ui_kit::{block, tooltip_layer};

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.window_title();
        if title != self.applied_title {
            window.set_window_title(&title);
            self.applied_title = title;
        }
        // Counting the words of a huge document takes a while: only when the text changed.
        let (id, revision) = {
            let view = self.editor.read(cx);
            (self.editor.entity_id(), view.core.revision())
        };
        if self.words.0 != Some(id) || self.words.1 != revision {
            let count = self.editor.read(cx).core.doc.word_count();
            self.words = (Some(id), revision, count);
        }
        let words = self.words.2;
        let pages = self.editor.update(cx, |view, _| view.page_info(window));
        let t = cx.global::<Theme>();
        let (bg, text) = (t.bg, t.text);
        let workspace = div()
            .flex_1()
            .flex()
            .gap(space::md())
            .overflow_hidden()
            .child(styles_panel(self, cx))
            .child(
                block(cx)
                    .flex_1()
                    .overflow_hidden()
                    .child(self.editor.clone()),
            )
            .child(side_panel(self, window, cx));
        div()
            .size_full()
            .on_action(
                cx.listener(|this, _: &NewDocument, window, cx| this.new_document(window, cx)),
            )
            .on_action(cx.listener(|this, _: &OpenPalette, window, cx| {
                this.palette.update(cx, |p, cx| p.open(window, cx));
            }))
            .on_action(cx.listener(|this, _: &InsertLink, window, cx| {
                this.link_editor
                    .update(cx, |editor, cx| editor.open(window, cx));
            }))
            .on_action(cx.listener(|this, _: &FindInDocument, window, cx| {
                this.find_bar
                    .update(cx, |bar, cx| bar.open(false, window, cx));
            }))
            .on_action(cx.listener(|this, _: &ReplaceInDocument, window, cx| {
                this.find_bar
                    .update(cx, |bar, cx| bar.open(true, window, cx));
            }))
            .on_action(cx.listener(|this, _: &FindNext, window, cx| {
                this.find_bar.update(cx, |bar, cx| {
                    if bar.is_open() {
                        bar.step(1, cx)
                    } else {
                        bar.open(false, window, cx)
                    }
                });
            }))
            .on_action(cx.listener(|this, _: &FindPrevious, window, cx| {
                this.find_bar.update(cx, |bar, cx| {
                    if bar.is_open() {
                        bar.step(-1, cx)
                    } else {
                        bar.open(false, window, cx)
                    }
                });
            }))
            .on_action(cx.listener(|this, _: &NewTab, _, cx| this.new_tab(cx)))
            .on_action(
                cx.listener(|this, _: &PrintDocument, window, cx| this.print_document(window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &CloseTab, window, cx| {
                    this.close_tab(this.active, window, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &NextTab, _, cx| {
                this.switch_to((this.active + 1) % this.docs.len(), cx)
            }))
            .on_action(cx.listener(|this, _: &PrevTab, _, cx| {
                this.switch_to((this.active + this.docs.len() - 1) % this.docs.len(), cx)
            }))
            .on_drop(cx.listener(|this, files: &ExternalPaths, window, cx| {
                this.drop_files(files.paths(), window, cx)
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
                    .child(self.find_bar.clone())
                    .child(workspace)
                    .child(status_bar(
                        self.lang,
                        pages,
                        words,
                        self.editor.read(cx).zoom,
                        self.editor.read(cx).multi_page,
                        crate::spell::enabled(cx),
                        cx,
                    )),
            )
            .children(self.context_menu.map(|at| context_menu(self, at, cx)))
            .children(self.tab_menu.map(|at| tab_menu(self, at, cx)))
            .child(self.image_dialog.clone())
            .children(super::about::about_dialog(self, cx))
            .child(tooltip_layer(cx))
    }
}
