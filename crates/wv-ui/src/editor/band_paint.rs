use gpui_kit::*;
use wv_core::{Align, CharStyle};
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
    let z = v.zoom;
    let height = ascent + descent;
    let (header, footer) = v.bands.2.pick(v.page, &v.bands.0, &v.bands.1);
    for (band, top) in [
        (header, setup.header_distance),
        (footer, setup.height - setup.footer_distance - height),
    ] {
        let text = band.render(v.page + 1, pages);
        if text.is_empty() {
            continue;
        }
        let width = m.width(&text, &style);
        let x = match band.align {
            Align::Right => setup.width - setup.margin_right - width,
            Align::Center => (setup.width - width) / 2.0,
            _ => setup.margin_left,
        };
        let shaped = m.shape_zoomed(&text, &style, v.band_color.into(), z);
        let _ = shaped.paint(
            at(x.max(0.0), top),
            px(height * z),
            TextAlign::Left,
            None,
            window,
            cx,
        );
    }
}
