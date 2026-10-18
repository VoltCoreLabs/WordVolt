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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) space_before: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) space_after: Option<u8>,
    pub(super) runs: Vec<RunDto>,
}
