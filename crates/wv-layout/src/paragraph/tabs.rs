use wv_core::{Paragraph, TabKind, TabLeader};

use super::Piece;
use crate::Measurer;

/// Default tab stops: every 0.5 inch (px).
const DEFAULT_TAB: f32 = 48.0;

/// Width of a tab that starts `x` px from the left edge of the text: up to the next custom stop
/// (a centre, right or decimal stop leaves room for `follow(kind)`, the width of the text after
/// the tab that goes before / on it), or, past the last one, up to the next default stop.
pub(super) fn tab_width(
    x: f32,
    stops: &[Stop],
    follow: &dyn Fn(TabKind) -> f32,
) -> (f32, TabLeader) {
    let aligned = stops.iter().find_map(|(s, kind, leader)| {
        let target = *s - follow(*kind);
        (*s > x + 0.5 && target > x + 0.5).then_some((target, *leader))
    });
    let (target, leader) = aligned.unwrap_or_else(|| {
        (
            ((x / DEFAULT_TAB).floor() + 1.0) * DEFAULT_TAB,
            TabLeader::None,
        )
    });
    (target - x, leader)
}

/// A custom tab stop: its place (px), how text lines up on it and its leader.
pub(super) type Stop = (f32, TabKind, TabLeader);

/// How much of the text after a tab goes before the stop: all of it for a right stop, half for a
/// centre stop, what comes before the decimal separator for a decimal one.
fn before_stop(kind: TabKind, rest: &str, style: &wv_core::CharStyle, m: &dyn Measurer) -> f32 {
    let segment = rest.split('\t').next().unwrap_or("");
    match kind {
        TabKind::Left => 0.0,
        TabKind::Right => m.width(segment, style),
        TabKind::Center => m.width(segment, style) / 2.0,
        TabKind::Decimal => {
            let end = segment.find(['.', ',']).unwrap_or(segment.len());
            m.width(&segment[..end], style)
        }
    }
}

/// Gives the tabs among the (whitespace) `spaces` the width they have when the first one starts
/// `x` px from the left edge of the text.
pub(super) fn resolve_tabs(
    p: &Paragraph,
    text: &str,
    spaces: &mut [Piece],
    mut x: f32,
    stops: &[Stop],
    m: &dyn Measurer,
) {
    for piece in spaces {
        let part = &text[piece.range.clone()];
        if part.contains('\t') {
            let style = &p.runs[piece.run].style;
            let mut width = 0.0;
            for (i, ch) in part.char_indices() {
                width += if ch == '\t' {
                    let rest = &text[piece.range.start + i + 1..];
                    let (w, leader) =
                        tab_width(x + width, stops, &|kind| before_stop(kind, rest, style, m));
                    piece.leader = leader;
                    w
                } else {
                    m.width(ch.encode_utf8(&mut [0; 4]), style)
                };
            }
            piece.width = width;
        }
        x += piece.width;
    }
}
