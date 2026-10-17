use std::collections::BTreeSet;

use pdf_writer::{Content, Name, Str};
use wv_core::{Align, Band, CharStyle, Document};
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

    fn picture(&mut self, id: u32, x: f32, top: f32, width: f32, height: f32) {
        let name = format!("I{id}");
        let (px, py) = (self.x(x), self.y(top + height));
        self.content
            .save_state()
            .transform([width * PT, 0.0, 0.0, height * PT, px, py])
            .x_object(Name(name.as_bytes()))
            .restore_state();
    }
}

fn draw_bands(
    c: &mut Canvas,
    bands: (&Band, &Band),
    page: usize,
    pages: usize,
    layout: &DocLayout,
) {
    let setup = &layout.setup;
    for (band, is_header) in [(bands.0, true), (bands.1, false)] {
        let text = band.render(page + 1, pages);
        let style = band.style.char_style();
        let (ascent, descent) = (c.fonts.ascent(&style), c.fonts.descent(&style));
        let top = if is_header {
            setup.header_distance
        } else {
            setup.height - setup.footer_distance - (ascent + descent)
        };
        let width = c.fonts.width(&text, &style);
        let x = match band.align {
            Align::Right => setup.width - setup.margin_right - width,
            Align::Center => (setup.width - width) / 2.0,
            _ => setup.margin_left,
        };
        let rgb = band.style.color.unwrap_or(BAND);
        c.text(&text, &style, x.max(0.0), top + ascent, rgb);
    }
}

fn draw_tables(c: &mut Canvas, layout: &DocLayout, page: usize) {
    let (left, top) = (layout.setup.margin_left, layout.setup.margin_top);
    for table in &layout.pages[page].tables {
        let (first, last) = (table.row_y[0], table.row_y[table.row_y.len() - 1]);
        let width = table.col_x[table.col_x.len() - 1];
        for y in &table.row_y {
            c.fill(left, top + y - 0.5, width, 1.0, TABLE);
        }
        for x in &table.col_x {
            c.fill(
                left + x - 0.5,
                top + first - 0.5,
                1.0,
                last - first + 1.0,
                TABLE,
            );
        }
    }
}

/// Draws page `page` of the layout.
pub fn draw_page(doc: &Document, layout: &DocLayout, fonts: &FontSet, page: usize) -> PageContent {
    let mut c = Canvas {
        content: Content::new(),
        fonts,
        height: layout.setup.height,
    };
    let mut images = BTreeSet::new();
    let setup = &layout.setup;
    draw_tables(&mut c, layout, page);
    draw_bands(
        &mut c,
        doc.bands_for(page),
        page,
        layout.pages.len(),
        layout,
    );
    for slice in &layout.pages[page].slices {
        let para = &layout.paragraphs[slice.para];
        let first_y = para.lines[slice.lines.start].y;
        let slice_top = setup.margin_top + slice.y;
        if slice.lines.start == 0 {
            if let Some(image) = para.image {
                let x = setup.margin_left + slice.x + para.lines[0].x;
                c.picture(image.id, x, slice_top, image.width, image.height);
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
                let background = f.style.highlight.or(f.style.mono.then_some(CODE_BG));
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
    PageContent {
        data: c.content.finish().to_vec(),
        images,
    }
}
