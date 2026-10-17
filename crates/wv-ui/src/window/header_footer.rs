use gpui_kit::*;
use wv_core::{Align, Band};
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius, space};
use crate::ui_kit::{
    ButtonKind, Icon, LineEdit, Outcome, Tip, button, icon_button, pop_in, track_bounds,
};

pub enum HeaderFooterEvent {
    Set(Band, Band),
}

const FIELD_WIDTH: f32 = 280.0;

/// Button with a popover for the header and footer text (`{page}` = page number).
pub struct HeaderFooterEditor {
    lang: Lang,
    open: bool,
    /// Bounds of the open panel (see `track_bounds`).
    panel_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    epoch: u64,
    current: (Band, Band),
    /// Alignment chosen in the open popover.
    aligns: (Align, Align),
    header: LineEdit,
    footer: LineEdit,
    header_focus: FocusHandle,
    footer_focus: FocusHandle,
}

impl EventEmitter<HeaderFooterEvent> for HeaderFooterEditor {}

impl HeaderFooterEditor {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            open: false,
            panel_bounds: Default::default(),
            epoch: 0,
            current: Default::default(),
            aligns: (Align::Center, Align::Center),
            header: LineEdit::default(),
            footer: LineEdit::default(),
            header_focus: cx.focus_handle(),
            footer_focus: cx.focus_handle(),
        }
    }

    /// Texts of the document (pre-fill when the popover opens; ignored while it is open).
    pub fn set_current(&mut self, header: &Band, footer: &Band) {
        if !self.open {
            self.current = (header.clone(), footer.clone());
        }
    }

    fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = !self.open;
        if self.open {
            self.epoch += 1;
            self.header.set(&self.current.0.text, false);
            self.footer.set(&self.current.1.text, false);
            self.aligns = (self.current.0.align, self.current.1.align);
            window.focus(&self.header_focus, cx);
        }
        cx.notify();
    }

    fn apply(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        let header = Band::new(self.header.text()).aligned(self.aligns.0);
        let footer = Band::new(self.footer.text()).aligned(self.aligns.1);
        self.current = (header.clone(), footer.clone());
        cx.emit(HeaderFooterEvent::Set(header, footer));
        cx.notify();
    }

    fn switch_field(&mut self, in_header: bool, window: &mut Window, cx: &mut Context<Self>) {
        let target = if in_header {
            &self.footer_focus
        } else {
            &self.header_focus
        };
        window.focus(target, cx);
        cx.notify();
    }

    fn on_key(&mut self, in_header: bool, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let field = if in_header {
            &mut self.header
        } else {
            &mut self.footer
        };
        match field.key(ev, cx) {
            Outcome::Submit => self.apply(cx),
            Outcome::Cancel => {
                self.open = false;
                cx.notify();
            }
            _ => cx.notify(),
        }
        cx.stop_propagation();
    }

    fn panel(&mut self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = cx.global::<Theme>();
        let (surface, border, accent, muted) = (th.surface, th.border, th.accent, th.text_muted);
        let lang = self.lang;
        let focus_header = self.header_focus.is_focused(window);
        let focus_footer = self.footer_focus.is_focused(window);
        let field =
            |id: &'static str,
             edit: &LineEdit,
             focus: &FocusHandle,
             focused: bool,
             in_header: bool,
             cx: &mut Context<Self>| {
                div()
                    .id(id)
                    .track_focus(focus)
                    .key_context("TabField")
                    .on_action(cx.listener(
                        move |this, _: &crate::actions::FieldNext, window, cx| {
                            this.switch_field(in_header, window, cx)
                        },
                    ))
                    .on_key_down(cx.listener(move |this, ev: &KeyDownEvent, _, cx| {
                        this.on_key(in_header, ev, cx)
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            window.focus(
                                if in_header {
                                    &this.header_focus
                                } else {
                                    &this.footer_focus
                                },
                                cx,
                            )
                        }),
                    )
                    .w(px(FIELD_WIDTH))
                    .h(px(32.))
                    .px(space::md())
                    .flex()
                    .items_center()
                    .rounded(radius::md())
                    .border_1()
                    .border_color(if focused { accent } else { border })
                    .text_sm()
                    .cursor_text()
                    .child(edit.view(FIELD_WIDTH - 24.0, window, cx))
            };
        let aligns = self.aligns;
        let label = |key: Key, in_header: bool, cx: &mut Context<Self>| {
            let current = if in_header { aligns.0 } else { aligns.1 };
            let buttons = [
                ("l", Icon::AlignLeft, Align::Left),
                ("c", Icon::AlignCenter, Align::Center),
                ("r", Icon::AlignRight, Align::Right),
            ]
            .map(|(id, icon, align)| {
                icon_button(
                    ElementId::from(SharedString::from(format!("band-align-{in_header}-{id}"))),
                    icon,
                    current == align,
                    cx,
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    if in_header {
                        this.aligns.0 = align;
                    } else {
                        this.aligns.1 = align;
                    }
                    cx.notify();
                }))
            });
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(div().text_xs().text_color(muted).child(tr(lang, key)))
                .child(div().flex().gap(space::xs()).children(buttons))
        };
        let header = field(
            "band-header",
            &self.header,
            &self.header_focus,
            focus_header,
            true,
            cx,
        );
        let footer = field(
            "band-footer",
            &self.footer,
            &self.footer_focus,
            focus_footer,
            false,
            cx,
        );
        div()
            .id("band-panel")
            .occlude()
            .relative()
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(6.))
            .bg(surface)
            .border_1()
            .border_color(border)
            .rounded(radius::md())
            .shadow_lg()
            .child(track_bounds(self.panel_bounds.clone()))
            .child(label(Key::HeaderLabel, true, cx))
            .child(header)
            .child(label(Key::FooterLabel, false, cx))
            .child(footer)
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(tr(lang, Key::BandHint)),
            )
            .child(
                button("band-apply", tr(lang, Key::Apply), ButtonKind::Primary, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.apply(cx))),
            )
    }
}

impl Render for HeaderFooterEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.lang;
        let button = icon_button("band-btn", Icon::PageHeader, self.open, cx)
            .tip(tr(lang, Key::TipHeaderFooter))
            .on_click(cx.listener(|this, _, window, cx| this.toggle(window, cx)));
        let pop_key = format!("band-pop-{}", self.epoch);
        let menu = self.open.then(|| {
            deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .offset(point(px(0.), px(36.)))
                    .child(pop_in(self.panel(window, cx), pop_key)),
            )
            .with_priority(10)
        });
        div()
            .relative()
            .on_mouse_down_out(cx.listener(|this, ev: &MouseDownEvent, _, cx| {
                if this.open && !this.panel_bounds.get().contains(&ev.position) {
                    this.open = false;
                    cx.notify();
                }
            }))
            .child(button)
            .children(menu)
    }
}
