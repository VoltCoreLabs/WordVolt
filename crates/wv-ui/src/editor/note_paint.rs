use gpui_kit::*;
use wv_core::{CharStyle, Script};
use wv_layout::{Measurer, NOTES_GAP, script_shift};

use super::measure::GpuiMeasurer;
use super::page_paint::PageView;

/// Draws the number of a footnote mark (raised, small) where the mark's character is.
/// `place` is the left edge, the top of the line and the baseline of the line (layout px).
pub(super) fn paint_note_mark(
    v: &PageView,
    m: &GpuiMeasurer,
    mark: (&CharStyle, usize, Hsla),
    place: (f32, f32, f32),
    at: &dyn Fn(f32, f32) -> Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let (style, number, color) = mark;
    let raised = CharStyle {
        note: None,
        script: Script::Super,
        ..style.clone()
    };
    let (ascent, descent) = (m.ascent(&raised), m.descent(&raised));
    let (x, top, baseline) = place;
    let y = top + baseline - ascent + script_shift(&raised);
    let shaped = m.shape_zoomed(&number.to_string(), &raised, color, v.zoom);
    let _ = shaped.paint(
        at(x, y),
        px((ascent + descent) * v.zoom),
        TextAlign::Left,
        None,
        window,
        cx,
    );
}

/// The short line above the footnotes of the page.
pub(super) fn paint_note_separator(
    v: &PageView,
    at: &dyn Fn(f32, f32) -> Point<Pixels>,
    window: &mut Window,
) {
    let Some(top) = v.layout.pages[v.page].note_top else {
        return;
    };
    let setup = v.layout.setup_of(v.page);
    let line = Bounds::new(
        at(setup.margin_left, setup.margin_top + top + NOTES_GAP / 2.0),
        size(
            px(setup.content_width() / 3.0 * v.zoom),
            px(1.0_f32.max(v.zoom)),
        ),
    );
    window.paint_quad(fill(line, v.bar_color));
}
