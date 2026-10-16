//! Document editing surface: renders paragraphs, caret, selection; handles keyboard and mouse.

mod keys;
mod mouse;
mod paragraph;
mod vertical;
mod view;

pub use view::EditorView;
