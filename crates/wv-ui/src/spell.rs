use std::sync::Arc;

use gpui_kit::*;
use wv_spell::Speller;

/// Spell checking state shared by all windows: the checker (once its dictionaries are loaded) and
/// whether it is switched on.
pub struct SpellGlobal {
    pub speller: Option<Arc<Speller>>,
    pub enabled: bool,
    /// The dictionaries are being loaded.
    pub loading: bool,
}

impl Global for SpellGlobal {}

/// The checker, when spell checking is on and loaded.
pub fn active(cx: &App) -> Option<Arc<Speller>> {
    cx.try_global::<SpellGlobal>()
        .filter(|g| g.enabled)
        .and_then(|g| g.speller.clone())
}

pub fn enabled(cx: &App) -> bool {
    cx.try_global::<SpellGlobal>().is_some_and(|g| g.enabled)
}
