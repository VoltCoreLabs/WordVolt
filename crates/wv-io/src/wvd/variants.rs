use serde::{Deserialize, Serialize};
use wv_core::{Band, BandVariants, CharStyle};

use super::schema::{RunDto, band_align_from, band_align_name, run_dto, run_of};

/// The runs of a band when it has any formatting; empty for plain text (the text field is enough).
pub fn styled_runs(band: &Band) -> Vec<RunDto> {
    let plain = band.runs().iter().all(|r| r.style == CharStyle::default());
    if plain {
        Vec::new()
    } else {
        band.runs().iter().map(run_dto).collect()
    }
}

/// A band from its stored text, formatted runs (preferred when present) and alignment name.
pub fn band_of(text: String, runs: Vec<RunDto>, align: &str) -> Band {
    let band = if runs.is_empty() {
        Band::new(text)
    } else {
        Band::from_runs(runs.into_iter().map(run_of).collect())
    };
    band.aligned(band_align_from(align))
}

/// One header or footer: text, formatted runs (only when formatted) and alignment ("left" /
/// "right"; empty = centred).
#[derive(Serialize, Deserialize, Default)]
struct BandDto {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    runs: Vec<RunDto>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    align: String,
}

impl BandDto {
    fn of(band: &Band) -> Self {
        Self {
            text: band.text(),
            runs: styled_runs(band),
            align: band_align_name(band.align),
        }
    }

    fn band(self) -> Band {
        band_of(self.text, self.runs, &self.align)
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
