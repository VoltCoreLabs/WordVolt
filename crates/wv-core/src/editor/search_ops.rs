use super::Editor;
use crate::document::Document;
use crate::position::{Position, Selection};
use crate::search::{SearchOptions, find_all};

impl Editor {
    /// Matches of `query` in the whole document.
    pub fn find_matches(&self, query: &str, opts: SearchOptions) -> Vec<(Position, Position)> {
        find_all(self.doc.paragraphs.iter().map(|p| p.text()), query, opts)
    }

    /// Selects `start..end` (the caret ends up at `end`).
    pub fn select_range(&mut self, start: Position, end: Position) {
        self.sel = Selection {
            anchor: start,
            head: end,
        };
        self.pending_style = None;
    }

    /// Replaces a match with `text`; the replacement takes the formatting of the first matched character.
    pub fn replace_match(&mut self, start: Position, end: Position, text: &str) {
        self.pending_style = self.doc.paragraphs[start.para].style_after(start.offset);
        self.replace_range(start, end, text);
    }

    /// Replaces every match in one undo step; returns how many were replaced.
    pub fn replace_all(&mut self, query: &str, replacement: &str, opts: SearchOptions) -> usize {
        let matches = self.find_matches(query, opts);
        let (Some(first), Some(last)) = (matches.first(), matches.last()) else {
            return 0;
        };
        let (from, to) = (first.0.para, last.0.para);
        // Work on a copy of the affected paragraphs, last match first so earlier offsets stay valid.
        let mut work = Editor::new(Document {
            paragraphs: self.doc.paragraphs[from..=to].to_vec(),
            ..Document::default()
        });
        let shift = |p: Position| Position::new(p.para - from, p.offset);
        for (start, end) in matches.iter().rev() {
            work.replace_match(shift(*start), shift(*end), replacement);
        }
        let caret = Position::new(work.sel.head.para + from, work.sel.head.offset);
        self.edit_paragraphs(
            from,
            to - from + 1,
            work.doc.paragraphs,
            Selection::caret(caret),
        );
        matches.len()
    }
}
