use gpui_kit::*;
use wv_core::Float;

use super::image_handles::{Handle, PlacedImage};
use super::view::EditorView;

/// Least size (px) a picture is scaled down to.
const LEAST: f32 = 16.0;

/// Angle step (degrees) when turning with Shift held.
const SNAP: f32 = 15.0;

impl EditorView {
    /// Scales the picture by the handle: corners keep the proportions, the edges of a floating
    /// picture stretch one side; a turned picture always scales as a whole. The opposite side of
    /// the box stays where it is.
    pub(super) fn resize_to(
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
        let (gain, frame, angle) = (d.gain, d.frame0, d.angle0);
        let turned = angle.rem_euclid(360.0) >= 0.01;
        // The box the turned picture takes up: its new size.
        let (w0, h0) = (pic.width, pic.height);
        let aspect = w0 / h0.max(1.0);
        let (mut w, mut h) = (w0, h0);
        if pic.float.is_none() {
            // In the line of text only the width changes (the height follows the proportions).
            let dx = dx * gain;
            w = (w0 + if handle.right() { dx } else { -dx }).max(LEAST);
            h = w / aspect;
        } else if handle.is_corner() {
            w = (w0 + if handle.right() { dx } else { -dx }).max(LEAST);
            h = w / aspect;
        } else {
            if handle.right() {
                w = (w0 + dx).max(LEAST);
            }
            if handle.left() {
                w = (w0 - dx).max(LEAST);
            }
            if handle.bottom() {
                h = (h0 + dy).max(LEAST);
            }
            if handle.top() {
                h = (h0 - dy).max(LEAST);
            }
        }
        let mut new_frame = (w, h);
        if turned {
            let vertical = (handle.top() || handle.bottom()) && !(handle.left() || handle.right());
            let s = if vertical { h / h0 } else { w / w0 };
            (w, h) = (w0 * s, h0 * s);
            new_frame = (frame.0 * s, frame.1 * s);
        }
        let x = if handle.left() { pic.x + w0 - w } else { pic.x };
        let y = if handle.top() { pic.y + h0 - h } else { pic.y };
        let float = pic.float.map(|f| Float { x, y, ..f });
        self.apply(cx, |c| {
            c.drag_image_geometry(new_frame.0, new_frame.1, float, first)
        });
    }

    /// Turns the picture so that the rotation handle follows the pointer around the centre of the
    /// box; with Shift in steps of 15 degrees.
    pub(super) fn rotate_to(
        &mut self,
        pic: PlacedImage,
        pointer: (f32, f32),
        shift: bool,
        first: bool,
        cx: &mut Context<Self>,
    ) {
        let Some((x, y, w, h)) = self.image_window_rect(&pic) else {
            return;
        };
        if let Some(d) = self.resizing.as_mut() {
            d.first = false;
        }
        let (cx_, cy_) = (x + w / 2.0, y + h / 2.0);
        // Straight up from the centre is 0 degrees, clockwise is positive.
        let mut angle = (pointer.1 - cy_).atan2(pointer.0 - cx_).to_degrees() + 90.0;
        angle = angle.rem_euclid(360.0);
        angle = if shift {
            (angle / SNAP).round() * SNAP
        } else {
            angle.round()
        };
        self.apply(cx, |c| c.drag_image_angle(angle, first));
    }
}
