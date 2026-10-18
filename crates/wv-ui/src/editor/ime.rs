use std::ops::Range;

use gpui_kit::*;

use super::measure::GpuiMeasurer;
use super::view::EditorView;

/// Typed and composed text arrives here (the platform text input: layouts, dead keys, IME of
/// Chinese/Japanese/Korean, emoji picker). Offsets are UTF-16 inside the paragraph of the caret;
/// the logic itself lives in `wv_core::Editor` (`ime_*`).
impl EntityInputHandler for EditorView {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let (text, clamped) = self.core.ime_text(range);
        *adjusted = Some(clamped);
        Some(text)
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let (range, reversed) = self.core.ime_selection();
        Some(UTF16Selection { range, reversed })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.core.ime_marked()
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.core.ime_unmark();
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply(cx, |c| c.ime_replace(range, text, false));
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        _: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply(cx, |c| c.ime_replace(range, new_text, true));
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        window: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let at = self.core.ime_position(range.start);
        let layout = self.layout.clone()?;
        let m = GpuiMeasurer::new(window, self.measure_cache.clone());
        let caret = layout.caret(at, &m)?;
        let page = self.page_bounds.borrow().get(caret.page).copied()?;
        Some(Bounds::new(
            point(
                page.origin.x + px(caret.x * self.zoom),
                page.origin.y + px(caret.y * self.zoom),
            ),
            size(px(1.), px(caret.height * self.zoom)),
        ))
    }

    fn character_index_for_point(
        &mut self,
        _: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }
}
