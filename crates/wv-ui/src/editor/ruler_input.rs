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
