//! Document model, editing, undo/redo. No UI dependencies. See README.md.

pub mod document;
pub mod editor;
pub mod history;
pub mod position;
pub mod search;
pub mod style;
pub mod time;

/// Font of text in a document that names none.
pub const DEFAULT_FONT: &str = "SF Pro Text";
/// Size (pt) of text in a document that names none.
pub const DEFAULT_SIZE_PT: u16 = 12;

pub use document::{
    BAND_DISTANCE, BAND_SIZE_PT, Band, BandVariants, Bookmark, Comment, DocSettings, Document,
    ImageData, ImageFormat, MAX_COLUMNS, MIN_COLUMN, Margins, NOTE_MARK, NumberFormat, PageSpec,
    Paragraph, Revision, Run, TableLook,
};
pub use editor::CellSelection;
pub use editor::MAX_IMAGE_WIDTH;
pub use editor::{CaseKind, Editor, Heading, TableInfo, TocEntry};
pub use position::{Position, Selection};
pub use search::SearchOptions;
pub use style::{
    Align, Cell, Change, CharStyle, Crop, Float, ImageRef, ListKind, MAX_TABS, NamedStyle,
    ParaMetrics, ParaStyle, Script, TabKind, TabStop, UserStyle, Wrap, turned_box,
};
