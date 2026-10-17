use gpui_kit::*;
use wv_i18n::{Key, t};

use crate::ui_kit::{Icon, block_group, icon_button};
use crate::window::state::MainWindow;

/// Insert tab: pictures (soon) and tables.
pub fn groups(this: &MainWindow, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let in_table = this.editor.read(cx).core.table_info().is_some();
    this.table_edit_menu
        .update(cx, |menu, _| menu.set_active(in_table));
    let objects = block_group(
        t(this.lang, Key::GroupObjects),
        vec![
            icon_button("ins-image", Icon::Image, false, cx).into_any_element(),
            this.table_menu.clone().into_any_element(),
            this.table_edit_menu.clone().into_any_element(),
        ],
        cx,
    );
    vec![objects.into_any_element()]
}
