use std::sync::Arc;

use super::Editor;
use crate::document::{ImageData, ImageFormat, Paragraph};
use crate::position::{Position, Selection};
use crate::style::ImageRef;

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
        picture.style.image = Some(ImageRef {
            id: id as u32,
            width: width * scale,
            height: height * scale,
        });
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
