use gpui_kit::*;
use wv_core::{MAX_TABS, ParaMetrics, TabKind, TabStop};
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
    /// The indent from the right edge.
    Right,
    Tab(usize),
}

/// A marker being dragged and the paragraph values it gives so far.
pub(super) struct Drag {
    pub grab: Grab,
    pub metrics: ParaMetrics,
    /// A tab stop pulled off the strip: it is removed when let go.
    pub removing: bool,
    /// The marker was pressed but has not moved yet; letting go then changes the kind of a tab
    /// stop that was there before.
    pub still: bool,
    /// The tab stop was made by this press.
    pub made: bool,
}

/// The marker under the pointer, or a new tab stop of `kind` where the strip itself is pressed
/// (`metrics` then has the new stop; the second value tells that it was made).
pub(super) fn press(
    g: &Geom,
    m: &mut ParaMetrics,
    (x, y): (f32, f32),
    kind: TabKind,
) -> Option<(Grab, bool)> {
    if y < g.top || y > g.top + HEIGHT {
        return None;
    }
    let left = g.x(f32::from(m.left_pt));
    let first = g.x(f32::from(m.left_pt) + f32::from(m.first_line));
    let right = g.x(g.text_pt() - f32::from(m.right_pt));
    let upper = y < g.top + HEIGHT / 2.0;
    if upper && (x - first).abs() <= GRAB {
        return Some((Grab::First, false));
    }
    if !upper && (x - left).abs() <= GRAB {
        return Some((Grab::Left, false));
    }
    if !upper && (x - right).abs() <= GRAB {
        return Some((Grab::Right, false));
    }
    if let Some(i) = m
        .tabs
        .iter()
        .position(|t| (x - g.x(f32::from(t.pos))).abs() <= GRAB)
    {
        return Some((Grab::Tab(i), false));
    }
    let pt = g.pt(x);
    if pt >= 0.0 && pt <= g.text_pt() && m.tabs.len() < MAX_TABS {
        m.tabs.push(TabStop {
            pos: pt.round() as u16,
            kind,
            leader: wv_core::TabLeader::None,
        });
        return Some((Grab::Tab(m.tabs.len() - 1), true));
    }
    None
}

/// The values after the marker was dragged to window (x, y).
pub(super) fn drag(g: &Geom, d: &mut Drag, x: f32, y: f32) {
    let pt = g.pt(x).round();
    let max = g.text_pt().floor().max(0.0);
    let before = d.metrics.clone();
    match d.grab {
        Grab::Left => d.metrics.left_pt = pt.clamp(0.0, max) as u16,
        Grab::First => {
            let first = (pt - f32::from(d.metrics.left_pt)).clamp(0.0, 255.0);
            d.metrics.first_line = first as u8;
        }
        Grab::Right => d.metrics.right_pt = (max - pt).clamp(0.0, 255.0) as u8,
        Grab::Tab(i) => {
            if let Some(tab) = d.metrics.tabs.get_mut(i) {
                tab.pos = pt.clamp(0.0, max) as u16;
            }
            d.removing = (y - (g.top + HEIGHT / 2.0)).abs() > REMOVE_DISTANCE;
        }
    }
    if d.metrics != before || d.removing {
        d.still = false;
    }
}

/// The paragraph values a finished drag gives (a removed tab stop is gone; a stop that was only
/// clicked has the next kind) and the kind that was set, if any.
pub(super) fn finish(d: &Drag) -> (ParaMetrics, Option<TabKind>) {
    let mut m = d.metrics.clone();
    let mut set = None;
    if let Grab::Tab(i) = d.grab
        && i < m.tabs.len()
    {
        if d.removing {
            m.tabs.remove(i);
        } else if d.still && !d.made {
            m.tabs[i].kind = m.tabs[i].kind.next();
            set = Some(m.tabs[i].kind);
        }
    }
    (m, set)
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
    // Tab stops: an L, a mirrored L, an upside-down T, and that with a dot, by kind.
    for (i, tab) in m.tabs.iter().enumerate() {
        if removing == Some(i) {
            continue;
        }
        let x = g.x(f32::from(tab.pos));
        let bar = |window: &mut Window, dx: f32, y: f32, w: f32, h: f32| {
            window.paint_quad(fill(
                Bounds::new(point(px(x + dx), px(g.top + y)), size(px(w), px(h))),
                colors.marker,
            ));
        };
        match tab.kind {
            TabKind::Left => {
                bar(window, -1.0, 6.0, 2.0, 10.0);
                bar(window, -1.0, 14.0, 6.0, 2.0);
            }
            TabKind::Right => {
                bar(window, -1.0, 6.0, 2.0, 10.0);
                bar(window, -5.0, 14.0, 6.0, 2.0);
            }
            TabKind::Center | TabKind::Decimal => {
                bar(window, -1.0, 6.0, 2.0, 10.0);
                bar(window, -4.0, 14.0, 9.0, 2.0);
                if tab.kind == TabKind::Decimal {
                    bar(window, 3.0, 10.0, 2.0, 2.0);
                }
            }
        }
    }
    // Indent markers: the first line's points down, the paragraph's points up with a base.
    let first_x = g.x(f32::from(m.left_pt) + f32::from(m.first_line));
    let left_x = g.x(f32::from(m.left_pt));
    triangle(window, first_x, g.top + 1.0, true, colors.marker);
    triangle(window, left_x, g.top + 11.0, false, colors.marker);
    triangle(
        window,
        g.x(g.text_pt() - f32::from(m.right_pt)),
        g.top + 11.0,
        false,
        colors.marker,
    );
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
    use wv_core::{ParaMetrics, TabKind, TabStop};

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
            tabs: vec![TabStop::left(200)],
            right_pt: 10,
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
        let kind = TabKind::Left;
        assert_eq!(
            press(&g, &mut m, (first, 52.0), kind),
            Some((Grab::First, false))
        );
        assert_eq!(
            press(&g, &mut m, (left, 70.0), kind),
            Some((Grab::Left, false))
        );
        let right = g.x(g.text_pt() - 10.0);
        assert_eq!(
            press(&g, &mut m, (right, 70.0), kind),
            Some((Grab::Right, false))
        );
        assert_eq!(
            press(&g, &mut m, (g.x(200.0), 60.0), kind),
            Some((Grab::Tab(0), false))
        );
        // The empty strip makes a tab stop of the chosen kind; outside the text or the strip
        // nothing happens.
        assert_eq!(
            press(&g, &mut m, (g.x(300.0), 60.0), TabKind::Right),
            Some((Grab::Tab(1), true))
        );
        assert_eq!(
            m.tabs[1],
            TabStop {
                pos: 300,
                kind: TabKind::Right,
                leader: wv_core::TabLeader::None,
            }
        );
        assert_eq!(press(&g, &mut m, (g.x(300.0), 10.0), kind), None);
        assert_eq!(press(&g, &mut m, (g.x(-20.0), 60.0), kind), None);
    }

    #[test]
    fn dragging_changes_indents_and_tabs() {
        let g = geom();
        let mut d = Drag {
            grab: Grab::Left,
            metrics: metrics(),
            removing: false,
            still: true,
            made: false,
        };
        drag(&g, &mut d, g.x(72.0), 60.0);
        assert_eq!((d.metrics.left_pt, d.metrics.first_line), (72, 18));
        d.grab = Grab::First;
        drag(&g, &mut d, g.x(90.0), 52.0);
        assert_eq!(d.metrics.first_line, 18);
        drag(&g, &mut d, g.x(10.0), 52.0);
        assert_eq!(d.metrics.first_line, 0, "no hanging indent");
        d.grab = Grab::Right;
        drag(&g, &mut d, g.x(g.text_pt() - 40.0), 60.0);
        assert_eq!(d.metrics.right_pt, 40);
        d.grab = Grab::Tab(0);
        drag(&g, &mut d, g.x(250.0), 60.0);
        assert_eq!(d.metrics.tabs, [TabStop::left(250)]);
        assert!(!d.removing);
        drag(&g, &mut d, g.x(250.0), 200.0);
        assert!(d.removing);
        assert!(finish(&d).0.tabs.is_empty());
    }

    #[test]
    fn clicking_a_tab_stop_cycles_its_kind_but_dragging_does_not() {
        let g = geom();
        let mut d = Drag {
            grab: Grab::Tab(0),
            metrics: metrics(),
            removing: false,
            still: true,
            made: false,
        };
        let (m, set) = finish(&d);
        assert_eq!(set, Some(TabKind::Center));
        assert_eq!(m.tabs[0].kind, TabKind::Center);
        drag(&g, &mut d, g.x(220.0), 60.0);
        let (m, set) = finish(&d);
        assert_eq!((set, m.tabs[0]), (None, TabStop::left(220)));
        // A stop made by this press keeps the kind it was made with.
        d.still = true;
        d.made = true;
        assert_eq!(finish(&d).1, None);
    }
}
