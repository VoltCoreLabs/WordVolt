use serde::{Deserialize, Serialize};

use super::convert::*;
use super::variants::VariantsDto;

#[derive(Serialize, Deserialize)]
pub(super) struct File {
    pub(super) format: String,
    pub(super) version: u32,
    pub(super) paragraphs: Vec<Para>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(super) header: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(super) footer: String,
    /// "left" / "right"; empty = centred.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(super) header_align: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(super) footer_align: String,
    /// Formatted runs of the header / footer (only when it has formatting).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) header_runs: Vec<RunDto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) footer_runs: Vec<RunDto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) images: Vec<ImageDto>,
    /// Page size and margins in px; absent = A4 with 2.54 cm margins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) page: Option<PageDto>,
    /// First-page and even-page headers/footers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) band_variants: Option<VariantsDto>,
    /// Column widths of tables (older files): (table id, fractions of the text width).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) table_widths: Vec<(u32, Vec<f32>)>,
    /// How tables are sized.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) table_looks: Vec<TableLookDto>,
    /// Comments (the text they are about carries their id).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) comments: Vec<CommentDto>,
    /// Authors and dates of tracked changes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) revisions: Vec<CommentDto>,
    /// Page setup of the sections that end with a break.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) sections: Vec<SectionDto>,
    /// Styles defined by the user.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) styles: Vec<super::styles::UserStyleDto>,
    /// Headers and footers of the sections that end with a break.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) section_bands: Vec<super::variants::SectionBandsDto>,
    /// The file's default font and size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) default_font: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) default_size_pt: Option<u16>,
    /// Bookmarks (the text they mark carries their id).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) bookmarks: Vec<BookmarkDto>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct BookmarkDto {
    pub(super) id: u32,
    pub(super) name: String,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(super) struct PageDto {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) top: f32,
    pub(super) right: f32,
    pub(super) bottom: f32,
    pub(super) left: f32,
    #[serde(default)]
    pub(super) gutter: f32,
    #[serde(default = "band_distance")]
    pub(super) header_distance: f32,
    #[serde(default = "band_distance")]
    pub(super) footer_distance: f32,
    #[serde(default = "one_column")]
    pub(super) columns: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) page_start: Option<u32>,
    /// Index into `NumberFormat::ALL` (absent = decimal).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(super) number_format: u8,
}

impl PageDto {
    pub(super) fn of(page: &wv_core::PageSpec) -> Self {
        Self {
            width: page.width,
            height: page.height,
            top: page.margin_top,
            right: page.margin_right,
            bottom: page.margin_bottom,
            left: page.margin_left,
            gutter: page.gutter,
            header_distance: page.header_distance,
            footer_distance: page.footer_distance,
            columns: page.columns,
            page_start: page.page_start,
            number_format: wv_core::NumberFormat::ALL
                .iter()
                .position(|f| *f == page.number_format)
                .unwrap_or(0) as u8,
        }
    }

    /// `None` for a size that cannot be a page.
    pub(super) fn spec(self) -> Option<wv_core::PageSpec> {
        (self.width >= 100.0 && self.height >= 100.0).then(|| wv_core::PageSpec {
            width: self.width,
            height: self.height,
            margin_top: self.top.max(0.0),
            margin_right: self.right.max(0.0),
            margin_bottom: self.bottom.max(0.0),
            margin_left: self.left.max(0.0),
            gutter: self.gutter.max(0.0),
            header_distance: self.header_distance.max(0.0),
            footer_distance: self.footer_distance.max(0.0),
            columns: self.columns.clamp(1, wv_core::MAX_COLUMNS),
            page_start: self.page_start,
            number_format: wv_core::NumberFormat::ALL
                .get(usize::from(self.number_format))
                .copied()
                .unwrap_or_default(),
        })
    }
}

/// Page setup of the section that ends with the paragraph carrying `section = id`.
#[derive(Serialize, Deserialize)]
pub(super) struct SectionDto {
    pub(super) id: u32,
    pub(super) page: PageDto,
}

pub(super) fn one_column() -> u8 {
    1
}

pub(super) fn band_distance() -> f32 {
    wv_core::BAND_DISTANCE
}

/// Sizes of a table: column fractions, the indent (fraction) and least row heights (px).
#[derive(Serialize, Deserialize)]
pub(super) struct TableLookDto {
    pub(super) table: u32,
    pub(super) widths: Vec<f32>,
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub(super) indent: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) rows: Vec<f32>,
}

pub(super) fn is_zero_f32(v: &f32) -> bool {
    *v == 0.0
}

/// Embedded picture: format name and base64 data.
#[derive(Serialize, Deserialize)]
pub(super) struct ImageDto {
    pub(super) format: String,
    pub(super) data: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct ImageRefDto {
    pub(super) id: u32,
    pub(super) width: f32,
    pub(super) height: f32,
    /// Floating picture: position on the page and how the text goes around it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) float: Option<FloatDto>,
    /// Degrees clockwise.
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub(super) angle: f32,
    /// Cut off the left, top, right and bottom (fractions of the picture).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) crop: Option<[f32; 4]>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct FloatDto {
    pub(super) x: f32,
    pub(super) y: f32,
    /// `topbottom` (default), `front` or `behind`.
    #[serde(default)]
    pub(super) wrap: String,
}

/// Table cell of a paragraph.
#[derive(Serialize, Deserialize, Clone, Copy)]
pub(super) struct CellDto {
    pub(super) table: u32,
    pub(super) row: u16,
    pub(super) col: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) shade: Option<u32>,
    /// Merged cells: columns / rows covered (absent = 1).
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub(super) colspan: u8,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub(super) rowspan: u8,
}

pub(super) fn one() -> u8 {
    1
}

pub(super) fn is_one(n: &u8) -> bool {
    *n == 1
}

#[derive(Serialize, Deserialize)]
pub(super) struct Para {
    #[serde(default)]
    pub(super) align: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(super) style: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub(super) quote: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(super) list: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) cell: Option<CellDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) image: Option<ImageRefDto>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub(super) toc: bool,
    /// The paragraph ends a section (id into `File::sections`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) section: Option<u32>,
    /// The paragraph has this user style.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) custom: Option<u32>,
    /// The paragraph is (part of) the footnote with this id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) note: Option<u32>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(super) indent: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) line_spacing: Option<f32>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(super) first_line: u8,
    #[serde(default, skip_serializing_if = "is_false")]
    pub(super) page_break: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(super) left_extra: u8,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) tabs: Vec<u16>,
    /// Kinds of the tab stops (index into `TabKind::ALL`); absent = all left.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) tab_kinds: Vec<u8>,
    /// Leaders of the tab stops (index into `TabLeader::ALL`); absent = none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) tab_leaders: Vec<u8>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(super) right_indent: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) space_before: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) space_after: Option<u8>,
    pub(super) runs: Vec<RunDto>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct CommentDto {
    pub(super) id: u32,
    #[serde(default)]
    pub(super) author: String,
    #[serde(default)]
    pub(super) text: String,
    #[serde(default)]
    pub(super) date: String,
}
