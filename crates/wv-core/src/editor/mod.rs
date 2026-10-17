//! Editor = document + selection + history.

mod case_ops;
mod core;
mod image_ops;
mod movement;
mod para_ops;
mod search_ops;
mod style_ops;
mod table_ops;
mod table_widths;

pub use table_widths::MIN_COLUMN;
#[cfg(test)]
mod tests;
mod text_ops;
mod toc_ops;

pub use case_ops::CaseKind;
pub use core::Editor;
pub use image_ops::MAX_IMAGE_WIDTH;
pub use table_ops::TableInfo;
pub use toc_ops::{Heading, TocEntry};
