use std::sync::Arc;

use super::Editor;
use crate::document::{ImageData, ImageFormat, Paragraph};
use crate::position::{Position, Selection};
use crate::style::{Crop, Float, ImageRef};

/// Widest picture inserted at its natural size (px); larger ones are scaled down to this.
pub const MAX_IMAGE_WIDTH: f32 = 600.0;

impl Editor {
    /// Inserts a picture of natural size `width` x `height` px after the caret's paragraph (an empty
    /// paragraph is reused). Not inside tables. The caret moves to the paragraph after the picture.
    pub fn insert_image(&mut self, format: ImageFormat, bytes: Vec<u8>, width: f32, height: f32) {
        if self.table_info().is_some() || width < 1.0 || height < 1.0 {
            return;
        }
        let id = match self.doc.images.iter().position(|i| *i.bytes == bytes) {
            Some(i) => i,
            None => {
                self.doc.images.push(ImageData {
                    format,
                    bytes: Arc::new(bytes),
                });
                self.doc.images.len() - 1
            }
        };
        let scale = (MAX_IMAGE_WIDTH.min(self.max_image_width()) / width).min(1.0);
        let mut picture = Paragraph::default();
        picture.style.image = Some(ImageRef::inline(id as u32, width * scale, height * scale));
        let at = self.sel.head.para;
        let current = &self.doc.paragraphs[at];
        let replace = current.is_empty() && current.style.cell.is_none();
        let (index, removed) = if replace { (at, 1) } else { (at + 1, 0) };
        picture.style.align = current.style.align;
        let mut paras = vec![picture];
        if index + removed >= self.doc.paragraphs.len() {
            paras.push(Paragraph::default());
        }
        let caret = Position::new(index + 1, 0);
        self.edit_paragraphs(index, removed, paras, Selection::caret(caret));
    }

    /// Widest a picture can be: the text width of the page (it never leaves the sheet).
    fn max_image_width(&self) -> f32 {
        let p = &self.doc.page;
        p.width - p.margin_left - p.margin_right - p.gutter
    }

    /// The picture paragraph under the caret.
    pub fn image_at_caret(&self) -> Option<ImageRef> {
        self.doc.paragraphs[self.sel.head.para].style.image
    }

    /// Scales the caret's picture to `width` px keeping its proportions.
    pub fn set_image_width(&mut self, width: f32) {
        let Some(image) = self.image_at_caret() else {
            return;
        };
        let width = width.clamp(16.0, self.max_image_width().max(16.0));
        let height = image.height * width / image.width;
        let para = self.sel.head.para;
        self.edit_para_styles(para, para, |s| {
            s.image = Some(ImageRef {
                width,
                height,
                ..image
            })
        });
    }

    /// Like `set_image_width` for a drag: the first call (`first`) starts an undo step, the
    /// following ones change that step.
    pub fn drag_image_width(&mut self, width: f32, first: bool) {
        if first {
            return self.set_image_width(width);
        }
        let Some(image) = self.image_at_caret() else {
            return;
        };
        let width = width.clamp(16.0, self.max_image_width().max(16.0));
        let para = self.sel.head.para;
        let mut changed = self.doc.paragraphs[para].clone();
        changed.style.image = Some(ImageRef {
            width,
            height: image.height * width / image.width,
            ..image
        });
        self.amend_last_edit(para, changed);
    }

    /// Changes the caret's picture with `f` (the size stays on the page). One undo step; for a
    /// drag the first call (`first`) starts it and the following ones change that step.
    pub fn edit_image(&mut self, first: bool, f: impl FnOnce(&mut ImageRef)) {
        let Some(mut image) = self.image_at_caret() else {
            return;
        };
        f(&mut image);
        (image.width, image.height) = self.clamp_image(image.width, image.height);
        let para = self.sel.head.para;
        if first {
            self.edit_para_styles(para, para, |s| s.image = Some(image));
        } else {
            let mut changed = self.doc.paragraphs[para].clone();
            changed.style.image = Some(image);
            self.amend_last_edit(para, changed);
        }
    }

    /// Sets the size and placement of the caret's picture (one undo step): `float` `None` puts it
    /// back into the line of text.
    pub fn set_image_geometry(&mut self, width: f32, height: f32, float: Option<Float>) {
        self.drag_image_geometry(width, height, float, true);
    }

    /// `set_image_geometry` for a drag: the first call (`first`) starts an undo step, the following
    /// ones change that step.
    pub fn drag_image_geometry(
        &mut self,
        width: f32,
        height: f32,
        float: Option<Float>,
        first: bool,
    ) {
        self.edit_image(first, |i| {
            (i.width, i.height, i.float) = (width, height, float);
        });
    }

    /// Turns the caret's picture by `degrees` clockwise (negative: counter-clockwise) around its
    /// centre. One undo step.
    pub fn turn_image(&mut self, degrees: f32) {
        self.edit_image(true, |i| set_angle(i, i.angle + degrees));
    }

    /// Sets the angle of the caret's picture (a drag: the first call (`first`) starts the undo
    /// step, the following ones change it). The picture turns around its centre.
    pub fn drag_image_angle(&mut self, angle: f32, first: bool) {
        self.edit_image(first, |i| set_angle(i, angle));
    }

    /// Cuts `crop` off the picture, which shows `width` x `height` px of it afterwards (and sits at
    /// `float`). Like `drag_image_geometry` it is one undo step per drag.
    pub fn drag_image_crop(
        &mut self,
        crop: Crop,
        width: f32,
        height: f32,
        float: Option<Float>,
        first: bool,
    ) {
        self.edit_image(first, |i| {
            (i.crop, i.width, i.height, i.float) = (crop, width, height, float);
        });
    }

    /// Brings back what was cut off the picture (it grows to its full size).
    pub fn reset_image_crop(&mut self) {
        let Some(image) = self.image_at_caret() else {
            return;
        };
        let (vw, vh) = image.crop.visible();
        self.edit_image(true, |i| {
            let cut = i.crop;
            i.width /= vw;
            i.height /= vh;
            i.crop = Crop::default();
            // What comes back on the left and top pushes the picture's corner out.
            if let Some(f) = i.float.as_mut() {
                f.x -= cut.left * i.width;
                f.y -= cut.top * i.height;
            }
        });
    }

    /// Picture sizes stay between 16 px and the text area of the page.
    fn clamp_image(&self, width: f32, height: f32) -> (f32, f32) {
        let p = &self.doc.page;
        let max_h = p.height - p.margin_top - p.margin_bottom;
        (
            width.clamp(16.0, self.max_image_width().max(16.0)),
            height.clamp(16.0, max_h.max(16.0)),
        )
    }

    /// Moves the caret's picture paragraph in front of paragraph `before` (so it is on that
    /// paragraph's page) and sets its placement. Not across tables. One undo step.
    pub fn move_image_to(&mut self, before: usize, float: Option<Float>) -> bool {
        let from = self.sel.head.para;
        let Some(image) = self.image_at_caret() else {
            return false;
        };
        let before = before.min(self.doc.paragraphs.len());
        let mut picture = self.doc.paragraphs[from].clone();
        picture.style.image = Some(ImageRef { float, ..image });
        if before == from || before == from + 1 {
            self.edit_para_styles(from, from, |s| s.image = picture.style.image);
            return true;
        }
        let (lo, hi) = (from.min(before), (from + 1).max(before));
        if self.doc.paragraphs[lo..hi]
            .iter()
            .any(|p| p.style.cell.is_some())
        {
            return false;
        }
        let mut paras: Vec<Paragraph> = self.doc.paragraphs[lo..hi].to_vec();
        paras.remove(from - lo);
        let at = if before > from {
            before - 1 - lo
        } else {
            before - lo
        };
        paras.insert(at, picture);
        let caret = Position::new(lo + at, 0);
        self.edit_paragraphs(lo, hi - lo, paras, Selection::caret(caret));
        true
    }

    /// Scales the caret's picture by `factor`, keeping its proportions.
    pub fn scale_image(&mut self, factor: f32) {
        if let Some(image) = self.image_at_caret() {
            self.set_image_width(image.width * factor);
        }
    }

    /// Removes the picture from the caret's paragraph, leaving an empty paragraph.
    pub(super) fn remove_image_at_caret(&mut self) -> bool {
        let para = self.sel.head.para;
        if self.doc.paragraphs[para].style.image.is_none() {
            return false;
        }
        self.edit_para_styles(para, para, |s| s.image = None);
        true
    }
}

/// Turns `i` to `angle` degrees keeping the centre of its box where it is.
fn set_angle(i: &mut ImageRef, angle: f32) {
    let before = i.bounds();
    i.angle = angle.rem_euclid(360.0);
    let after = i.bounds();
    if let Some(f) = i.float.as_mut() {
        f.x += (before.0 - after.0) / 2.0;
        f.y += (before.1 - after.1) / 2.0;
    }
}
