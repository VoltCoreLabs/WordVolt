use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

use gpui_kit::*;
use wv_core::{Document, Editor};

use super::paragraph::paragraph_el;
use super::scroll::ScrollState;
use crate::theme::{Theme, space};

pub const PAGE_WIDTH: f32 = 620.;
pub const PAGE_HEIGHT: f32 = 877.;
pub const PAGE_MARGIN: f32 = 64.;

/// The editable sheet inside a smooth-scrolling viewport. Owns the document (`core`)
/// and the per-frame text layouts used for hit-testing.
pub struct EditorView {
    pub core: Editor,
    pub(super) focus: FocusHandle,
    pub(super) layouts: Rc<RefCell<Vec<TextLayout>>>,
    pub(super) selecting: bool,
    /// Time of the last edit/caret move; drives the caret fade/blink animation.
    pub(super) caret_epoch: Rc<Cell<Instant>>,
    pub(super) scroll: Rc<ScrollState>,
    /// Scroll to the caret on the next paint (set after keyboard/mouse edits).
    pub(super) reveal_caret: Rc<Cell<bool>>,
    /// Column (x) kept while moving the caret up/down.
    pub(super) goal_x: Option<Pixels>,
    focused_once: bool,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            core: Editor::new(Document::default()),
            focus: cx.focus_handle(),
            layouts: Rc::default(),
            selecting: false,
            caret_epoch: Rc::new(Cell::new(Instant::now())),
            scroll: Rc::default(),
            reveal_caret: Rc::default(),
            goal_x: None,
            focused_once: false,
        }
    }

    /// Replaces the document (open / new): fresh history, caret at the start, scrolled to the top.
    pub fn load(&mut self, doc: Document, cx: &mut Context<Self>) {
        self.core = Editor::new(doc);
        self.scroll.reset();
        self.goal_x = None;
        self.caret_epoch.set(Instant::now());
        cx.notify();
    }

    /// Runs an editing action (from keyboard or ribbon) and redraws.
    pub fn apply(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut Editor)) {
        f(&mut self.core);
        self.goal_x = None;
        self.caret_epoch.set(Instant::now());
        self.reveal_caret.set(true);
        cx.notify();
    }
}

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focused_once {
            self.focused_once = true;
            window.focus(&self.focus, cx);
        }
        let theme = cx.global::<Theme>();
        let (page, border, thumb) = (theme.page, theme.border, theme.border_strong);
        self.layouts.borrow_mut().clear();
        let focused = self.focus.is_focused(window);
        let paragraphs: Vec<_> = (0..self.core.doc.paragraphs.len())
            .map(|ix| paragraph_el(self, ix, focused, cx))
            .collect();
        let offset = self.scroll.tick(window);

        let sheet = div()
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
            .children(paragraphs);

        let (content_scroll, viewport_scroll) = (self.scroll.clone(), self.scroll.clone());
        let content_probe = canvas(
            move |b, _, _| content_scroll.set_content_height(b.size.height),
            |_, _, _, _| {},
        )
        .absolute()
        .size_full();
        let viewport_probe = canvas(
            move |b, _, _| {
                viewport_scroll.set_viewport(b);
                (viewport_scroll, b)
            },
            move |_, (scroll, b), window, _| scroll.paint_scrollbar(b, thumb, window),
        )
        .absolute()
        .size_full();

        div()
            .id("doc-scroll")
            .relative()
            .size_full()
            .overflow_hidden()
            .on_mouse_move(
                cx.listener(|this, ev: &MouseMoveEvent, _, cx| this.on_drag_move(ev, cx)),
            )
            .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, _, cx| {
                this.scroll
                    .scroll_by(-f32::from(ev.delta.pixel_delta(px(48.)).y));
                cx.notify();
            }))
            .child(
                div()
                    .relative()
                    .w_full()
                    .flex()
                    .justify_center()
                    .py(space::xl())
                    .mt(px(-offset))
                    .child(sheet)
                    .child(content_probe),
            )
            .child(viewport_probe)
    }
}
