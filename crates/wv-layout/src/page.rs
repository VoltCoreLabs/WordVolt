use std::ops::Range;

use std::rc::Rc;

use crate::paragraph::ParagraphLayout;
use crate::table::{Block, TABLE_GAP, TableBox};

pub(crate) const EPS: f32 = 0.01;

/// Paper size and margins in logical pixels (96 per inch).
#[derive(Clone, Debug, PartialEq)]
pub struct PageSetup {
    pub width: f32,
    pub height: f32,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    /// Distance from the page edge to the header / footer text.
    pub header_distance: f32,
    pub footer_distance: f32,
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
            header_distance: 47.2,
            footer_distance: 47.2,
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
            // The gutter is part of the left margin as far as the text is concerned.
            margin_left: spec.margin_left + spec.gutter,
            header_distance: spec.header_distance,
            footer_distance: spec.footer_distance,
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
    pub(crate) fn is_empty(&self) -> bool {
        self.slices.is_empty() && self.tables.is_empty()
    }
}

/// Flows body paragraphs and tables onto pages. A paragraph may continue on the next page (between
/// lines); a table row never splits (the table continues on the next page). The gap above a
/// paragraph is dropped at the top of a page.
pub fn paginate(
    paragraphs: &[Rc<ParagraphLayout>],
    blocks: &[Block],
    setup: &PageSetup,
) -> Vec<Page> {
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
                crate::table_page::paginate_table(paragraphs, table, setup, &mut pages, &mut y);
                y += TABLE_GAP;
            }
        }
    }
    pages
}
