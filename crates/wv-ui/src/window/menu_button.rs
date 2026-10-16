use gpui_kit::*;
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius};
use crate::ui_kit::{Icon, Tip, icon_button, pop_in};

/// A menu entry: a translated label or plain text (numbers).
#[derive(Clone, Copy)]
pub enum MenuItem {
    Tr(Key),
    Text(&'static str),
}

pub enum MenuEvent {
    /// Index of the chosen entry.
    Picked(usize),
}

/// Icon button that opens a list of commands.
pub struct MenuButton {
    id: &'static str,
    glyph: Icon,
    tip: Key,
    items: Vec<MenuItem>,
    lang: Lang,
    active: bool,
    open: bool,
    /// Counts openings so the pop-in animation restarts each time.
    epoch: u64,
}

impl EventEmitter<MenuEvent> for MenuButton {}

impl MenuButton {
    pub fn new(
        id: &'static str,
        glyph: Icon,
        tip: Key,
        items: Vec<MenuItem>,
        lang: Lang,
        _: &mut Context<Self>,
    ) -> Self {
        Self {
            id,
            glyph,
            tip,
            items,
            lang,
            active: false,
            open: false,
            epoch: 0,
        }
    }

    /// Lights the button up (e.g. the selection is displayed in capitals).
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl Render for MenuButton {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (lang, id) = (self.lang, self.id);
        let button = icon_button(
            SharedString::from(format!("{id}-btn")),
            self.glyph,
            self.active || self.open,
            cx,
        )
        .tip(tr(lang, self.tip))
        .on_click(cx.listener(|this, _, _, cx| {
            this.open = !this.open;
            this.epoch += 1;
            cx.notify();
        }));
        let menu = self.open.then(|| {
            let th = cx.global::<Theme>();
            let (surface, border, hover) = (th.surface, th.border, th.surface_alt);
            let rows: Vec<_> = self
                .items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    div()
                        .id((SharedString::from(format!("{id}-item")), index))
                        .h(px(30.))
                        .px(px(10.))
                        .flex()
                        .items_center()
                        .rounded(radius::sm())
                        .text_sm()
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.open = false;
                                cx.emit(MenuEvent::Picked(index));
                                cx.notify();
                            }),
                        )
                        .child(match item {
                            MenuItem::Tr(key) => tr(lang, *key).to_string(),
                            MenuItem::Text(text) => (*text).to_string(),
                        })
                })
                .collect();
            let panel = div()
                .id(SharedString::from(format!("{id}-menu")))
                .occlude()
                .min_w(px(240.))
                .p(px(4.))
                .flex()
                .flex_col()
                .bg(surface)
                .border_1()
                .border_color(border)
                .rounded(radius::md())
                .shadow_lg()
                .children(rows);
            deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .offset(point(px(0.), px(36.)))
                    .child(pop_in(panel, format!("{id}-pop-{}", self.epoch))),
            )
            .with_priority(10)
        });
        div()
            .relative()
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                if this.open {
                    this.open = false;
                    cx.notify();
                }
            }))
            .child(button)
            .children(menu)
    }
}
