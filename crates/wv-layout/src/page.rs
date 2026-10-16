use std::ops::Range;

use crate::paragraph::ParagraphLayout;

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
#[derive(Clone, Debug, PartialEq)]
pub struct Slice {
    pub para: usize,
    pub lines: Range<usize>,
    pub y: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Page {
    pub slices: Vec<Slice>,
}

/// Flows laid-out paragraphs onto pages; a paragraph may continue on the next page (between lines).
/// The gap above a paragraph is dropped at the top of a page.
pub fn paginate(paragraphs: &[ParagraphLayout], setup: &PageSetup) -> Vec<Page> {
    let limit = setup.content_height();
    let mut pages = vec![Page::default()];
    let mut y = 0.0;
    for (para_ix, para) in paragraphs.iter().enumerate() {
        for (line_ix, line) in para.lines.iter().enumerate() {
            let page_has_content = pages.last().is_some_and(|p| !p.slices.is_empty());
            let lead = if line_ix == 0 && y > 0.0 {
                para.space_before
            } else {
                0.0
            };
            if y + lead + line.height > limit + EPS && page_has_content {
                pages.push(Page::default());
                y = 0.0;
            } else {
                y += lead;
            }
            let page = pages.last_mut().expect("at least one page");
            match page.slices.last_mut() {
                Some(s) if s.para == para_ix && s.lines.end == line_ix => {
                    s.lines.end += 1;
                    s.height += line.height;
                }
                _ => page.slices.push(Slice {
                    para: para_ix,
                    lines: line_ix..line_ix + 1,
                    y,
                    height: line.height,
                }),
            }
            y += line.height;
        }
        y += para.space_after.unwrap_or(setup.paragraph_spacing);
    }
    pages
}
