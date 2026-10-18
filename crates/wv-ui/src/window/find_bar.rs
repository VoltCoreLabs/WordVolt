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

    /// The text being searched for.
    pub fn query(&self) -> &str {
        self.find.text()
    }

    /// All matches of the search (start, end).
    pub fn results(&self) -> &[(Position, Position)] {
        &self.matches
    }

    /// Index of the current match.
    pub fn current(&self) -> Option<usize> {
        self.current
    }

    /// Makes match `index` the current one and selects it in the document.
    pub fn go_to(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.matches.len() {
            self.current = Some(index);
            self.message = None;
            self.select_current(cx);
            self.publish(cx);
            cx.notify();
        }
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

mod view;
