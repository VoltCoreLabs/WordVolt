use gpui_kit::{Font, FontFallbacks, FontFeatures, FontStyle, FontWeight};

/// Default UI font: SF Pro Text if installed, otherwise bundled Inter, then Segoe UI.
/// (SF Pro's licence forbids bundling it, so it is never shipped with the app.)
pub fn ui_font() -> Font {
    Font {
        family: "SF Pro Text".into(),
        features: FontFeatures::default(),
        fallbacks: Some(FontFallbacks::from_fonts(vec![
            "Inter Variable".into(),
            "Inter".into(),
            "Segoe UI".into(),
        ])),
        weight: FontWeight::NORMAL,
        style: FontStyle::Normal,
    }
}
