use wv_core::ImageRef;

/// Pixels (96 dpi) <-> EMU.
const EMU_PER_PX: f32 = 9525.0;

/// Relationship id of picture `id` (see `package::document_rels`).
pub fn rel_id(id: u32) -> String {
    format!("rIdI{}", id + 1)
}

/// `w:drawing` run with an inline picture; `unique` numbers the drawing object.
pub fn drawing_xml(image: ImageRef, unique: u32) -> String {
    let (cx, cy) = (
        (image.width * EMU_PER_PX).round() as u64,
        (image.height * EMU_PER_PX).round() as u64,
    );
    let rid = rel_id(image.id);
    format!(
        "<w:r><w:drawing><wp:inline xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\"><wp:extent cx=\"{cx}\" cy=\"{cy}\"/><wp:docPr id=\"{unique}\" name=\"Picture {unique}\"/><a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:pic xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:nvPicPr><pic:cNvPr id=\"{unique}\" name=\"{rid}\"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed=\"{rid}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"
    )
}

/// EMU -> px for a size read from `wp:extent`.
pub fn emu_to_px(emu: &str) -> f32 {
    emu.parse::<f32>().map_or(0.0, |v| v / EMU_PER_PX)
}
