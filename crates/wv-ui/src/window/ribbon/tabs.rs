use gpui_kit::*;

use crate::theme::space;
use wv_i18n::{Key, t};

use crate::ui_kit::{Icon, Tip, icon_button, pill_tabs, tab};
use crate::window::state::{MainWindow, Tab};

/// Tabs on the left; collapse-ribbon and side-panel toggles on the right.
pub fn tab_bar(this: &MainWindow, cx: &mut Context<MainWindow>) -> Div {
    let tabs = pill_tabs(
        &this.tab_pills,
        Tab::ALL.iter().position(|t| *t == this.tab).unwrap_or(0),
        Tab::ALL.map(|t| {
            tab(t.label(this.lang), this.tab == t, cx).on_click(cx.listener(
                move |this, _, _, cx| {
                    this.tab = t;
                    this.ribbon_open = true;
                    cx.notify();
                },
            ))
        }),
        cx,
    );
    let collapse = if this.ribbon_open {
        Icon::CaretUp
    } else {
        Icon::CaretDown
    };
    div()
        .flex()
        .items_center()
        .gap(space::xs())
        .child(tabs)
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .gap(space::sm())
                .child(
                    icon_button("toggle-panel", Icon::ListNumbers, this.panel_open, cx)
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
