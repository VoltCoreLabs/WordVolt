use std::collections::BTreeSet;

use pdf_writer::Name;
use wv_core::Crop;
use wv_layout::DocLayout;

use super::{Canvas, PT};

/// What a picture looks like on the page: the box it takes up (x, top, width, height in layout
/// px), its size before turning, the angle (degrees clockwise) and what is cut off its sides.
pub(super) struct Placed {
    pub id: u32,
    pub rect: (f32, f32, f32, f32),
    pub frame: (f32, f32),
    pub angle: f32,
    pub crop: Crop,
}

impl Canvas<'_> {
    /// A picture turned about the centre of its box and with `crop` cut off its sides: the whole
    /// picture is drawn, turned, and clipped to the part that stays.
    pub(super) fn picture(&mut self, p: &Placed) {
        let (x, top, bw, bh) = p.rect;
        let name = format!("I{}", p.id);
        let (vw, vh) = p.crop.visible();
        let (fw, fh) = (p.frame.0 / vw, p.frame.1 / vh);
        // The visible part is centred in the box; the centre of the whole picture is off it by what
        // was cut on each side, turned with the picture.
        let (sin, cos) = p.angle.to_radians().sin_cos();
        let off = (
            (0.5 - (p.crop.left + vw / 2.0)) * fw,
            (0.5 - (p.crop.top + vh / 2.0)) * fh,
        );
        let (cx, cy) = (
            x + bw / 2.0 + off.0 * cos - off.1 * sin,
            top + bh / 2.0 + off.0 * sin + off.1 * cos,
        );
        // To PDF: y goes up, so a clockwise turn is a counter-clockwise one by minus the angle.
        let (c, s) = (cos, -sin);
        let (wd, hd) = (fw * PT, fh * PT);
        let (px, py) = (self.x(cx), self.y(cy));
        let matrix = [
            wd * c,
            wd * s,
            -hd * s,
            hd * c,
            px - (wd / 2.0 * c - hd / 2.0 * s),
            py - (wd / 2.0 * s + hd / 2.0 * c),
        ];
        self.content.save_state();
        self.content.transform(matrix);
        if !p.crop.is_none() {
            // In the unit square of the picture: v goes up, so the bottom cut is at the bottom.
            self.content
                .rect(p.crop.left, p.crop.bottom, vw, vh)
                .clip_nonzero()
                .end_path();
        }
        self.content.x_object(Name(name.as_bytes())).restore_state();
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
        c.picture(&Placed {
            id: f.id,
            rect: (
                setup.margin_left + f.x,
                setup.margin_top + f.y,
                f.width,
                f.height,
            ),
            frame: f.frame,
            angle: f.angle,
            crop: f.crop,
        });
        images.insert(f.id);
    }
}
