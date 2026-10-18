use gpui_kit::*;
use wv_core::CharStyle;
use wv_layout::{Line, Measurer};

use super::measure::GpuiMeasurer;
use super::page_view::PageView;

/// Non-printing characters of one line: a dot for every space, an arrow for every tab and a
/// pilcrow after the last line of a paragraph. `place` is the left edge of the line's text and its
/// top (layout px).
pub(super) fn paint_line_marks(
    v: &PageView,
    m: &GpuiMeasurer,
    (line, text): (&Line, &str),
    place: (f32, f32),
    at: &dyn Fn(f32, f32) -> Point<Pixels>,
    (window, cx): (&mut Window, &mut App),
) {
    let color: Hsla = v.bar_color.into();
    let (left, top) = place;
    let mut draw = |glyph: &str, style: &CharStyle, x: f32| {
        let plain = CharStyle {
            underline: false,
            strike: false,
            highlight: None,
            link: None,
            note: None,
            comment: None,
            spoiler: false,
            ..style.clone()
        };
        let (ascent, descent) = (m.ascent(&plain), m.descent(&plain));
        let y = top + line.baseline - ascent;
        let _ = m.shape_zoomed(glyph, &plain, color, v.zoom).paint(
            at(x, y),
            px((ascent + descent) * v.zoom),
            TextAlign::Left,
            None,
            window,
            cx,
        );
    };
    for f in &line.fragments {
        let piece = &text[f.range.clone()];
        for (i, ch) in piece.char_indices() {
            let glyph = match ch {
                ' ' => "·",
                '\t' => "→",
                _ => continue,
            };
            let before = m.width(&piece[..i], &f.style);
            draw(glyph, &f.style, left + f.x + before);
        }
    }
    if line.is_last
        && let Some(f) = line.fragments.last()
    {
        draw("¶", &f.style, left + f.x + f.width);
    }
}
