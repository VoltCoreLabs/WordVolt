//! Text layout and pagination. See README.md.

mod document;
mod line;
mod measure;
mod page;
mod paragraph;

pub use document::{CaretRect, DocLayout, SelectionRect};
pub use line::{Fragment, Line};
pub use measure::{DEFAULT_SIZE_PT, Measurer, font_px};
pub use page::{Page, PageSetup, Slice};
pub use paragraph::{Options, ParagraphLayout, layout_paragraph};

#[cfg(test)]
mod test_support;
