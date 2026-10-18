use std::io::Cursor;

use wv_core::ImageFormat;

/// Format and pixel size of an encoded PNG/JPEG/GIF/WebP picture; `None` if it is none of those or
/// damaged.
pub fn probe_image(bytes: &[u8]) -> Option<(ImageFormat, u32, u32)> {
    let reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let format = match reader.format()? {
        image::ImageFormat::Png => ImageFormat::Png,
        image::ImageFormat::Jpeg => ImageFormat::Jpeg,
        image::ImageFormat::Gif => ImageFormat::Gif,
        image::ImageFormat::WebP => ImageFormat::Webp,
        _ => return None,
    };
    let (width, height) = reader.into_dimensions().ok()?;
    Some((format, width, height))
}

/// The picture turned by `quarters` quarter turns clockwise, as PNG bytes; `None` if it cannot be
/// read.
pub fn rotate_image(bytes: &[u8], quarters: u8) -> Option<Vec<u8>> {
    let decoded = image::load_from_memory(bytes).ok()?;
    let turned = match quarters % 4 {
        0 => decoded,
        1 => decoded.rotate90(),
        2 => decoded.rotate180(),
        _ => decoded.rotate270(),
    };
    let mut out = Cursor::new(Vec::new());
    turned.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    // 1x1 transparent PNG.
    const PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn probes_png_and_rejects_garbage() {
        assert_eq!(probe_image(PNG), Some((ImageFormat::Png, 1, 1)));
        assert_eq!(probe_image(b"not an image"), None);
    }
}
