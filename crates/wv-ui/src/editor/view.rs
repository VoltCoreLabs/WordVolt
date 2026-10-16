use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

use gpui_kit::*;
use wv_core::{Document, Editor};
use wv_layout::{DocLayout, PageSetup};

use super::measure::{GpuiMeasurer, MeasureCache};
use super::page_paint::{PageView, paint_page};
use super::scroll::ScrollState;
use crate::theme::{Theme, space};

/// Space between pages.
const PAGE_GAP: f32 = 24.0;

/// The paged editing surface inside a smooth-scrolling viewport. Owns the document (`core`);
/// the page layout is recomputed whenever the document changes.
pub struct EditorView {
    pub core: Editor,
    pub(super) focus: FocusHandle,
    pub(super) selecting: bool,
    /// Time of the last edit/caret move; drives the caret fade/blink animation.
    pub(super) caret_epoch: Rc<Cell<Instant>>,
    pub(super) scroll: Rc<ScrollState>,
    /// Scroll to the caret on the next paint (set after keyboard/mouse edits).
    pub(super) reveal_caret: Rc<Cell<bool>>,
    /// Caret column (page x) kept while moving up/down.
    pub(super) goal_x: Option<f32>,
    pub(super) layout: Option<Rc<DocLayout>>,
    layout_key: (u64, u64),
    doc_epoch: u64,
    measure_cache: Rc<RefCell<MeasureCache>>,
    /// Window bounds of every page, written while painting (used for mouse hit-testing).
    pub(super) page_bounds: Rc<RefCell<Vec<Bounds<Pixels>>>>,
    focused_once: bool,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            core: Editor::new(Document::default()),
            focus: cx.focus_handle(),
            selecting: false,
            caret_epoch: Rc::new(Cell::new(Instant::now())),
            scroll: Rc::default(),
            reveal_caret: Rc::default(),
            goal_x: None,
            layout: None,
            layout_key: (0, 0),
            doc_epoch: 0,
            measure_cache: Rc::default(),
            page_bounds: Rc::default(),
            focused_once: false,
        }
    }

    /// Replaces the document (open / new): fresh history, caret at the start, scrolled to the top.
    pub fn load(&mut self, doc: Document, cx: &mut Context<Self>) {
        self.core = Editor::new(doc);
        self.doc_epoch += 1;
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

    /// (page with the caret, total pages), both 1-based; recomputes the layout if the document changed.
    pub fn page_info(&mut self, window: &Window) -> (usize, usize) {
        let layout = self.ensure_layout(window);
        let current = layout
            .caret(self.core.sel.head, &self.measurer(window))
            .map_or(1, |c| c.page + 1);
        (current, layout.pages.len())
    }

    pub(super) fn measurer(&self, window: &Window) -> GpuiMeasurer {
        GpuiMeasurer::new(window, self.measure_cache.clone())
    }

    fn ensure_layout(&mut self, window: &Window) -> Rc<DocLayout> {
        let key = (self.core.revision(), self.doc_epoch);
        match &self.layout {
            Some(layout) if key == self.layout_key => layout.clone(),
            _ => {
                let layout = Rc::new(DocLayout::new(
                    &self.core.doc,
                    PageSetup::a4(),
                    &self.measurer(window),
                ));
                self.layout = Some(layout.clone());
                self.layout_key = key;
                layout
            }
        }
    }
}

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focused_once {
            self.focused_once = true;
            window.focus(&self.focus, cx);
        }
        let layout = self.ensure_layout(window);
        let theme = cx.global::<Theme>();
        let (page_bg, border, thumb) = (theme.page, theme.border, theme.border_strong);
        let focused = self.focus.is_focused(window);
        self.page_bounds
            .borrow_mut()
            .resize(layout.pages.len(), Bounds::default());

        let pages: Vec<_> = (0..layout.pages.len())
            .map(|ix| {
                let view = PageView {
                    layout: layout.clone(),
                    measure: self.measure_cache.clone(),
                    page: ix,
                    selection: self.core.sel.range(),
                    head: self.core.sel.head,
                    focused,
                    text: theme.text,
                    selection_color: theme.accent_subtle,
                    epoch: self.caret_epoch.clone(),
                    scroll: self.scroll.clone(),
                    reveal: self.reveal_caret.clone(),
                };
                let bounds_store = self.page_bounds.clone();
                div()
                    .relative()
                    .flex_none()
                    .w(px(layout.setup.width))
                    .h(px(layout.setup.height))
                    .bg(page_bg)
                    .border_1()
                    .border_color(border)
                    .child(
                        canvas(
                            move |bounds, _, _| {
                                if let Some(slot) = bounds_store.borrow_mut().get_mut(ix) {
                                    *slot = bounds;
                                }
                            },
                            move |bounds, (), window, cx| paint_page(&view, bounds, window, cx),
                        )
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full(),
                    )
            })
            .collect();

        let offset = self.scroll.tick(window);
        let (content_scroll, viewport_scroll) = (self.scroll.clone(), self.scroll.clone());
        let content_probe = canvas(
            move |b, _, _| content_scroll.set_content_height(b.size.height),
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        let viewport_probe = canvas(
            move |b, _, _| {
                viewport_scroll.set_viewport(b);
                (viewport_scroll, b)
            },
            move |_, (scroll, b), window, _| scroll.paint_scrollbar(b, thumb, window),
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();

        div()
            .id("doc-scroll")
            .track_focus(&self.focus)
            .key_context("Editor")
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_drag_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, _, cx| {
                this.scroll
                    .scroll_by(-f32::from(ev.delta.pixel_delta(px(48.)).y));
                cx.notify();
            }))
            .cursor_text()
            .relative()
            .size_full()
            .overflow_hidden()
            .child(
                div()
                    .relative()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(PAGE_GAP))
                    .py(space::xl())
                    .mt(px(-offset))
                    .children(pages)
                    .child(content_probe),
            )
            .child(viewport_probe)
    }
}
