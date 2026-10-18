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
    let (header, footer, variants, numbered) = {
        let core = &this.editor.read(cx).core;
        let (header, footer) = core.bands_here();
        let numbered = footer.has_page_number();
        (header, footer, core.doc.variants.clone(), numbered)
    };
    this.band_editor.update(cx, |editor, _| {
        editor.set_current(&header, &footer, &variants)
    });
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
    let toc = block_group(
        t(this.lang, Key::GroupToc),
        vec![
            icon_button("ins-toc", Icon::ListNumbers, false, cx)
                .tip(t(this.lang, Key::TipToc))
                .on_click(cx.listener(|this, _, window, cx| {
                    let title = t(this.lang, Key::TocTitle);
                    this.editor
                        .update(cx, |view, cx| view.update_toc(title, window, cx));
                    this.editor.update(cx, |view, cx| view.request_focus(cx));
                }))
                .into_any_element(),
        ],
        cx,
    );
    let pages = block_group(
        t(this.lang, Key::GroupPages),
        vec![
            icon_button("ins-page-break", Icon::PageBreak, false, cx)
                .tip(t(this.lang, Key::TipPageBreak))
                .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.insert_page_break())))
                .into_any_element(),
            icon_button("ins-blank-page", Icon::PageSingle, false, cx)
                .tip(t(this.lang, Key::TipBlankPage))
                .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.insert_blank_page())))
                .into_any_element(),
        ],
        cx,
    );
    let at_caret = {
        let core = &this.editor.read(cx).core;
        core.comment_at(core.sel.head).and_then(|id| {
            let c = core.doc.comments.iter().find(|c| c.id == id)?;
            let date = c.date.replace('T', " ");
            let meta = format!("{} {}", c.author, date.trim_end_matches('Z'));
            Some(((id, c.text.clone()), meta.trim().to_string()))
        })
    };
    let (at_caret, meta) = at_caret.unzip();
    this.comment_editor
        .update(cx, |e, _| e.set_current(at_caret, meta.unwrap_or_default()));
    let notes = block_group(
        t(this.lang, Key::GroupNotes),
        vec![
            button("ins-footnote", "AB¹", ButtonKind::Default, cx)
                .tip(t(this.lang, Key::TipFootnote))
                .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.insert_footnote())))
                .into_any_element(),
            this.comment_editor.clone().into_any_element(),
        ],
        cx,
    );
    let mut groups = vec![
        pages.into_any_element(),
        objects.into_any_element(),
        bands.into_any_element(),
        toc.into_any_element(),
        notes.into_any_element(),
    ];
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
