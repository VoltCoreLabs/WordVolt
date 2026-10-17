use miniz_oxide::deflate::compress_to_vec_zlib;
use pdf_writer::{Filter, Finish, Pdf, Ref};
use wv_core::ImageData;

use super::embed::Ids;

/// Embeds the picture as an RGB image (with a soft mask when it is transparent).
/// `None` when the bytes cannot be decoded.
pub fn write_image(pdf: &mut Pdf, ids: &mut Ids, data: &ImageData) -> Option<Ref> {
    let rgba = image::load_from_memory(&data.bytes).ok()?.into_rgba8();
    let (width, height) = rgba.dimensions();
    let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
    let mut alpha = Vec::with_capacity(rgba.len() / 4);
    for pixel in rgba.pixels() {
        rgb.extend_from_slice(&pixel.0[..3]);
        alpha.push(pixel.0[3]);
    }
    let id = ids.next();
    let mask = alpha.iter().any(|a| *a != 255).then(|| ids.next());
    let packed = compress_to_vec_zlib(&rgb, 6);
    let mut image = pdf.image_xobject(id, &packed);
    image.filter(Filter::FlateDecode);
    image.width(width as i32).height(height as i32);
    image.color_space().device_rgb();
    image.bits_per_component(8);
    if let Some(mask) = mask {
        image.s_mask(mask);
    }
    image.finish();
    if let Some(mask) = mask {
        let packed = compress_to_vec_zlib(&alpha, 6);
        let mut soft = pdf.image_xobject(mask, &packed);
        soft.filter(Filter::FlateDecode);
        soft.width(width as i32).height(height as i32);
        soft.color_space().device_gray();
        soft.bits_per_component(8);
    }
    Some(id)
}
