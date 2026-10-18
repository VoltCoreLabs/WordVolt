use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius, space};
use crate::ui_kit::{ButtonKind, LineEdit, Outcome, button, pop_in, track_bounds};

#[derive(Clone)]
pub enum RefEvent {
    /// Bookmark the selection under this name.
    Add(String),
    Delete(u32),
    /// A cross-reference to the bookmark with this name.
    Bookmark(String),
    /// A cross-reference to the heading paragraph.
    Heading(usize),
}

/// What the popover lists: the bookmarks (id, name) and the headings (paragraph, level, text).
#[derive(Default)]
pub struct RefItems {
    pub bookmarks: Vec<(u32, String)>,
    pub headings: Vec<(usize, u8, String)>,
}

/// "Bookmarks" button with a popover: a name field (Enter bookmarks the selection), the bookmarks
/// and the headings of the document; a click on one types a cross-reference to it.
pub struct RefsPanel {
    lang: Lang,
    items: RefItems,
    open: bool,
    /// Bounds of the open panel (see `track_bounds`).
    panel_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    /// Counts openings so the pop-in animation restarts each time.
    epoch: u64,
    edit: LineEdit,
    focus: FocusHandle,
}

impl EventEmitter<RefEvent> for RefsPanel {}

impl RefsPanel {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            items: RefItems::default(),
            open: false,
            panel_bounds: Default::default(),
            epoch: 0,
            edit: LineEdit::default(),
            focus: cx.focus_handle(),
        }
    }

    pub fn set_items(&mut self, items: RefItems) {
        self.items = items;
    }

    fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = true;
        self.epoch += 1;
        self.edit.set("", false);
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn choose(&mut self, event: RefEvent, cx: &mut Context<Self>) {
        self.open = false;
        cx.emit(event);
        cx.notify();
    }

    fn on_key(&mut self, ev: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match self.edit.key(ev, cx) {
            Outcome::Submit => {
                let name = self.edit.text().trim().to_string();
                if name.is_empty() {
                    self.open = false;
                    cx.notify();
                } else {
                    self.choose(RefEvent::Add(name), cx);
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

    fn row(
        &self,
        id: (&'static str, usize),
        label: String,
        indent: u8,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let hover = cx.global::<Theme>().surface_alt;
        div()
            .id(id)
            .h(px(28.))
            .pl(space::sm() + px(f32::from(indent) * 12.0))
            .pr(space::sm())
            .flex()
            .items_center()
            .rounded(radius::sm())
            .text_sm()
            .cursor_pointer()
            .overflow_hidden()
            .whitespace_nowrap()
            .hover(move |s| s.bg(hover))
            .child(label)
    }
}

impl Render for RefsPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        let (surface, border, muted, accent, hover) =
            (t.surface, t.border, t.text_muted, t.accent, t.surface_alt);
        let lang = self.lang;
        let trigger = button(
            "refs-btn",
            "🔖",
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
                .id("refs-name-field")
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
                .child(self.edit.view(260.0, window, cx));
            let bookmarks: Vec<_> = self
                .items
                .bookmarks
                .clone()
                .into_iter()
                .map(|(id, name)| {
                    let target = name.clone();
                    let delete = div()
                        .id(("refs-del", id as usize))
                        .w(px(24.))
                        .h(px(24.))
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .rounded(radius::sm())
                        .text_color(muted)
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover))
                        .child("×")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.choose(RefEvent::Delete(id), cx);
                                cx.stop_propagation();
                            }),
                        );
                    div()
                        .flex()
                        .items_center()
                        .child(
                            self.row(("refs-bm", id as usize), name, 0, cx)
                                .flex_1()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.choose(RefEvent::Bookmark(target.clone()), cx)
                                    }),
                                ),
                        )
                        .child(delete)
                })
                .collect();
            let headings: Vec<_> = self
                .items
                .headings
                .clone()
                .into_iter()
                .map(|(para, level, text)| {
                    self.row(("refs-h", para), text, level.saturating_sub(1), cx)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.choose(RefEvent::Heading(para), cx)
                            }),
                        )
                })
                .collect();
            let caption = |key: Key| div().text_xs().text_color(muted).child(tr(lang, key));
            let body = div()
                .id("refs-panel")
                .occlude()
                .relative()
                .w(px(320.))
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
                .child(caption(Key::BookmarkHint))
                .child(field)
                .when(!bookmarks.is_empty(), |d| {
                    d.child(caption(Key::BookmarkList)).children(bookmarks)
                })
                .when(!headings.is_empty(), |d| {
                    d.child(caption(Key::HeadingList)).child(
                        div()
                            .id("refs-headings")
                            .max_h(px(180.))
                            .overflow_y_scroll()
                            .flex()
                            .flex_col()
                            .children(headings),
                    )
                });
            deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .offset(point(px(0.), px(36.)))
                    .child(pop_in(body, format!("refs-pop-{}", self.epoch))),
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
