use gpui_kit::*;

use crate::theme::space;
use wv_i18n::{Key, t};

use crate::ui_kit::{Icon, Tip, icon_button, tab};
use crate::window::state::{MainWindow, Tab};

/// Tabs on the left; collapse-ribbon and side-panel toggles on the right.
pub fn tab_bar(this: &MainWindow, cx: &mut Context<MainWindow>) -> Div {
    let tabs = Tab::ALL.map(|t| {
        tab(t.label(this.lang), this.tab == t, cx).on_click(cx.listener(move |this, _, _, cx| {
            this.tab = t;
            this.ribbon_open = true;
            cx.notify();
        }))
    });
    let collapse = if this.ribbon_open {
        Icon::CaretUp
    } else {
        Icon::CaretDown
    };
    div()
        .flex()
        .items_end()
        .gap(space::xs())
        .children(tabs)
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .gap(space::sm())
                .pb(space::xs())
                .child(
                    icon_button("toggle-panel", Icon::PaintBucket, this.panel_open, cx)
                        .tip(t(this.lang, Key::TipPanel))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.panel_open = !this.panel_open;
                            cx.notify();
                        })),
                )
                .child(
                    icon_button("collapse-ribbon", collapse, false, cx)
                        .tip(t(this.lang, Key::TipRibbon))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.ribbon_open = !this.ribbon_open;
                            cx.notify();
                        })),
                ),
        )
}
