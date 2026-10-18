use gpui_kit::*;
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius, space};
use crate::ui_kit::{ButtonKind, LineEdit, Outcome, button, pop_in, track_bounds};

/// The name typed for a new style.
pub struct NewStyle(pub String);

/// "New style" button of the styles pane with a small popover: type a name, Enter defines a style
/// from the formatting at the caret.
pub struct StyleNamer {
    lang: Lang,
    open: bool,
    /// Bounds of the open panel (see `track_bounds`).
    panel_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    /// Counts openings so the pop-in animation restarts each time.
    epoch: u64,
    edit: LineEdit,
    focus: FocusHandle,
}

impl EventEmitter<NewStyle> for StyleNamer {}

impl StyleNamer {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            open: false,
            panel_bounds: Default::default(),
            epoch: 0,
            edit: LineEdit::default(),
            focus: cx.focus_handle(),
        }
    }

    fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = true;
        self.epoch += 1;
        self.edit.set("", false);
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn on_key(&mut self, ev: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match self.edit.key(ev, cx) {
            Outcome::Submit => {
                let name = self.edit.text().trim().to_string();
                self.open = false;
                if !name.is_empty() {
                    cx.emit(NewStyle(name));
                }
                cx.notify();
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

impl Render for StyleNamer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        let (surface, border, muted, accent) = (t.surface, t.border, t.text_muted, t.accent);
        let lang = self.lang;
        let trigger = button(
            "style-new",
            tr(lang, Key::NewStyle),
            if self.open {
                ButtonKind::Primary
            } else {
                ButtonKind::Default
            },
            cx,
        )
        .on_click(cx.listener(|this, _, window, cx| {
            if this.open {
                this.open = false;
                cx.notify();
            } else {
                this.open(window, cx);
            }
        }));
        let panel = self.open.then(|| {
            let field = div()
                .id("style-name-field")
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
                .child(self.edit.view(190.0, window, cx));
            let body = div()
                .id("style-name-panel")
                .occlude()
                .relative()
                .w(px(240.))
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
                .child(
                    div()
                        .text_xs()
                        .text_color(muted)
                        .child(tr(lang, Key::NewStyleHint)),
                )
                .child(field);
            deferred(
                anchored()
                    .anchor(Anchor::BottomLeft)
                    .offset(point(px(0.), px(-6.)))
                    .child(pop_in(body, format!("style-name-pop-{}", self.epoch))),
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
            .child(trigger)
            .children(panel)
    }
}
