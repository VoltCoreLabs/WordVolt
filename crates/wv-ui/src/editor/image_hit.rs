use gpui_kit::*;
use wv_core::Float;

use super::image_handles::PlacedImage;
use super::view::EditorView;

impl EditorView {
    /// Every picture on the rendered pages, topmost last.
    pub(super) fn placed_images(&self) -> Vec<PlacedImage> {
        let Some(layout) = self.layout.as_ref() else {
            return Vec::new();
        };
        let bounds = self.page_bounds.borrow();
        let mut out = Vec::new();
        for (page, p) in layout.pages.iter().enumerate() {
            if bounds.get(page).is_none_or(|b| b.size.height <= px(0.)) {
                continue;
            }
            for slice in p.slices.iter().filter(|s| s.lines.start == 0) {
                let para = &layout.paragraphs[slice.para];
                let Some(image) = para.image.filter(|i| i.float.is_none()) else {
                    continue;
                };
                out.push(PlacedImage {
                    page,
                    para: slice.para,
                    x: slice.x + para.lines[0].x,
                    y: slice.y,
                    width: image.width,
                    height: image.height,
                    float: None,
                });
            }
            for f in &p.floats {
                out.push(PlacedImage {
                    page,
                    para: f.para,
                    x: f.x,
                    y: f.y,
                    width: f.width,
                    height: f.height,
                    float: Some(Float {
                        x: f.x,
                        y: f.y,
                        wrap: f.wrap,
                    }),
                });
            }
        }
        out
    }

    /// The selected picture (the one in the caret's paragraph).
    pub(super) fn selected_image(&self) -> Option<PlacedImage> {
        let para = self.core.sel.head.para;
        self.core.image_at_caret()?;
        self.placed_images().into_iter().find(|i| i.para == para)
    }

    /// Window rectangle (x, y, w, h) of a picture.
    pub(super) fn image_window_rect(&self, image: &PlacedImage) -> Option<(f32, f32, f32, f32)> {
        let layout = self.layout.as_ref()?;
        let b = *self.page_bounds.borrow().get(image.page)?;
        let z = self.zoom;
        Some((
            f32::from(b.origin.x) + (layout.setup_of(image.page).margin_left + image.x) * z,
            f32::from(b.origin.y) + (layout.setup_of(image.page).margin_top + image.y) * z,
            image.width * z,
            image.height * z,
        ))
    }

    /// The picture under a window point (the topmost one).
    pub(super) fn image_at(&self, at: Point<Pixels>) -> Option<PlacedImage> {
        let (px_x, px_y) = (f32::from(at.x), f32::from(at.y));
        self.placed_images().into_iter().rev().find(|i| {
            self.image_window_rect(i).is_some_and(|(x, y, w, h)| {
                px_x >= x && px_x <= x + w && px_y >= y && px_y <= y + h
            })
        })
    }
}
