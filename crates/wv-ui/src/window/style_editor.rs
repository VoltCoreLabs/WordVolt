use gpui_kit::*;
use wv_core::{Align, UserStyle};
use wv_i18n::{Key, Lang, t as tr};

use super::style_fields::{self, COUNT};
use crate::theme::{Theme, radius, space};
use crate::ui_kit::{ButtonKind, LineEdit, Outcome, button, pop_in, track_bounds};

/// A style edited in the dialog: its id and new definition.
pub struct StyleEdited(pub u32, pub UserStyle);

const LABELS: [Key; COUNT] = [
    Key::StyleFieldName,
    Key::StyleFieldFont,
    Key::StyleFieldSize,
    Key::StyleFieldColor,
    Key::ParaLineSpacing,
    Key::ParaSpaceBefore,
    Key::ParaSpaceAfter,
];
const FIELD_WIDTH: f32 = 150.0;

/// Popover for the parameters of a user style: name, font, size, colour, bold/italic/underline,
/// alignment, line spacing and spacing around paragraphs.
pub struct StyleEditor {
    lang: Lang,
    open: bool,
    id: u32,
    style: UserStyle,
    panel_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    epoch: u64,
    edits: [LineEdit; COUNT],
    focus: [FocusHandle; COUNT],
}

impl EventEmitter<StyleEdited> for StyleEditor {}

impl StyleEditor {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            open: false,
            id: 0,
            style: UserStyle::default(),
            panel_bounds: Default::default(),
            epoch: 0,
            edits: Default::default(),
            focus: std::array::from_fn(|_| cx.focus_handle()),
        }
    }

    /// Opens the dialog for style `id`.
    pub fn open(&mut self, id: u32, style: UserStyle, window: &mut Window, cx: &mut Context<Self>) {
        self.open = true;
        self.epoch += 1;
        self.id = id;
        for (edit, text) in self.edits.iter_mut().zip(style_fields::texts(&style)) {
            edit.set(&text, false);
        }
        self.style = style;
        window.focus(&self.focus[0], cx);
        cx.notify();
    }

    fn typed(&self) -> UserStyle {
        let texts: [String; COUNT] = std::array::from_fn(|i| self.edits[i].text().to_string());
        style_fields::apply(&self.style, &texts)
    }

    fn apply(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        cx.emit(StyleEdited(self.id, self.typed()));
        cx.notify();
    }

    /// Flips a toggle (what is typed stays: it is kept in `style` first).
    fn flip(&mut self, change: impl FnOnce(&mut UserStyle), cx: &mut Context<Self>) {
        let mut typed = self.typed();
        change(&mut typed);
        self.style = typed;
        cx.notify();
    }

    fn on_key(&mut self, index: usize, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        match self.edits[index].key(ev, cx) {
            Outcome::Submit => self.apply(cx),
            Outcome::Cancel => {
                self.open = false;
                cx.notify();
            }
            _ => cx.notify(),
        }
        cx.stop_propagation();
    }

    fn field(&self, index: usize, window: &Window, cx: &mut Context<Self>) -> Div {
        let th = cx.global::<Theme>();
        let (border, accent, muted) = (th.border, th.accent, th.text_muted);
        let focused = self.focus[index].is_focused(window);
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(tr(self.lang, LABELS[index])),
            )
            .child(
                div()
                    .id(("style-field", index))
                    .track_focus(&self.focus[index])
                    .key_context("TabField")
                    .on_action(cx.listener(
                        move |this, _: &crate::actions::FieldNext, window, cx| {
                            window.focus(&this.focus[(index + 1) % COUNT], cx);
                            cx.notify();
                        },
                    ))
                    .on_key_down(
                        cx.listener(move |this, ev: &KeyDownEvent, _, cx| {
                            this.on_key(index, ev, cx)
                        }),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            window.focus(&this.focus[index], cx)
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
                    .child(self.edits[index].view(FIELD_WIDTH - 24.0, window, cx)),
            )
    }

    fn toggles(&self, cx: &mut Context<Self>) -> Div {
        let kind = |on: bool| {
            if on {
                ButtonKind::Primary
            } else {
                ButtonKind::Default
            }
        };
        let s = &self.style;
        let align = match s.align {
            None => "—",
            Some(Align::Left) => "◧",
            Some(Align::Center) => "▥",
            Some(Align::Right) => "◨",
            Some(Align::Justify) => "▤",
        };
        div()
            .flex()
            .gap(px(6.))
            .child(
                button("style-bold", "B", kind(s.bold), cx)
                    .on_click(cx.listener(|t, _, _, cx| t.flip(|s| s.bold = !s.bold, cx))),
            )
            .child(
                button("style-italic", "I", kind(s.italic), cx)
                    .on_click(cx.listener(|t, _, _, cx| t.flip(|s| s.italic = !s.italic, cx))),
            )
            .child(
                button("style-underline", "U", kind(s.underline), cx).on_click(
                    cx.listener(|t, _, _, cx| t.flip(|s| s.underline = !s.underline, cx)),
                ),
            )
            .child(
                button("style-align", align, ButtonKind::Default, cx).on_click(cx.listener(
                    |t, _, _, cx| t.flip(|s| s.align = style_fields::next_align(s.align), cx),
                )),
            )
    }
}

impl Render for StyleEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = cx.global::<Theme>();
        let (surface, border, muted) = (th.surface, th.border, th.text_muted);
        let lang = self.lang;
        let panel = self.open.then(|| {
            let row = |a: usize, b: usize, this: &Self, cx: &mut Context<Self>| {
                div()
                    .flex()
                    .gap(space::md())
                    .child(this.field(a, window, cx))
                    .child(this.field(b, window, cx))
            };
            let body = div()
                .id("style-editor-panel")
                .occlude()
                .relative()
                .p(px(12.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .bg(surface)
                .border_1()
                .border_color(border)
                .rounded(radius::md())
                .shadow_lg()
                .child(track_bounds(self.panel_bounds.clone()))
                .child(row(0, 1, self, cx))
                .child(row(2, 3, self, cx))
                .child(self.toggles(cx))
                .child(row(4, 5, self, cx))
                .child(self.field(6, window, cx))
                .child(
                    div()
                        .text_xs()
                        .text_color(muted)
                        .child(tr(lang, Key::StyleEditHint)),
                )
                .child(
                    button("style-apply", tr(lang, Key::Apply), ButtonKind::Primary, cx)
                        .on_click(cx.listener(|this, _, _, cx| this.apply(cx))),
                );
            deferred(
                anchored()
                    .anchor(Anchor::BottomLeft)
                    .offset(point(px(0.), px(-6.)))
                    .child(pop_in(body, format!("style-editor-pop-{}", self.epoch))),
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
            .children(panel)
    }
}
