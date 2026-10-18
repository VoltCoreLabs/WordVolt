use gpui_kit::*;
use wv_core::Float;

use super::image_handles::{Handle, PlacedImage};
use super::view::EditorView;

/// Least fraction of a picture that stays after cutting its sides.
const LEAST_VISIBLE: f32 = 0.05;

impl EditorView {
    /// Cuts the side(s) at `handle` of the picture to where the pointer is: the part that stays
    /// visible keeps its place and scale, only the edge moves.
    pub(super) fn crop_to(
        &mut self,
        handle: Handle,
        pic: PlacedImage,
        pointer: (f32, f32),
        first: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(d) = self.resizing.as_mut() else {
            return;
        };
        let z = self.zoom;
        let (dx, dy) = ((pointer.0 - d.start.0) / z, (pointer.1 - d.start.1) / z);
        d.first = false;
        let before = d.crop0;
        let (vw, vh) = before.visible();
        // Size of the whole picture at this scale.
        let (fw, fh) = (pic.width / vw, pic.height / vh);
        let mut crop = before;
        if handle.left() {
            crop.left = (before.left + dx / fw).clamp(0.0, 1.0 - before.right - LEAST_VISIBLE);
        }
        if handle.right() {
            crop.right = (before.right - dx / fw).clamp(0.0, 1.0 - crop.left - LEAST_VISIBLE);
        }
        if handle.top() {
            crop.top = (before.top + dy / fh).clamp(0.0, 1.0 - before.bottom - LEAST_VISIBLE);
        }
        if handle.bottom() {
            crop.bottom = (before.bottom - dy / fh).clamp(0.0, 1.0 - crop.top - LEAST_VISIBLE);
        }
        let (vw, vh) = crop.visible();
        let (w, h) = (fw * vw, fh * vh);
        // The visible part stays where it was: the corner moves by what was cut on the left/top.
        let x = pic.x + (crop.left - before.left) * fw;
        let y = pic.y + (crop.top - before.top) * fh;
        let float = pic.float.map(|f| Float { x, y, ..f });
        self.apply(cx, |c| c.drag_image_crop(crop, w, h, float, first));
    }
}
