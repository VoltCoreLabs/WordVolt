use std::borrow::Cow;

use gpui_kit::{App, AssetSource, Result, SharedString};

#[derive(rust_embed::RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../assets"]
#[include = "icons/**"]
#[include = "fonts/**"]
struct Embedded;

/// Our assets (Phosphor icons, fonts) with the gpui-kit bundle as fallback
/// (gpui-component needs its own Lucide icons).
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match Embedded::get(path) {
            Some(file) => Ok(Some(file.data)),
            None => gpui_kit::assets::Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut all: Vec<SharedString> = Embedded::iter()
            .filter(|name| name.starts_with(path))
            .map(|name| name.to_string().into())
            .collect();
        all.extend(gpui_kit::assets::Assets.list(path)?);
        Ok(all)
    }
}

/// Registers every bundled font file with the text system.
pub fn register_fonts(cx: &App) {
    let fonts: Vec<Cow<'static, [u8]>> = Embedded::iter()
        .filter(|name| {
            name.starts_with("fonts/") && (name.ends_with(".ttf") || name.ends_with(".otf"))
        })
        .filter_map(|name| Embedded::get(&name).map(|f| f.data))
        .collect();
    if let Err(err) = cx.text_system().add_fonts(fonts) {
        eprintln!("failed to register fonts: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_phosphor_icon() {
        let data = Assets.load("icons/phosphor/text-b.svg").unwrap();
        assert!(data.is_some(), "icon not found");
    }
}
