use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

use gpui_kit::*;
use wv_core::{Document, Editor, Position};
use wv_layout::{DocLayout, PageSetup};

use super::measure::{GpuiMeasurer, MeasureCache};
use super::scroll::ScrollState;

/// Space between pages.
pub(super) const PAGE_GAP: f32 = 24.0;

/// The paged editing surface inside a smooth-scrolling viewport. Owns the document (`core`);
/// the page layout is recomputed whenever the document changes.
/// Stored picture bytes (cache keys) and the GPUI images built from them.
pub(super) type PictureCache = Rc<super::image_paint::PictureSet>;

pub const MIN_ZOOM: f32 = 0.10;
pub const MAX_ZOOM: f32 = 4.0;
/// Space reserved at each side of the page when fitting it to the width.
pub(super) const FIT_PADDING: f32 = 24.0;

pub struct EditorView {
    pub core: Editor,
    pub(super) focus: FocusHandle,
    pub(super) selecting: bool,
    /// Set by a right click: the window asks for its context menu at this point.
    pub context_request: Option<Point<Pixels>>,
    pub(super) resizing: Option<super::image_drag::ImageDrag>,
    pub(super) border_drag: Option<super::table_resize::BorderDrag>,
    /// Time of the last edit/caret move; drives the caret fade/blink animation.
    pub(super) caret_epoch: Rc<Cell<Instant>>,
    pub(super) scroll: Rc<ScrollState>,
    /// Scroll to the caret on the next paint (set after keyboard/mouse edits).
    pub(super) reveal_caret: Rc<Cell<bool>>,
    /// Caret column (page x) kept while moving up/down.
    pub(super) goal_x: Option<f32>,
    pub(super) layout: Option<Rc<DocLayout>>,
    /// Laid-out paragraphs reused between layouts.
    pub(super) layout_cache: wv_layout::LayoutCache,
    /// Search matches to highlight and the index of the current one.
    pub(super) find: Rc<Vec<(Position, Position)>>,
    pub(super) find_current: Option<usize>,
    pub(super) layout_key: (u64, u64),
    pub(super) doc_epoch: u64,
    pub(super) measure_cache: Rc<RefCell<MeasureCache>>,
    /// Format painter: the formatting picked up, applied to the next selection.
    pub painter: Option<wv_core::CharStyle>,
    /// The handles of the selected picture cut its sides instead of scaling it.
    pub cropping: bool,
    /// Window bounds of every page, written while painting (used for mouse hit-testing).
    pub(super) page_bounds: Rc<RefCell<Vec<Bounds<Pixels>>>>,
    pub(super) focused_once: bool,
    /// Display scale (1.0 = 100%).
    pub zoom: f32,
    /// Pages sit side by side while they fit (else always one per row).
    pub multi_page: bool,
    /// Non-printing characters are drawn.
    pub show_marks: bool,
    /// The ruler strip is shown above the page.
    pub show_ruler: bool,
    /// A ruler marker is being dragged.
    pub(super) ruler_drag: Option<super::ruler::Drag>,
    /// The kind of tab stop a click on the ruler makes.
    pub(super) new_tab_kind: wv_core::TabKind,
    /// Horizontal scroll (px) when the zoomed page is wider than the viewport.
    pub(super) scroll_x: f32,
    /// Decoded-on-demand pictures of the document (rebuilt when the stored images change).
    pub(super) picture_cache: PictureCache,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let caret_epoch = Rc::new(Cell::new(Instant::now()));
        let blink = caret_epoch.clone();
        cx.spawn(async move |this, cx| {
            loop {
                let wait = super::caret::next_change(blink.get().elapsed().as_secs_f32());
                cx.background_executor()
                    .timer(std::time::Duration::from_secs_f32(wait))
                    .await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        })
        .detach();
        Self {
            core: Editor::new(Document::default()),
            focus: cx.focus_handle(),
            selecting: false,
            context_request: None,
            resizing: None,
            border_drag: None,
            caret_epoch,
            scroll: Rc::default(),
            reveal_caret: Rc::default(),
            goal_x: None,
            layout: None,
            layout_cache: wv_layout::LayoutCache::default(),
            find: Rc::default(),
            find_current: None,
            layout_key: (0, 0),
            doc_epoch: 0,
            measure_cache: Rc::default(),
            page_bounds: Rc::default(),
            painter: None,
            cropping: false,
            focused_once: false,
            zoom: 1.0,
            multi_page: crate::settings::load_multi_page(),
            show_marks: false,
            show_ruler: true,
            ruler_drag: None,
            new_tab_kind: wv_core::TabKind::Left,
            scroll_x: 0.0,
            picture_cache: Default::default(),
        }
    }

    /// Sets the display scale (clamped to 10%..400%), keeping the scrolled-to part in view.
    pub fn set_zoom(&mut self, zoom: f32, cx: &mut Context<Self>) {
        let zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        if (zoom - self.zoom).abs() > f32::EPSILON {
            self.scroll.rescale(zoom / self.zoom);
            self.zoom = zoom;
            self.scroll_x = 0.0;
            cx.notify();
        }
    }

    pub fn toggle_ruler(&mut self, cx: &mut Context<Self>) {
        self.show_ruler = !self.show_ruler;
        self.ruler_drag = None;
        cx.notify();
    }

    pub fn toggle_marks(&mut self, cx: &mut Context<Self>) {
        self.show_marks = !self.show_marks;
        cx.notify();
    }

    /// Switches between pages side by side and one page per row (remembered).
    pub fn toggle_multi_page(&mut self, cx: &mut Context<Self>) {
        self.multi_page = !self.multi_page;
        crate::settings::save_multi_page(self.multi_page);
        cx.notify();
    }

    /// Width of the widest page of any section.
    pub(super) fn widest_page(&self) -> f32 {
        let doc = &self.core.doc;
        doc.sections
            .values()
            .fold(doc.page.width, |w, s| w.max(s.width))
    }

    /// Zoom at which a page exactly fills the viewport width.
    pub fn fit_width_zoom(&self) -> f32 {
        let page = self.widest_page() + 2.0 * FIT_PADDING;
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
                let measurer = self.measurer(window);
                let layout = Rc::new(DocLayout::with_cache(
                    &self.core.doc,
                    PageSetup::from_spec(&self.core.doc.page),
                    &measurer,
                    &mut self.layout_cache,
                ));
                self.layout = Some(layout.clone());
                self.layout_key = key;
                layout
            }
        }
    }
}
