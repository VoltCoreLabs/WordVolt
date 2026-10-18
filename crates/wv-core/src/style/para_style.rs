#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ListKind {
    Bullet,
    Number,
}

/// Position of a paragraph inside a table. A table is a run of consecutive paragraphs that share
/// `table`, ordered by (row, col); a cell may hold several paragraphs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Cell {
    pub table: u32,
    pub row: u16,
    pub col: u16,
    /// Background of the cell (0xRRGGBB); `None` = none.
    pub shade: Option<u32>,
    /// Columns and rows the cell covers (merged cells); the covered positions have no cell of
    /// their own.
    pub colspan: u8,
    pub rowspan: u8,
}

impl Cell {
    /// The cell at (`row`, `col`) of `table`, without a background.
    pub fn at(table: u32, row: u16, col: u16) -> Self {
        Self {
            table,
            row,
            col,
            shade: None,
            colspan: 1,
            rowspan: 1,
        }
    }

    /// First row after the cell.
    pub fn row_end(&self) -> u16 {
        self.row + u16::from(self.rowspan.max(1))
    }

    /// First column after the cell.
    pub fn col_end(&self) -> u16 {
        self.col + u16::from(self.colspan.max(1))
    }

    /// The cell covers the position (`row`, `col`).
    pub fn covers(&self, row: u16, col: u16) -> bool {
        (self.row..self.row_end()).contains(&row) && (self.col..self.col_end()).contains(&col)
    }
}

/// How text goes around a floating picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Wrap {
    /// Text runs along the sides of the picture (it is laid out in the width that is left).
    Square,
    /// Text stops above the picture and goes on below it.
    TopBottom,
    /// The picture is drawn over the text; the text does not move.
    Front,
    /// The picture is drawn under the text; the text does not move.
    Behind,
}

/// A picture placed freely on the page instead of in the line of text. `x`/`y` (px) are the
/// picture's top-left corner from the top-left corner of the text area of the page where its
/// paragraph is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Float {
    pub x: f32,
    pub y: f32,
    pub wrap: Wrap,
}

/// A picture that fills the paragraph (its text is empty): index into `Document::images`, the
/// displayed size in px (96 dpi) and, for a floating picture, where it is on the page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageRef {
    pub id: u32,
    /// Displayed size (after rotating and cropping).
    pub width: f32,
    pub height: f32,
    pub float: Option<Float>,
    /// Quarter turns clockwise (0..=3).
    pub rotation: u8,
    /// What is cut off the displayed picture.
    pub crop: Crop,
}

/// The parts cut off the sides of a picture, as fractions of its (rotated) size.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Crop {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Crop {
    pub fn is_none(&self) -> bool {
        *self == Self::default()
    }

    /// Fraction of the uncropped picture that stays visible, horizontally and vertically.
    pub fn visible(&self) -> (f32, f32) {
        (
            (1.0 - self.left - self.right).max(0.05),
            (1.0 - self.top - self.bottom).max(0.05),
        )
    }

    /// The same cut after the picture is turned a quarter clockwise.
    pub fn turned(self) -> Self {
        Self {
            left: self.bottom,
            top: self.left,
            right: self.top,
            bottom: self.right,
        }
    }
}

impl ImageRef {
    /// A picture in the line of text.
    pub fn inline(id: u32, width: f32, height: f32) -> Self {
        Self {
            id,
            width,
            height,
            float: None,
            rotation: 0,
            crop: Crop::default(),
        }
    }
}

/// Most tab stops a paragraph can have.
pub const MAX_TABS: usize = 20;

/// The spacing and indent values edited together in the paragraph dialog.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParaMetrics {
    /// Left indent in points (for a list item: the part on top of its level).
    pub left_pt: u16,
    pub first_line: u8,
    pub space_before: Option<u8>,
    pub space_after: Option<u8>,
    pub line_spacing: Option<f32>,
    pub tabs: Vec<u16>,
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
    /// Left indent on top of `indent` steps, in points.
    pub left_extra: u8,
    /// Extra indent of the first line in points.
    pub first_line: u8,
    /// Tab stops in points from the left edge of the text (sorted); past the last one the
    /// default stops (every 0.5 inch) apply.
    pub tabs: Vec<u16>,
    /// Gap above / below the paragraph in points; `None` = the style's default.
    pub space_before: Option<u8>,
    pub space_after: Option<u8>,
    /// The paragraph starts on a new page.
    pub page_break: bool,
    /// Set for paragraphs that belong to a table cell.
    pub cell: Option<Cell>,
    /// Set for picture paragraphs.
    pub image: Option<ImageRef>,
    /// Line of the table of contents (replaced when the contents are updated).
    pub toc: bool,
}

// Sizes hash by their bits (an exact match is what the layout cache needs).
impl std::hash::Hash for ImageRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.id, self.width.to_bits(), self.height.to_bits()).hash(state);
        self.float
            .map(|f| (f.x.to_bits(), f.y.to_bits(), f.wrap))
            .hash(state);
        self.rotation.hash(state);
        [
            self.crop.left,
            self.crop.top,
            self.crop.right,
            self.crop.bottom,
        ]
        .map(f32::to_bits)
        .hash(state);
    }
}

impl std::hash::Hash for ParaStyle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.align.hash(state);
        self.named.hash(state);
        self.quote.hash(state);
        self.list.hash(state);
        self.indent.hash(state);
        self.line_spacing.map(f32::to_bits).hash(state);
        self.left_extra.hash(state);
        self.first_line.hash(state);
        self.tabs.hash(state);
        self.page_break.hash(state);
        self.space_before.hash(state);
        self.space_after.hash(state);
        self.cell.hash(state);
        self.image.hash(state);
        self.toc.hash(state);
    }
}
