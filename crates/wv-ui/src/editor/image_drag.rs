use gpui_kit::*;
use wv_core::{Crop, Float, Position, Wrap};

use super::image_handles::{CORNERS, HANDLE, HANDLES, Handle, PlacedImage};
use super::view::EditorView;

/// Pointer movement (px) that turns a click on a picture into a drag.
const DRAG_THRESHOLD: f32 = 4.0;

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Resize(Handle),
    /// Cutting the picture's side at the handle instead of scaling it.
    Crop(Handle),
    Move,
}

/// A picture drag (resize or move) in progress.
pub struct ImageDrag {
    mode: Mode,
    /// Pointer at the start (window px) and the picture then.
    pub(super) start: (f32, f32),
    picture: PlacedImage,
    /// What was cut off the picture at the start.
    pub(super) crop0: Crop,
    /// Where in the picture the pointer grabbed it (layout px).
    grab: (f32, f32),
    /// Width change per pixel dragged for an inline picture (a centred one grows on both sides).
    gain: f32,
    /// The drag has moved far enough to count.
    active: bool,
    /// No step recorded yet (the first change starts the undo step).
    pub(super) first: bool,
    /// Page the picture is anchored to now.
    page: usize,
}

impl EditorView {
    /// Starts a drag when the point is on a handle of the selected picture, or on any picture
    /// (which becomes the selected one and is moved).
    pub(super) fn start_image_drag(&mut self, at: Point<Pixels>, cx: &mut Context<Self>) -> bool {
        let (px_x, px_y) = (f32::from(at.x), f32::from(at.y));
        if let Some(sel) = self.selected_image()
            && let Some((x, y, w, h)) = self.image_window_rect(&sel)
        {
            let handles: &[Handle] = if sel.float.is_some() || self.cropping {
                &HANDLES
            } else {
                &CORNERS
            };
            let hit = handles.iter().copied().find(|c| {
                let (hx, hy) = c.of(x, y, w, h);
                (px_x - hx).abs() <= HANDLE && (px_y - hy).abs() <= HANDLE
            });
            if let Some(handle) = hit {
                let mode = if self.cropping {
                    Mode::Crop(handle)
                } else {
                    Mode::Resize(handle)
                };
                self.begin_drag(mode, sel, (px_x, px_y), true);
                return true;
            }
        }
        let Some(picture) = self.image_at(at) else {
            return false;
        };
        if self.core.sel.head.para != picture.para {
            self.apply(cx, |core| {
                core.set_head(Position::new(picture.para, 0), false)
            });
            self.reveal_caret.set(false);
        }
        self.begin_drag(Mode::Move, picture, (px_x, px_y), false);
        true
    }

    fn begin_drag(&mut self, mode: Mode, picture: PlacedImage, start: (f32, f32), active: bool) {
        let z = self.zoom;
        let rect = self.image_window_rect(&picture).unwrap_or_default();
        let crop0 = self
            .core
            .image_at_caret()
            .map(|i| i.crop)
            .unwrap_or_default();
        self.resizing = Some(ImageDrag {
            mode,
            crop0,
            start,
            picture,
            grab: ((start.0 - rect.0) / z, (start.1 - rect.1) / z),
            gain: if self.core.align() == wv_core::Align::Center {
                2.0
            } else {
                1.0
            },
            active,
            first: true,
            page: picture.page,
        });
    }

    /// Continues a picture drag; `false` when none is in progress.
    pub(super) fn drag_image(&mut self, at: Point<Pixels>, cx: &mut Context<Self>) -> bool {
        let Some(d) = self.resizing.as_mut() else {
            return false;
        };
        let pointer = (f32::from(at.x), f32::from(at.y));
        if !d.active {
            let moved = (pointer.0 - d.start.0).hypot(pointer.1 - d.start.1);
            if moved < DRAG_THRESHOLD {
                return true;
            }
            d.active = true;
        }
        let (mode, picture, first) = (d.mode, d.picture, d.first);
        match mode {
            Mode::Resize(handle) => self.resize_to(handle, picture, pointer, first, cx),
            Mode::Crop(handle) => self.crop_to(handle, picture, pointer, first, cx),
            Mode::Move => self.move_to(pointer, cx),
        }
        self.reveal_caret.set(false);
        true
    }

    fn resize_to(
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
        let gain = d.gain;
        let Some(float) = pic.float else {
            // In the line of text only the width changes (the height follows the proportions).
            let dx = dx * gain;
            let width = pic.width + if handle.right() { dx } else { -dx };
            self.apply(cx, |c| c.drag_image_width(width, first));
            return;
        };
        let aspect = pic.width / pic.height.max(1.0);
        let (mut x, mut y, mut w, mut h) = (pic.x, pic.y, pic.width, pic.height);
        if handle.is_corner() {
            // Corners keep the proportions; the opposite corner stays where it is.
            let grow = if handle.right() { dx } else { -dx };
            w = (pic.width + grow).max(16.0);
            h = w / aspect;
            if handle.left() {
                x = pic.x + pic.width - w;
            }
            if handle.top() {
                y = pic.y + pic.height - h;
            }
        } else {
            if handle.right() {
                w = (pic.width + dx).max(16.0);
            }
            if handle.left() {
                w = (pic.width - dx).max(16.0);
                x = pic.x + pic.width - w;
            }
            if handle.bottom() {
                h = (pic.height + dy).max(16.0);
            }
            if handle.top() {
                h = (pic.height - dy).max(16.0);
                y = pic.y + pic.height - h;
            }
        }
        let moved = Float { x, y, ..float };
        self.apply(cx, |c| c.drag_image_geometry(w, h, Some(moved), first));
    }

    /// Moves the picture so the grabbed point follows the pointer; crossing to another page
    /// anchors the picture to that page.
    fn move_to(&mut self, pointer: (f32, f32), cx: &mut Context<Self>) {
        let Some(layout) = self.layout.clone() else {
            return;
        };
        let Some(d) = self.resizing.as_mut() else {
            return;
        };
        let z = self.zoom;
        let target = {
            let bounds = self.page_bounds.borrow();
            let on = bounds
                .iter()
                .enumerate()
                .find(|(_, b)| {
                    b.size.height > px(0.) && b.contains(&point(px(pointer.0), px(pointer.1)))
                })
                .map(|(i, _)| i)
                .unwrap_or(d.page);
            (on, bounds.get(on).copied())
        };
        let (page, Some(b)) = target else {
            return;
        };
        let s = &layout.setup;
        let pic = d.picture;
        let x = (pointer.0 - f32::from(b.origin.x)) / z - s.margin_left - d.grab.0;
        let y = (pointer.1 - f32::from(b.origin.y)) / z - s.margin_top - d.grab.1;
        // Anywhere on the sheet.
        let x = x.clamp(-s.margin_left, s.width - s.margin_left - pic.width);
        let y = y.clamp(-s.margin_top, s.height - s.margin_top - pic.height);
        let wrap = pic.float.map_or(Wrap::TopBottom, |f| f.wrap);
        let float = Float { x, y, wrap };
        let first = d.first;
        d.first = false;
        if page != d.page {
            d.page = page;
            d.first = true;
            // The first paragraph on the page: the picture goes in front of it.
            let current = self.core.sel.head.para;
            let before = layout.pages[page]
                .slices
                .iter()
                .map(|s| s.para)
                .find(|p| *p != current)
                .unwrap_or(0);
            self.apply(cx, |c| _ = c.move_image_to(before, Some(float)));
            return;
        }
        self.apply(cx, |c| {
            let Some(image) = c.image_at_caret() else {
                return;
            };
            c.drag_image_geometry(image.width, image.height, Some(float), first)
        });
    }

    /// Ends a picture drag.
    pub(super) fn end_image_drag(&mut self) {
        self.resizing = None;
    }
}
