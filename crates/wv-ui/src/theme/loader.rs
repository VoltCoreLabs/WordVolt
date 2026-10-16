use gpui_kit::{Rgba, rgb};
use serde::Deserialize;

use super::Theme;

const LIGHT: &str = include_str!("../../../../assets/theme/light.toml");

#[derive(Deserialize)]
struct ThemeFile {
    bg: String,
    surface: String,
    surface_alt: String,
    border: String,
    border_strong: String,
    text: String,
    text_muted: String,
    accent: String,
    accent_hover: String,
    accent_subtle: String,
    on_accent: String,
    danger: String,
    page: String,
    code_bg: String,
    spoiler: String,
    find_match: String,
    find_current: String,
}

fn color(hex: &str) -> Result<Rgba, String> {
    let digits = hex.trim().trim_start_matches('#');
    u32::from_str_radix(digits, 16)
        .map(rgb)
        .map_err(|_| format!("bad colour {hex:?}"))
}

impl Theme {
    /// Built-in light theme (the same file that is hot-reloaded in dev).
    pub fn light() -> Self {
        Self::parse(LIGHT).expect("built-in light theme is valid")
    }

    pub fn parse(source: &str) -> Result<Self, String> {
        let f: ThemeFile = toml::from_str(source).map_err(|e| e.to_string())?;
        Ok(Self {
            bg: color(&f.bg)?,
            surface: color(&f.surface)?,
            surface_alt: color(&f.surface_alt)?,
            border: color(&f.border)?,
            border_strong: color(&f.border_strong)?,
            text: color(&f.text)?,
            text_muted: color(&f.text_muted)?,
            accent: color(&f.accent)?,
            accent_hover: color(&f.accent_hover)?,
            accent_subtle: color(&f.accent_subtle)?,
            on_accent: color(&f.on_accent)?,
            danger: color(&f.danger)?,
            page: color(&f.page)?,
            code_bg: color(&f.code_bg)?,
            spoiler: color(&f.spoiler)?,
            find_match: color(&f.find_match)?,
            find_current: color(&f.find_current)?,
        })
    }
}
