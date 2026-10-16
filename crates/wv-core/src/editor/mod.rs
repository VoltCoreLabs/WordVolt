//! Editor = document + selection + history.

mod core;
mod movement;
mod style_ops;
#[cfg(test)]
mod tests;
mod text_ops;

pub use core::Editor;
