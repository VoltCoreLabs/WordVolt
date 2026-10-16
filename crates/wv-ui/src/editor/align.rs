use gpui_kit::*;
use wv_core::Align;

// GPUI's `TextLayout::position_for_index` / `index_for_position` ignore text alignment, while painting
// applies it per visual line. These helpers add the same per-line offset so caret and clicks match the text.

/// One visual line: (end byte index, x of its first glyph, x of its end), all before alignment.
fn visual_lines(layout: &TextLayout) -> Vec<(usize, Pixels, Pixels)> {
    let lines = layout.line_layouts();
    let Some(wrapped) = lines.first() else {
        return vec![];
    };
    let unwrapped = &wrapped.unwrapped_layout;
    let mut out = Vec::new();
    let mut start_x = px(0.);
    for b in wrapped.wrap_boundaries.iter() {
        let glyph = &unwrapped.runs[b.run_ix].glyphs[b.glyph_ix];
        out.push((glyph.index, start_x, glyph.position.x));
        start_x = glyph.position.x;
    }
    out.push((wrapped.len(), start_x, unwrapped.width));
    out
}

fn line_offset(layout: &TextLayout, line: &(usize, Pixels, Pixels), align: Align) -> Pixels {
    let free = layout.bounds().size.width - (line.2 - line.1);
    match align {
        Align::Center => free / 2.0,
        Align::Right => free,
        Align::Left | Align::Justify => px(0.),
    }
}

/// Window position of the caret at byte `index`.
pub fn caret_point(layout: &TextLayout, index: usize, align: Align) -> Option<Point<Pixels>> {
    let p = layout.position_for_index(index)?;
    let lines = visual_lines(layout);
    let line = lines.iter().find(|l| index <= l.0).or(lines.last())?;
    Some(point(p.x + line_offset(layout, line, align), p.y))
}

/// Byte index under a window position.
pub fn index_at(layout: &TextLayout, pos: Point<Pixels>, align: Align) -> usize {
    let lines = visual_lines(layout);
    let row = ((pos.y - layout.bounds().top()) / layout.line_height())
        .floor()
        .max(0.0) as usize;
    let shift = lines
        .get(row.min(lines.len().saturating_sub(1)))
        .map_or(px(0.), |l| line_offset(layout, l, align));
    layout
        .index_for_position(point(pos.x - shift, pos.y))
        .unwrap_or_else(|i| i)
}
