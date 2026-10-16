use gpui_kit::*;
use unicode_segmentation::UnicodeSegmentation;

use crate::theme::Theme;

/// What a key press did to the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Nothing,
    Changed,
    Submit,
    Cancel,
}

/// Minimal single-line text editing (typing, arrows, Home/End, Backspace/Delete, copy/cut/paste,
/// select-all). The owner view handles focus and calls [`LineEdit::key`] from its key handler.
#[derive(Default)]
pub struct LineEdit {
    text: String,
    cursor: usize,
    /// Whole text selected: the next typed character replaces it.
    all: bool,
}

impl LineEdit {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set(&mut self, text: &str, select_all: bool) {
        self.text = text.to_string();
        self.cursor = self.text.len();
        self.all = select_all && !text.is_empty();
    }

    fn insert(&mut self, s: &str) {
        if self.all {
            self.text.clear();
            self.cursor = 0;
            self.all = false;
        }
        self.text.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    fn prev(&self) -> usize {
        self.text[..self.cursor]
            .grapheme_indices(true)
            .next_back()
            .map_or(0, |(i, _)| i)
    }

    fn next(&self) -> usize {
        self.cursor
            + self.text[self.cursor..]
                .graphemes(true)
                .next()
                .map_or(0, str::len)
    }

    pub fn key(&mut self, ev: &KeyDownEvent, cx: &mut App) -> Outcome {
        let k = &ev.keystroke;
        let m = k.modifiers;
        let command = (m.control && !m.alt) || m.platform;
        match (k.key.as_str(), command) {
            ("enter", _) => Outcome::Submit,
            ("escape", _) => Outcome::Cancel,
            ("backspace", _) if self.all => self.clear(),
            ("backspace", _) if self.cursor > 0 => {
                let from = self.prev();
                self.text.drain(from..self.cursor);
                self.cursor = from;
                Outcome::Changed
            }
            ("delete", _) if self.all => self.clear(),
            ("delete", _) if self.cursor < self.text.len() => {
                let to = self.next();
                self.text.drain(self.cursor..to);
                Outcome::Changed
            }
            ("left", _) => self.move_to(if self.all { 0 } else { self.prev() }),
            ("right", _) => self.move_to(if self.all {
                self.text.len()
            } else {
                self.next()
            }),
            ("home", _) => self.move_to(0),
            ("end", _) => self.move_to(self.text.len()),
            ("a", true) => {
                self.all = !self.text.is_empty();
                Outcome::Nothing
            }
            ("c", true) => {
                self.copy(cx);
                Outcome::Nothing
            }
            ("x", true) => {
                self.copy(cx);
                if self.all {
                    self.clear()
                } else {
                    Outcome::Nothing
                }
            }
            ("v", true) => match cx.read_from_clipboard().and_then(|item| item.text()) {
                Some(pasted) => {
                    self.insert(&pasted.replace(['\r', '\n'], " "));
                    Outcome::Changed
                }
                None => Outcome::Nothing,
            },
            (_, false) => match k
                .key_char
                .as_deref()
                .filter(|t| !t.chars().any(char::is_control))
            {
                Some(typed) => {
                    self.insert(typed);
                    Outcome::Changed
                }
                None => Outcome::Nothing,
            },
            _ => Outcome::Nothing,
        }
    }

    fn clear(&mut self) -> Outcome {
        self.text.clear();
        self.cursor = 0;
        self.all = false;
        Outcome::Changed
    }

    fn move_to(&mut self, at: usize) -> Outcome {
        self.cursor = at;
        self.all = false;
        Outcome::Nothing
    }

    fn copy(&self, cx: &mut App) {
        if self.all {
            cx.write_to_clipboard(ClipboardItem::new_string(self.text.clone()));
        }
    }

    /// The text with a caret (or highlighted when everything is selected).
    pub fn view(&self, cx: &App) -> Div {
        let t = cx.global::<Theme>();
        let row = div().flex().items_center();
        if self.all {
            return row.child(div().bg(t.accent_subtle).child(self.text.clone()));
        }
        row.child(self.text[..self.cursor].to_string())
            .child(div().w(px(1.)).h(px(16.)).bg(t.text))
            .child(self.text[self.cursor..].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::LineEdit;

    #[test]
    fn grapheme_navigation_and_replace_on_select_all() {
        let mut e = LineEdit::default();
        e.set("a👨‍👩‍👧b", true);
        assert!(e.all);
        e.insert("X");
        assert_eq!(e.text(), "X");
        e.set("a👨‍👩‍👧b", false);
        e.cursor = e.text.len() - 1;
        assert_eq!(&e.text[e.prev()..e.cursor], "👨‍👩‍👧");
        e.cursor = 1;
        assert_eq!(&e.text[e.cursor..e.next()], "👨‍👩‍👧");
    }
}
