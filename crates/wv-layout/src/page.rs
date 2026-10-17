use std::ops::Range;

use crate::paragraph::ParagraphLayout;
use crate::table::{Block, CELL_PADDING, MIN_ROW_TEXT, TABLE_GAP, TableBlock, TableBox};

const EPS: f32 = 0.01;

/// Paper size and margins in logical pixels (96 per inch).
#[derive(Clone, Debug, PartialEq)]
pub struct PageSetup {
    pub width: f32,
    pub height: f32,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    /// Gap after every paragraph.
    pub paragraph_spacing: f32,
    /// Line height multiplier.
    pub line_spacing: f32,
}

impl PageSetup {
    /// A4 (210 x 297 mm) with 2.54 cm margins.
    pub fn a4() -> Self {
        Self {
            width: 793.7,
            height: 1122.5,
            margin_top: 96.0,
            margin_right: 96.0,
            margin_bottom: 96.0,
            margin_left: 96.0,
            paragraph_spacing: 8.0,
            line_spacing: 1.15,
        }
    }

    /// The document's page size and margins with the default text spacing.
    pub fn from_spec(spec: &wv_core::PageSpec) -> Self {
        Self {
            width: spec.width,
            height: spec.height,
            margin_top: spec.margin_top,
            margin_right: spec.margin_right,
            margin_bottom: spec.margin_bottom,
            margin_left: spec.margin_left,
            ..Self::a4()
        }
    }

    pub fn content_width(&self) -> f32 {
        self.width - self.margin_left - self.margin_right
    }

    pub fn content_height(&self) -> f32 {
        self.height - self.margin_top - self.margin_bottom
    }
}

impl Default for PageSetup {
    fn default() -> Self {
        Self::a4()
    }
}

/// Lines `lines` of paragraph `para`, placed at `y` (from the top of the page content area).
/// `x`/`width` locate the text box: the whole text width for body text, one cell inside a table.
#[derive(Clone, Debug, PartialEq)]
pub struct Slice {
    pub para: usize,
    pub lines: Range<usize>,
    pub y: f32,
    pub height: f32,
    pub x: f32,
    pub width: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Page {
    pub slices: Vec<Slice>,
    pub tables: Vec<TableBox>,
}

impl Page {
    fn is_empty(&self) -> bool {
        self.slices.is_empty() && self.tables.is_empty()
    }
}

/// Flows body paragraphs and tables onto pages. A paragraph may continue on the next page (between
/// lines); a table row never splits (the table continues on the next page). The gap above a
/// paragraph is dropped at the top of a page.
pub fn paginate(paragraphs: &[ParagraphLayout], blocks: &[Block], setup: &PageSetup) -> Vec<Page> {
    let limit = setup.content_height();
    let content_width = setup.content_width();
    let mut pages = vec![Page::default()];
    let mut y = 0.0;
    for block in blocks {
        match block {
            Block::Para(para_ix) => {
                let para = &paragraphs[*para_ix];
                for (line_ix, line) in para.lines.iter().enumerate() {
                    let lead = if line_ix == 0 && y > 0.0 {
                        para.space_before
                    } else {
                        0.0
                    };
                    if y + lead + line.height > limit + EPS
                        && !pages.last().is_some_and(Page::is_empty)
                    {
                        pages.push(Page::default());
                        y = 0.0;
                    } else {
                        y += lead;
                    }
                    let page = pages.last_mut().expect("at least one page");
                    match page.slices.last_mut() {
                        Some(s) if s.para == *para_ix && s.lines.end == line_ix && s.x == 0.0 => {
                            s.lines.end += 1;
                            s.height += line.height;
                        }
                        _ => page.slices.push(Slice {
                            para: *para_ix,
                            lines: line_ix..line_ix + 1,
                            y,
                            height: line.height,
                            x: 0.0,
                            width: content_width,
                        }),
                    }
                    y += line.height;
                }
                y += para.space_after.unwrap_or(setup.paragraph_spacing);
            }
            Block::Table(table) => {
                paginate_table(paragraphs, table, setup, &mut pages, &mut y);
                y += TABLE_GAP;
            }
        }
    }
    pages
}

fn paginate_table(
    paragraphs: &[ParagraphLayout],
    table: &TableBlock,
    setup: &PageSetup,
    pages: &mut Vec<Page>,
    y: &mut f32,
) {
    let limit = setup.content_height();
    let col_width = setup.content_width() / table.cols.max(1) as f32;
    let col_x: Vec<f32> = (0..=table.cols).map(|c| c as f32 * col_width).collect();
    let inner = (col_width - 2.0 * CELL_PADDING).max(20.0);
    let gap = |p: &ParagraphLayout| p.space_after.unwrap_or(setup.paragraph_spacing);
    let mut open_box: Option<usize> = None;
    for row in &table.rows {
        let text_height = row
            .iter()
            .map(|cell| {
                let total: f32 = cell.iter().map(|i| paragraphs[*i].height).sum();
                let gaps: f32 = cell
                    .iter()
                    .take(cell.len().saturating_sub(1))
                    .map(|i| gap(&paragraphs[*i]))
                    .sum();
                total + gaps
            })
            .fold(MIN_ROW_TEXT, f32::max);
        let row_height = text_height + 2.0 * CELL_PADDING;
        if *y + row_height > limit + EPS && !pages.last().is_some_and(Page::is_empty) {
            pages.push(Page::default());
            *y = 0.0;
            open_box = None;
        }
        let page = pages.last_mut().expect("at least one page");
        let box_ix = *open_box.get_or_insert_with(|| {
            page.tables.push(TableBox {
                col_x: col_x.clone(),
                row_y: vec![*y],
            });
            page.tables.len() - 1
        });
        page.tables[box_ix].row_y.push(*y + row_height);
        for (c, cell) in row.iter().enumerate() {
            let mut cell_y = *y + CELL_PADDING;
            for (k, &para_ix) in cell.iter().enumerate() {
                let para = &paragraphs[para_ix];
                page.slices.push(Slice {
                    para: para_ix,
                    lines: 0..para.lines.len(),
                    y: cell_y,
                    height: para.height,
                    x: col_x[c] + CELL_PADDING,
                    width: inner,
                });
                cell_y += para.height + if k + 1 < cell.len() { gap(para) } else { 0.0 };
            }
        }
        *y += row_height;
    }
}
