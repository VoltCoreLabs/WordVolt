//! Text layout and pagination. See README.md.

mod around;
mod band_line;
mod document;
mod layout_cache;
mod leader;
mod line;
mod measure;
mod numbering;
mod page;
mod paginate;
mod paragraph;
mod raster;
mod table;
mod table_page;

pub use band_line::{BandLine, BandPart, band_line};
pub use document::{CaretRect, DocLayout, SelectionRect};
pub use layout_cache::LayoutCache;
pub use leader::leader;
pub use line::{Fragment, Line};
pub use measure::{DEFAULT_SIZE_PT, Measurer, base_px, font_px, script_shift};
pub use page::{Page, PageSetup, PlacedFloat, Slice};
pub use paragraph::{
    INDENT_STEP, ImageBox, LIST_HANGING, Obstacle, Options, ParagraphLayout, QUOTE_INDENT,
    layout_paragraph, layout_paragraph_around,
};
pub use raster::{Strip, silhouette, turned_png, turned_raster};
pub use table::{CELL_PADDING, CellRect, Shade, TableBox};

#[cfg(test)]
mod test_support;
