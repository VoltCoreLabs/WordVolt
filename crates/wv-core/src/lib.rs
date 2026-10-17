//! Document model, editing, undo/redo. No UI dependencies. See README.md.

pub mod document;
pub mod editor;
pub mod history;
pub mod position;
pub mod search;
pub mod style;

pub use document::{
    Band, DocSettings, Document, ImageData, ImageFormat, Margins, PageSpec, Paragraph, Run,
};
pub use editor::MAX_IMAGE_WIDTH;
pub use editor::{CaseKind, Editor, Heading, TableInfo, TocEntry};
pub use position::{Position, Selection};
pub use search::SearchOptions;
pub use style::{Align, Cell, CharStyle, ImageRef, ListKind, NamedStyle, ParaStyle, Script};
