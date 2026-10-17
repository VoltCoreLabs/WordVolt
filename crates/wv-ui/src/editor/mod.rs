//! Document editing surface: paged view drawn from the wv-layout engine; keyboard, mouse, scrolling.

mod band_paint;
mod caret;
mod image_paint;
mod keys;
mod locate;
mod measure;
mod mouse;
mod nav;
mod page_paint;
mod resize;
mod scroll;
mod toc;
mod view;

pub use view::EditorView;
