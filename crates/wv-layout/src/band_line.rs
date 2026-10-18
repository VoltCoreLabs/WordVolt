use wv_core::{BAND_SIZE_PT, Band, CharStyle};

use crate::Measurer;

/// One formatted piece of a header/footer line.
pub struct BandPart {
    pub text: String,
    /// The style with the band's default size filled in.
    pub style: CharStyle,
    /// Offset from the start of the line.
    pub x: f32,
    pub width: f32,
    pub ascent: f32,
}

/// A header or footer line for one page, measured.
pub struct BandLine {
    pub parts: Vec<BandPart>,
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
}

impl BandLine {
    pub fn height(&self) -> f32 {
        self.ascent + self.descent
    }
}

/// Lays out `band` for the page numbered `page` (as written) of `pages`; `None` when it shows
/// nothing.
pub fn band_line(band: &Band, page: &str, pages: usize, m: &dyn Measurer) -> Option<BandLine> {
    let (mut parts, mut x) = (Vec::new(), 0.0);
    let (mut ascent, mut descent) = (0.0_f32, 0.0_f32);
    for run in band.render_runs(page, pages) {
        if run.text.is_empty() {
            continue;
        }
        let mut style = run.style;
        style.size_pt.get_or_insert(BAND_SIZE_PT);
        let width = m.width(&run.text, &style);
        let (a, d) = (m.ascent(&style), m.descent(&style));
        (ascent, descent) = (ascent.max(a), descent.max(d));
        parts.push(BandPart {
            text: run.text,
            style,
            x,
            width,
            ascent: a,
        });
        x += width;
    }
    (!parts.is_empty()).then_some(BandLine {
        parts,
        width: x,
        ascent,
        descent,
    })
}
