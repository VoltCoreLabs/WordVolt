use wv_core::{Document, Position};

use std::rc::Rc;

use crate::Measurer;
use crate::layout_cache::LayoutCache;
use crate::numbering::Counters;
use crate::page::{Page, PageSetup, Slice, paginate};
use crate::paragraph::{Options, ParagraphLayout};
use crate::table;

/// Caret position in page coordinates (origin = top-left corner of the page, margins included).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretRect {
    pub page: usize,
    pub x: f32,
    pub y: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionRect {
    pub page: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Whole-document layout: every paragraph laid out and flowed onto pages.
pub struct DocLayout {
    pub setup: PageSetup,
    pub paragraphs: Vec<Rc<ParagraphLayout>>,
    pub pages: Vec<Page>,
    /// For every paragraph the (page, slice index) pairs where it is placed, in order.
    locations: Vec<Vec<(usize, usize)>>,
}

/// Width of the mark drawn for a selected paragraph break on an empty line.
const BREAK_MARK: f32 = 6.0;

impl DocLayout {
    pub fn new(doc: &Document, setup: PageSetup, m: &dyn Measurer) -> Self {
        Self::with_cache(doc, setup, m, &mut LayoutCache::default())
    }

    /// Like `new`, reusing the layout of paragraphs that did not change since the last time.
    pub fn with_cache(
        doc: &Document,
        setup: PageSetup,
        m: &dyn Measurer,
        cache: &mut LayoutCache,
    ) -> Self {
        cache.begin();
        // Text is laid out for one column; tables and paragraphs in cells likewise.
        let content_width = setup.column_width();
        let fractions = table::column_fractions(doc);
        let mut counters = Counters::default();
        let paragraphs: Vec<Rc<ParagraphLayout>> = doc
            .paragraphs
            .iter()
            .map(|p| {
                let marker = counters.marker(&p.style);
                let width = match p.style.cell {
                    Some(cell) => table::inner_width(
                        content_width,
                        &fractions[&cell.table],
                        cell.col,
                        cell.colspan,
                    ),
                    None => content_width,
                };
                let options = Options {
                    width,
                    line_spacing: setup.line_spacing,
                };
                match p.with_named_style() {
                    Some(resolved) => cache.layout(&resolved, &options, marker, m),
                    None => cache.layout(p, &options, marker, m),
                }
            })
            .collect();
        cache.end();
        let pages = paginate(&paragraphs, &table::blocks(doc), &setup);
        let mut locations = vec![Vec::new(); paragraphs.len()];
        for (page, p) in pages.iter().enumerate() {
            for (slice_ix, s) in p.slices.iter().enumerate() {
                locations[s.para].push((page, slice_ix));
            }
        }
        Self {
            setup,
            paragraphs,
            pages,
            locations,
        }
    }

    /// Page index, slice and top of line `line` of paragraph `para` (page coordinates).
    fn locate(&self, para: usize, line: usize) -> Option<(usize, &Slice, f32)> {
        let layout = self.paragraphs.get(para)?;
        self.locations
            .get(para)?
            .iter()
            .find_map(|&(page, slice_ix)| {
                let s = &self.pages[page].slices[slice_ix];
                s.lines.contains(&line).then(|| {
                    let top = self.setup.margin_top + s.y + layout.lines[line].y
                        - layout.lines[s.lines.start].y;
                    (page, s, top)
                })
            })
    }

    pub fn caret(&self, pos: Position, m: &dyn Measurer) -> Option<CaretRect> {
        let para = self.paragraphs.get(pos.para)?;
        let line_ix = para.line_of(pos.offset.min(para.text.len()));
        let line = &para.lines[line_ix];
        let (page, slice, y) = self.locate(pos.para, line_ix)?;
        Some(CaretRect {
            page,
            x: self.setup.margin_left + slice.x + line.x_at(&para.text, pos.offset, m),
            y,
            height: line.height,
        })
    }

    /// Document position nearest to a point on `page` (page coordinates).
    pub fn position_at(&self, page: usize, x: f32, y: f32, m: &dyn Measurer) -> Position {
        let page = &self.pages[page.min(self.pages.len() - 1)];
        let (x, y) = (x - self.setup.margin_left, y - self.setup.margin_top);
        // The text box under the point, or the nearest one (cells sit side by side at the same height).
        let distance = |from: f32, size: f32, at: f32| {
            if at < from {
                from - at
            } else {
                (at - (from + size)).max(0.0)
            }
        };
        let nearest = page.slices.iter().min_by(|a, b| {
            let score =
                |s: &Slice| distance(s.y, s.height, y).powi(2) + distance(s.x, s.width, x).powi(2);
            score(a).total_cmp(&score(b))
        });
        let Some(slice) = nearest else {
            return Position::default();
        };
        let para = &self.paragraphs[slice.para];
        let first_y = para.lines[slice.lines.start].y;
        let line_ix = slice
            .lines
            .clone()
            .find(|i| y < slice.y + para.lines[*i].y - first_y + para.lines[*i].height)
            .unwrap_or(slice.lines.end - 1);
        Position::new(
            slice.para,
            para.lines[line_ix].index_at(&para.text, x - slice.x, m),
        )
    }

    /// Byte range of the visual line holding `pos` (for Home/End): trailing spaces of wrapped lines are excluded.
    pub fn line_range(&self, pos: Position) -> (usize, usize) {
        let para = &self.paragraphs[pos.para.min(self.paragraphs.len() - 1)];
        let line = &para.lines[para.line_of(pos.offset.min(para.text.len()))];
        (
            line.range.start,
            if line.is_last {
                line.range.end
            } else {
                line.visible_end
            },
        )
    }

    /// Rectangles to highlight for the selection `start..end` (document order).
    pub fn selection_rects(
        &self,
        start: Position,
        end: Position,
        m: &dyn Measurer,
    ) -> Vec<SelectionRect> {
        (0..self.pages.len())
            .flat_map(|page| self.selection_rects_on_page(start, end, page, m))
            .collect()
    }

    /// The part of the selection `start..end` that lies on page `page` (cheap: only that page's
    /// lines are looked at).
    pub fn selection_rects_on_page(
        &self,
        start: Position,
        end: Position,
        page: usize,
        m: &dyn Measurer,
    ) -> Vec<SelectionRect> {
        let mut rects = Vec::new();
        let Some(p) = self.pages.get(page) else {
            return rects;
        };
        for slice in &p.slices {
            let para_ix = slice.para;
            if para_ix < start.para || para_ix > end.para {
                continue;
            }
            let para = &self.paragraphs[para_ix];
            let from = if para_ix == start.para {
                start.offset
            } else {
                0
            };
            let to = if para_ix == end.para {
                end.offset
            } else {
                para.text.len()
            };
            let first_y = para.lines[slice.lines.start].y;
            for line_ix in slice.lines.clone() {
                let line = &para.lines[line_ix];
                let (a, b) = (from.max(line.range.start), to.min(line.range.end));
                let break_selected = line.range.is_empty() && para_ix < end.para;
                if a >= b && !break_selected {
                    continue;
                }
                let y = self.setup.margin_top + slice.y + line.y - first_y;
                let x1 = self.setup.margin_left + slice.x + line.x_at(&para.text, a, m);
                let width = if a < b {
                    line.x_at(&para.text, b, m) - line.x_at(&para.text, a, m)
                } else {
                    BREAK_MARK
                };
                rects.push(SelectionRect {
                    page,
                    x: x1,
                    y,
                    width,
                    height: line.height,
                });
            }
        }
        rects
    }
}

#[cfg(test)]
mod tests;
