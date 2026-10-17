//! Text layout and pagination. See README.md.

mod document;
mod line;
mod measure;
mod numbering;
mod page;
mod paragraph;
mod table;

pub use document::{CaretRect, DocLayout, SelectionRect};
pub use line::{Fragment, Line};
pub use measure::{DEFAULT_SIZE_PT, Measurer, base_px, font_px, script_shift};
pub use page::{Page, PageSetup, Slice};
pub use paragraph::{
    INDENT_STEP, ImageBox, LIST_HANGING, Options, ParagraphLayout, QUOTE_INDENT, layout_paragraph,
};
pub use table::{CELL_PADDING, TableBox};

#[cfg(test)]
mod test_support;
