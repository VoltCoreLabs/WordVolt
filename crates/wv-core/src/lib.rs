//! Document model, editing, undo/redo. No UI dependencies. See README.md.

pub mod document;
pub mod editor;
pub mod history;
pub mod position;
pub mod search;
pub mod style;

pub use document::{Document, Paragraph, Run};
pub use editor::{CaseKind, Editor, TableInfo};
pub use position::{Position, Selection};
pub use search::SearchOptions;
pub use style::{Align, Cell, CharStyle, ListKind, NamedStyle, ParaStyle, Script};
