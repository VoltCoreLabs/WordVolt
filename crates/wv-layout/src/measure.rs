use wv_core::{CharStyle, Script};

/// Font size used when a run has no explicit size (Word's default: 12 pt).
pub const DEFAULT_SIZE_PT: u16 = 12;

/// Super/subscript glyphs are this fraction of the normal size.
const SCRIPT_SCALE: f32 = 0.65;

/// Size in logical pixels of the text of the style before the superscript/subscript reduction (1 pt = 96/72 px).
pub fn base_px(style: &CharStyle) -> f32 {
    f32::from(style.size_pt.unwrap_or(DEFAULT_SIZE_PT)) * 96.0 / 72.0
}

/// Font size in logical pixels as drawn (smaller for super/subscript).
pub fn font_px(style: &CharStyle) -> f32 {
    match style.script {
        Script::Normal => base_px(style),
        Script::Super | Script::Sub => base_px(style) * SCRIPT_SCALE,
    }
}

/// How far the baseline of this text is moved down from the line's baseline (negative = up).
pub fn script_shift(style: &CharStyle) -> f32 {
    match style.script {
        Script::Normal => 0.0,
        Script::Super => -0.35 * base_px(style),
        Script::Sub => 0.14 * base_px(style),
    }
}

/// Width of a footnote mark: room for two digits in the superscript size (the number is drawn there).
pub fn note_width(style: &CharStyle) -> f32 {
    base_px(style) * 0.75
}

/// Width of `text` in `style`: a footnote mark has the width of its number, whatever the font says.
pub fn run_width(m: &dyn Measurer, text: &str, style: &CharStyle) -> f32 {
    if style.note.is_some() {
        note_width(style) * text.chars().count() as f32
    } else {
        m.width(text, style)
    }
}

/// Text metrics supplied by the UI (or a fake in tests). Lengths in logical pixels.
pub trait Measurer {
    /// Advance width of `text` set in `style`.
    fn width(&self, text: &str, style: &CharStyle) -> f32;
    /// Distance from the baseline up to the top of the tallest glyph.
    fn ascent(&self, style: &CharStyle) -> f32;
    /// Distance from the baseline down to the lowest glyph.
    fn descent(&self, style: &CharStyle) -> f32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_text_is_smaller_and_shifted() {
        let normal = CharStyle::default();
        let sup = CharStyle {
            script: Script::Super,
            ..Default::default()
        };
        let sub = CharStyle {
            script: Script::Sub,
            ..Default::default()
        };
        assert_eq!(font_px(&normal), 16.0);
        assert!(font_px(&sup) < 12.0 && font_px(&sup) == font_px(&sub));
        assert!(
            script_shift(&sup) < 0.0 && script_shift(&sub) > 0.0 && script_shift(&normal) == 0.0
        );
    }
}
