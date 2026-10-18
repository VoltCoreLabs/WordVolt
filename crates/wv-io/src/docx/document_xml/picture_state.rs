use wv_core::{ImageRef, Paragraph};

use super::State;

#[derive(Default)]
pub(super) struct PictureState {
    /// Placement of the drawing being read when it is anchored (floating) instead of inline.
    pub(super) anchor: Option<PictureAnchor>,
    /// Inside `w:positionV` (true) / `w:positionH` (false), and inside its `w:posOffset`.
    pub(super) axis: Option<bool>,
    pub(super) in_offset: bool,
    /// The offset is measured from the page edge, not from the margin.
    pub(super) relative_page: bool,
    pub(super) rels: Vec<String>,
    pub(super) in_drawing: bool,
    pub(super) embed: Option<String>,
    /// Angle (degrees clockwise) and the cut of the drawing being read.
    pub(super) angle: f32,
    /// `a:ext` of the picture: its size before it is turned.
    pub(super) frame: Option<(f32, f32)>,
    pub(super) crop: wv_core::Crop,
    pub(super) size: (f32, f32),
    /// The paragraph that follows a picture inside one `w:p`; dropped if it stays empty.
    pub(super) after: bool,
}

/// Where an anchored picture is (px from the margin) and how the text goes around it.
#[derive(Default)]
pub(super) struct PictureAnchor {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) behind: bool,
    /// How the text goes around it; `Front` also stands for "no wrapping" (see `behind`).
    pub(super) wrap: Option<wv_core::Wrap>,
}

impl State {
    /// A picture ends the text paragraph before it and becomes a paragraph of its own.
    pub(super) fn end_drawing(&mut self) {
        let pictures = &mut self.pictures;
        pictures.in_drawing = false;
        let Some(rid) = pictures.embed.take() else {
            return;
        };
        let (mut width, mut height) = pictures.size;
        if width < 1.0 || height < 1.0 {
            (width, height) = (200.0, 150.0);
        }
        let id = pictures.rels.len() as u32;
        pictures.rels.push(rid);
        if self.current.as_ref().is_some_and(|p| !p.is_empty()) {
            self.line_break(false);
        }
        // The picture's own size (before turning) when Word gives it, else the box.
        let (width, height) = self.pictures.frame.unwrap_or((width, height));
        let mut image = ImageRef::inline(id, width, height);
        image.angle = self.pictures.angle;
        image.crop = self.pictures.crop;
        image.float = self.pictures.anchor.take().map(|a| {
            let wrap = match (a.wrap, a.behind) {
                (Some(wv_core::Wrap::Front), true) => wv_core::Wrap::Behind,
                (Some(wrap), _) => wrap,
                (None, true) => wv_core::Wrap::Behind,
                (None, false) => wv_core::Wrap::TopBottom,
            };
            wv_core::Float {
                x: a.x,
                y: a.y,
                wrap,
            }
        });
        self.image = Some(image);
        let (align, named) = (self.align, self.named);
        self.end_paragraph();
        self.current = Some(Paragraph::default());
        self.align = align;
        self.named = named;
        self.pictures.after = true;
    }

    /// The text of `w:posOffset`: EMU from the margin (or the page edge, one inch of margin is
    /// assumed for that) as a position of the picture being read.
    pub(super) fn picture_offset(&mut self, text: &str) {
        let page_margin = if self.pictures.relative_page {
            96.0
        } else {
            0.0
        };
        let px = crate::docx::picture::emu_to_px(text.trim()) - page_margin;
        if let (Some(vertical), Some(anchor)) = (self.pictures.axis, self.pictures.anchor.as_mut())
        {
            if vertical {
                anchor.y = px.max(0.0);
            } else {
                anchor.x = px.max(0.0);
            }
        }
    }
}
