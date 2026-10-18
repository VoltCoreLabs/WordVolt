use image::{Rgba, RgbaImage};
use wv_core::Crop;

/// Alpha above which a pixel counts as part of a picture's outline.
const SOLID: u8 = 40;

/// Height (px) of a strip of the outline text goes around.
const STRIP: f32 = 6.0;

/// The most a picture is magnified when it is turned (raster pixels per layout pixel).
const MAX_SCALE: f32 = 2.0;

/// Size of the smallest box that holds a `w` x `h` picture turned by `angle` degrees.
pub fn bounds_of(w: f32, h: f32, angle: f32) -> (f32, f32) {
    wv_core::turned_box(w, h, angle)
}

/// One horizontal strip of a picture's outline: the part between `x0` and `x1` is taken (all in
/// layout px from the top-left corner of the box that holds the turned picture).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strip {
    pub top: f32,
    pub bottom: f32,
    pub x0: f32,
    pub x1: f32,
}

/// The visible part of the picture (`crop` of its source cut off) stretched to `frame` px and turned
/// clockwise by `angle` degrees, as a raster that fits the turned frame; at most `max_side` px wide
/// or tall. Outside the frame it is transparent. `None` if the bytes are no picture.
pub fn turned_raster(
    bytes: &[u8],
    frame: (f32, f32),
    crop: Crop,
    angle: f32,
    max_side: u32,
) -> Option<RgbaImage> {
    let src = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (sw, sh) = (src.width() as f32, src.height() as f32);
    let (x0, y0) = (crop.left * sw, crop.top * sh);
    let (cw, ch) = (
        ((1.0 - crop.left - crop.right) * sw).max(1.0),
        ((1.0 - crop.top - crop.bottom) * sh).max(1.0),
    );
    let (bw, bh) = bounds_of(frame.0, frame.1, angle);
    let k = (max_side as f32 / bw.max(bh).max(1.0)).min(MAX_SCALE);
    let (ow, oh) = (
        (bw * k).ceil().max(1.0) as u32,
        (bh * k).ceil().max(1.0) as u32,
    );
    let (fw, fh) = (frame.0 * k, frame.1 * k);
    let (sin, cos) = angle.to_radians().sin_cos();
    let mut out = RgbaImage::new(ow, oh);
    for oy in 0..oh {
        for ox in 0..ow {
            let (dx, dy) = (
                ox as f32 + 0.5 - ow as f32 / 2.0,
                oy as f32 + 0.5 - oh as f32 / 2.0,
            );
            // Back to the picture's own frame (the turn undone).
            let fx = dx * cos + dy * sin + fw / 2.0;
            let fy = -dx * sin + dy * cos + fh / 2.0;
            if fx < 0.0 || fy < 0.0 || fx >= fw || fy >= fh {
                continue;
            }
            let (sx, sy) = (x0 + fx / fw * cw, y0 + fy / fh * ch);
            out.put_pixel(ox, oy, sample(&src, sx - 0.5, sy - 0.5));
        }
    }
    Some(out)
}

/// `turned_raster` encoded as PNG.
pub fn turned_png(
    bytes: &[u8],
    frame: (f32, f32),
    crop: Crop,
    angle: f32,
    max_side: u32,
) -> Option<Vec<u8>> {
    let raster = turned_raster(bytes, frame, crop, angle, max_side)?;
    let mut out = std::io::Cursor::new(Vec::new());
    raster.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

/// The outline of the turned picture as strips from top to bottom: where it is not transparent.
/// Strips with nothing in them are left out.
pub fn silhouette(bytes: &[u8], frame: (f32, f32), crop: Crop, angle: f32) -> Vec<Strip> {
    let Some(raster) = turned_raster(bytes, frame, crop, angle, 128) else {
        return Vec::new();
    };
    let (bw, bh) = bounds_of(frame.0, frame.1, angle);
    // Raster pixels per layout pixel.
    let (kx, ky) = (raster.width() as f32 / bw, raster.height() as f32 / bh);
    let mut out = Vec::new();
    let mut top = 0.0_f32;
    while top < bh {
        let bottom = (top + STRIP).min(bh);
        let (r0, r1) = (
            (top * ky) as u32,
            ((bottom * ky).ceil() as u32).min(raster.height()),
        );
        let (mut lo, mut hi) = (u32::MAX, 0);
        for y in r0..r1 {
            for x in 0..raster.width() {
                if raster.get_pixel(x, y).0[3] > SOLID {
                    lo = lo.min(x);
                    hi = hi.max(x + 1);
                }
            }
        }
        if lo < hi {
            out.push(Strip {
                top,
                bottom,
                x0: lo as f32 / kx,
                x1: hi as f32 / kx,
            });
        }
        top = bottom;
    }
    out
}

/// Bilinear sample at (`x`, `y`) in source pixels.
fn sample(img: &RgbaImage, x: f32, y: f32) -> Rgba<u8> {
    let (w, h) = (img.width() as i64, img.height() as i64);
    let (x0, y0) = (x.floor() as i64, y.floor() as i64);
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let at = |px: i64, py: i64| {
        img.get_pixel(px.clamp(0, w - 1) as u32, py.clamp(0, h - 1) as u32)
            .0
    };
    let (p00, p10, p01, p11) = (
        at(x0, y0),
        at(x0 + 1, y0),
        at(x0, y0 + 1),
        at(x0 + 1, y0 + 1),
    );
    let mut out = [0u8; 4];
    for c in 0..4 {
        let top = f32::from(p00[c]) * (1.0 - fx) + f32::from(p10[c]) * fx;
        let bottom = f32::from(p01[c]) * (1.0 - fx) + f32::from(p11[c]) * fx;
        out[c] = (top * (1.0 - fy) + bottom * fy).round() as u8;
    }
    Rgba(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fully opaque PNG of one colour.
    fn solid(w: u32, h: u32) -> Vec<u8> {
        let img = RgbaImage::from_pixel(w, h, Rgba([200, 50, 20, 255]));
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }

    #[test]
    fn a_quarter_turn_swaps_the_box() {
        let (w, h) = bounds_of(100.0, 40.0, 90.0);
        assert!((w - 40.0).abs() < 0.01 && (h - 100.0).abs() < 0.01);
        let (w, h) = bounds_of(100.0, 100.0, 45.0);
        assert!((w - 141.42).abs() < 0.1 && (h - 141.42).abs() < 0.1);
    }

    #[test]
    fn the_raster_fits_the_turned_frame_and_has_transparent_corners() {
        let r = turned_raster(&solid(40, 40), (100.0, 100.0), Crop::default(), 45.0, 128).unwrap();
        // The box (141 px) is scaled down to the 128 px asked for: corners are outside the turned
        // square, the middle inside.
        assert!((126..=130).contains(&r.width()), "{}", r.width());
        assert_eq!(r.get_pixel(0, 0).0[3], 0);
        assert_eq!(r.get_pixel(r.width() / 2, r.height() / 2).0[3], 255);
    }

    #[test]
    fn the_silhouette_of_a_turned_square_is_widest_in_the_middle() {
        let strips = silhouette(&solid(40, 40), (100.0, 100.0), Crop::default(), 45.0);
        assert!(strips.len() > 10);
        let width = |s: &Strip| s.x1 - s.x0;
        let middle = strips.iter().map(width).fold(0.0, f32::max);
        assert!(middle > 120.0);
        assert!(width(&strips[0]) < 30.0, "a point at the top");
    }

    #[test]
    fn an_upright_picture_is_a_rectangle() {
        let strips = silhouette(&solid(40, 20), (80.0, 40.0), Crop::default(), 0.0);
        assert!(
            strips
                .iter()
                .all(|s| s.x0 < 1.0 && (s.x1 - 80.0).abs() < 1.5)
        );
        assert!((strips.last().unwrap().bottom - 40.0).abs() < 0.01);
    }
}
