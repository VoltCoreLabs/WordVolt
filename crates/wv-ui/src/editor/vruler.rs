use gpui_kit::*;
use wv_layout::{DocLayout, Measurer};

use super::measure::GpuiMeasurer;
use super::ruler::{Colors, shown_page};

/// Width of the vertical ruler strip at the left edge of the page view (px).
pub(super) const WIDTH: f32 = 22.0;
/// Pixels per centimetre.
const CM_PX: f32 = 96.0 / 2.54;
/// How far (px) a margin edge is grabbed.
const GRAB: f32 = 5.0;
/// Least height (px) a margin drag leaves for the text.
const MIN_TEXT: f32 = 48.0;

/// Where the vertical ruler is: the strip's left edge and the vertical extent of the view, and the
/// page it measures (its top on the screen, margins and the zoom).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct VGeom {
    pub left: f32,
    pub view_top: f32,
    pub view_bottom: f32,
    pub page_top: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub page_height: f32,
    pub zoom: f32,
}

impl VGeom {
    /// Window y of the top edge of the text.
    pub fn text_top(&self) -> f32 {
        self.page_top + self.margin_top * self.zoom
    }

    /// Window y of the bottom edge of the text.
    pub fn text_bottom(&self) -> f32 {
        self.page_top + (self.page_height - self.margin_bottom) * self.zoom
    }

    fn page_bottom(&self) -> f32 {
        self.page_top + self.page_height * self.zoom
    }

    fn on_strip(&self, x: f32, y: f32) -> bool {
        x >= self.left && x <= self.left + WIDTH && y >= self.view_top && y <= self.view_bottom
    }
}

/// The edge of a margin being dragged.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum VGrab {
    Top,
    Bottom,
}

/// A margin being dragged and its value (px) so far.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct VDrag {
    pub grab: VGrab,
    pub margin: f32,
}

/// Where the vertical ruler is for the page with the caret (`None` when no page is shown).
pub(super) fn geometry(
    layout: &DocLayout,
    page_bounds: &[Bounds<Pixels>],
    caret_page: usize,
    zoom: f32,
    view: (f32, f32, f32),
) -> Option<VGeom> {
    let (page, b) = shown_page(page_bounds, caret_page)?;
    let setup = layout.setup_of(page);
    Some(VGeom {
        left: view.0,
        view_top: view.1,
        view_bottom: view.2,
        page_top: f32::from(b.origin.y),
        margin_top: setup.margin_top,
        margin_bottom: setup.margin_bottom,
        page_height: setup.height,
        zoom,
    })
}

/// The margin edge under the pointer.
pub(super) fn press(g: &VGeom, (x, y): (f32, f32)) -> Option<VDrag> {
    if !g.on_strip(x, y) {
        return None;
    }
    if (y - g.text_top()).abs() <= GRAB {
        return Some(VDrag {
            grab: VGrab::Top,
            margin: g.margin_top,
        });
    }
    if (y - g.text_bottom()).abs() <= GRAB {
        return Some(VDrag {
            grab: VGrab::Bottom,
            margin: g.margin_bottom,
        });
    }
    None
}

/// The margin (px) after the edge was dragged to window `y`; the text keeps a little room.
pub(super) fn drag(g: &VGeom, d: &mut VDrag, y: f32) {
    d.margin = match d.grab {
        VGrab::Top => (y - g.page_top) / g.zoom,
        VGrab::Bottom => (g.page_bottom() - y) / g.zoom,
    }
    .round();
    let other = match d.grab {
        VGrab::Top => g.margin_bottom,
        VGrab::Bottom => g.margin_top,
    };
    d.margin = d
        .margin
        .clamp(0.0, (g.page_height - other - MIN_TEXT).max(0.0));
}

/// Paints the strip: shaded margins and the centimetre scale counted from the top of the text
/// (up into the top margin and down to the bottom of the page). A dragged margin shows its value.
pub(super) fn paint(
    g: &VGeom,
    drag: Option<VDrag>,
    colors: &Colors,
    meas: &GpuiMeasurer,
    (window, cx): (&mut Window, &mut App),
) {
    let mut g = *g;
    match drag {
        Some(VDrag {
            grab: VGrab::Top,
            margin,
        }) => g.margin_top = margin,
        Some(VDrag {
            grab: VGrab::Bottom,
            margin,
        }) => g.margin_bottom = margin,
        None => {}
    }
    let bar = |y0: f32, y1: f32, color: Rgba, window: &mut Window| {
        let (a, b) = (y0.max(g.view_top), y1.min(g.view_bottom));
        if b > a {
            window.paint_quad(fill(
                Bounds::new(point(px(g.left), px(a)), size(px(WIDTH), px(b - a))),
                color,
            ));
        }
    };
    bar(g.view_top, g.view_bottom, colors.bar, window);
    bar(g.page_top, g.text_top(), colors.margin, window);
    bar(g.text_bottom(), g.page_bottom(), colors.margin, window);
    let style = wv_core::CharStyle {
        size_pt: Some(7),
        ..Default::default()
    };
    let step = CM_PX * g.zoom / 2.0;
    let from = ((g.page_top - g.text_top()) / step).ceil() as i32;
    let to = ((g.page_bottom() - g.text_top()) / step).floor() as i32;
    for i in from..=to {
        let y = g.text_top() + step * i as f32;
        if y < g.view_top || y > g.view_bottom {
            continue;
        }
        let major = i % 2 == 0;
        let len = if major { 7.0 } else { 4.0 };
        window.paint_quad(fill(
            Bounds::new(
                point(px(g.left + WIDTH - len - 1.0), px(y)),
                size(px(len), px(1.)),
            ),
            colors.tick,
        ));
        if major && i != 0 {
            let label = (i.abs() / 2).to_string();
            let shaped = meas.shape_zoomed(&label, &style, colors.text.into(), 1.0);
            let _ = shaped.paint(
                point(px(g.left + 2.0), px(y + 1.0)),
                px(meas.ascent(&style) + meas.descent(&style)),
                TextAlign::Left,
                None,
                window,
                cx,
            );
        }
    }
    for edge in [g.text_top(), g.text_bottom()] {
        if edge >= g.view_top && edge <= g.view_bottom {
            window.paint_quad(fill(
                Bounds::new(point(px(g.left), px(edge - 1.0)), size(px(WIDTH), px(2.))),
                colors.marker,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{VDrag, VGeom, VGrab, WIDTH, drag, press};

    fn geom() -> VGeom {
        VGeom {
            left: 0.0,
            view_top: 22.0,
            view_bottom: 700.0,
            page_top: 100.0,
            margin_top: 96.0,
            margin_bottom: 96.0,
            page_height: 1056.0,
            zoom: 1.0,
        }
    }

    #[test]
    fn a_margin_edge_is_grabbed_only_on_the_strip() {
        let g = geom();
        assert_eq!(
            press(&g, (5.0, g.text_top() + 2.0)).map(|d| d.grab),
            Some(VGrab::Top)
        );
        assert_eq!(press(&g, (5.0, g.text_top() + 30.0)), None);
        assert_eq!(press(&g, (WIDTH + 10.0, g.text_top())), None);
        let bottom = geom();
        let bottom = VGeom {
            view_bottom: 2000.0,
            ..bottom
        };
        assert_eq!(
            press(&bottom, (5.0, bottom.text_bottom())).map(|d| d.grab),
            Some(VGrab::Bottom)
        );
    }

    #[test]
    fn dragging_sets_the_margin_and_keeps_room_for_text() {
        let g = geom();
        let mut d = VDrag {
            grab: VGrab::Top,
            margin: 96.0,
        };
        drag(&g, &mut d, g.page_top + 48.0);
        assert_eq!(d.margin, 48.0);
        drag(&g, &mut d, g.page_top - 30.0);
        assert_eq!(d.margin, 0.0);
        drag(&g, &mut d, g.page_top + 5000.0);
        assert_eq!(d.margin, 1056.0 - 96.0 - 48.0);
        let mut bottom = VDrag {
            grab: VGrab::Bottom,
            margin: 96.0,
        };
        drag(&g, &mut bottom, g.page_top + 1056.0 - 120.0);
        assert_eq!(bottom.margin, 120.0);
    }

    #[test]
    fn zoom_scales_the_pointer() {
        let g = VGeom {
            zoom: 2.0,
            ..geom()
        };
        let mut d = VDrag {
            grab: VGrab::Top,
            margin: 96.0,
        };
        drag(&g, &mut d, g.page_top + 100.0);
        assert_eq!(d.margin, 50.0);
    }
}
