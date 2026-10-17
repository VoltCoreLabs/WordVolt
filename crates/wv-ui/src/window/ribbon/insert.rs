use gpui_kit::*;
use wv_i18n::{Key, t};

use crate::ui_kit::{ButtonKind, Icon, Tip, block_group, button, icon_button};
use crate::window::state::MainWindow;

/// Insert tab: pictures (soon) and tables.
pub fn groups(this: &MainWindow, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let in_table = this.editor.read(cx).core.table_info().is_some();
    this.table_edit_menu
        .update(cx, |menu, _| menu.set_active(in_table));
    let objects = block_group(
        t(this.lang, Key::GroupObjects),
        vec![
            icon_button("ins-image", Icon::Image, false, cx)
                .tip(t(this.lang, Key::TipImage))
                .on_click(cx.listener(|this, _, window, cx| this.pick_and_insert_image(window, cx)))
                .into_any_element(),
            this.table_menu.clone().into_any_element(),
            this.table_edit_menu.clone().into_any_element(),
        ],
        cx,
    );
    let (header, footer, numbered) = {
        let doc = &this.editor.read(cx).core.doc;
        (
            doc.header.clone(),
            doc.footer.clone(),
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
    let mut groups = vec![objects.into_any_element(), bands.into_any_element()];
    if this.editor.read(cx).core.image_at_caret().is_some() {
        let picture = block_group(
            t(this.lang, Key::GroupPicture),
            vec![
                button("pic-smaller", "−", ButtonKind::Default, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.scale_image(0.8))))
                    .into_any_element(),
                button("pic-larger", "+", ButtonKind::Default, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.scale_image(1.25))))
                    .into_any_element(),
                button(
                    "pic-fit",
                    t(this.lang, Key::PicFit),
                    ButtonKind::Default,
                    cx,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.edit(cx, |c| c.set_image_width(wv_core::MAX_IMAGE_WIDTH))
                }))
                .into_any_element(),
            ],
            cx,
        );
        groups.push(picture.into_any_element());
    }
    groups
}
