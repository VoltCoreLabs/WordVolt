use serde::{Deserialize, Serialize};
use wv_core::{Band, BandStyle, BandVariants};

use super::schema::{band_align_from, band_align_name};

/// Formatting of a header or footer.
#[derive(Serialize, Deserialize, Default, PartialEq)]
pub struct StyleDto {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    bold: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    italic: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    underline: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    size_pt: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color: Option<u32>,
}

impl StyleDto {
    pub fn of(style: &BandStyle) -> Self {
        Self {
            bold: style.bold,
            italic: style.italic,
            underline: style.underline,
            size_pt: style.size_pt,
            color: style.color,
        }
    }

    pub fn style(&self) -> BandStyle {
        BandStyle {
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
            size_pt: self.size_pt.map(|p| p.clamp(6, 72)),
            color: self.color,
        }
    }

    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// One header or footer: text and alignment ("left" / "right"; empty = centred).
#[derive(Serialize, Deserialize, Default)]
struct BandDto {
    #[serde(default, skip_serializing_if = "StyleDto::is_default")]
    style: StyleDto,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    text: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    align: String,
}

impl BandDto {
    fn of(band: &Band) -> Self {
        Self {
            text: band.text.clone(),
            align: band_align_name(band.align),
            style: StyleDto::of(&band.style),
        }
    }

    fn band(self) -> Band {
        Band::new(self.text)
            .aligned(band_align_from(&self.align))
            .styled(self.style.style())
    }
}

/// First-page and even-page headers/footers with their switches.
#[derive(Serialize, Deserialize, Default)]
pub struct VariantsDto {
    #[serde(default)]
    different_first: bool,
    #[serde(default)]
    odd_even: bool,
    #[serde(default)]
    first_header: BandDto,
    #[serde(default)]
    first_footer: BandDto,
    #[serde(default)]
    even_header: BandDto,
    #[serde(default)]
    even_footer: BandDto,
}

impl VariantsDto {
    /// `None` for the default (nothing to store).
    pub fn of(v: &BandVariants) -> Option<Self> {
        (*v != BandVariants::default()).then(|| Self {
            different_first: v.different_first,
            odd_even: v.odd_even,
            first_header: BandDto::of(&v.first.0),
            first_footer: BandDto::of(&v.first.1),
            even_header: BandDto::of(&v.even.0),
            even_footer: BandDto::of(&v.even.1),
        })
    }

    pub fn variants(self) -> BandVariants {
        BandVariants {
            different_first: self.different_first,
            odd_even: self.odd_even,
            first: (self.first_header.band(), self.first_footer.band()),
            even: (self.even_header.band(), self.even_footer.band()),
        }
    }
}
