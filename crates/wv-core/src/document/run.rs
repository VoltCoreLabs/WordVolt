use crate::style::CharStyle;

/// A piece of text with uniform formatting.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Run {
    pub text: String,
    pub style: CharStyle,
}

impl Run {
    pub fn new(text: impl Into<String>, style: CharStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}
