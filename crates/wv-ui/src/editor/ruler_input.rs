use gpui_kit::*;

use super::ruler::{self, Drag};
use super::view::EditorView;

impl EditorView {
    /// Where the ruler is now (`None` when it is hidden or no page is shown).
    fn ruler_geom(&self, window: &Window) -> Option<ruler::Geom> {
        if !self.show_ruler {
            return None;
        }
        let layout = self.layout.as_ref()?;
        let caret_page = layout
            .caret(self.core.sel.head, &self.measurer(window))
            .map_or(0, |c| c.page);
        ruler::geometry(
            layout,
            &self.page_bounds.borrow(),
            caret_page,
            self.zoom,
            self.scroll.viewport_top(),
        )
    }

    /// A press on the ruler grabs a marker (or makes a tab stop). Returns whether it was on it.
    pub(super) fn ruler_down(
        &mut self,
        at: Point<Pixels>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(g) = self.ruler_geom(window) else {
            return false;
        };
        let mut metrics = self.core.para_metrics();
        let pressed = ruler::press(
            &g,
            &mut metrics,
            (f32::from(at.x), f32::from(at.y)),
            self.new_tab_kind,
        );
        let Some((grab, made)) = pressed else {
            return false;
        };
        self.ruler_drag = Some(Drag {
            grab,
            metrics,
            removing: false,
            still: true,
            made,
        });
        cx.notify();
        true
    }

    /// Moves the grabbed marker; the paragraph changes when the button is let go.
    pub(super) fn ruler_move(
        &mut self,
        at: Point<Pixels>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.ruler_drag.is_none() {
            return false;
        }
        if let (Some(g), Some(d)) = (self.ruler_geom(window), self.ruler_drag.as_mut()) {
            ruler::drag(&g, d, f32::from(at.x), f32::from(at.y));
            cx.notify();
        }
        true
    }

    pub(super) fn ruler_up(&mut self, cx: &mut Context<Self>) {
        if let Some(d) = self.ruler_drag.take() {
            let (metrics, kind) = ruler::finish(&d);
            // Stops made next are of the kind that was last chosen.
            if let Some(kind) = kind {
                self.new_tab_kind = kind;
            }
            if metrics != self.core.para_metrics() {
                self.apply(cx, |core| core.set_para_metrics(&metrics));
            } else {
                cx.notify();
            }
        }
    }
}

impl EditorView {
    /// Where the vertical ruler is now (`None` when the ruler is hidden or no page is shown).
    fn vruler_geom(&self, window: &Window) -> Option<super::vruler::VGeom> {
        if !self.show_ruler {
            return None;
        }
        let layout = self.layout.as_ref()?;
        let caret_page = layout
            .caret(self.core.sel.head, &self.measurer(window))
            .map_or(0, |c| c.page);
        let top = self.scroll.viewport_top();
        super::vruler::geometry(
            layout,
            &self.page_bounds.borrow(),
            caret_page,
            self.zoom,
            (
                self.scroll.viewport_left(),
                top + ruler::HEIGHT,
                top + self.scroll.viewport_height(),
            ),
        )
    }

    /// A press on a margin edge of the vertical ruler grabs it. Returns whether it was on one.
    pub(super) fn vruler_down(
        &mut self,
        at: Point<Pixels>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(g) = self.vruler_geom(window) else {
            return false;
        };
        self.vruler_drag = super::vruler::press(&g, (f32::from(at.x), f32::from(at.y)));
        if self.vruler_drag.is_some() {
            cx.notify();
        }
        self.vruler_drag.is_some()
    }

    pub(super) fn vruler_move(
        &mut self,
        at: Point<Pixels>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        if let (Some(g), Some(d)) = (self.vruler_geom(window), self.vruler_drag.as_mut()) {
            super::vruler::drag(&g, d, f32::from(at.y));
            cx.notify();
        }
    }

    /// Letting go sets the margin of the section the caret is in.
    pub(super) fn vruler_up(&mut self, cx: &mut Context<Self>) {
        let Some(d) = self.vruler_drag.take() else {
            return;
        };
        let mut page = self.core.page_here();
        match d.grab {
            super::vruler::VGrab::Top => page.margin_top = d.margin,
            super::vruler::VGrab::Bottom => page.margin_bottom = d.margin,
        }
        if page != self.core.page_here() {
            self.apply(cx, |core| core.set_page(page));
        } else {
            cx.notify();
        }
    }
}
