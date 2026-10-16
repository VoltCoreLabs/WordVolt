use gpui_kit::{Global, Rgba};

/// Colour tokens of the current theme. Stored as a GPUI global.
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
}

impl Global for Theme {}
