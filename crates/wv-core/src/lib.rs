//! Document model, editing, undo/redo. No UI dependencies. See README.md.

pub mod document;
pub mod editor;
pub mod history;
pub mod position;
pub mod style;

pub use document::{Document, Paragraph, Run};
pub use editor::Editor;
pub use position::{Position, Selection};
pub use style::{Align, CharStyle, ParaStyle};
