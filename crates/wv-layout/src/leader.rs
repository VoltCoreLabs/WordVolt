use wv_core::CharStyle;

use crate::Measurer;

/// Gap kept between the leader dots and the text on either side.
const PAD: f32 = 4.0;

/// Dots filling a tab of `width`: the text and its x offset inside the tab (right-aligned, so the
/// dots end just before what follows). `None` when there is no room.
pub fn leader(width: f32, style: &CharStyle, m: &dyn Measurer) -> Option<(String, f32)> {
    let dot = m.width("·", style);
    let room = width - 2.0 * PAD;
    if dot <= 0.0 || room < dot {
        return None;
    }
    let count = (room / dot).floor() as usize;
    Some(("·".repeat(count), width - PAD - count as f32 * dot))
}
