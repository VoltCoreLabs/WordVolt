#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKind {
    Bullet,
    Number,
}

/// Paragraph formatting.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParaStyle {
    pub align: Align,
    /// Built-in style (Normal, Title, Heading 1, ...).
    pub named: super::NamedStyle,
    /// Block quote: indented with a bar on the left.
    pub quote: bool,
    /// List item (bullet or number). The nesting level is `indent`.
    pub list: Option<ListKind>,
    /// Left indent in steps of 0.5 inch (0..=8); for list items this is the nesting level.
    pub indent: u8,
    /// Line height multiplier; `None` = the document default.
    pub line_spacing: Option<f32>,
}
