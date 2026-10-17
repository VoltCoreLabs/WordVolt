//! Editor = document + selection + history.

mod case_ops;
mod core;
mod movement;
mod para_ops;
mod search_ops;
mod style_ops;
mod table_ops;
#[cfg(test)]
mod tests;
mod text_ops;

pub use case_ops::CaseKind;
pub use core::Editor;
pub use table_ops::TableInfo;
