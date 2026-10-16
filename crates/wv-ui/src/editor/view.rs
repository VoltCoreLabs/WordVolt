use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::*;
use wv_core::{Document, Editor};

use super::paragraph::paragraph_el;
use crate::theme::Theme;

pub const PAGE_WIDTH: f32 = 620.;
pub const PAGE_HEIGHT: f32 = 877.;
pub const PAGE_MARGIN: f32 = 64.;

/// The editable sheet. Owns the document (`core`) and the per-frame text layouts used for hit-testing.
pub struct EditorView {
    pub core: Editor,
    pub(super) focus: FocusHandle,
    pub(super) layouts: Rc<RefCell<Vec<TextLayout>>>,
    pub(super) selecting: bool,
    pub(super) caret_visible: bool,
    /// Column (x) kept while moving the caret up/down.
    pub(super) goal_x: Option<Pixels>,
    focused_once: bool,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Caret blink.
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(530))
                    .await;
                if this
                    .update(cx, |view, cx| {
                        view.caret_visible = !view.caret_visible;
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        Self {
            core: Editor::new(Document::default()),
            focus: cx.focus_handle(),
            layouts: Rc::default(),
            selecting: false,
            caret_visible: true,
            goal_x: None,
            focused_once: false,
        }
    }

    /// Runs an editing action (from keyboard or ribbon) and redraws.
    pub fn apply(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut Editor)) {
        f(&mut self.core);
        self.goal_x = None;
        self.caret_visible = true;
        cx.notify();
    }
}

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focused_once {
            self.focused_once = true;
            window.focus(&self.focus, cx);
        }
        let page = cx.global::<Theme>().page;
        let border = cx.global::<Theme>().border;
        self.layouts.borrow_mut().clear();
        let focused = self.focus.is_focused(window);
        let paragraphs: Vec<_> = (0..self.core.doc.paragraphs.len())
            .map(|ix| paragraph_el(self, ix, focused, cx))
            .collect();
        div()
            .id("sheet")
            .track_focus(&self.focus)
            .key_context("Editor")
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_sheet_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .cursor_text()
            .flex_none()
            .w(px(PAGE_WIDTH))
            .min_h(px(PAGE_HEIGHT))
            .p(px(PAGE_MARGIN))
            .bg(page)
            .border_1()
            .border_color(border)
            .text_size(px(16.))
            .line_height(px(24.))
            .flex()
            .flex_col()
            .children(paragraphs)
    }
}
