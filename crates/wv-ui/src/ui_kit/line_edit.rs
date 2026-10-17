use std::ops::Range;

use gpui_kit::*;
use unicode_segmentation::UnicodeSegmentation;

use crate::theme::{Theme, ui_font};

/// What a key press did to the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Nothing,
    Changed,
    Submit,
    Cancel,
}

/// Minimal single-line text editing (typing, arrows, Home/End, Backspace/Delete, copy/cut/paste,
/// select-all, Shift+arrows/Home/End to select). The owner view handles focus and calls
/// [`LineEdit::key`] from its key handler.
#[derive(Default)]
pub struct LineEdit {
    text: String,
    cursor: usize,
    /// The other end of the selection (the selection is `anchor..cursor`, either way round).
    anchor: Option<usize>,
}

impl LineEdit {
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The selected bytes, if any.
    pub fn selection(&self) -> Option<Range<usize>> {
        let a = self.anchor?;
        (a != self.cursor).then(|| a.min(self.cursor)..a.max(self.cursor))
    }

    pub fn set(&mut self, text: &str, select_all: bool) {
        self.text = text.to_string();
        self.cursor = self.text.len();
        self.anchor = (select_all && !text.is_empty()).then_some(0);
    }

    /// Removes the selection; true if there was one.
    fn delete_selection(&mut self) -> bool {
        let Some(range) = self.selection() else {
            self.anchor = None;
            return false;
        };
        self.text.drain(range.clone());
        self.cursor = range.start;
        self.anchor = None;
        true
    }

    fn insert(&mut self, s: &str) {
        self.delete_selection();
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
            ("backspace", _) if self.delete_selection() => Outcome::Changed,
            ("backspace", _) if self.cursor > 0 => {
                let from = self.prev();
                self.text.drain(from..self.cursor);
                self.cursor = from;
                Outcome::Changed
            }
            ("delete", _) if self.delete_selection() => Outcome::Changed,
            ("delete", _) if self.cursor < self.text.len() => {
                let to = self.next();
                self.text.drain(self.cursor..to);
                Outcome::Changed
            }
            ("left", _) => {
                let to = match self.selection() {
                    Some(r) if !m.shift => r.start,
                    _ => self.prev(),
                };
                self.move_to(to, m.shift)
            }
            ("right", _) => {
                let to = match self.selection() {
                    Some(r) if !m.shift => r.end,
                    _ => self.next(),
                };
                self.move_to(to, m.shift)
            }
            ("home", _) => self.move_to(0, m.shift),
            ("end", _) => self.move_to(self.text.len(), m.shift),
            ("a", true) => {
                self.anchor = (!self.text.is_empty()).then_some(0);
                self.cursor = self.text.len();
                Outcome::Nothing
            }
            ("c", true) => {
                self.copy(cx);
                Outcome::Nothing
            }
            ("x", true) => {
                self.copy(cx);
                if self.delete_selection() {
                    Outcome::Changed
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

    /// Moves the caret; with `extend` the selection grows from where it started.
    fn move_to(&mut self, at: usize, extend: bool) -> Outcome {
        if extend {
            self.anchor.get_or_insert(self.cursor);
        } else {
            self.anchor = None;
        }
        self.cursor = at;
        Outcome::Nothing
    }

    fn copy(&self, cx: &mut App) {
        if let Some(range) = self.selection() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.text[range].to_string()));
        }
    }

    /// Width of `text` in the 14 px UI font (what `text_sm` draws).
    fn text_width(text: &str, window: &Window, color: Hsla) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let run = TextRun {
            len: text.len(),
            font: ui_font(),
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        f32::from(
            window
                .text_system()
                .shape_line(text.to_string().into(), px(14.), &[run], None)
                .width(),
        )
    }

    /// The text with a caret (or the selection highlighted). `visible` is the width available for
    /// the text: longer text is clipped and scrolled so that the caret stays in view.
    pub fn view(&self, visible: f32, window: &Window, cx: &App) -> Div {
        let t = cx.global::<Theme>();
        let row = div().flex().flex_none().items_center().whitespace_nowrap();
        let before = Self::text_width(&self.text[..self.cursor], window, t.text.into());
        let caret = || div().w(px(1.)).h(px(16.)).bg(t.text);
        let row = match self.selection() {
            Some(sel) => {
                let (start, end) = (sel.start, sel.end);
                let selected = div()
                    .bg(t.accent_subtle)
                    .child(self.text[start..end].to_string());
                let row = row.child(self.text[..start].to_string());
                let row = if self.cursor == start {
                    row.child(caret())
                } else {
                    row
                };
                let row = row.child(selected);
                let row = if self.cursor == end {
                    row.child(caret())
                } else {
                    row
                };
                row.child(self.text[end..].to_string())
            }
            None => row
                .child(self.text[..self.cursor].to_string())
                .child(caret())
                .child(self.text[self.cursor..].to_string()),
        };
        let shift = (before + 4.0 - visible).max(0.0);
        div().w_full().overflow_hidden().child(row.ml(px(-shift)))
    }
}

#[cfg(test)]
mod tests {
    use super::LineEdit;

    #[test]
    fn grapheme_navigation_and_replace_on_select_all() {
        let mut e = LineEdit::default();
        e.set("a👨‍👩‍👧b", true);
        assert_eq!(e.selection(), Some(0..e.text().len()));
        e.insert("X");
        assert_eq!(e.text(), "X");
        e.set("a👨‍👩‍👧b", false);
        e.cursor = e.text.len() - 1;
        assert_eq!(&e.text[e.prev()..e.cursor], "👨‍👩‍👧");
        e.cursor = 1;
        assert_eq!(&e.text[e.cursor..e.next()], "👨‍👩‍👧");
    }

    #[test]
    fn selection_extends_and_is_replaced_by_typing() {
        let mut e = LineEdit::default();
        e.set("hello world", false);
        e.move_to(6, false);
        e.move_to(11, true);
        assert_eq!(e.selection(), Some(6..11));
        e.insert("there");
        assert_eq!(e.text(), "hello there");
        assert_eq!(e.selection(), None);
        e.move_to(5, false);
        e.move_to(0, true);
        assert!(e.delete_selection());
        assert_eq!(e.text(), " there");
    }
}
