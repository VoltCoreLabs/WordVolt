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
    /// Text follows the outline of the picture (its transparent parts let the text in).
    Tight,
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
    /// Size of the picture before it is turned (after cropping).
    pub width: f32,
    pub height: f32,
    pub float: Option<Float>,
    /// Degrees clockwise (0..360) around the centre.
    pub angle: f32,
    /// What is cut off the picture (before it is turned).
    pub crop: Crop,
}

/// Size of the smallest box that holds a `w` x `h` picture turned by `angle` degrees.
pub fn turned_box(w: f32, h: f32, angle: f32) -> (f32, f32) {
    let (sin, cos) = angle.to_radians().sin_cos();
    (w * cos.abs() + h * sin.abs(), w * sin.abs() + h * cos.abs())
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
}

impl ImageRef {
    /// The box the turned picture takes up (what the text is laid out around).
    pub fn bounds(&self) -> (f32, f32) {
        turned_box(self.width, self.height, self.angle)
    }

    /// A picture in the line of text.
    pub fn inline(id: u32, width: f32, height: f32) -> Self {
        Self {
            id,
            width,
            height,
            float: None,
            angle: 0.0,
            crop: Crop::default(),
        }
    }
}

/// How text lines up at a tab stop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TabKind {
    /// The text starts at the stop.
    #[default]
    Left,
    /// The text is centred on the stop.
    Center,
    /// The text ends at the stop.
    Right,
    /// The text is lined up on its decimal separator.
    Decimal,
}

impl TabKind {
    pub const ALL: [TabKind; 4] = [Self::Left, Self::Center, Self::Right, Self::Decimal];

    /// The next kind (what a click on a stop in the ruler cycles through).
    pub fn next(self) -> Self {
        Self::ALL[(Self::ALL.iter().position(|k| *k == self).unwrap_or(0) + 1) % 4]
    }

    /// The name Word uses for it in `w:tab w:val`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::Decimal => "decimal",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "left" | "start" => Some(Self::Left),
            "center" => Some(Self::Center),
            "right" | "end" => Some(Self::Right),
            "decimal" => Some(Self::Decimal),
            _ => None,
        }
    }
}

/// What fills the space a tab leaves before its stop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TabLeader {
    #[default]
    None,
    Dots,
    Hyphens,
    Underscore,
}

impl TabLeader {
    pub const ALL: [TabLeader; 4] = [Self::None, Self::Dots, Self::Hyphens, Self::Underscore];

    /// The next leader (what the paragraph dialog cycles through).
    pub fn next(self) -> Self {
        Self::ALL[(Self::ALL.iter().position(|k| *k == self).unwrap_or(0) + 1) % 4]
    }

    /// The name Word uses for it in `w:tab w:leader`.
    pub fn id(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Dots => "dot",
            Self::Hyphens => "hyphen",
            Self::Underscore => "underscore",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "dot" | "middleDot" => Self::Dots,
            "hyphen" | "heavy" => Self::Hyphens,
            "underscore" => Self::Underscore,
            _ => Self::None,
        }
    }
}

/// A tab stop: its place in points from the left edge of the text, how text lines up there and
/// what fills the gap before it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TabStop {
    pub pos: u16,
    pub kind: TabKind,
    pub leader: TabLeader,
}

impl TabStop {
    pub fn left(pos: u16) -> Self {
        Self {
            pos,
            kind: TabKind::Left,
            leader: TabLeader::None,
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
    pub tabs: Vec<TabStop>,
    /// Right indent in points.
    pub right_pt: u8,
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
    /// Tab stops (sorted by place); past the last one the default stops (every 0.5 inch) apply.
    pub tabs: Vec<TabStop>,
    /// Right indent in points.
    pub right_indent: u8,
    /// Gap above / below the paragraph in points; `None` = the style's default.
    pub space_before: Option<u8>,
    pub space_after: Option<u8>,
    /// The paragraph starts on a new page.
    pub page_break: bool,
    /// Set for paragraphs that belong to a table cell.
    pub cell: Option<Cell>,
    /// Set for picture paragraphs.
    pub image: Option<ImageRef>,
    /// The paragraph belongs to the footnote with this id (it is drawn at the bottom of the page of
    /// the footnote's mark, not in the text).
    pub note: Option<u32>,
    /// Line of the table of contents (replaced when the contents are updated).
    pub toc: bool,
    /// The paragraph ends a section (a section break, next page): the page setup of that section
    /// is `Document::sections[id]`. The paragraphs after the last break use `Document::page`.
    pub section: Option<u32>,
    /// The paragraph has this user style (`Document::styles`).
    pub custom: Option<u32>,
}

// Sizes hash by their bits (an exact match is what the layout cache needs).
impl std::hash::Hash for ImageRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.id, self.width.to_bits(), self.height.to_bits()).hash(state);
        self.float
            .map(|f| (f.x.to_bits(), f.y.to_bits(), f.wrap))
            .hash(state);
        self.angle.to_bits().hash(state);
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
        self.right_indent.hash(state);
        self.page_break.hash(state);
        self.space_before.hash(state);
        self.space_after.hash(state);
        self.cell.hash(state);
        self.image.hash(state);
        self.toc.hash(state);
        self.note.hash(state);
    }
}

#[cfg(test)]
mod leader_tests {
    use super::TabLeader;

    #[test]
    fn leader_ids_round_trip_and_cycle() {
        for l in TabLeader::ALL {
            assert_eq!(TabLeader::from_id(l.id()), l);
        }
        assert_eq!(TabLeader::Underscore.next(), TabLeader::None);
        assert_eq!(TabLeader::from_id("middleDot"), TabLeader::Dots);
    }
}
