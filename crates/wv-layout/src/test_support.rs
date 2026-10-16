use wv_core::CharStyle;

use crate::{Measurer, font_px};

/// Every char is `0.5 * font size` wide (bold: 0.6), ascent 0.8, descent 0.2. Deterministic for tests.
pub struct Fixed;

impl Measurer for Fixed {
    fn width(&self, text: &str, style: &CharStyle) -> f32 {
        let per_char = if style.bold { 0.6 } else { 0.5 };
        text.chars().count() as f32 * font_px(style) * per_char
    }

    fn ascent(&self, style: &CharStyle) -> f32 {
        font_px(style) * 0.8
    }

    fn descent(&self, style: &CharStyle) -> f32 {
        font_px(style) * 0.2
    }
}

/// Font size 12 pt = 16 px -> 8 px per char with `Fixed`.
pub const CHAR: f32 = 8.0;
