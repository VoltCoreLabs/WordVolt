use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use gpui_kit::*;
use wv_core::{Document, Editor, Position};
use wv_layout::{DocLayout, PageSetup};

use super::measure::{GpuiMeasurer, MeasureCache};
use super::page_paint::{PageView, paint_page};
use super::scroll::ScrollState;
use crate::theme::{Theme, space};

/// Space between pages.
const PAGE_GAP: f32 = 24.0;

/// The paged editing surface inside a smooth-scrolling viewport. Owns the document (`core`);
/// the page layout is recomputed whenever the document changes.
/// Stored picture bytes (cache keys) and the GPUI images built from them.
pub(super) type PictureCache = (Vec<Arc<Vec<u8>>>, Rc<Vec<Arc<Image>>>);

pub const MIN_ZOOM: f32 = 0.25;
pub const MAX_ZOOM: f32 = 4.0;
/// Space reserved at each side of the page when fitting it to the width.
const FIT_PADDING: f32 = 24.0;

pub struct EditorView {
    pub core: Editor,
    pub(super) focus: FocusHandle,
    pub(super) selecting: bool,
    /// Set by a right click: the window asks for its context menu at this point.
    pub context_request: Option<Point<Pixels>>,
    pub(super) resizing: Option<super::resize::Resize>,
    /// Time of the last edit/caret move; drives the caret fade/blink animation.
    pub(super) caret_epoch: Rc<Cell<Instant>>,
    pub(super) scroll: Rc<ScrollState>,
    /// Scroll to the caret on the next paint (set after keyboard/mouse edits).
    pub(super) reveal_caret: Rc<Cell<bool>>,
    /// Caret column (page x) kept while moving up/down.
    pub(super) goal_x: Option<f32>,
    pub(super) layout: Option<Rc<DocLayout>>,
    /// Search matches to highlight and the index of the current one.
    find: Rc<Vec<(Position, Position)>>,
    find_current: Option<usize>,
    layout_key: (u64, u64),
    doc_epoch: u64,
    measure_cache: Rc<RefCell<MeasureCache>>,
    /// Window bounds of every page, written while painting (used for mouse hit-testing).
    pub(super) page_bounds: Rc<RefCell<Vec<Bounds<Pixels>>>>,
    focused_once: bool,
    /// Display scale (1.0 = 100%).
    pub zoom: f32,
    /// Horizontal scroll (px) when the zoomed page is wider than the viewport.
    pub(super) scroll_x: f32,
    /// Decoded-on-demand pictures of the document (rebuilt when the stored images change).
    pub(super) picture_cache: PictureCache,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            core: Editor::new(Document::default()),
            focus: cx.focus_handle(),
            selecting: false,
            context_request: None,
            resizing: None,
            caret_epoch: Rc::new(Cell::new(Instant::now())),
            scroll: Rc::default(),
            reveal_caret: Rc::default(),
            goal_x: None,
            layout: None,
            find: Rc::default(),
            find_current: None,
            layout_key: (0, 0),
            doc_epoch: 0,
            measure_cache: Rc::default(),
            page_bounds: Rc::default(),
            focused_once: false,
            zoom: 1.0,
            scroll_x: 0.0,
            picture_cache: Default::default(),
        }
    }

    /// Sets the display scale (clamped to 25%..400%), keeping the scrolled-to part in view.
    pub fn set_zoom(&mut self, zoom: f32, cx: &mut Context<Self>) {
        let zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        if (zoom - self.zoom).abs() > f32::EPSILON {
            self.scroll.rescale(zoom / self.zoom);
            self.zoom = zoom;
            self.scroll_x = 0.0;
            cx.notify();
        }
    }

    /// Zoom at which a page exactly fills the viewport width.
    pub fn fit_width_zoom(&self) -> f32 {
        let page = self.core.doc.page.width + 2.0 * FIT_PADDING;
        (self.scroll.viewport_width() / page).clamp(MIN_ZOOM, MAX_ZOOM)
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

    /// Sets the search highlights (all matches, and the current one).
    pub fn set_find(
        &mut self,
        matches: Vec<(Position, Position)>,
        current: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        self.find = Rc::new(matches);
        self.find_current = current;
        cx.notify();
    }

    /// Gives the keyboard focus back to the document (e.g. after picking a font).
    pub fn request_focus(&mut self, cx: &mut Context<Self>) {
        self.focused_once = false;
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

    pub(super) fn ensure_layout(&mut self, window: &Window) -> Rc<DocLayout> {
        let key = (self.core.revision(), self.doc_epoch);
        match &self.layout {
            Some(layout) if key == self.layout_key => layout.clone(),
            _ => {
                let layout = Rc::new(DocLayout::new(
                    &self.core.doc,
                    PageSetup::from_spec(&self.core.doc.page),
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

        let images = self.picture_sources();
        let doc = &self.core.doc;
        let bands = Rc::new((doc.header.clone(), doc.footer.clone(), doc.variants.clone()));
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
                    code_bg: theme.code_bg,
                    spoiler_color: theme.spoiler,
                    link_color: theme.accent,
                    bar_color: theme.border_strong,
                    find: self.find.clone(),
                    find_current: self.find_current,
                    find_color: theme.find_match,
                    find_current_color: theme.find_current,
                    table_color: theme.border_strong,
                    bands: bands.clone(),
                    images: images.clone(),
                    band_color: theme.text_muted,
                    zoom: self.zoom,
                };
                let bounds_store = self.page_bounds.clone();
                div()
                    .relative()
                    .flex_none()
                    .w(px(layout.setup.width * self.zoom))
                    .h(px(layout.setup.height * self.zoom))
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
            .on_action(cx.listener(|this, _: &crate::actions::EditorTab, _, cx| {
                this.apply(cx, |c| c.tab_key(false))
            }))
            .on_action(
                cx.listener(|this, _: &crate::actions::EditorTabBack, _, cx| {
                    this.apply(cx, |c| c.tab_key(true))
                }),
            )
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_down(MouseButton::Right, cx.listener(Self::on_right_down))
            .on_mouse_move(cx.listener(Self::on_drag_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, _, cx| {
                let delta = ev.delta.pixel_delta(px(48.));
                let (dx, dy) = (f32::from(delta.x), f32::from(delta.y));
                if ev.modifiers.secondary() {
                    // Ctrl/Cmd + wheel zooms.
                    let step = if dy > 0.0 { 1.1 } else { 1.0 / 1.1 };
                    this.set_zoom(this.zoom * step, cx);
                } else if ev.modifiers.shift || dx.abs() > dy.abs() {
                    let content = this.core.doc.page.width * this.zoom + 2.0 * FIT_PADDING;
                    let max = (content - this.scroll.viewport_width()).max(0.0);
                    let step = if dx != 0.0 { dx } else { dy };
                    this.scroll_x = (this.scroll_x - step).clamp(0.0, max);
                    cx.notify();
                } else {
                    this.scroll.scroll_by(-dy);
                    cx.notify();
                }
            }))
            .cursor_text()
            .relative()
            .size_full()
            .overflow_hidden()
            .child(
                div()
                    .relative()
                    .w_full()
                    .min_w(px(layout.setup.width * self.zoom + 2.0 * FIT_PADDING))
                    .ml(px(-self.scroll_x))
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
