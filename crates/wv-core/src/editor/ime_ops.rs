use std::ops::Range;

use super::Editor;
use crate::position::{Position, Selection};

/// Text being composed by an input method.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Marked {
    para: usize,
    /// Byte range in the paragraph.
    range: Range<usize>,
    /// Document revision right after the composition was put in (it is still the latest edit).
    revision: u64,
}

/// Byte offset in `text` of a UTF-16 offset (the platform text input counts UTF-16 units).
fn byte_of(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (i, c) in text.char_indices() {
        if units >= utf16 {
            return i;
        }
        units += c.len_utf16();
    }
    text.len()
}

fn utf16_of(text: &str, byte: usize) -> usize {
    text[..byte.min(text.len())].encode_utf16().count()
}

impl Editor {
    fn caret_text(&self) -> String {
        self.doc.paragraphs[self.sel.head.para].text()
    }

    /// The part of the caret's paragraph at the UTF-16 `range`, and the range clamped to the text.
    pub fn ime_text(&self, range: Range<usize>) -> (String, Range<usize>) {
        let text = self.caret_text();
        let (a, b) = (byte_of(&text, range.start), byte_of(&text, range.end));
        let b = b.max(a);
        (
            text[a..b].to_string(),
            utf16_of(&text, a)..utf16_of(&text, b),
        )
    }

    /// The selection as a UTF-16 range inside the caret's paragraph, and whether the head comes
    /// before the anchor.
    pub fn ime_selection(&self) -> (Range<usize>, bool) {
        let sel = self.sel;
        let text = self.caret_text();
        let anchor = if sel.anchor.para == sel.head.para {
            sel.anchor.offset
        } else {
            sel.head.offset
        };
        let (a, b) = (utf16_of(&text, anchor), utf16_of(&text, sel.head.offset));
        (a.min(b)..a.max(b), b < a)
    }

    /// The composition being typed as a UTF-16 range of its paragraph.
    pub fn ime_marked(&self) -> Option<Range<usize>> {
        let m = self.marked.as_ref()?;
        let text = self.doc.paragraphs.get(m.para)?.text();
        Some(utf16_of(&text, m.range.start)..utf16_of(&text, m.range.end))
    }

    pub fn ime_unmark(&mut self) {
        self.marked = None;
    }

    /// Position in the caret's paragraph of a UTF-16 offset.
    pub fn ime_position(&self, utf16: usize) -> Position {
        let para = self.sel.head.para;
        Position::new(para, byte_of(&self.caret_text(), utf16))
    }

    /// Types `text` the way the platform text input asks: over the UTF-16 `range` of the caret's
    /// paragraph, else over the composition in progress, else over the selection. With `mark` the
    /// text becomes the composition (the next call replaces it). A whole composition is one undo
    /// step: each update takes the previous one back first.
    pub fn ime_replace(&mut self, range: Option<Range<usize>>, text: &str, mark: bool) {
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        let composing = self.marked.take();
        let mut undone = false;
        let (start, end) = match (range, composing) {
            (Some(r), _) => {
                let t = self.caret_text();
                let para = self.sel.head.para;
                (
                    Position::new(para, byte_of(&t, r.start)),
                    Position::new(para, byte_of(&t, r.end)),
                )
            }
            (None, Some(m))
                if m.revision == self.revision() && {
                    undone = self.undo();
                    undone
                } =>
            {
                self.sel.range()
            }
            (None, Some(m)) if m.para < self.doc.paragraphs.len() => (
                Position::new(m.para, m.range.start),
                Position::new(m.para, m.range.end.min(self.doc.paragraphs[m.para].len())),
            ),
            _ => self.sel.range(),
        };
        // Cancelling a composition only takes it back: what it replaced stays.
        if undone && text.is_empty() {
            return;
        }
        if text.is_empty() && start == end {
            self.sel = Selection::caret(start);
            return;
        }
        self.replace_range(start, end, &text);
        if mark && !text.is_empty() {
            self.marked = Some(Marked {
                para: start.para,
                range: start.offset..start.offset + text.len(),
                revision: self.revision(),
            });
        }
    }
}
