use wv_core::CharStyle;

/// Font size used when a run has no explicit size (Word's default: 12 pt).
pub const DEFAULT_SIZE_PT: u16 = 12;

/// Font size in logical pixels (1 pt = 96/72 px).
pub fn font_px(style: &CharStyle) -> f32 {
    f32::from(style.size_pt.unwrap_or(DEFAULT_SIZE_PT)) * 96.0 / 72.0
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
