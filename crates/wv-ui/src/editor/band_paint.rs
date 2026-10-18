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
    let setup = v.layout.setup_of(v.page);
    let pages = v.layout.pages.len();
    let z = v.zoom;
    let b = &v.bands;
    let own = v.layout.pages[v.page]
        .section
        .and_then(|id| b.sections.get(&id));
    let (main_header, main_footer) = own.map_or((&b.header, &b.footer), |(h, f)| (h, f));
    let (header, footer) = b.variants.pick(v.page, main_header, main_footer);
    for (band, is_header) in [(header, true), (footer, false)] {
        let Some(line) = wv_layout::band_line(band, &v.layout.pages[v.page].number, pages, m)
        else {
            continue;
        };
        let top = if is_header {
            setup.header_distance
        } else {
            setup.height - setup.footer_distance - line.height()
        };
        let x = match band.align {
            Align::Right => setup.width - setup.margin_right - line.width,
            Align::Center => (setup.width - line.width) / 2.0,
            _ => setup.margin_left,
        };
        for part in &line.parts {
            let color = part.style.color.map_or(v.band_color, rgb);
            let shaped = m.shape_zoomed(&part.text, &part.style, color.into(), z);
            let height = m.ascent(&part.style) + m.descent(&part.style);
            let _ = shaped.paint(
                at(x.max(0.0) + part.x, top + line.ascent - part.ascent),
                px(height * z),
                TextAlign::Left,
                None,
                window,
                cx,
            );
        }
    }
}
