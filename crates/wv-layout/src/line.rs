use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use wv_core::CharStyle;

use crate::Measurer;

/// A stretch of text of one run on a line. `x` is relative to the line's content start.
#[derive(Clone, Debug, PartialEq)]
pub struct Fragment {
    pub range: Range<usize>,
    pub style: CharStyle,
    pub x: f32,
    pub width: f32,
}

/// One visual line of a paragraph. Byte ranges are offsets into the paragraph text.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// Whole line including trailing spaces that hang past the right edge.
    pub range: Range<usize>,
    /// End of the visible text (before trailing spaces).
    pub visible_end: usize,
    pub fragments: Vec<Fragment>,
    /// Alignment offset of the content from the left edge.
    pub x: f32,
    /// Visible content width (without trailing spaces).
    pub width: f32,
    /// Top of the line inside the paragraph.
    pub y: f32,
    pub height: f32,
    pub baseline: f32,
    /// Extra width added to every space (justified lines).
    pub space_extra: f32,
    pub is_last: bool,
}

impl Line {
    fn spaces(&self, text: &str, from: usize, to: usize) -> usize {
        let to = to.min(self.visible_end).max(from);
        text[from..to].bytes().filter(|b| *b == b' ').count()
    }

    /// X of the caret before byte `b` of fragment `f`. A tab has a width of its own (not what the
    /// font measures): inside it only its two edges are caret places.
    fn frag_x(&self, text: &str, f: &Fragment, b: usize, m: &dyn Measurer) -> f32 {
        if b >= f.range.end && text[f.range.clone()].contains('\t') {
            return self.x + f.x + f.width;
        }
        self.x
            + f.x
            + m.width(&text[f.range.start..b], &f.style)
            + self.space_extra * self.spaces(text, f.range.start, b) as f32
    }

    /// X (from the paragraph's left edge) of the caret before byte `byte` of the paragraph text.
    pub fn x_at(&self, text: &str, byte: usize, m: &dyn Measurer) -> f32 {
        let Some(f) = self
            .fragments
            .iter()
            .find(|f| byte < f.range.end)
            .or(self.fragments.last())
        else {
            return self.x;
        };
        let byte = byte.clamp(f.range.start, f.range.end);
        self.frag_x(text, f, byte, m)
    }

    /// Byte offset of the caret position nearest to `x` (from the paragraph's left edge).
    pub fn index_at(&self, text: &str, x: f32, m: &dyn Measurer) -> usize {
        let limit = if self.is_last {
            self.range.end
        } else {
            self.visible_end
        };
        let mut best = (self.range.start, f32::MAX);
        for f in &self.fragments {
            let boundaries = text[f.range.clone()]
                .grapheme_indices(true)
                .map(|(i, _)| f.range.start + i)
                .chain(std::iter::once(f.range.end));
            for b in boundaries {
                if b > limit {
                    break;
                }
                let bx = self.frag_x(text, f, b, m);
                let distance = (bx - x).abs();
                if distance < best.1 {
                    best = (b, distance);
                }
            }
        }
        best.0
    }
}
