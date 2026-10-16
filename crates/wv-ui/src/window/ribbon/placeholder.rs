use gpui_kit::*;
use wv_i18n::{Key, Lang, t};

use crate::ui_kit::{Icon, block_group, icon_button};
use crate::window::state::{MainWindow, Tab};

/// Tabs that are not built yet show one example group.
pub fn groups(tab: Tab, lang: Lang, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let (caption, glyph) = match tab {
        Tab::Layout => (t(lang, Key::GroupPage), Icon::Printer),
        _ => (t(lang, Key::GroupZoom), Icon::Search),
    };
    vec![
        block_group(
            caption,
            vec![icon_button("placeholder", glyph, false, cx).into_any_element()],
            cx,
        )
        .into_any_element(),
    ]
}
