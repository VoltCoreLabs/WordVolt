use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

use gpui_kit::*;
use wv_core::{Band, BandVariants, Position};
use wv_layout::DocLayout;

use super::measure::MeasureCache;

/// Everything needed to paint one page; cheap to clone into the canvas closure.
pub struct PageView {
    pub layout: Rc<DocLayout>,
    pub measure: Rc<RefCell<MeasureCache>>,
    pub page: usize,
    pub selection: (Position, Position),
    pub head: Position,
    pub focused: bool,
    pub text: Rgba,
    pub selection_color: Rgba,
    /// A block of table cells is selected (drawn as cells instead of text lines).
    pub cell_selection: Option<wv_core::CellSelection>,
    pub epoch: Rc<Cell<Instant>>,
    pub code_bg: Rgba,
    /// Background of text that has a comment.
    pub comment_bg: Rgba,
    pub spoiler_color: Rgba,
    pub link_color: Rgba,
    pub bar_color: Rgba,
    /// Search matches (all, and which one is current) with their colours.
    pub find: Rc<Vec<(Position, Position)>>,
    pub find_current: Option<usize>,
    pub find_color: Rgba,
    pub find_current_color: Rgba,
    /// Table borders.
    pub table_color: Rgba,
    /// Wavy underline of misspelled words.
    pub misspell_color: Rgba,
    /// The selected picture's handles cut its sides.
    pub cropping: bool,
    /// Header and footer text, and their colour.
    pub bands: Rc<(Band, Band, BandVariants)>,
    /// Pictures of the document by `ImageRef::id`.
    pub images: Rc<super::image_paint::PictureSet>,
    pub band_color: Rgba,
    /// Display scale (1.0 = 100%).
    pub zoom: f32,
    /// Draw the non-printing characters (¶, dots, arrows).
    pub show_marks: bool,
    /// Paragraphs that end a section.
    pub section_ends: Rc<Vec<usize>>,
}
