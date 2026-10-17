use gpui_kit::*;
use wv_core::{Position, SearchOptions};
use wv_i18n::{Key, Lang, match_of, replaced_count, t as tr};

use crate::editor::EditorView;
use crate::theme::{Theme, radius, space};
use crate::ui_kit::{ButtonKind, Icon, LineEdit, Outcome, Tip, block, button, icon_button};

/// Find / replace bar above the document: highlights every match, counts them, steps through them
/// and replaces one or all (one undo step).
pub struct FindBar {
    editor: Entity<EditorView>,
    /// Keeps the matches fresh while the document changes.
    watch: Subscription,
    lang: Lang,
    open: bool,
    show_replace: bool,
    find: LineEdit,
    replace: LineEdit,
    find_focus: FocusHandle,
    replace_focus: FocusHandle,
    match_case: bool,
    whole_word: bool,
    matches: Vec<(Position, Position)>,
    current: Option<usize>,
    /// Result of the last "replace all" shown instead of the counter.
    message: Option<String>,
    /// Document revision the matches were computed for (to refresh them when the text changes).
    revision: u64,
}

impl FindBar {
    pub fn new(editor: Entity<EditorView>, lang: Lang, cx: &mut Context<Self>) -> Self {
        let watch = Self::watch(&editor, cx);
        Self {
            watch,
            editor,
            lang,
            open: false,
            show_replace: false,
            find: LineEdit::default(),
            replace: LineEdit::default(),
            find_focus: cx.focus_handle(),
            replace_focus: cx.focus_handle(),
            match_case: false,
            whole_word: false,
            matches: Vec::new(),
            current: None,
            message: None,
            revision: 0,
        }
    }

    /// Recomputes the matches when the document changes from outside (typing, undo).
    fn watch(editor: &Entity<EditorView>, cx: &mut Context<Self>) -> Subscription {
        cx.observe(editor, |this, editor, cx| {
            let revision = editor.read(cx).core.revision();
            if this.open && revision != this.revision {
                this.revision = revision;
                // An old "replaced N" no longer applies.
                this.message = None;
                this.recompute(false, cx);
            }
        })
    }

    /// Searches another document (the tab changed).
    pub fn set_editor(&mut self, editor: Entity<EditorView>, cx: &mut Context<Self>) {
        self.watch = Self::watch(&editor, cx);
        self.revision = editor.read(cx).core.revision();
        self.editor = editor;
        self.matches.clear();
        self.current = None;
        self.message = None;
        if self.open {
            self.recompute(false, cx);
        }
        cx.notify();
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Ctrl+F (`with_replace` = false) or Ctrl+H. A single-line selection becomes the query.
    pub fn open(&mut self, with_replace: bool, window: &mut Window, cx: &mut Context<Self>) {
        let selected = self.editor.read(cx).core.selected_text();
        let query = if !selected.is_empty() && !selected.contains('\n') {
            selected
        } else {
            self.find.text().to_string()
        };
        self.find.set(&query, true);
        self.open = true;
        self.show_replace |= with_replace;
        self.message = None;
        self.revision = self.editor.read(cx).core.revision();
        self.recompute(true, cx);
        window.focus(&self.find_focus, cx);
        cx.notify();
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        self.matches.clear();
        self.current = None;
        self.editor.update(cx, |view, cx| {
            view.set_find(Vec::new(), None, cx);
            view.request_focus(cx);
        });
        cx.notify();
    }

    fn options(&self) -> SearchOptions {
        SearchOptions {
            match_case: self.match_case,
            whole_word: self.whole_word,
        }
    }

    /// Finds the matches again; with `select` the first match at or after the caret becomes current.
    fn recompute(&mut self, select: bool, cx: &mut Context<Self>) {
        let query = self.find.text().to_string();
        let opts = self.options();
        let (matches, caret) = {
            let view = self.editor.read(cx);
            (
                view.core.find_matches(&query, opts),
                view.core.sel.range().0,
            )
        };
        self.current = if matches.is_empty() {
            None
        } else {
            Some(
                matches
                    .iter()
                    .position(|(start, _)| *start >= caret)
                    .unwrap_or(0),
            )
        };
        self.matches = matches;
        if select {
            self.select_current(cx);
        }
        self.publish(cx);
        cx.notify();
    }

    fn select_current(&mut self, cx: &mut Context<Self>) {
        if let Some((start, end)) = self.current.and_then(|i| self.matches.get(i)).copied() {
            self.editor.update(cx, |view, cx| {
                view.apply(cx, |core| core.select_range(start, end))
            });
        }
    }

    fn publish(&self, cx: &mut Context<Self>) {
        let (matches, current) = (self.matches.clone(), self.current);
        self.editor
            .update(cx, |view, cx| view.set_find(matches, current, cx));
    }

    /// Next (`1`) or previous (`-1`) match, wrapping around.
    pub fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = self.matches.len() as isize;
        if count == 0 {
            return;
        }
        let from = self
            .current
            .map_or(if delta > 0 { -1 } else { 0 }, |c| c as isize);
        self.current = Some((from + delta).rem_euclid(count) as usize);
        self.message = None;
        self.select_current(cx);
        self.publish(cx);
        cx.notify();
    }

    fn replace_one(&mut self, cx: &mut Context<Self>) {
        let Some((start, end)) = self.current.and_then(|i| self.matches.get(i)).copied() else {
            return;
        };
        let text = self.replace.text().to_string();
        self.editor.update(cx, |view, cx| {
            view.apply(cx, |core| core.replace_match(start, end, &text))
        });
        self.revision = self.editor.read(cx).core.revision();
        self.message = None;
        self.recompute(true, cx);
    }

    fn replace_all(&mut self, cx: &mut Context<Self>) {
        let (query, text, opts) = (
            self.find.text().to_string(),
            self.replace.text().to_string(),
            self.options(),
        );
        let count = self.editor.update(cx, |view, cx| {
            let mut count = 0;
            view.apply(cx, |core| count = core.replace_all(&query, &text, opts));
            count
        });
        self.revision = self.editor.read(cx).core.revision();
        self.recompute(false, cx);
        self.message = Some(replaced_count(self.lang, count));
        cx.notify();
    }

    fn on_find_key(&mut self, ev: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key = ev.keystroke.key.as_str();
        if key == "enter" {
            self.step(if ev.keystroke.modifiers.shift { -1 } else { 1 }, cx);
        } else {
            match self.find.key(ev, cx) {
                Outcome::Cancel => self.close(cx),
                Outcome::Changed => {
                    self.message = None;
                    self.recompute(true, cx);
                }
                _ => cx.notify(),
            }
        }
        cx.stop_propagation();
    }

    fn on_replace_key(&mut self, ev: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key = ev.keystroke.key.as_str();
        if key == "enter" {
            self.replace_one(cx);
        } else {
            match self.replace.key(ev, cx) {
                Outcome::Cancel => self.close(cx),
                _ => cx.notify(),
            }
        }
        cx.stop_propagation();
    }

    fn counter(&self) -> String {
        if let Some(message) = &self.message {
            return message.clone();
        }
        match (self.find.text().is_empty(), self.current) {
            (true, _) => String::new(),
            (false, Some(current)) => match_of(self.lang, current + 1, self.matches.len()),
            (false, None) => tr(self.lang, Key::NoMatches).to_string(),
        }
    }
}

impl Render for FindBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.open {
            return div().into_any_element();
        }
        let lang = self.lang;
        let th = cx.global::<Theme>();
        let (border, accent, muted) = (th.border, th.accent, th.text_muted);

        let field = |id: &'static str,
                     edit: &LineEdit,
                     placeholder: Key,
                     focus: &FocusHandle,
                     focused: bool,
                     window: &Window,
                     cx: &App| {
            let content = if edit.text().is_empty() && !focused {
                div()
                    .text_color(muted)
                    .child(tr(lang, placeholder))
                    .into_any_element()
            } else {
                edit.view(260.0, window, cx).into_any_element()
            };
            div()
                .id(id)
                .track_focus(focus)
                .flex_1()
                .h(px(32.))
                .px(space::md())
                .flex()
                .items_center()
                .overflow_hidden()
                .rounded(radius::md())
                .border_1()
                .border_color(if focused { accent } else { border })
                .text_sm()
                .cursor_text()
                .child(content)
        };

        let find_focused = self.find_focus.is_focused(window);
        let replace_focused = self.replace_focus.is_focused(window);
        let find_field = field(
            "find-field",
            &self.find,
            Key::FindPlaceholder,
            &self.find_focus,
            find_focused,
            window,
            cx,
        )
        .key_context("TabField")
        .on_action(
            cx.listener(|this, _: &crate::actions::FieldNext, window, cx| {
                if this.show_replace {
                    window.focus(&this.replace_focus, cx);
                }
            }),
        )
        .on_key_down(cx.listener(Self::on_find_key))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _, window, cx| window.focus(&this.find_focus, cx)),
        );
        let counter = self.counter();
        let top = div()
            .flex()
            .items_center()
            .gap(space::sm())
            .child(find_field)
            .child(
                div()
                    .min_w(px(90.))
                    .text_sm()
                    .text_color(muted)
                    .child(counter),
            )
            .child(
                icon_button("find-prev", Icon::CaretUp, false, cx)
                    .tip(tr(lang, Key::TipFindPrev))
                    .on_click(cx.listener(|this, _, _, cx| this.step(-1, cx))),
            )
            .child(
                icon_button("find-next", Icon::CaretDown, false, cx)
                    .tip(tr(lang, Key::TipFindNext))
                    .on_click(cx.listener(|this, _, _, cx| this.step(1, cx))),
            )
            .child(
                icon_button("find-case", Icon::TextAa, self.match_case, cx)
                    .tip(tr(lang, Key::TipMatchCase))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.match_case = !this.match_case;
                        this.recompute(true, cx);
                    })),
            )
            .child(
                icon_button("find-word", Icon::WholeWord, self.whole_word, cx)
                    .tip(tr(lang, Key::TipWholeWord))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.whole_word = !this.whole_word;
                        this.recompute(true, cx);
                    })),
            )
            .child(
                icon_button("find-close", Icon::Close, false, cx)
                    .tip(tr(lang, Key::TipFindClose))
                    .on_click(cx.listener(|this, _, _, cx| this.close(cx))),
            );
        let replace_row = self.show_replace.then(|| {
            let replace_field = field(
                "replace-field",
                &self.replace,
                Key::ReplacePlaceholder,
                &self.replace_focus,
                replace_focused,
                window,
                cx,
            )
            .key_context("TabField")
            .on_action(
                cx.listener(|this, _: &crate::actions::FieldNext, window, cx| {
                    window.focus(&this.find_focus, cx)
                }),
            )
            .on_key_down(cx.listener(Self::on_replace_key))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| window.focus(&this.replace_focus, cx)),
            );
            div()
                .flex()
                .items_center()
                .gap(space::sm())
                .child(replace_field)
                .child(
                    button(
                        "replace-one",
                        tr(lang, Key::ReplaceOne),
                        ButtonKind::Default,
                        cx,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.replace_one(cx))),
                )
                .child(
                    button(
                        "replace-all",
                        tr(lang, Key::ReplaceAll),
                        ButtonKind::Primary,
                        cx,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.replace_all(cx))),
                )
        });
        block(cx)
            .p(space::sm())
            .flex()
            .flex_col()
            .gap(space::sm())
            .child(top)
            .children(replace_row)
            .into_any_element()
    }
}
