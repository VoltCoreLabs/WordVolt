use super::{Align, CharStyle, NamedStyle, ParaStyle};

/// A paragraph style the user defined (or a style of a file that is not a built-in one): the
/// formatting it gives to its paragraphs, on top of the built-in style it is based on. What the
/// paragraph or its text sets directly wins over it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserStyle {
    pub name: String,
    pub base: NamedStyle,
    pub font: Option<String>,
    pub size_pt: Option<u16>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<u32>,
    pub align: Option<Align>,
    pub line_spacing: Option<f32>,
    pub space_before: Option<u8>,
    pub space_after: Option<u8>,
}

impl UserStyle {
    /// A style that looks like `text` and `para` (the formatting at the caret).
    pub fn from_formatting(name: &str, para: &ParaStyle, text: &CharStyle) -> Self {
        Self {
            name: name.to_string(),
            base: para.named,
            font: text.font.clone(),
            size_pt: text.size_pt,
            bold: text.bold,
            italic: text.italic,
            underline: text.underline,
            color: text.color,
            align: Some(para.align),
            line_spacing: para.line_spacing,
            space_before: para.space_before,
            space_after: para.space_after,
        }
    }

    /// Fills in what the style defines and the text leaves open.
    pub fn apply_to_text(&self, style: &mut CharStyle) {
        if style.font.is_none() {
            style.font = self.font.clone();
        }
        if style.size_pt.is_none() {
            style.size_pt = self.size_pt;
        }
        if style.color.is_none() {
            style.color = self.color;
        }
        style.bold |= self.bold;
        style.italic |= self.italic;
        style.underline |= self.underline;
    }

    /// Fills in what the style defines and the paragraph leaves open.
    pub fn apply_to_paragraph(&self, style: &mut ParaStyle) {
        if style.align == Align::Left
            && let Some(align) = self.align
        {
            style.align = align;
        }
        style.line_spacing = style.line_spacing.or(self.line_spacing);
        style.space_before = style.space_before.or(self.space_before);
        style.space_after = style.space_after.or(self.space_after);
        style.named = self.base;
    }
}
