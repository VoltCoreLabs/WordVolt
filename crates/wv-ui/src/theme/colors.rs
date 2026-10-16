use gpui_kit::{Global, Rgba};

/// Colour tokens of the current theme. Stored as a GPUI global.
#[allow(dead_code)] // tokens are defined ahead of use
pub struct Theme {
    pub bg: Rgba,
    pub surface: Rgba,
    pub surface_alt: Rgba,
    pub border: Rgba,
    pub border_strong: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub accent_subtle: Rgba,
    pub on_accent: Rgba,
    pub danger: Rgba,
    pub page: Rgba,
    /// Background of inline code.
    pub code_bg: Rgba,
    /// Block that hides spoiler text.
    pub spoiler: Rgba,
    /// Highlight of every search match and of the current one.
    pub find_match: Rgba,
    pub find_current: Rgba,
}

impl Global for Theme {}
