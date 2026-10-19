use gpui_kit::*;
use wv_core::Position;

use super::caret::{FADE_IN, caret_alpha};
use super::measure::GpuiMeasurer;
use super::page_paint::PageView;

/// The caret (blinking) when it is on this page.
pub(super) fn paint_caret(
    v: &PageView,
    m: &GpuiMeasurer,
    at: &dyn Fn(f32, f32) -> Point<Pixels>,
    sz: &dyn Fn(f32, f32) -> Size<Pixels>,
    window: &mut Window,
) {
    if v.focused
        && let Some(c) = v.layout.caret(v.head, m)
        && c.page == v.page
    {
        let secs = v.epoch.get().elapsed().as_secs_f32();
        let alpha = caret_alpha(secs);
        window.paint_quad(fill(
            Bounds::new(at(c.x, c.y), sz(1.5, c.height)),
            Rgba { a: alpha, ..v.text },
        ));
        if secs < FADE_IN {
            window.request_animation_frame();
        }
    }
}

/// Byte ranges of the misspelled words of a paragraph (none when spell checking is off).
pub(super) fn spell_marks(text: &str, cx: &App) -> std::sync::Arc<Vec<std::ops::Range<usize>>> {
    crate::spell::active(cx)
        .map(|speller| speller.misspelled(text))
        .unwrap_or_default()
}

/// A spoiler shows its text while the caret or the selection touches it.
pub(super) fn revealed(v: &PageView, para: usize, range: &std::ops::Range<usize>) -> bool {
    let (start, end) = v.selection;
    let (lo, hi) = (
        Position::new(para, range.start),
        Position::new(para, range.end),
    );
    (v.head >= lo && v.head <= hi) || (start != end && start <= hi && end >= lo)
}
