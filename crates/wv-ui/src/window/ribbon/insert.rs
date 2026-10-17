use gpui_kit::*;
use wv_i18n::{Key, t};

use crate::ui_kit::{Icon, Tip, block_group, icon_button};
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
    let (header, footer, numbered) = {
        let doc = &this.editor.read(cx).core.doc;
        (
            doc.header.text.clone(),
            doc.footer.text.clone(),
            doc.footer.has_page_number(),
        )
    };
    this.band_editor
        .update(cx, |editor, _| editor.set_current(&header, &footer));
    let page_number = icon_button("ins-page-number", Icon::Hash, numbered, cx)
        .tip(t(this.lang, Key::TipPageNumber))
        .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.toggle_page_number())));
    let bands = block_group(
        t(this.lang, Key::TipHeaderFooter),
        vec![
            this.band_editor.clone().into_any_element(),
            page_number.into_any_element(),
        ],
        cx,
    );
    vec![objects.into_any_element(), bands.into_any_element()]
}
