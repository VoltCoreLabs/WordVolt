use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_i18n::{Key, Lang, t as tr};

use super::palette_items::{Action, Item, matching};
use crate::theme::{Theme, radius, space};
use crate::ui_kit::{LineEdit, Outcome, pop_in, track_bounds};

/// The user picked a command.
pub struct Picked(pub Action);

/// Most commands the list shows at once.
const ROWS: usize = 10;

/// Search box of the title bar: type to find a command by name, Up/Down and Enter (or a click) run
/// it. The list is a popover without a backdrop, like the command palette of code editors.
pub struct CommandPalette {
    lang: Lang,
    open: bool,
    selected: usize,
    /// Bounds of the open list (see `track_bounds`).
    panel_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    /// Bounds of the box itself: a click on it is not "outside".
    box_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    /// Counts openings so the pop-in animation restarts each time.
    epoch: u64,
    edit: LineEdit,
    focus: FocusHandle,
}

impl EventEmitter<Picked> for CommandPalette {}

impl CommandPalette {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            open: false,
            selected: 0,
            panel_bounds: Default::default(),
            box_bounds: Default::default(),
            epoch: 0,
            edit: LineEdit::default(),
            focus: cx.focus_handle(),
        }
    }

    /// Focuses the box and shows the list (the shortcut, or a click on the box).
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            self.epoch += 1;
            self.edit.set("", false);
            self.selected = 0;
        }
        self.open = true;
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        cx.notify();
    }

    fn found(&self) -> Vec<&'static Item> {
        let lang = self.lang;
        let mut found = matching(self.edit.text(), &|key| tr(lang, key).to_string());
        found.truncate(ROWS);
        found
    }

    fn choose(&mut self, action: Action, cx: &mut Context<Self>) {
        self.open = false;
        cx.emit(Picked(action));
        cx.notify();
    }

    fn on_key(&mut self, ev: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let count = self.found().len();
        match ev.keystroke.key.as_str() {
            "down" => self.selected = (self.selected + 1).min(count.saturating_sub(1)),
            "up" => self.selected = self.selected.saturating_sub(1),
            _ => match self.edit.key(ev, cx) {
                Outcome::Submit => {
                    if let Some(item) = self.found().get(self.selected) {
                        self.choose(item.action, cx);
                    }
                }
                Outcome::Cancel => self.close(cx),
                Outcome::Changed => self.selected = 0,
                _ => {}
            },
        }
        cx.notify();
        cx.stop_propagation();
    }
}

impl Render for CommandPalette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        let (surface, alt, border, muted, accent, subtle) = (
            t.surface,
            t.surface_alt,
            t.border,
            t.text_muted,
            t.accent,
            t.accent_subtle,
        );
        let lang = self.lang;
        let empty = self.edit.text().is_empty();
        let input = div()
            .id("palette-input")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| this.open(window, cx)),
            )
            .relative()
            .w(px(240.))
            .h(px(28.))
            .px(space::md())
            .flex()
            .items_center()
            .rounded(radius::md())
            .border_1()
            .border_color(if self.open { accent } else { border })
            .bg(alt)
            .text_sm()
            .cursor_text()
            .child(track_bounds(self.box_bounds.clone()))
            .when(self.open, |d| d.child(self.edit.view(210.0, window, cx)))
            .when(!self.open || empty, |d| {
                d.child(
                    div()
                        .absolute()
                        .top_0()
                        .left(space::md())
                        .h_full()
                        .flex()
                        .items_center()
                        .text_color(muted)
                        .child(tr(lang, Key::PaletteHint)),
                )
            });
        let panel = self.open.then(|| {
            let found = self.found();
            let rows: Vec<_> = found
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let action = item.action;
                    div()
                        .id(("palette-row", i))
                        .h(px(30.))
                        .px(space::md())
                        .flex()
                        .items_center()
                        .justify_between()
                        .rounded(radius::sm())
                        .text_sm()
                        .cursor_pointer()
                        .when(i == self.selected, |d| d.bg(subtle))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| this.choose(action, cx)),
                        )
                        .child(tr(lang, item.name))
                        .child(div().text_xs().text_color(muted).child(item.shortcut))
                })
                .collect();
            let body = div()
                .id("palette-panel")
                .occlude()
                .relative()
                .w(px(380.))
                .p(px(4.))
                .flex()
                .flex_col()
                .bg(surface)
                .border_1()
                .border_color(border)
                .rounded(radius::md())
                .shadow_lg()
                .child(track_bounds(self.panel_bounds.clone()))
                .children(rows)
                .when(found.is_empty(), |d| {
                    d.child(
                        div()
                            .h(px(30.))
                            .px(space::md())
                            .flex()
                            .items_center()
                            .text_sm()
                            .text_color(muted)
                            .child(tr(lang, Key::PaletteNone)),
                    )
                });
            deferred(
                anchored()
                    .anchor(Anchor::TopRight)
                    .offset(point(px(240.), px(34.)))
                    .child(pop_in(body, format!("palette-pop-{}", self.epoch))),
            )
            .with_priority(10)
        });
        div()
            .relative()
            .on_mouse_down_out(cx.listener(|this, ev: &MouseDownEvent, _, cx| {
                let inside = this.panel_bounds.get().contains(&ev.position)
                    || this.box_bounds.get().contains(&ev.position);
                if this.open && !inside {
                    this.close(cx);
                }
            }))
            .child(input)
            .children(panel)
    }
}
