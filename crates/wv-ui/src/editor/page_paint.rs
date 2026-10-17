use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use gpui_kit::*;
use wv_core::{Band, BandVariants, Position};
use wv_layout::{DocLayout, Measurer, script_shift};

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
    pub code_bg: Rgba,
    pub spoiler_color: Rgba,
    pub link_color: Rgba,
    pub bar_color: Rgba,
    /// Search matches (all, and which one is current) with their colours.
    pub find: Rc<Vec<(Position, Position)>>,
    pub find_current: Option<usize>,
    pub find_color: Rgba,
    pub find_current_color: Rgba,
    /// Table borders.
    pub table_color: Rgba,
    /// Header and footer text, and their colour.
    pub bands: Rc<(Band, Band, BandVariants)>,
    /// Pictures of the document by `ImageRef::id`.
    pub images: Rc<Vec<Arc<Image>>>,
    pub band_color: Rgba,
    /// Display scale (1.0 = 100%).
    pub zoom: f32,
}

/// More matches than this are still found and counted, but not all highlighted.
const MAX_HIGHLIGHTS: usize = 2000;

/// Slack around the viewport in which lines are still painted.
const CULL: f32 = 80.0;

pub fn paint_page(v: &PageView, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    let m = GpuiMeasurer::new(window, v.measure.clone());
    let origin = bounds.origin;
    let z = v.zoom;
    let at = |x: f32, y: f32| point(origin.x + px(x * z), origin.y + px(y * z));
    let sz = |w: f32, h: f32| size(px(w * z), px(h * z));
    // Visible region = clip rectangle of the scroll container (no cross-element bookkeeping).
    let clip = window.content_mask().bounds;
    let (view_top, view_bottom) = (f32::from(clip.top()), f32::from(clip.bottom()));
    let visible = |top: f32, height: f32| {
        let top = f32::from(origin.y) + top * v.zoom;
        top + height * v.zoom > view_top - CULL && top < view_bottom + CULL
    };

    // A page that is off screen paints nothing (only the caret may still ask to be scrolled to).
    if !visible(0.0, v.layout.setup.height) {
        paint_caret(v, &m, origin, &at, &sz, window);
        return;
    }

    let (start, end) = v.selection;
    if start != end {
        for r in v
            .layout
            .selection_rects_on_page(start, end, v.page, &m)
            .iter()
            .filter(|r| visible(r.y, r.height))
        {
            window.paint_quad(fill(
                Bounds::new(at(r.x, r.y), sz(r.width, r.height)),
                v.selection_color,
            ));
        }
    }

    // Search matches go over the selection (the current match is usually selected too). Only the
    // matches in the paragraphs of this page are looked at.
    let page_paras =
        v.layout.pages[v.page]
            .slices
            .iter()
            .fold(None, |acc: Option<(usize, usize)>, s| {
                Some(acc.map_or((s.para, s.para), |(lo, hi)| {
                    (lo.min(s.para), hi.max(s.para))
                }))
            });
    for (i, (from, to)) in v.find.iter().enumerate().take(MAX_HIGHLIGHTS) {
        let Some((lo, hi)) = page_paras else { break };
        if to.para < lo || from.para > hi {
            continue;
        }
        let color = if Some(i) == v.find_current {
            v.find_current_color
        } else {
            v.find_color
        };
        for r in v
            .layout
            .selection_rects_on_page(*from, *to, v.page, &m)
            .iter()
            .filter(|r| visible(r.y, r.height))
        {
            window.paint_quad(fill(
                Bounds::new(at(r.x, r.y), sz(r.width, r.height)),
                color,
            ));
        }
    }

    super::table_paint::paint_tables(v, &at, &visible, window);

    super::band_paint::paint_bands(v, &m, &at, window, cx);
    super::image_paint::paint_images(v, &at, &visible, window, cx);

    let setup = &v.layout.setup;
    for slice in &v.layout.pages[v.page].slices {
        let para = &v.layout.paragraphs[slice.para];
        let first_y = para.lines[slice.lines.start].y;
        if slice.lines.start == 0
            && let Some(marker) = &para.marker
        {
            let line = &para.lines[0];
            let top = setup.margin_top + slice.y;
            if visible(top, line.height) {
                // The marker looks like the first character of the item, without decorations.
                let mut style = line
                    .fragments
                    .first()
                    .map(|f| f.style.clone())
                    .unwrap_or_default();
                (
                    style.underline,
                    style.strike,
                    style.spoiler,
                    style.highlight,
                    style.link,
                ) = (false, false, false, None, None);
                let color: Hsla = style.color.map_or(v.text, rgb).into();
                let (ascent, descent) = (m.ascent(&style), m.descent(&style));
                let x = setup.margin_left + slice.x + para.indent - 6.0 - m.width(marker, &style);
                let y = top + line.baseline - ascent;
                let shaped = m.shape_zoomed(marker, &style, color, z);
                let _ = shaped.paint(
                    at(x, y),
                    px((ascent + descent) * z),
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                );
            }
        }
        if para.quote {
            let bar = Bounds::new(
                at(
                    setup.margin_left + slice.x + 4.0,
                    setup.margin_top + slice.y,
                ),
                sz(3., slice.height),
            );
            window.paint_quad(fill(bar, v.bar_color));
        }
        for line in &para.lines[slice.lines.clone()] {
            let top = setup.margin_top + slice.y + line.y - first_y;
            if !visible(top, line.height) {
                continue;
            }
            for f in &line.fragments {
                let text = &para.text[f.range.clone()];
                let color: Hsla = f
                    .style
                    .color
                    .map_or(
                        if f.style.link.is_some() {
                            v.link_color
                        } else {
                            v.text
                        },
                        rgb,
                    )
                    .into();
                let (ascent, descent) = (m.ascent(&f.style), m.descent(&f.style));
                let y = top + line.baseline - ascent + script_shift(&f.style);
                let mut x = setup.margin_left + slice.x + line.x + f.x;
                let background = f
                    .style
                    .highlight
                    .map(rgb)
                    .or(f.style.mono.then_some(v.code_bg));
                if let Some(background) = background {
                    window.paint_quad(fill(
                        Bounds::new(at(x, top), sz(f.width, line.height)),
                        background,
                    ));
                }
                if f.style.spoiler && !revealed(v, slice.para, &f.range) {
                    let block = Bounds::new(at(x, top + 2.0), sz(f.width, line.height - 4.0));
                    window.paint_quad(fill(
                        block,
                        Rgba {
                            a: 0.75,
                            ..v.spoiler_color
                        },
                    ));
                    continue;
                }
                let paint = |piece: &str, x: f32, window: &mut Window, cx: &mut App| {
                    if !piece.trim().is_empty() || f.style.underline {
                        let shaped = m.shape_zoomed(piece, &f.style, color, z);
                        let _ = shaped.paint(
                            at(x, y),
                            px((ascent + descent) * z),
                            TextAlign::Left,
                            None,
                            window,
                            cx,
                        );
                    }
                };
                if text == "\t" {
                    // A tab (contents entry): dot leader up to the page number.
                    if let Some((dots, offset)) = wv_layout::leader(f.width, &f.style, &m) {
                        paint(&dots, x + offset, window, cx);
                    }
                    continue;
                }
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

    paint_caret(v, &m, origin, &at, &sz, window);
}

/// The caret (blinking) when it is on this page; the first time it is painted after a move it
/// asks the scroll area to bring it into view.
fn paint_caret(
    v: &PageView,
    m: &GpuiMeasurer,
    origin: Point<Pixels>,
    at: &dyn Fn(f32, f32) -> Point<Pixels>,
    sz: &dyn Fn(f32, f32) -> Size<Pixels>,
    window: &mut Window,
) {
    let z = v.zoom;
    if v.focused
        && let Some(c) = v.layout.caret(v.head, m)
        && c.page == v.page
    {
        let alpha = caret_alpha(v.epoch.get().elapsed().as_secs_f32());
        window.paint_quad(fill(
            Bounds::new(at(c.x, c.y), sz(1.5, c.height)),
            Rgba { a: alpha, ..v.text },
        ));
        if v.reveal.replace(false) {
            v.scroll.reveal(origin.y + px(c.y * z), px(c.height * z));
        }
        window.request_animation_frame();
    }
}

/// A spoiler shows its text while the caret or the selection touches it.
fn revealed(v: &PageView, para: usize, range: &std::ops::Range<usize>) -> bool {
    let (start, end) = v.selection;
    let (lo, hi) = (
        Position::new(para, range.start),
        Position::new(para, range.end),
    );
    (v.head >= lo && v.head <= hi) || (start != end && start <= hi && end >= lo)
}
