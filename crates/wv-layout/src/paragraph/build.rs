use wv_core::{Align, CharStyle, Paragraph};

use super::{Options, ParagraphLayout, Piece};
use crate::Measurer;
use crate::line::{Fragment, Line};

pub(super) fn build(
    p: &Paragraph,
    text: String,
    rows: Vec<Vec<Piece>>,
    opts: &Options,
    first: f32,
    m: &dyn Measurer,
) -> ParagraphLayout {
    let mut default_style = p.runs.first().map(|r| r.style.clone()).unwrap_or_default();
    p.style.named.apply(&mut default_style);
    let last_row = rows.len() - 1;
    let mut lines = Vec::with_capacity(rows.len());
    let mut y = 0.0;
    for (i, row) in rows.iter().enumerate() {
        // The first line is narrower by the first-line indent and starts that far in.
        let o = Options {
            width: if i == 0 {
                opts.width - first
            } else {
                opts.width
            },
            ..*opts
        };
        let mut line = build_line(p, &text, row, i == last_row, y, &default_style, &o, m);
        if i == 0 {
            line.x += first;
        }
        y += line.height;
        lines.push(line);
    }
    ParagraphLayout {
        text,
        quote: false,
        indent: 0.0,
        marker: None,
        space_before: 0.0,
        space_after: None,
        lines,
        height: y,
        image: None,
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
    // In the contents, a tab pushes the rest of the line (the page number) to the right edge.
    let is_tab = |x: &Piece| p.style.toc && &text[x.range.clone()] == "\t";
    let tab_extra = if row.iter().any(is_tab) {
        (opts.width - visible.iter().map(|x| x.width).sum::<f32>()).max(0.0)
    } else {
        0.0
    };
    let visible_width: f32 = visible.iter().map(|x| x.width).sum::<f32>() + tab_extra;
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
        let tab = is_tab(piece);
        let width = piece.width + extra + if tab { tab_extra } else { 0.0 };
        let after_tab = fragments
            .last()
            .is_some_and(|f| text[f.range.clone()].ends_with('\t'));
        match fragments.last_mut() {
            Some(f)
                if f.range.end == piece.range.start
                    && f.style == p.runs[piece.run].style
                    && !tab
                    && !after_tab =>
            {
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
