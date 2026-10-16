use gpui_kit::*;

use crate::ui_kit::{Icon, block_group, icon_button};
use crate::window::state::{MainWindow, Tab};

/// Tabs that are not built yet show one example group.
pub fn groups(tab: Tab, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let (caption, glyph) = match tab {
        Tab::Layout => ("Page", Icon::Printer),
        _ => ("Zoom", Icon::Search),
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
