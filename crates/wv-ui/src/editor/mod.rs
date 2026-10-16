//! Document editing surface: renders paragraphs, caret, selection; handles keyboard, mouse, scrolling.

mod align;
mod caret;
mod keys;
mod mouse;
mod paragraph;
mod scroll;
mod vertical;
mod view;

pub use view::EditorView;
