use quick_xml::events::BytesStart;
use wv_core::{ImageRef, Paragraph};

use super::State;
use crate::docx::read_util::attr;

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
    /// Start tags of a drawing: size, turn, cut, anchor and wrapping. `false` when `e` is none
    /// of those.
    pub(super) fn start_picture(&mut self, e: &BytesStart, empty: bool) -> bool {
        let name = e.local_name();
        match name.as_ref() {
            b"drawing" if !empty => {
                self.pictures.in_drawing = true;
                self.pictures.embed = None;
                self.pictures.size = (0.0, 0.0);
                self.pictures.anchor = None;
                self.pictures.angle = 0.0;
                self.pictures.frame = None;
                self.pictures.crop = wv_core::Crop::default();
            }
            b"xfrm" if self.pictures.in_drawing => {
                let rot = attr(e, b"rot")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(0.0);
                self.pictures.angle = (rot / 60_000.0).rem_euclid(360.0);
            }
            b"ext" if self.pictures.in_drawing => {
                let px = |name: &[u8]| {
                    attr(e, name).map_or(0.0, |v| crate::docx::picture::emu_to_px(&v))
                };
                let (w, h) = (px(b"cx"), px(b"cy"));
                if w >= 1.0 && h >= 1.0 {
                    self.pictures.frame = Some((w, h));
                }
            }
            b"srcRect" if self.pictures.in_drawing => {
                let part = |name: &[u8]| {
                    attr(e, name)
                        .and_then(|v| v.parse::<f32>().ok())
                        .map_or(0.0, |v| (v / 100_000.0).clamp(0.0, 0.9))
                };
                self.pictures.crop = wv_core::Crop {
                    left: part(b"l"),
                    top: part(b"t"),
                    right: part(b"r"),
                    bottom: part(b"b"),
                };
            }
            b"anchor" if self.pictures.in_drawing => {
                self.pictures.anchor = Some(PictureAnchor {
                    behind: attr(e, b"behindDoc").is_some_and(|v| v == "1" || v == "true"),
                    ..PictureAnchor::default()
                });
            }
            b"positionH" | b"positionV" if self.pictures.anchor.is_some() && !empty => {
                self.pictures.axis = Some(name.as_ref() == b"positionV");
                self.pictures.relative_page = attr(e, b"relativeFrom").as_deref() == Some("page");
            }
            b"posOffset" if self.pictures.axis.is_some() && !empty => {
                self.pictures.in_offset = true
            }
            b"wrapNone" if self.pictures.anchor.is_some() => {
                if let Some(a) = self.pictures.anchor.as_mut() {
                    a.wrap = Some(wv_core::Wrap::Front);
                }
            }
            b"wrapTopAndBottom" if self.pictures.anchor.is_some() => {
                if let Some(a) = self.pictures.anchor.as_mut() {
                    a.wrap = Some(wv_core::Wrap::TopBottom);
                }
            }
            b"wrapSquare" if self.pictures.anchor.is_some() => {
                if let Some(a) = self.pictures.anchor.as_mut() {
                    a.wrap = Some(wv_core::Wrap::Square);
                }
            }
            b"wrapTight" | b"wrapThrough" if self.pictures.anchor.is_some() => {
                if let Some(a) = self.pictures.anchor.as_mut() {
                    a.wrap = Some(wv_core::Wrap::Tight);
                }
            }
            b"extent" if self.pictures.in_drawing && self.pictures.size.0 == 0.0 => {
                let px = |name: &[u8]| {
                    attr(e, name).map_or(0.0, |v| crate::docx::picture::emu_to_px(&v))
                };
                self.pictures.size = (px(b"cx"), px(b"cy"));
            }
            b"blip" if self.pictures.in_drawing => self.pictures.embed = attr(e, b"embed"),
            _ => return false,
        }
        true
    }

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
