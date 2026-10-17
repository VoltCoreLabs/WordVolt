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

/// Position of a paragraph inside a table. A table is a run of consecutive paragraphs that share
/// `table`, ordered by (row, col); a cell may hold several paragraphs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub table: u32,
    pub row: u16,
    pub col: u16,
    /// Background of the cell (0xRRGGBB); `None` = none.
    pub shade: Option<u32>,
}

impl Cell {
    /// The cell at (`row`, `col`) of `table`, without a background.
    pub fn at(table: u32, row: u16, col: u16) -> Self {
        Self {
            table,
            row,
            col,
            shade: None,
        }
    }
}

/// A picture that fills the paragraph (its text is empty): index into `Document::images` and the
/// displayed size in px (96 dpi).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageRef {
    pub id: u32,
    pub width: f32,
    pub height: f32,
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
    /// Set for paragraphs that belong to a table cell.
    pub cell: Option<Cell>,
    /// Set for picture paragraphs.
    pub image: Option<ImageRef>,
    /// Line of the table of contents (replaced when the contents are updated).
    pub toc: bool,
}
