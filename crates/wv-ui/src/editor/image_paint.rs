use gpui_kit::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use wv_core::{Crop, ImageFormat};

use super::image_handles::{
    CORNERS, HANDLE, HANDLES, Handle, ROTATE_GAP, ROTATE_RADIUS, rotate_handle,
};
use super::page_paint::PageView;
use super::view::EditorView;

/// How a picture is shown: size before turning, angle (degrees) and the cut.
#[derive(Clone, Copy)]
pub struct Look {
    pub id: u32,
    pub frame: (f32, f32),
    pub angle: f32,
    pub crop: Crop,
}

/// Key of a turned picture: picture, angle (half degrees), cut (thousandths), frame (px).
type TurnedKey = (u32, i32, [i32; 4], i32, i32);

/// The document's pictures as GPUI images; a turned picture is made when first needed.
#[derive(Default)]
pub struct PictureSet {
    keys: Vec<Arc<Vec<u8>>>,
    base: Vec<Arc<Image>>,
    turned: RefCell<HashMap<TurnedKey, Arc<Image>>>,
}

impl PictureSet {
    /// The picture as it is shown: the stored one when it is not turned (the cut is done when it
    /// is painted); a turned one is made as a raster of the turned, cut picture.
    pub fn get(&self, look: &Look) -> Option<Arc<Image>> {
        let base = self.base.get(look.id as usize)?;
        if look.angle.rem_euclid(360.0) < 0.01 {
            return Some(base.clone());
        }
        let c = look.crop;
        let key = (
            look.id,
            (look.angle * 2.0).round() as i32,
            [c.left, c.top, c.right, c.bottom].map(|v| (v * 1000.0).round() as i32),
            look.frame.0.round() as i32,
            look.frame.1.round() as i32,
        );
        let mut turned = self.turned.borrow_mut();
        if turned.len() > 24 && !turned.contains_key(&key) {
            turned.clear();
        }
        let image = turned.entry(key).or_insert_with(|| {
            wv_layout::turned_png(
                &self.keys[look.id as usize],
                look.frame,
                look.crop,
                look.angle,
                TURNED_SIDE,
            )
            .map(|png| Arc::new(Image::from_bytes(gpui_kit::ImageFormat::Png, png)))
            .unwrap_or_else(|| base.clone())
        });
        Some(image.clone())
    }
}

/// Longest side (px) of the raster of a turned picture.
const TURNED_SIDE: u32 = 800;

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

/// Paints a picture in `bounds`, the box that holds it turned and cut.
fn paint_picture(
    v: &PageView,
    look: &Look,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(source) = v.images.get(look) else {
        return;
    };
    let Some(data) = source.use_render_image(window, cx) else {
        return;
    };
    // A turned picture is a raster of exactly the box; an upright one is laid out whole so that
    // its visible part lands in the box.
    let full = if look.angle.rem_euclid(360.0) < 0.01 {
        let crop = look.crop;
        let (vw, vh) = crop.visible();
        let (full_w, full_h) = (bounds.size.width / vw, bounds.size.height / vh);
        Bounds::new(
            point(
                bounds.origin.x - full_w * crop.left,
                bounds.origin.y - full_h * crop.top,
            ),
            size(full_w, full_h),
        )
    } else {
        bounds
    };
    let _ = window.paint_image(bounds, full, Corners::default(), data, 0, false);
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
        let (box_w, box_h) = image.bounds();
        if !visible(top, box_h) {
            continue;
        }
        let x = setup.margin_left + slice.x + para.lines[0].x;
        let bounds = Bounds::new(at(x, top), size(px(box_w * v.zoom), px(box_h * v.zoom)));
        let look = Look {
            id: image.id,
            frame: (image.width, image.height),
            angle: image.angle,
            crop: image.crop,
        };
        paint_picture(v, &look, bounds, window, cx);
        if v.head.para == slice.para {
            let handles: &[Handle] = if v.cropping { &HANDLES } else { &CORNERS };
            paint_handles(v, bounds, handles, !v.cropping, window);
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
        let look = Look {
            id: f.id,
            frame: f.frame,
            angle: f.angle,
            crop: f.crop,
        };
        if behind {
            if is_behind {
                paint_picture(v, &look, bounds, window, cx);
            }
            continue;
        }
        if !is_behind {
            paint_picture(v, &look, bounds, window, cx);
        }
        // The handles go over everything, also over the text in front of a "behind" picture.
        if v.head.para == f.para {
            paint_handles(v, bounds, &HANDLES, !v.cropping, window);
        }
    }
}

/// Outline and handles of the selected picture.
fn paint_handles(
    v: &PageView,
    bounds: Bounds<Pixels>,
    handles: &[Handle],
    rotate: bool,
    window: &mut Window,
) {
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
    if rotate {
        // The handle that turns the picture, above the middle of the top edge.
        let (hx, hy) = rotate_handle(f32::from(o.x), f32::from(o.y), f32::from(s.width));
        let r = px(ROTATE_RADIUS);
        let centre = point(px(hx), px(hy));
        window.paint_quad(fill(
            Bounds::new(
                point(centre.x - line / 2.0, centre.y),
                size(line, px(ROTATE_GAP)),
            ),
            accent,
        ));
        let dot = Bounds::new(centre - point(r, r), size(r * 2.0, r * 2.0));
        if dot.dilate(px(3.0)).contains(&window.mouse_position()) {
            window.set_window_cursor_style(CursorStyle::Crosshair);
        }
        window.paint_quad(quad(
            dot,
            Corners::all(r),
            rgb(0xffffff),
            line,
            accent,
            BorderStyle::default(),
        ));
    }
}
