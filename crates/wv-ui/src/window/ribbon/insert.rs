use gpui_kit::*;
use wv_i18n::{Key, Lang, t};

use crate::ui_kit::{Icon, block_group, icon_button};
use crate::window::state::MainWindow;

pub fn groups(lang: Lang, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let objects = block_group(
        t(lang, Key::GroupObjects),
        vec![
            icon_button("ins-image", Icon::Image, false, cx).into_any_element(),
            icon_button("ins-table", Icon::Table, false, cx).into_any_element(),
            icon_button("ins-link", Icon::Link, false, cx).into_any_element(),
        ],
        cx,
    );
    vec![objects.into_any_element()]
}
