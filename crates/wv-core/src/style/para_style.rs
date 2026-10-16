#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Paragraph formatting.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParaStyle {
    pub align: Align,
}
