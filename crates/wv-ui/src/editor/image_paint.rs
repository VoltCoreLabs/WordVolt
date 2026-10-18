use gpui_kit::*;

use std::rc::Rc;
use std::sync::Arc;

use wv_core::ImageFormat;

use super::image_handles::{CORNERS, HANDLE, HANDLES, Handle};
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

/// Paints one picture at `bounds`.
fn paint_picture(v: &PageView, id: u32, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    let Some(source) = v.images.get(id as usize) else {
        return;
    };
    if let Some(data) = source.clone().use_render_image(window, cx) {
        let _ = window.paint_image(bounds, bounds, Corners::default(), data, 0, false);
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
        paint_picture(v, image.id, bounds, window, cx);
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
                paint_picture(v, f.id, bounds, window, cx);
            }
            continue;
        }
        if !is_behind {
            paint_picture(v, f.id, bounds, window, cx);
        }
        // The handles go over everything, also over the text in front of a "behind" picture.
        if v.head.para == f.para {
            paint_handles(v, bounds, &HANDLES, window);
        }
    }
}

/// Outline and handles of the selected picture.
fn paint_handles(v: &PageView, bounds: Bounds<Pixels>, handles: &[Handle], window: &mut Window) {
    let accent = v.link_color;
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
