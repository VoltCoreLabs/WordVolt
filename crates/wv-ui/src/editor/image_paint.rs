use gpui_kit::*;

use std::rc::Rc;
use std::sync::Arc;

use wv_core::ImageFormat;

use super::page_paint::PageView;
use super::view::EditorView;

impl EditorView {
    /// GPUI images for the document's pictures (kept while the stored bytes stay the same).
    pub(super) fn picture_sources(&mut self) -> Rc<Vec<Arc<Image>>> {
        let images = &self.core.doc.images;
        let same = self.picture_cache.0.len() == images.len()
            && self
                .picture_cache
                .0
                .iter()
                .zip(images)
                .all(|(a, b)| Arc::ptr_eq(a, &b.bytes));
        if !same {
            let sources = images
                .iter()
                .map(|i| {
                    let format = match i.format {
                        ImageFormat::Png => gpui_kit::ImageFormat::Png,
                        ImageFormat::Jpeg => gpui_kit::ImageFormat::Jpeg,
                        ImageFormat::Gif => gpui_kit::ImageFormat::Gif,
                        ImageFormat::Webp => gpui_kit::ImageFormat::Webp,
                    };
                    Arc::new(Image::from_bytes(format, i.bytes.to_vec()))
                })
                .collect();
            let keys = images.iter().map(|i| i.bytes.clone()).collect();
            self.picture_cache = (keys, Rc::new(sources));
        }
        self.picture_cache.1.clone()
    }
}

/// Paints the pictures of the picture paragraphs on this page.
pub fn paint_images(
    v: &PageView,
    at: &dyn Fn(f32, f32) -> Point<Pixels>,
    visible: &dyn Fn(f32, f32) -> bool,
    window: &mut Window,
    cx: &mut App,
) {
    let setup = &v.layout.setup;
    for slice in &v.layout.pages[v.page].slices {
        let para = &v.layout.paragraphs[slice.para];
        let (Some(image), true) = (para.image, slice.lines.start == 0) else {
            continue;
        };
        let top = setup.margin_top + slice.y;
        if !visible(top, image.height) {
            continue;
        }
        let Some(source) = v.images.get(image.id as usize) else {
            continue;
        };
        let x = setup.margin_left + slice.x + para.lines[0].x;
        let bounds = Bounds::new(
            at(x, top),
            size(px(image.width * v.zoom), px(image.height * v.zoom)),
        );
        if let Some(data) = source.clone().use_render_image(window, cx) {
            let _ = window.paint_image(bounds, bounds, Corners::default(), data, 0, false);
        }
    }
}
