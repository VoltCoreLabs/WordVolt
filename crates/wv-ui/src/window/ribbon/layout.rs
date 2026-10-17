use gpui_kit::*;
use wv_i18n::{Key, t};

use crate::ui_kit::{Icon, Tip, block_group, icon_button};
use crate::window::state::MainWindow;

/// Layout tab: paper size, margins and orientation of the document's pages.
pub fn groups(this: &MainWindow, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let landscape = this.editor.read(cx).core.doc.page.is_landscape();
    let orientation = |id: &'static str,
                       icon: Icon,
                       tip: Key,
                       to_landscape: bool,
                       cx: &mut Context<MainWindow>| {
        icon_button(id, icon, landscape == to_landscape, cx)
            .tip(t(this.lang, tip))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.edit(cx, move |c| {
                    c.set_page(c.doc.page.with_landscape(to_landscape))
                })
            }))
            .into_any_element()
    };
    let page = block_group(
        t(this.lang, Key::GroupPage),
        vec![
            this.paper_menu.clone().into_any_element(),
            this.margins_menu.clone().into_any_element(),
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
    vec![page.into_any_element()]
}
