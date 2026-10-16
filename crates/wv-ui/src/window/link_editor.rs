use gpui_kit::*;
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius, space};
use crate::ui_kit::{Icon, LineEdit, Outcome, Tip, icon_button};

pub enum LinkEvent {
    /// `None` removes the link.
    Set(Option<String>),
}

/// Link button with a small popover: type an address, Enter applies, an empty address removes the link.
pub struct LinkEditor {
    lang: Lang,
    current: Option<String>,
    open: bool,
    edit: LineEdit,
    focus: FocusHandle,
}

impl EventEmitter<LinkEvent> for LinkEditor {}

impl LinkEditor {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            current: None,
            open: false,
            edit: LineEdit::default(),
            focus: cx.focus_handle(),
        }
    }

    /// Link at the caret/selection (shown as active and pre-filled).
    pub fn set_current(&mut self, link: Option<String>) {
        self.current = link;
    }

    /// Opens the address popover (button or Ctrl+K), pre-filled with the current link.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = true;
        self.edit
            .set(self.current.as_deref().unwrap_or("https://"), true);
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn apply(&mut self, link: Option<String>, cx: &mut Context<Self>) {
        self.open = false;
        self.current = link.clone();
        cx.emit(LinkEvent::Set(link));
        cx.notify();
    }

    fn on_key(&mut self, ev: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match self.edit.key(ev, cx) {
            Outcome::Submit => {
                let address = self.edit.text().trim().to_string();
                self.apply(
                    (!address.is_empty() && address != "https://").then_some(address),
                    cx,
                );
            }
            Outcome::Cancel => {
                self.open = false;
                cx.notify();
            }
            _ => cx.notify(),
        }
        cx.stop_propagation();
    }
}

impl Render for LinkEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        let (surface, border, muted, accent, hover) =
            (t.surface, t.border, t.text_muted, t.accent, t.surface_alt);
        let lang = self.lang;
        let button = icon_button(
            "link-btn",
            Icon::Link,
            self.current.is_some() || self.open,
            cx,
        )
        .tip(tr(lang, Key::TipLink))
        .on_click(cx.listener(|this, _, window, cx| {
            if this.open {
                this.open = false;
                cx.notify();
            } else {
                this.open(window, cx);
            }
        }));
        let panel = self.open.then(|| {
            let remove = self.current.is_some().then(|| {
                div()
                    .id("link-remove")
                    .h(px(28.))
                    .px(space::sm())
                    .flex()
                    .items_center()
                    .rounded(radius::sm())
                    .text_sm()
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| this.apply(None, cx)),
                    )
                    .child(tr(lang, Key::RemoveLink))
            });
            let field = div()
                .id("link-field")
                .track_focus(&self.focus)
                .on_key_down(cx.listener(Self::on_key))
                .h(px(32.))
                .px(space::md())
                .flex()
                .items_center()
                .rounded(radius::md())
                .border_1()
                .border_color(accent)
                .text_sm()
                .cursor_text()
                .child(self.edit.view(240.0, window, cx));
            let body = div()
                .id("link-panel")
                .occlude()
                .w(px(300.))
                .p(px(12.))
                .flex()
                .flex_col()
                .gap(px(6.))
                .bg(surface)
                .border_1()
                .border_color(border)
                .rounded(radius::md())
                .shadow_lg()
                .child(
                    div()
                        .text_xs()
                        .text_color(muted)
                        .child(tr(lang, Key::LinkAddress)),
                )
                .child(field)
                .children(remove);
            deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .offset(point(px(0.), px(36.)))
                    .child(body),
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
            .children(panel)
    }
}
