use std::ops::Range;

use crate::table::TableBox;
use wv_core::Wrap;

pub(crate) const EPS: f32 = 0.01;

/// Space between text columns: 0.5 inch.
pub const COLUMN_GAP: f32 = 48.0;

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
    /// Text columns.
    pub columns: u8,
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
            columns: 1,
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
            columns: spec.columns.max(1),
            ..Self::a4()
        }
    }

    pub fn content_width(&self) -> f32 {
        self.width - self.margin_left - self.margin_right
    }

    /// Width of one text column.
    pub fn column_width(&self) -> f32 {
        let n = f32::from(self.columns.max(1));
        (self.content_width() - COLUMN_GAP * (n - 1.0)) / n
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
    /// Page size and margins of the section the page belongs to.
    pub setup: PageSetup,
    pub slices: Vec<Slice>,
    pub tables: Vec<TableBox>,
    /// Floating pictures of the page (their paragraphs take no room in the text).
    pub floats: Vec<PlacedFloat>,
    /// Where the footnotes of the page start (the separator line), from the top of the text area.
    pub note_top: Option<f32>,
}

impl Page {
    pub(crate) fn is_empty(&self) -> bool {
        self.slices.is_empty() && self.tables.is_empty()
    }
}

/// A floating picture on a page; `x`/`y` are from the top-left corner of the text area.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacedFloat {
    /// The picture paragraph.
    pub para: usize,
    pub id: u32,
    pub x: f32,
    pub y: f32,
    /// The box of the turned picture (what the text goes around).
    pub width: f32,
    pub height: f32,
    pub wrap: Wrap,
    /// The picture itself: its size before turning, the angle and the cut.
    pub frame: (f32, f32),
    pub angle: f32,
    pub crop: wv_core::Crop,
}
