use wv_core::Paragraph;

use super::Piece;
use crate::Measurer;

/// Default tab stops: every 0.5 inch (px).
const DEFAULT_TAB: f32 = 48.0;

/// Width of a tab that starts `x` px from the left edge of the text: up to the next custom stop,
/// or, past the last one, up to the next default stop.
pub(super) fn tab_width(x: f32, stops: &[f32]) -> f32 {
    let target = stops
        .iter()
        .copied()
        .find(|s| *s > x + 0.5)
        .unwrap_or_else(|| ((x / DEFAULT_TAB).floor() + 1.0) * DEFAULT_TAB);
    target - x
}

/// Gives the tabs among the (whitespace) `spaces` the width they have when the first one starts
/// `x` px from the left edge of the text.
pub(super) fn resolve_tabs(
    p: &Paragraph,
    text: &str,
    spaces: &mut [Piece],
    mut x: f32,
    stops: &[f32],
    m: &dyn Measurer,
) {
    for piece in spaces {
        let part = &text[piece.range.clone()];
        if part.contains('\t') {
            let style = &p.runs[piece.run].style;
            let mut width = 0.0;
            for ch in part.chars() {
                width += if ch == '\t' {
                    tab_width(x + width, stops)
                } else {
                    m.width(ch.encode_utf8(&mut [0; 4]), style)
                };
            }
            piece.width = width;
        }
        x += piece.width;
    }
}
