use wv_core::{CharStyle, TabLeader};

use crate::Measurer;

/// Gap kept between the leader dots and the text on either side.
const PAD: f32 = 4.0;

/// Dots filling a tab of `width`: the text and its x offset inside the tab (right-aligned, so the
/// dots end just before what follows). `None` when there is no room.
pub fn leader(width: f32, style: &CharStyle, m: &dyn Measurer) -> Option<(String, f32)> {
    fill(width, style, m, TabLeader::Dots)
}

/// The characters of `kind` filling a tab of `width` (see `leader`); `None` for no leader.
pub fn fill(
    width: f32,
    style: &CharStyle,
    m: &dyn Measurer,
    kind: TabLeader,
) -> Option<(String, f32)> {
    let mark = match kind {
        TabLeader::None => return None,
        TabLeader::Dots => "·",
        TabLeader::Hyphens => "-",
        TabLeader::Underscore => "_",
    };
    let unit = m.width(mark, style);
    // Underscores join into a line, so they run up to both neighbours.
    let pad = if kind == TabLeader::Underscore {
        0.0
    } else {
        PAD
    };
    let room = width - 2.0 * pad;
    if unit <= 0.0 || room < unit {
        return None;
    }
    let count = (room / unit).floor() as usize;
    Some((mark.repeat(count), width - pad - count as f32 * unit))
}
