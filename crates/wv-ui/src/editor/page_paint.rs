use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

use gpui_kit::*;
use wv_core::Position;
use wv_layout::{DocLayout, Measurer};

use super::caret::caret_alpha;
use super::measure::{GpuiMeasurer, MeasureCache};
use super::scroll::ScrollState;

/// Everything needed to paint one page; cheap to clone into the canvas closure.
pub struct PageView {
    pub layout: Rc<DocLayout>,
    pub measure: Rc<RefCell<MeasureCache>>,
    pub page: usize,
    pub selection: (Position, Position),
    pub head: Position,
    pub focused: bool,
    pub text: Rgba,
    pub selection_color: Rgba,
    pub epoch: Rc<Cell<Instant>>,
    pub scroll: Rc<ScrollState>,
    pub reveal: Rc<Cell<bool>>,
}

/// Slack around the viewport in which lines are still painted.
const CULL: f32 = 80.0;

pub fn paint_page(v: &PageView, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    let m = GpuiMeasurer::new(window, v.measure.clone());
    let origin = bounds.origin;
    let at = |x: f32, y: f32| point(origin.x + px(x), origin.y + px(y));
    let (view_top, view_bottom) = v.scroll.visible();
    let visible = |top: f32, height: f32| {
        let top = f32::from(origin.y) + top;
        top + height > view_top - CULL && top < view_bottom + CULL
    };

    let (start, end) = v.selection;
    if start != end {
        for r in v
            .layout
            .selection_rects(start, end, &m)
            .iter()
            .filter(|r| r.page == v.page && visible(r.y, r.height))
        {
            window.paint_quad(fill(
                Bounds::new(at(r.x, r.y), size(px(r.width), px(r.height))),
                v.selection_color,
            ));
        }
    }

    let setup = &v.layout.setup;
    for slice in &v.layout.pages[v.page].slices {
        let para = &v.layout.paragraphs[slice.para];
        let first_y = para.lines[slice.lines.start].y;
        for line in &para.lines[slice.lines.clone()] {
            let top = setup.margin_top + slice.y + line.y - first_y;
            if !visible(top, line.height) {
                continue;
            }
            for f in &line.fragments {
                let text = &para.text[f.range.clone()];
                let color: Hsla = f.style.color.map_or(v.text, rgb).into();
                let (ascent, descent) = (m.ascent(&f.style), m.descent(&f.style));
                let y = top + line.baseline - ascent;
                let mut x = setup.margin_left + line.x + f.x;
                let paint = |piece: &str, x: f32, window: &mut Window, cx: &mut App| {
                    if !piece.trim().is_empty() || f.style.underline {
                        let shaped = m.shape(piece, &f.style, color);
                        let _ = shaped.paint(
                            at(x, y),
                            px(ascent + descent),
                            TextAlign::Left,
                            None,
                            window,
                            cx,
                        );
                    }
                };
                if line.space_extra == 0.0 {
                    paint(text, x, window, cx);
                } else {
                    // Justified: words are placed one by one so the extra space goes between them.
                    let mut offset = f.range.start;
                    for piece in text.split_inclusive(' ') {
                        paint(piece.trim_end_matches(' '), x, window, cx);
                        x += m.width(piece, &f.style);
                        offset += piece.len();
                        if piece.ends_with(' ') && offset <= line.visible_end {
                            x += line.space_extra;
                        }
                    }
                }
            }
        }
    }

    if v.focused
        && let Some(c) = v.layout.caret(v.head, &m)
        && c.page == v.page
    {
        let alpha = caret_alpha(v.epoch.get().elapsed().as_secs_f32());
        window.paint_quad(fill(
            Bounds::new(at(c.x, c.y), size(px(1.5), px(c.height))),
            Rgba { a: alpha, ..v.text },
        ));
        if v.reveal.replace(false) {
            v.scroll.reveal(origin.y + px(c.y), px(c.height));
        }
        window.request_animation_frame();
    }
}
