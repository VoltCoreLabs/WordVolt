//! Editor = document + selection + history.

mod case_ops;
mod core;
mod movement;
mod style_ops;
#[cfg(test)]
mod tests;
mod text_ops;

pub use case_ops::CaseKind;
pub use core::Editor;
