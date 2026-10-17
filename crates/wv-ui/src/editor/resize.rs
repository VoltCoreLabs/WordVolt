use gpui_kit::*;

use super::view::EditorView;

/// Side of a resize handle on screen (px).
pub const HANDLE: f32 = 9.0;

/// Corner of a picture being dragged.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub const CORNERS: [Corner; 4] = [
    Corner::TopLeft,
    Corner::TopRight,
    Corner::BottomLeft,
    Corner::BottomRight,
];

impl Corner {
    /// The corner's point on a rectangle.
    pub fn of(self, x: f32, y: f32, width: f32, height: f32) -> (f32, f32) {
        match self {
            Self::TopLeft => (x, y),
            Self::TopRight => (x + width, y),
            Self::BottomLeft => (x, y + height),
            Self::BottomRight => (x + width, y + height),
        }
    }

    fn grows_right(self) -> bool {
        matches!(self, Self::TopRight | Self::BottomRight)
    }
}

/// A picture resize drag in progress.
pub struct Resize {
    corner: Corner,
    start_x: f32,
    start_width: f32,
    /// Width change per pixel dragged (a centred picture grows on both sides).
    gain: f32,
    /// No step recorded yet (the first change starts the undo step).
    first: bool,
}

impl EditorView {
    /// Where the picture under the caret is on screen (window coordinates).
    pub(super) fn image_screen_rect(&self) -> Option<Bounds<Pixels>> {
        let layout = self.layout.as_ref()?;
        let para = self.core.sel.head.para;
        let image = layout.paragraphs.get(para)?.image?;
        let (page, slice) = layout.pages.iter().enumerate().find_map(|(i, p)| {
            p.slices
                .iter()
                .find(|s| s.para == para && s.lines.start == 0)
                .map(|s| (i, s))
        })?;
        let origin = self.page_bounds.borrow().get(page)?.origin;
        let (x, y) = (
            layout.setup.margin_left + slice.x + layout.paragraphs[para].lines[0].x,
            layout.setup.margin_top + slice.y,
        );
        let z = self.zoom;
        Some(Bounds::new(
            point(origin.x + px(x * z), origin.y + px(y * z)),
            size(px(image.width * z), px(image.height * z)),
        ))
    }

    /// Starts a resize when the point is on a handle of the selected picture.
    pub(super) fn start_resize(&mut self, at: Point<Pixels>) -> bool {
        let (Some(rect), Some(image)) = (self.image_screen_rect(), self.core.image_at_caret())
        else {
            return false;
        };
        let (px_x, px_y) = (f32::from(at.x), f32::from(at.y));
        let hit = CORNERS.into_iter().find(|c| {
            let (x, y) = c.of(
                f32::from(rect.origin.x),
                f32::from(rect.origin.y),
                f32::from(rect.size.width),
                f32::from(rect.size.height),
            );
            (px_x - x).abs() <= HANDLE && (px_y - y).abs() <= HANDLE
        });
        let Some(corner) = hit else { return false };
        self.resizing = Some(Resize {
            corner,
            start_x: px_x,
            start_width: image.width,
            gain: if self.core.align() == wv_core::Align::Center {
                2.0
            } else {
                1.0
            },
            first: true,
        });
        true
    }

    /// Continues a resize drag; `false` when none is in progress.
    pub(super) fn drag_resize(&mut self, at: Point<Pixels>, cx: &mut Context<Self>) -> bool {
        let zoom = self.zoom;
        let Some(r) = self.resizing.as_mut() else {
            return false;
        };
        let dx = (f32::from(at.x) - r.start_x) / zoom * r.gain;
        let width = r.start_width + if r.corner.grows_right() { dx } else { -dx };
        let first = std::mem::take(&mut r.first);
        self.apply(cx, |core| core.drag_image_width(width, first));
        self.reveal_caret.set(false);
        true
    }
}
