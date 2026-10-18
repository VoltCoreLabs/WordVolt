use gpui_kit::*;
use wv_core::Float;

/// Side of a resize handle on screen (px).
pub const HANDLE: f32 = 9.0;

/// Distance (px) from the top edge of a picture to its rotation handle, and the handle's radius.
pub const ROTATE_GAP: f32 = 28.0;
pub const ROTATE_RADIUS: f32 = 6.0;

/// Where the rotation handle of a picture is (above the middle of its top edge).
pub fn rotate_handle(x: f32, y: f32, w: f32) -> (f32, f32) {
    (x + w / 2.0, y - ROTATE_GAP)
}

/// A handle on the outline of a picture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Handle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

pub const HANDLES: [Handle; 8] = [
    Handle::TopLeft,
    Handle::Top,
    Handle::TopRight,
    Handle::Right,
    Handle::BottomRight,
    Handle::Bottom,
    Handle::BottomLeft,
    Handle::Left,
];

/// The four corners (the handles of a picture in the line of text).
pub const CORNERS: [Handle; 4] = [
    Handle::TopLeft,
    Handle::TopRight,
    Handle::BottomRight,
    Handle::BottomLeft,
];

impl Handle {
    /// The handle's point on a rectangle.
    pub fn of(self, x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
        let (cx, cy) = (x + w / 2.0, y + h / 2.0);
        match self {
            Self::TopLeft => (x, y),
            Self::Top => (cx, y),
            Self::TopRight => (x + w, y),
            Self::Right => (x + w, cy),
            Self::BottomRight => (x + w, y + h),
            Self::Bottom => (cx, y + h),
            Self::BottomLeft => (x, y + h),
            Self::Left => (x, cy),
        }
    }

    pub fn cursor(self) -> CursorStyle {
        match self {
            Self::TopLeft | Self::BottomRight => CursorStyle::ResizeUpLeftDownRight,
            Self::TopRight | Self::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
            Self::Top | Self::Bottom => CursorStyle::ResizeUpDown,
            Self::Left | Self::Right => CursorStyle::ResizeLeftRight,
        }
    }

    pub(super) fn is_corner(self) -> bool {
        matches!(
            self,
            Self::TopLeft | Self::TopRight | Self::BottomLeft | Self::BottomRight
        )
    }

    pub(super) fn left(self) -> bool {
        matches!(self, Self::TopLeft | Self::Left | Self::BottomLeft)
    }

    pub(super) fn right(self) -> bool {
        matches!(self, Self::TopRight | Self::Right | Self::BottomRight)
    }

    pub(super) fn top(self) -> bool {
        matches!(self, Self::TopLeft | Self::Top | Self::TopRight)
    }

    pub(super) fn bottom(self) -> bool {
        matches!(self, Self::BottomLeft | Self::Bottom | Self::BottomRight)
    }
}

/// A picture of the document as it is on a page: the page and the rectangle in px from the
/// top-left corner of the text area.
#[derive(Clone, Copy, Debug)]
pub struct PlacedImage {
    pub page: usize,
    pub para: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub float: Option<Float>,
}
