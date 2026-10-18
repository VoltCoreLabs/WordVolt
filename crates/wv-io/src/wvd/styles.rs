use serde::{Deserialize, Serialize};
use wv_core::{NamedStyle, UserStyle};

use super::convert::{align_from, align_name};

/// A style the user defined (see `wv_core::UserStyle`).
#[derive(Serialize, Deserialize)]
pub(super) struct UserStyleDto {
    pub(super) id: u32,
    pub(super) name: String,
    #[serde(default)]
    pub(super) base: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) font: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) size_pt: Option<u16>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(super) bold: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(super) italic: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(super) underline: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) color: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) line_spacing: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) space_before: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) space_after: Option<u8>,
}

impl UserStyleDto {
    pub(super) fn of(id: u32, s: &UserStyle) -> Self {
        Self {
            id,
            name: s.name.clone(),
            base: s.base.id().to_string(),
            font: s.font.clone(),
            size_pt: s.size_pt,
            bold: s.bold,
            italic: s.italic,
            underline: s.underline,
            color: s.color,
            align: s.align.map(|a| align_name(a).to_string()),
            line_spacing: s.line_spacing,
            space_before: s.space_before,
            space_after: s.space_after,
        }
    }

    pub(super) fn style(self) -> (u32, UserStyle) {
        let style = UserStyle {
            name: self.name,
            base: NamedStyle::from_id(&self.base).unwrap_or_default(),
            font: self.font,
            size_pt: self.size_pt,
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
            color: self.color,
            align: self.align.as_deref().map(align_from),
            line_spacing: self.line_spacing,
            space_before: self.space_before,
            space_after: self.space_after,
        };
        (self.id, style)
    }
}
