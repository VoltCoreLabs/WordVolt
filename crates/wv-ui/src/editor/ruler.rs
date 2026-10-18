use gpui_kit::*;
use wv_core::{MAX_TABS, ParaMetrics};
use wv_layout::{DocLayout, Measurer};

use super::measure::GpuiMeasurer;

/// Height of the ruler strip at the top of the page view (px).
pub(super) const HEIGHT: f32 = 22.0;
/// One point in layout pixels.
const PT_PX: f32 = 96.0 / 72.0;
/// Pixels per centimetre.
const CM_PX: f32 = 96.0 / 2.54;
/// How far (px) a marker is grabbed from its centre.
const GRAB: f32 = 6.0;
/// A tab stop dragged this far off the strip (px) is removed.
const REMOVE_DISTANCE: f32 = 28.0;

/// Where the ruler is on the screen: the left edge of the page whose text it measures, its margins
/// and the zoom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Geom {
    pub page_left: f32,
    pub top: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub page_width: f32,
    pub zoom: f32,
}

impl Geom {
    /// Window x of `pt` points from the left edge of the text.
    pub fn x(&self, pt: f32) -> f32 {
        self.page_left + (self.margin_left + pt * PT_PX) * self.zoom
    }

    /// Points from the left edge of the text at window x.
    pub fn pt(&self, x: f32) -> f32 {
        ((x - self.page_left) / self.zoom - self.margin_left) / PT_PX
    }

    /// Width of the text area in points.
    pub fn text_pt(&self) -> f32 {
        (self.page_width - self.margin_left - self.margin_right) / PT_PX
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Grab {
    /// The indent of the whole paragraph (the lower marker).
    Left,
    /// The extra indent of the first line (the upper marker).
    First,
    Tab(usize),
}

/// A marker being dragged and the paragraph values it gives so far.
pub(super) struct Drag {
    pub grab: Grab,
    pub metrics: ParaMetrics,
    /// A tab stop pulled off the strip: it is removed when let go.
    pub removing: bool,
}

/// The marker under the pointer, or a new tab stop where the strip itself is pressed (its index
/// then is `metrics.tabs.len()`; `metrics` has the new stop).
pub(super) fn press(g: &Geom, m: &mut ParaMetrics, x: f32, y: f32) -> Option<Grab> {
    if y < g.top || y > g.top + HEIGHT {
        return None;
    }
    let left = g.x(f32::from(m.left_pt));
    let first = g.x(f32::from(m.left_pt) + f32::from(m.first_line));
    let upper = y < g.top + HEIGHT / 2.0;
    if upper && (x - first).abs() <= GRAB {
        return Some(Grab::First);
    }
    if !upper && (x - left).abs() <= GRAB {
        return Some(Grab::Left);
    }
    if let Some(i) = m
        .tabs
        .iter()
        .position(|t| (x - g.x(f32::from(*t))).abs() <= GRAB)
    {
        return Some(Grab::Tab(i));
    }
    let pt = g.pt(x);
    if pt >= 0.0 && pt <= g.text_pt() && m.tabs.len() < MAX_TABS {
        m.tabs.push(pt.round() as u16);
        return Some(Grab::Tab(m.tabs.len() - 1));
    }
    None
}

/// The values after the marker was dragged to window (x, y).
pub(super) fn drag(g: &Geom, d: &mut Drag, x: f32, y: f32) {
    let pt = g.pt(x).round();
    let max = g.text_pt().floor().max(0.0);
    match d.grab {
        Grab::Left => d.metrics.left_pt = pt.clamp(0.0, max) as u16,
        Grab::First => {
            let first = (pt - f32::from(d.metrics.left_pt)).clamp(0.0, 255.0);
            d.metrics.first_line = first as u8;
        }
        Grab::Tab(i) => {
            if let Some(tab) = d.metrics.tabs.get_mut(i) {
                *tab = pt.clamp(0.0, max) as u16;
            }
            d.removing = (y - (g.top + HEIGHT / 2.0)).abs() > REMOVE_DISTANCE;
        }
    }
}

/// The paragraph values a finished drag gives (a removed tab stop is gone, the stops sorted).
pub(super) fn finish(d: &Drag) -> ParaMetrics {
    let mut m = d.metrics.clone();
    if let (true, Grab::Tab(i)) = (d.removing, d.grab)
        && i < m.tabs.len()
    {
        m.tabs.remove(i);
    }
    m
}

/// Where the ruler is for the page with the caret (or, off screen, the first page that is shown).
pub(super) fn geometry(
    layout: &DocLayout,
    page_bounds: &[Bounds<Pixels>],
    caret_page: usize,
    zoom: f32,
    top: f32,
) -> Option<Geom> {
    let shown = |i: usize| page_bounds.get(i).filter(|b| b.size.width > px(0.));
    let page = if shown(caret_page).is_some() {
        caret_page
    } else {
        (0..page_bounds.len()).find(|i| shown(*i).is_some())?
    };
    let b = shown(page)?;
    let setup = layout.setup_of(page);
    Some(Geom {
        page_left: f32::from(b.origin.x),
        top,
        margin_left: setup.margin_left,
        margin_right: setup.margin_right,
        page_width: setup.width,
        zoom,
    })
}

/// Colours of the ruler.
pub(super) struct Colors {
    pub bar: Rgba,
    pub margin: Rgba,
    pub tick: Rgba,
    pub text: Rgba,
    pub marker: Rgba,
}

/// Paints the strip: shaded margins, centimetre scale and the markers of the paragraph.
pub(super) fn paint(
    g: &Geom,
    (m, removing): (&ParaMetrics, Option<usize>),
    colors: &Colors,
    left: f32,
    right: f32,
    meas: &GpuiMeasurer,
    (window, cx): (&mut Window, &mut App),
) {
    let bar = |x0: f32, x1: f32, color: Rgba, window: &mut Window| {
        let (a, b) = (x0.max(left), x1.min(right));
        if b > a {
            window.paint_quad(fill(
                Bounds::new(point(px(a), px(g.top)), size(px(b - a), px(HEIGHT))),
                color,
            ));
        }
    };
    bar(left, right, colors.bar, window);
    let (text_left, text_right) = (g.x(0.0), g.x(g.text_pt()));
    bar(g.page_left, text_left, colors.margin, window);
    bar(
        text_right,
        g.page_left + g.page_width * g.zoom,
        colors.margin,
        window,
    );
    // Centimetre scale from the left edge of the text, half centimetres in between.
    let style = wv_core::CharStyle {
        size_pt: Some(7),
        ..Default::default()
    };
    let step = CM_PX * g.zoom / 2.0;
    let from = ((left - text_left) / step).ceil() as i32;
    let to = ((right - text_left) / step).floor() as i32;
    for i in from..=to {
        let x = text_left + step * i as f32;
        if x < g.page_left || x > g.page_left + g.page_width * g.zoom {
            continue;
        }
        let major = i % 2 == 0;
        let (len, y) = if major { (7.0, 15.0) } else { (4.0, 16.5) };
        window.paint_quad(fill(
            Bounds::new(
                point(px(x), px(g.top + y - len + 4.0)),
                size(px(1.), px(len)),
            ),
            colors.tick,
        ));
        if major && i > 0 {
            let label = (i / 2).to_string();
            let shaped = meas.shape_zoomed(&label, &style, colors.text.into(), 1.0);
            let _ = shaped.paint(
                point(px(x + 2.0), px(g.top + 2.0)),
                px(meas.ascent(&style) + meas.descent(&style)),
                TextAlign::Left,
                None,
                window,
                cx,
            );
        }
    }
    // Tab stops: a small "L".
    for (i, tab) in m.tabs.iter().enumerate() {
        if removing == Some(i) {
            continue;
        }
        let x = g.x(f32::from(*tab));
        window.paint_quad(fill(
            Bounds::new(point(px(x - 1.0), px(g.top + 6.0)), size(px(2.), px(10.))),
            colors.marker,
        ));
        window.paint_quad(fill(
            Bounds::new(point(px(x - 1.0), px(g.top + 14.0)), size(px(6.), px(2.))),
            colors.marker,
        ));
    }
    // Indent markers: the first line's points down, the paragraph's points up with a base.
    let first_x = g.x(f32::from(m.left_pt) + f32::from(m.first_line));
    let left_x = g.x(f32::from(m.left_pt));
    triangle(window, first_x, g.top + 1.0, true, colors.marker);
    triangle(window, left_x, g.top + 11.0, false, colors.marker);
    window.paint_quad(fill(
        Bounds::new(
            point(px(left_x - 5.0), px(g.top + 17.0)),
            size(px(10.), px(3.)),
        ),
        colors.marker,
    ));
}

/// A small triangle of stacked bars, pointing down (`down`) or up; `top` is its upper edge.
fn triangle(window: &mut Window, x: f32, top: f32, down: bool, color: Rgba) {
    for row in 0..5 {
        let width = if down {
            10.0 - 2.0 * row as f32
        } else {
            2.0 + 2.0 * row as f32
        };
        window.paint_quad(fill(
            Bounds::new(
                point(px(x - width / 2.0), px(top + row as f32 * 1.5)),
                size(px(width), px(1.5)),
            ),
            color,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{Drag, Geom, Grab, drag, finish, press};
    use wv_core::ParaMetrics;

    fn geom() -> Geom {
        Geom {
            page_left: 100.0,
            top: 50.0,
            margin_left: 96.0,
            margin_right: 96.0,
            page_width: 800.0,
            zoom: 1.0,
        }
    }

    fn metrics() -> ParaMetrics {
        ParaMetrics {
            left_pt: 36,
            first_line: 18,
            space_before: None,
            space_after: None,
            line_spacing: None,
            tabs: vec![200],
        }
    }

    #[test]
    fn points_and_pixels_convert_both_ways() {
        let g = geom();
        assert_eq!(g.x(0.0), 196.0);
        assert!((g.pt(g.x(72.0)) - 72.0).abs() < 1e-3);
        assert!((g.text_pt() - 456.0).abs() < 1e-3);
    }

    #[test]
    fn the_markers_and_the_strip_are_pressed() {
        let g = geom();
        let mut m = metrics();
        let (first, left) = (g.x(54.0), g.x(36.0));
        assert_eq!(press(&g, &mut m, first, 52.0), Some(Grab::First));
        assert_eq!(press(&g, &mut m, left, 70.0), Some(Grab::Left));
        assert_eq!(press(&g, &mut m, g.x(200.0), 60.0), Some(Grab::Tab(0)));
        // The empty strip makes a tab stop; outside the text or the strip nothing happens.
        assert_eq!(press(&g, &mut m, g.x(300.0), 60.0), Some(Grab::Tab(1)));
        assert_eq!(m.tabs, [200, 300]);
        assert_eq!(press(&g, &mut m, g.x(300.0), 10.0), None);
        assert_eq!(press(&g, &mut m, g.x(-20.0), 60.0), None);
    }

    #[test]
    fn dragging_changes_indents_and_tabs() {
        let g = geom();
        let mut d = Drag {
            grab: Grab::Left,
            metrics: metrics(),
            removing: false,
        };
        drag(&g, &mut d, g.x(72.0), 60.0);
        assert_eq!((d.metrics.left_pt, d.metrics.first_line), (72, 18));
        d.grab = Grab::First;
        drag(&g, &mut d, g.x(90.0), 52.0);
        assert_eq!(d.metrics.first_line, 18);
        drag(&g, &mut d, g.x(10.0), 52.0);
        assert_eq!(d.metrics.first_line, 0, "no hanging indent");
        d.grab = Grab::Tab(0);
        drag(&g, &mut d, g.x(250.0), 60.0);
        assert_eq!(d.metrics.tabs, [250]);
        assert!(!d.removing);
        drag(&g, &mut d, g.x(250.0), 200.0);
        assert!(d.removing);
        assert!(finish(&d).tabs.is_empty());
    }
}
