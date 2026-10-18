//! Editor = document + selection + history.

mod bookmark_ops;
mod case_ops;
mod clipboard_ops;
mod comment_ops;
mod core;
mod footnote_ops;
mod image_ops;
mod ime_ops;
mod movement;
mod para_ops;
mod search_ops;
mod style_ops;
mod table_look;
mod table_merge;

pub use table_merge::CellSelection;
mod section_ops;
mod style_def_ops;
mod table_ops;
mod table_rowcol;
#[cfg(test)]
mod tests;
mod text_ops;
mod toc_ops;
mod track_ops;

pub use case_ops::CaseKind;
pub use core::Editor;
pub use image_ops::MAX_IMAGE_WIDTH;
pub use table_ops::TableInfo;
pub use toc_ops::{Heading, TocEntry};
