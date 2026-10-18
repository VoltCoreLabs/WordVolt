use std::collections::BTreeSet;

use pdf_writer::Name;
use wv_core::Crop;
use wv_layout::DocLayout;

use super::{Canvas, PT};

impl Canvas<'_> {
    /// A picture: `x`/`top`/`width`/`height` is the visible part; rotated by quarter turns and with
    /// `crop` cut off its sides (the whole picture is drawn and clipped to the visible part).
    pub(super) fn picture(
        &mut self,
        id: u32,
        rect: (f32, f32, f32, f32),
        rotation: u8,
        crop: Crop,
    ) {
        let (x, top, width, height) = rect;
        let name = format!("I{id}");
        let (vw, vh) = crop.visible();
        let (fw, fh) = (width / vw, height / vh);
        let (fx, ftop) = (x - crop.left * fw, top - crop.top * fh);
        let (px, py) = (self.x(fx), self.y(ftop + fh));
        let (wd, hd) = (fw * PT, fh * PT);
        // Maps the unit square of the picture to its place, turned clockwise by `rotation`.
        let matrix = match rotation % 4 {
            0 => [wd, 0.0, 0.0, hd, px, py],
            1 => [0.0, -hd, wd, 0.0, px, py + hd],
            2 => [-wd, 0.0, 0.0, -hd, px + wd, py + hd],
            _ => [0.0, hd, -wd, 0.0, px + wd, py],
        };
        self.content.save_state();
        if !crop.is_none() {
            let (cx, cy) = (self.x(x), self.y(top + height));
            self.content
                .rect(cx, cy, width * PT, height * PT)
                .clip_nonzero()
                .end_path();
        }
        self.content
            .transform(matrix)
            .x_object(Name(name.as_bytes()))
            .restore_state();
    }
}

/// Floating pictures of the page: those behind the text first, the others on top of it.
pub(super) fn place_floats(
    c: &mut Canvas,
    images: &mut BTreeSet<u32>,
    layout: &DocLayout,
    page: usize,
    behind: bool,
) {
    let setup = &layout.setup;
    for f in layout.pages[page]
        .floats
        .iter()
        .filter(|f| (f.wrap == wv_core::Wrap::Behind) == behind)
    {
        c.picture(
            f.id,
            (
                setup.margin_left + f.x,
                setup.margin_top + f.y,
                f.width,
                f.height,
            ),
            f.rotation,
            f.crop,
        );
        images.insert(f.id);
    }
}
