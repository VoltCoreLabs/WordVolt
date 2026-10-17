use gpui_kit::*;
use wv_core::{Band, CharStyle};
use wv_layout::Measurer;

use super::measure::GpuiMeasurer;
use super::page_paint::PageView;

const BAND_PT: u16 = 10;

/// Paints the header and footer text, centred in the top and bottom margins.
pub fn paint_bands(
    v: &PageView,
    m: &GpuiMeasurer,
    at: &dyn Fn(f32, f32) -> Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let setup = &v.layout.setup;
    let pages = v.layout.pages.len();
    let style = CharStyle {
        size_pt: Some(BAND_PT),
        ..Default::default()
    };
    let (ascent, descent) = (m.ascent(&style), m.descent(&style));
    let height = ascent + descent;
    let (header, footer): &(Band, Band) = &v.bands;
    for (band, centre) in [
        (header, setup.margin_top / 2.0),
        (footer, setup.height - setup.margin_bottom / 2.0),
    ] {
        let text = band.render(v.page + 1, pages);
        if text.is_empty() {
            continue;
        }
        let x = (setup.width - m.width(&text, &style)) / 2.0;
        let shaped = m.shape(&text, &style, v.band_color.into());
        let _ = shaped.paint(
            at(x.max(setup.margin_left / 2.0), centre - height / 2.0),
            px(height),
            TextAlign::Left,
            None,
            window,
            cx,
        );
    }
}
