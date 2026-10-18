use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use wv_core::{Align, Paragraph};

use crate::Measurer;
use crate::line::Line;
use build::build;

const EPS: f32 = 0.01;

/// Left indent of a block quote (px).
pub const QUOTE_INDENT: f32 = 24.0;

/// One indent step: 0.5 inch.
pub const INDENT_STEP: f32 = 48.0;

/// Room for the bullet/number in front of a list item's text (px).
pub const LIST_HANGING: f32 = 24.0;

/// Gap after a list item (list items sit closer together than ordinary paragraphs).
const LIST_SPACE_AFTER: f32 = 2.0;

#[derive(Clone, Copy)]
pub struct Options {
    /// Width available for text (page width minus margins).
    pub width: f32,
    /// Line height multiplier (1.0 = single).
    pub line_spacing: f32,
}

/// A laid-out paragraph. Its lines are positioned from y = 0.
#[derive(Clone, Debug, PartialEq)]
pub struct ParagraphLayout {
    pub text: String,
    /// Block quote: drawn with a bar and indented by [`QUOTE_INDENT`].
    pub quote: bool,
    /// Left edge of the text (indent steps, list hanging and quote indent).
    pub indent: f32,
    /// Bullet or number drawn in front of the first line (set by `DocLayout`, which counts the items).
    pub marker: Option<String>,
    /// Gap above the paragraph (named styles such as headings); dropped at the top of a page.
    pub space_before: f32,
    /// Gap after the paragraph; `None` = the page setup's default.
    pub space_after: Option<f32>,
    pub lines: Vec<Line>,
    pub height: f32,
    /// Picture of a picture paragraph (it is drawn on the paragraph's only line).
    pub image: Option<ImageBox>,
}

/// Picture placement inside a paragraph layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageBox {
    pub id: u32,
    pub width: f32,
    pub height: f32,
}

impl ParagraphLayout {
    /// Index of the line the caret at `byte` is drawn on (a wrap point belongs to the next line).
    pub fn line_of(&self, byte: usize) -> usize {
        self.lines
            .iter()
            .position(|l| byte < l.range.end)
            .unwrap_or(self.lines.len() - 1)
    }
}

pub(super) struct Piece {
    pub(super) range: Range<usize>,
    pub(super) run: usize,
    pub(super) width: f32,
    pub(super) space: bool,
}

/// Lays out one paragraph for the given width.
pub fn layout_paragraph(p: &Paragraph, opts: &Options, m: &dyn Measurer) -> ParagraphLayout {
    let quote = if p.style.quote { QUOTE_INDENT } else { 0.0 };
    let list_gap = if p.style.list.is_some() {
        LIST_HANGING
    } else {
        0.0
    };
    let indent = f32::from(p.style.indent) * INDENT_STEP + list_gap + quote;
    let opts = &Options {
        width: opts.width - indent,
        line_spacing: p.style.line_spacing.unwrap_or(opts.line_spacing),
    };
    let text = p.text();
    let spans = run_spans(p);
    let mut lines: Vec<Vec<Piece>> = Vec::new();
    let mut current: Vec<Piece> = Vec::new();
    let mut current_width = 0.0;
    for atom in atoms(&text) {
        let word_end = atom.start + text[atom.clone()].trim_end().len();
        let word = pieces(p, &text, &spans, atom.start..word_end, false, m);
        let spaces = pieces(p, &text, &spans, word_end..atom.end, true, m);
        let word_width: f32 = word.iter().map(|x| x.width).sum();
        if !current.is_empty() && current_width + word_width > opts.width + EPS {
            lines.push(std::mem::take(&mut current));
            current_width = 0.0;
        }
        if word_width > opts.width + EPS {
            for piece in word {
                for part in split_graphemes(p, &text, piece, m) {
                    if !current.is_empty() && current_width + part.width > opts.width + EPS {
                        lines.push(std::mem::take(&mut current));
                        current_width = 0.0;
                    }
                    current_width += part.width;
                    current.push(part);
                }
            }
        } else {
            current_width += word_width;
            current.extend(word);
        }
        current_width += spaces.iter().map(|x| x.width).sum::<f32>();
        current.extend(spaces);
    }
    lines.push(current);
    let mut layout = build(p, text, lines, opts, m);
    layout.quote = p.style.quote;
    layout.indent = indent;
    layout.space_before = p.style.named.space_before();
    layout.space_after = p
        .style
        .list
        .map(|_| LIST_SPACE_AFTER)
        .or(p.style.named.space_after());
    layout.lines.iter_mut().for_each(|line| line.x += indent);
    if let Some(img) = p.style.image {
        let free = (opts.width - img.width).max(0.0);
        let offset = match p.style.align {
            Align::Center => free / 2.0,
            Align::Right => free,
            _ => 0.0,
        };
        layout.image = Some(ImageBox {
            id: img.id,
            width: img.width,
            height: img.height,
        });
        if let Some(line) = layout.lines.first_mut() {
            (line.x, line.y, line.width) = (indent + offset, 0.0, img.width);
            (line.height, line.baseline) = (img.height, img.height);
        }
        layout.height = img.height;
    }
    layout
}

fn run_spans(p: &Paragraph) -> Vec<Range<usize>> {
    let mut start = 0;
    p.runs
        .iter()
        .map(|r| {
            let span = start..start + r.text.len();
            start = span.end;
            span
        })
        .collect()
}

/// Segments between line-break opportunities (UAX #14); the last one ends at the text end.
fn atoms(text: &str) -> Vec<Range<usize>> {
    let mut start = 0;
    let mut out = Vec::new();
    for (end, _) in unicode_linebreak::linebreaks(text) {
        if end > start {
            out.push(start..end);
            start = end;
        }
    }
    out
}

fn pieces(
    p: &Paragraph,
    text: &str,
    spans: &[Range<usize>],
    range: Range<usize>,
    space: bool,
    m: &dyn Measurer,
) -> Vec<Piece> {
    spans
        .iter()
        .enumerate()
        .filter_map(|(run, span)| {
            let part = range.start.max(span.start)..range.end.min(span.end);
            (part.start < part.end).then(|| Piece {
                // A tab has no width of its own (the layout stretches it, see `build_line`).
                width: if p.style.toc && &text[part.clone()] == "\t" {
                    0.0
                } else {
                    m.width(&text[part.clone()], &p.runs[run].style)
                },
                range: part,
                run,
                space,
            })
        })
        .collect()
}

fn split_graphemes<'a>(
    p: &'a Paragraph,
    text: &'a str,
    piece: Piece,
    m: &'a dyn Measurer,
) -> impl Iterator<Item = Piece> + 'a {
    text[piece.range.clone()]
        .grapheme_indices(true)
        .map(move |(i, g)| {
            let start = piece.range.start + i;
            Piece {
                range: start..start + g.len(),
                run: piece.run,
                width: m.width(g, &p.runs[piece.run].style),
                space: false,
            }
        })
}

mod build;
#[cfg(test)]
mod tests;
