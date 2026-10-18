use gpui_kit::*;
use wv_i18n::{Key, t};

use crate::ui_kit::{ButtonKind, Icon, Tip, block_group, button, icon_button};
use crate::window::state::MainWindow;

/// Layout tab: paper size, margins and orientation of the document's pages.
pub fn groups(this: &MainWindow, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let spec = this.editor.read(cx).core.page_here();
    let landscape = spec.is_landscape();
    this.page_setup
        .update(cx, |editor, _| editor.set_current(spec));
    let orientation = |id: &'static str,
                       icon: Icon,
                       tip: Key,
                       to_landscape: bool,
                       cx: &mut Context<MainWindow>| {
        icon_button(id, icon, landscape == to_landscape, cx)
            .tip(t(this.lang, tip))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.edit(cx, move |c| {
                    c.set_page(c.page_here().with_landscape(to_landscape))
                })
            }))
            .into_any_element()
    };
    let page = block_group(
        t(this.lang, Key::GroupPage),
        vec![
            this.paper_menu.clone().into_any_element(),
            this.margins_menu.clone().into_any_element(),
            this.columns_menu.clone().into_any_element(),
            this.page_setup.clone().into_any_element(),
            orientation(
                "page-portrait",
                Icon::PagePortrait,
                Key::TipPortrait,
                false,
                cx,
            ),
            orientation(
                "page-landscape",
                Icon::PageLandscape,
                Key::TipLandscape,
                true,
                cx,
            ),
        ],
        cx,
    );
    let sections = block_group(
        t(this.lang, Key::GroupSections),
        vec![
            button(
                "section-break",
                t(this.lang, Key::SectionBreak),
                ButtonKind::Default,
                cx,
            )
            .tip(t(this.lang, Key::TipSectionBreak))
            .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.insert_section_break())))
            .into_any_element(),
            button(
                "section-unbreak",
                t(this.lang, Key::RemoveSectionBreak),
                ButtonKind::Default,
                cx,
            )
            .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.remove_section_break())))
            .into_any_element(),
        ],
        cx,
    );
    vec![page.into_any_element(), sections.into_any_element()]
}
