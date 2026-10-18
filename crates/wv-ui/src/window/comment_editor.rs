use gpui_kit::*;
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius, space};
use crate::ui_kit::{ButtonKind, LineEdit, Outcome, Tip, button, pop_in, track_bounds};

#[derive(Clone)]
pub enum CommentEvent {
    Add(String),
    Edit(u32, String),
    Delete(u32),
}

/// Comment button with a small popover: type the text, Enter adds the comment to the selection (or
/// changes the one at the caret); an existing comment can be deleted.
pub struct CommentEditor {
    lang: Lang,
    current: Option<(u32, String)>,
    /// Author and date of the comment at the caret.
    meta: String,
    open: bool,
    /// Bounds of the open panel (see `track_bounds`).
    panel_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    /// Counts openings so the pop-in animation restarts each time.
    epoch: u64,
    edit: LineEdit,
    focus: FocusHandle,
}

impl EventEmitter<CommentEvent> for CommentEditor {}

impl CommentEditor {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            current: None,
            meta: String::new(),
            open: false,
            panel_bounds: Default::default(),
            epoch: 0,
            edit: LineEdit::default(),
            focus: cx.focus_handle(),
        }
    }

    /// Comment at the caret (shown as active and pre-filled).
    pub fn set_current(&mut self, comment: Option<(u32, String)>, meta: String) {
        self.current = comment;
        self.meta = meta;
    }

    /// Opens the popover, pre-filled with the comment at the caret.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = true;
        self.epoch += 1;
        let text = self.current.as_ref().map_or("", |(_, t)| t.as_str());
        self.edit.set(text, true);
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn apply(&mut self, event: CommentEvent, cx: &mut Context<Self>) {
        self.open = false;
        cx.emit(event);
        cx.notify();
    }

    fn on_key(&mut self, ev: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match self.edit.key(ev, cx) {
            Outcome::Submit => {
                let text = self.edit.text().trim().to_string();
                if text.is_empty() {
                    self.open = false;
                    cx.notify();
                } else {
                    let event = match &self.current {
                        Some((id, _)) => CommentEvent::Edit(*id, text),
                        None => CommentEvent::Add(text),
                    };
                    self.apply(event, cx);
                }
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

impl Render for CommentEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        let (surface, border, muted, accent, hover) =
            (t.surface, t.border, t.text_muted, t.accent, t.surface_alt);
        let lang = self.lang;
        let button = button(
            "comment-btn",
            "💬",
            if self.current.is_some() || self.open {
                ButtonKind::Primary
            } else {
                ButtonKind::Default
            },
            cx,
        )
        .tip(tr(lang, Key::TipComment))
        .on_click(cx.listener(|this, _, window, cx| {
            if this.open {
                this.open = false;
                cx.notify();
            } else {
                this.open(window, cx);
            }
        }));
        let panel = self.open.then(|| {
            let existing = self.current.as_ref().map(|(id, _)| *id);
            let remove = existing.map(|id| {
                div()
                    .id("comment-remove")
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
                        cx.listener(move |this, _, _, cx| this.apply(CommentEvent::Delete(id), cx)),
                    )
                    .child(tr(lang, Key::DeleteComment))
            });
            let field = div()
                .id("comment-field")
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
                .id("comment-panel")
                .occlude()
                .relative()
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
                .child(track_bounds(self.panel_bounds.clone()))
                .child(
                    div()
                        .text_xs()
                        .text_color(muted)
                        .child(if self.meta.is_empty() {
                            tr(lang, Key::CommentText).to_string()
                        } else {
                            self.meta.clone()
                        }),
                )
                .child(field)
                .children(remove);
            deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .offset(point(px(0.), px(36.)))
                    .child(pop_in(body, format!("comment-pop-{}", self.epoch))),
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
            .children(panel)
    }
}
