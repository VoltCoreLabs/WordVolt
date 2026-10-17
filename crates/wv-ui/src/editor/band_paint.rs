use gpui_kit::*;
use wv_core::Align;
use wv_layout::Measurer;

use super::measure::GpuiMeasurer;
use super::page_paint::PageView;

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
    let z = v.zoom;
    let (header, footer) = v.bands.2.pick(v.page, &v.bands.0, &v.bands.1);
    for (band, is_header) in [(header, true), (footer, false)] {
        let text = band.render(v.page + 1, pages);
        if text.is_empty() {
            continue;
        }
        let style = band.style.char_style();
        let height = m.ascent(&style) + m.descent(&style);
        let top = if is_header {
            setup.header_distance
        } else {
            setup.height - setup.footer_distance - height
        };
        let width = m.width(&text, &style);
        let x = match band.align {
            Align::Right => setup.width - setup.margin_right - width,
            Align::Center => (setup.width - width) / 2.0,
            _ => setup.margin_left,
        };
        let color = band.style.color.map_or(v.band_color, rgb);
        let shaped = m.shape_zoomed(&text, &style, color.into(), z);
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
