use std::collections::BTreeSet;

use pdf_writer::{Content, Name, Str};
use wv_core::{Align, Band, CharStyle, Document, Script};
use wv_layout::{DocLayout, Measurer, script_shift};

use super::fonts::FontSet;

/// 1 layout pixel (1/96 in) in PDF points.
const PT: f32 = 0.75;

const TEXT: u32 = 0x202020;
const LINK: u32 = 0x1a56db;
const BAND: u32 = 0x666666;
const TABLE: u32 = 0x999999;
const QUOTE_BAR: u32 = 0x9aa0a6;
const CODE_BG: u32 = 0xf0f0f0;
const SPOILER: u32 = 0x333333;
const COMMENT_BG: u32 = 0xfff1b8;

/// One page's content stream and the pictures it uses.
pub struct PageContent {
    pub data: Vec<u8>,
    pub images: BTreeSet<u32>,
}

struct Canvas<'a> {
    content: Content,
    fonts: &'a FontSet<'a>,
    height: f32,
}

impl Canvas<'_> {
    fn x(&self, px: f32) -> f32 {
        px * PT
    }

    fn y(&self, px: f32) -> f32 {
        (self.height - px) * PT
    }

    fn color(&mut self, rgb: u32) {
        let c = |shift: u32| ((rgb >> shift) & 0xff) as f32 / 255.0;
        self.content.set_fill_rgb(c(16), c(8), c(0));
    }

    /// Filled rectangle; `top` is the upper edge (layout pixels).
    fn fill(&mut self, x: f32, top: f32, width: f32, height: f32, rgb: u32) {
        self.color(rgb);
        let (px, py) = (self.x(x), self.y(top + height));
        self.content.rect(px, py, width * PT, height * PT);
        self.content.fill_nonzero();
    }

    /// Text with its baseline at `baseline`; underline and strike-through as thin bars.
    fn text(&mut self, text: &str, style: &CharStyle, x: f32, baseline: f32, rgb: u32) {
        if text.is_empty() {
            return;
        }
        let size = wv_layout::font_px(style);
        let width = self.fonts.width(text, style);
        if let Some(index) = self.fonts.face_index(style)
            && !text.trim().is_empty()
        {
            let bytes: Vec<u8> = self
                .fonts
                .encode(index, text, style)
                .iter()
                .flat_map(|g| g.to_be_bytes())
                .collect();
            let name = format!("F{index}");
            self.color(rgb);
            let (px, py) = (self.x(x), self.y(baseline));
            self.content
                .begin_text()
                .set_font(Name(name.as_bytes()), size * PT)
                .set_text_matrix([1.0, 0.0, 0.0, 1.0, px, py])
                .show(Str(&bytes))
                .end_text();
        }
        let thickness = (size * 0.06).max(0.8);
        if style.underline || style.link.is_some() {
            self.fill(x, baseline + size * 0.12, width, thickness, rgb);
        }
        if style.strike {
            self.fill(x, baseline - size * 0.3, width, thickness, rgb);
        }
    }
}

fn draw_bands(
    c: &mut Canvas,
    bands: (&Band, &Band),
    page: usize,
    pages: usize,
    layout: &DocLayout,
) {
    let setup = &layout.pages[page].setup;
    for (band, is_header) in [(bands.0, true), (bands.1, false)] {
        let Some(line) = wv_layout::band_line(band, &layout.pages[page].number, pages, c.fonts)
        else {
            continue;
        };
        let top = if is_header {
            setup.header_distance
        } else {
            setup.height - setup.footer_distance - line.height()
        };
        let x = match band.align {
            Align::Right => setup.width - setup.margin_right - line.width,
            Align::Center => (setup.width - line.width) / 2.0,
            _ => setup.margin_left,
        };
        for part in &line.parts {
            let rgb = part.style.color.unwrap_or(BAND);
            c.text(
                &part.text,
                &part.style,
                x.max(0.0) + part.x,
                top + line.ascent,
                rgb,
            );
        }
    }
}

fn draw_tables(c: &mut Canvas, layout: &DocLayout, page: usize) {
    let setup = &layout.pages[page].setup;
    let (left, top) = (setup.margin_left, setup.margin_top);
    for table in &layout.pages[page].tables {
        for shade in &table.shades {
            c.fill(
                left + shade.x,
                top + shade.y,
                shade.width,
                shade.height,
                shade.color,
            );
        }
        // Borders around every cell (merged cells show no line inside).
        for cell in &table.cells {
            let (x, y, w, h) = (left + cell.x, top + cell.y, cell.width, cell.height);
            c.fill(x, y - 0.5, w, 1.0, TABLE);
            c.fill(x, y + h - 0.5, w, 1.0, TABLE);
            c.fill(x - 0.5, y, 1.0, h, TABLE);
            c.fill(x + w - 0.5, y, 1.0, h, TABLE);
        }
    }
}

/// Draws page `page` of the layout.
pub fn draw_page(doc: &Document, layout: &DocLayout, fonts: &FontSet, page: usize) -> PageContent {
    let mut c = Canvas {
        content: Content::new(),
        fonts,
        height: layout.pages[page].setup.height,
    };
    let mut images = BTreeSet::new();
    let setup = &layout.pages[page].setup;
    draw_tables(&mut c, layout, page);
    draw_bands(
        &mut c,
        doc.bands_for(layout.pages[page].section, page),
        page,
        layout.pages.len(),
        layout,
    );
    pictures::place_floats(&mut c, &mut images, layout, page, true);
    // The line above the footnotes of the page.
    if let Some(top) = layout.pages[page].note_top {
        c.fill(
            setup.margin_left,
            setup.margin_top + top + wv_layout::NOTES_GAP / 2.0,
            setup.content_width() / 3.0,
            0.75,
            QUOTE_BAR,
        );
    }
    for slice in &layout.pages[page].slices {
        let para = &layout.paragraphs[slice.para];
        let first_y = para.lines[slice.lines.start].y;
        let slice_top = setup.margin_top + slice.y;
        if slice.lines.start == 0 {
            if let Some(image) = para.image.filter(|i| i.float.is_none()) {
                let x = setup.margin_left + slice.x + para.lines[0].x;
                let (bw, bh) = image.bounds();
                c.picture(&pictures::Placed {
                    id: image.id,
                    rect: (x, slice_top, bw, bh),
                    frame: (image.width, image.height),
                    angle: image.angle,
                    crop: image.crop,
                });
                images.insert(image.id);
            }
            if let Some(marker) = &para.marker {
                let line = &para.lines[0];
                let mut style = line
                    .fragments
                    .first()
                    .map(|f| f.style.clone())
                    .unwrap_or_default();
                (style.underline, style.strike, style.highlight, style.link) =
                    (false, false, None, None);
                let x =
                    setup.margin_left + slice.x + para.indent - 6.0 - fonts.width(marker, &style);
                let rgb = style.color.unwrap_or(TEXT);
                c.text(marker, &style, x, slice_top + line.baseline, rgb);
            }
        }
        if para.quote {
            c.fill(
                setup.margin_left + slice.x + 4.0,
                slice_top,
                3.0,
                slice.height,
                QUOTE_BAR,
            );
        }
        for line in &para.lines[slice.lines.clone()] {
            let top = slice_top + line.y - first_y;
            for f in &line.fragments {
                let text = &para.text[f.range.clone()];
                let mut x = setup.margin_left + slice.x + line.x + f.x;
                let baseline = top + line.baseline + script_shift(&f.style);
                let background = f
                    .style
                    .highlight
                    .or(f.style.mono.then_some(CODE_BG))
                    .or(f.style.comment.map(|_| COMMENT_BG));
                if let Some(background) = background {
                    c.fill(x, top, f.width, line.height, background);
                }
                if f.style.spoiler {
                    c.fill(x, top + 2.0, f.width, line.height - 4.0, SPOILER);
                    continue;
                }
                let rgb = f
                    .style
                    .color
                    .unwrap_or(if f.style.link.is_some() { LINK } else { TEXT });
                // A footnote mark: its number, raised, instead of the character.
                if let Some(id) = f.style.note {
                    if let Some(n) = layout.note_numbers.get(&id) {
                        let raised = CharStyle {
                            note: None,
                            script: Script::Super,
                            ..f.style.clone()
                        };
                        let raised_baseline = top + line.baseline + script_shift(&raised);
                        c.text(&n.to_string(), &raised, x, raised_baseline, rgb);
                    }
                    continue;
                }
                if text == "\t" {
                    if let Some((dots, offset)) = wv_layout::leader(f.width, &f.style, fonts) {
                        c.text(&dots, &f.style, x + offset, baseline, rgb);
                    }
                    continue;
                }
                if line.space_extra == 0.0 {
                    c.text(text, &f.style, x, baseline, rgb);
                    continue;
                }
                // Justified: words one by one, the extra space goes between them.
                let mut offset = f.range.start;
                for piece in text.split_inclusive(' ') {
                    c.text(piece.trim_end_matches(' '), &f.style, x, baseline, rgb);
                    x += fonts.width(piece, &f.style);
                    offset += piece.len();
                    if piece.ends_with(' ') && offset <= line.visible_end {
                        x += line.space_extra;
                    }
                }
            }
        }
    }
    pictures::place_floats(&mut c, &mut images, layout, page, false);
    PageContent {
        data: c.content.finish().to_vec(),
        images,
    }
}

mod pictures;
