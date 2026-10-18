use wv_core::{ImageRef, Wrap};

/// Pixels (96 dpi) <-> EMU.
const EMU_PER_PX: f32 = 9525.0;

const WP: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";

/// Relationship id of picture `id` (see `package::document_rels`).
pub fn rel_id(id: u32) -> String {
    format!("rIdI{}", id + 1)
}

/// `w:drawing` run with the picture: inline, or anchored for a floating one; `unique` numbers the
/// drawing object.
pub fn drawing_xml(image: ImageRef, unique: u32) -> String {
    let (cx, cy) = (
        (image.width * EMU_PER_PX).round() as u64,
        (image.height * EMU_PER_PX).round() as u64,
    );
    let rid = rel_id(image.id);
    let graphic = format!(
        "<a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:pic xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:nvPicPr><pic:cNvPr id=\"{unique}\" name=\"{rid}\"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed=\"{rid}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic>"
    );
    let doc_pr = format!("<wp:docPr id=\"{unique}\" name=\"Picture {unique}\"/>");
    let drawing = match image.float {
        None => format!(
            "<wp:inline xmlns:wp=\"{WP}\" distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\"><wp:extent cx=\"{cx}\" cy=\"{cy}\"/>{doc_pr}{graphic}</wp:inline>"
        ),
        Some(float) => {
            let (x, y) = (
                (float.x * EMU_PER_PX).round() as i64,
                (float.y * EMU_PER_PX).round() as i64,
            );
            let (behind, wrap) = match float.wrap {
                Wrap::TopBottom => (0, "<wp:wrapTopAndBottom/>"),
                Wrap::Front => (0, "<wp:wrapNone/>"),
                Wrap::Behind => (1, "<wp:wrapNone/>"),
            };
            format!(
                "<wp:anchor xmlns:wp=\"{WP}\" distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\" simplePos=\"0\" relativeHeight=\"0\" behindDoc=\"{behind}\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\"><wp:simplePos x=\"0\" y=\"0\"/><wp:positionH relativeFrom=\"margin\"><wp:posOffset>{x}</wp:posOffset></wp:positionH><wp:positionV relativeFrom=\"margin\"><wp:posOffset>{y}</wp:posOffset></wp:positionV><wp:extent cx=\"{cx}\" cy=\"{cy}\"/><wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/>{wrap}{doc_pr}<wp:cNvGraphicFramePr/>{graphic}</wp:anchor>"
            )
        }
    };
    format!("<w:r><w:drawing>{drawing}</w:drawing></w:r>")
}

/// EMU -> px for a size read from `wp:extent`.
pub fn emu_to_px(emu: &str) -> f32 {
    emu.parse::<f32>().map_or(0.0, |v| v / EMU_PER_PX)
}
