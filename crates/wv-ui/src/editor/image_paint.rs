use gpui_kit::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use wv_core::{Crop, ImageFormat};

use super::image_handles::{CORNERS, HANDLE, HANDLES, Handle};
use super::page_paint::PageView;
use super::view::EditorView;

/// The document's pictures as GPUI images; a turned picture is made when first needed.
#[derive(Default)]
pub struct PictureSet {
    keys: Vec<Arc<Vec<u8>>>,
    base: Vec<Arc<Image>>,
    turned: RefCell<HashMap<(u32, u8), Arc<Image>>>,
}

impl PictureSet {
    /// Picture `id` turned by `rotation` quarter turns clockwise.
    pub fn get(&self, id: u32, rotation: u8) -> Option<Arc<Image>> {
        let base = self.base.get(id as usize)?;
        if rotation.is_multiple_of(4) {
            return Some(base.clone());
        }
        let mut turned = self.turned.borrow_mut();
        let image = turned.entry((id, rotation % 4)).or_insert_with(|| {
            wv_io::rotate_image(&self.keys[id as usize], rotation)
                .map(|png| Arc::new(Image::from_bytes(gpui_kit::ImageFormat::Png, png)))
                .unwrap_or_else(|| base.clone())
        });
        Some(image.clone())
    }
}

impl EditorView {
    /// GPUI images for the document's pictures (kept while the stored bytes stay the same).
    pub(super) fn picture_sources(&mut self) -> Rc<PictureSet> {
        let images = &self.core.doc.images;
        let same = self.picture_cache.keys.len() == images.len()
            && self
                .picture_cache
                .keys
                .iter()
                .zip(images)
                .all(|(a, b)| Arc::ptr_eq(a, &b.bytes));
        if !same {
            let base = images
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
            self.picture_cache = Rc::new(PictureSet {
                keys,
                base,
                turned: RefCell::default(),
            });
        }
        self.picture_cache.clone()
    }
}

/// Paints a picture (turned, with its crop cut off) in `bounds`, the visible part of it.
fn paint_picture(
    v: &PageView,
    image: (u32, u8, Crop),
    bounds: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let (id, rotation, crop) = image;
    let Some(source) = v.images.get(id, rotation) else {
        return;
    };
    // The whole picture is laid out so that its visible part lands in `bounds`.
    let (vw, vh) = crop.visible();
    let (full_w, full_h) = (bounds.size.width / vw, bounds.size.height / vh);
    let full = Bounds::new(
        point(
            bounds.origin.x - full_w * crop.left,
            bounds.origin.y - full_h * crop.top,
        ),
        size(full_w, full_h),
    );
    if let Some(data) = source.use_render_image(window, cx) {
        let _ = window.paint_image(bounds, full, Corners::default(), data, 0, false);
    }
}

/// Paints the pictures that are in the line of text on this page.
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
        let (Some(image), true) = (
            para.image.filter(|i| i.float.is_none()),
            slice.lines.start == 0,
        ) else {
            continue;
        };
        let top = setup.margin_top + slice.y;
        if !visible(top, image.height) {
            continue;
        }
        let x = setup.margin_left + slice.x + para.lines[0].x;
        let bounds = Bounds::new(
            at(x, top),
            size(px(image.width * v.zoom), px(image.height * v.zoom)),
        );
        paint_picture(
            v,
            (image.id, image.rotation, image.crop),
            bounds,
            window,
            cx,
        );
        if v.head.para == slice.para {
            paint_handles(v, bounds, &CORNERS, window);
        }
    }
}

/// Paints the floating pictures of this page: `behind` the ones under the text (before it), else
/// the others over the text, with the outline and handles of the selected one.
pub fn paint_floats(
    v: &PageView,
    at: &dyn Fn(f32, f32) -> Point<Pixels>,
    behind: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let setup = &v.layout.setup;
    for f in &v.layout.pages[v.page].floats {
        let is_behind = f.wrap == wv_core::Wrap::Behind;
        let bounds = Bounds::new(
            at(setup.margin_left + f.x, setup.margin_top + f.y),
            size(px(f.width * v.zoom), px(f.height * v.zoom)),
        );
        if behind {
            if is_behind {
                paint_picture(v, (f.id, f.rotation, f.crop), bounds, window, cx);
            }
            continue;
        }
        if !is_behind {
            paint_picture(v, (f.id, f.rotation, f.crop), bounds, window, cx);
        }
        // The handles go over everything, also over the text in front of a "behind" picture.
        if v.head.para == f.para {
            paint_handles(v, bounds, &HANDLES, window);
        }
    }
}

/// Outline and handles of the selected picture.
fn paint_handles(v: &PageView, bounds: Bounds<Pixels>, handles: &[Handle], window: &mut Window) {
    // Cropping shows dark handles.
    let accent = if v.cropping { v.text } else { v.link_color };
    let line = px(1.5);
    let (o, s) = (bounds.origin, bounds.size);
    for edge in [
        Bounds::new(o, size(s.width, line)),
        Bounds::new(point(o.x, o.y + s.height - line), size(s.width, line)),
        Bounds::new(o, size(line, s.height)),
        Bounds::new(point(o.x + s.width - line, o.y), size(line, s.height)),
    ] {
        window.paint_quad(fill(edge, accent));
    }
    let half = px(HANDLE / 2.0);
    for handle in handles.iter().copied() {
        let (x, y) = handle.of(
            f32::from(o.x),
            f32::from(o.y),
            f32::from(s.width),
            f32::from(s.height),
        );
        let outer = Bounds::new(
            point(px(x) - half, px(y) - half),
            size(px(HANDLE), px(HANDLE)),
        );
        if outer.dilate(px(3.0)).contains(&window.mouse_position()) {
            window.set_window_cursor_style(handle.cursor());
        }
        window.paint_quad(fill(outer, accent));
        window.paint_quad(fill(outer.dilate(px(-2.0)), rgb(0xffffff)));
    }
}
