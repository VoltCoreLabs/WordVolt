use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use wv_core::{Align, CharStyle, Paragraph};

use crate::Measurer;
use crate::line::{Fragment, Line};

const EPS: f32 = 0.01;

/// Left indent of a block quote (px).
pub const QUOTE_INDENT: f32 = 24.0;

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
    pub lines: Vec<Line>,
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

struct Piece {
    range: Range<usize>,
    run: usize,
    width: f32,
    space: bool,
}

/// Lays out one paragraph for the given width.
pub fn layout_paragraph(p: &Paragraph, opts: &Options, m: &dyn Measurer) -> ParagraphLayout {
    let indent = if p.style.quote { QUOTE_INDENT } else { 0.0 };
    let opts = &Options {
        width: opts.width - indent,
        ..*opts
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
    layout.lines.iter_mut().for_each(|line| line.x += indent);
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
                width: m.width(&text[part.clone()], &p.runs[run].style),
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

fn build(
    p: &Paragraph,
    text: String,
    rows: Vec<Vec<Piece>>,
    opts: &Options,
    m: &dyn Measurer,
) -> ParagraphLayout {
    let default_style = p.runs.first().map(|r| r.style.clone()).unwrap_or_default();
    let last_row = rows.len() - 1;
    let mut lines = Vec::with_capacity(rows.len());
    let mut y = 0.0;
    for (i, row) in rows.iter().enumerate() {
        let line = build_line(p, &text, row, i == last_row, y, &default_style, opts, m);
        y += line.height;
        lines.push(line);
    }
    ParagraphLayout {
        text,
        quote: false,
        lines,
        height: y,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_line(
    p: &Paragraph,
    text: &str,
    row: &[Piece],
    is_last: bool,
    y: f32,
    default_style: &CharStyle,
    opts: &Options,
    m: &dyn Measurer,
) -> Line {
    let trailing = row.iter().rev().take_while(|x| x.space).count();
    let visible = &row[..row.len() - trailing];
    let visible_width: f32 = visible.iter().map(|x| x.width).sum();
    let start = row.first().map_or(0, |x| x.range.start);
    let end = row.last().map_or(0, |x| x.range.end);
    let visible_end = visible.last().map_or(start, |x| x.range.end);

    let mut space_extra = 0.0;
    let x = match p.style.align {
        Align::Left => 0.0,
        Align::Center => ((opts.width - visible_width) / 2.0).max(0.0),
        Align::Right => (opts.width - visible_width).max(0.0),
        Align::Justify => {
            let gaps = text[start..visible_end]
                .bytes()
                .filter(|b| *b == b' ')
                .count();
            if !is_last && gaps > 0 && visible_width < opts.width {
                space_extra = (opts.width - visible_width) / gaps as f32;
            }
            0.0
        }
    };

    let mut fragments: Vec<Fragment> = Vec::new();
    let mut cursor = 0.0;
    for piece in row {
        let extra = space_extra
            * text[piece.range.clone()]
                .bytes()
                .filter(|b| *b == b' ')
                .count() as f32
            * f32::from(piece.range.end <= visible_end);
        let width = piece.width + extra;
        match fragments.last_mut() {
            Some(f) if f.range.end == piece.range.start && f.style == p.runs[piece.run].style => {
                f.range.end = piece.range.end;
                f.width += width;
            }
            _ => fragments.push(Fragment {
                range: piece.range.clone(),
                style: p.runs[piece.run].style.clone(),
                x: cursor,
                width,
            }),
        }
        cursor += width;
    }

    let styles: Vec<&CharStyle> = if fragments.is_empty() {
        vec![default_style]
    } else {
        fragments.iter().map(|f| &f.style).collect()
    };
    let ascent = styles.iter().map(|s| m.ascent(s)).fold(0.0, f32::max);
    let descent = styles.iter().map(|s| m.descent(s)).fold(0.0, f32::max);
    let height = (ascent + descent) * opts.line_spacing;
    Line {
        range: start..end,
        visible_end,
        fragments,
        x,
        width: visible_width,
        y,
        height,
        baseline: height - descent,
        space_extra,
        is_last,
    }
}

#[cfg(test)]
mod tests;
