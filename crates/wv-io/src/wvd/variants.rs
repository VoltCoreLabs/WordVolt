use serde::{Deserialize, Serialize};
use wv_core::{Band, BandVariants};

use super::schema::{band_align_from, band_align_name};

/// One header or footer: text and alignment ("left" / "right"; empty = centred).
#[derive(Serialize, Deserialize, Default)]
struct BandDto {
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
        }
    }

    fn band(self) -> Band {
        Band::new(self.text).aligned(band_align_from(&self.align))
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
