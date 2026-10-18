use unicode_segmentation::UnicodeSegmentation;

use super::Editor;
use crate::document::Paragraph;
use crate::position::{Position, Selection};

impl Editor {
    /// Replaces `start..end` with `text` (`\n` starts a new paragraph). Caret goes after the text.
    pub fn replace_range(&mut self, start: Position, end: Position, text: &str) {
        self.replace_range_flagged(start, end, text, false);
    }

    /// `replace_range`; with `page_break` the paragraph that starts after the (first) newline
    /// begins a new page. Paragraphs created by a split never inherit a page break.
    pub(super) fn replace_range_flagged(
        &mut self,
        start: Position,
        end: Position,
        text: &str,
        page_break: bool,
    ) {
        // Text edits stay inside one table cell (or outside tables): never merge cells or cell and body.
        if self.doc.paragraphs[start.para].style.cell != self.doc.paragraphs[end.para].style.cell {
            return;
        }
        // A picture paragraph holds no text.
        if !text.is_empty() && self.doc.paragraphs[start.para].style.image.is_some() {
            return;
        }
        if self.tracked_replace((start, end), text, page_break) {
            return;
        }
        let mut style = self
            .pending_style
            .take()
            .unwrap_or_else(|| self.doc.paragraphs[start.para].style_at(start.offset));
        // Text typed at the end of a link does not become part of it.
        if style.link.is_some() {
            let inside = self.doc.paragraphs[start.para]
                .style_after(start.offset)
                .is_some_and(|next| next.link == style.link);
            if !inside {
                style.link = None;
            }
        }
        let mut first = self.doc.paragraphs[start.para].clone();
        let mut last = self.doc.paragraphs[end.para].clone();
        let tail = last.split_off(end.offset);
        let last_style = last.style.clone();
        first.delete(start.offset, first.len());

        let mut lines = text.split('\n');
        first.insert(start.offset, lines.next().unwrap_or(""), &style);
        let mut paras = vec![first];
        for line in lines {
            let mut p = Paragraph {
                runs: vec![],
                style: paras[0].style.clone(),
            };
            p.style.page_break = false;
            p.insert(0, line, &style);
            paras.push(p);
        }
        // A section break stays at the end of the text that was before it.
        let section = last_style.section.or(paras[0].style.section);
        for p in &mut paras {
            p.style.section = None;
        }
        let last_ix = paras.len() - 1;
        paras[last_ix].style.section = section;
        // Text that joins a picture paragraph replaces the picture.
        if paras[0].style.image.is_some()
            && (paras.len() > 1 || !paras[0].is_empty() || !tail.is_empty())
        {
            paras[0].style = last_style;
        }
        let new_last = paras.len() - 1;
        if page_break && new_last > 0 {
            paras[1].style.page_break = true;
        }
        let caret_offset = paras[new_last].len();
        if new_last > 0 && tail.is_empty() {
            paras[new_last].style.named = paras[0].style.named.next();
        }
        paras[new_last].append(tail);

        let caret = Position::new(start.para + new_last, caret_offset);
        self.edit_paragraphs(
            start.para,
            end.para - start.para + 1,
            paras,
            Selection::caret(caret),
        );
    }

    /// Types text at the caret, replacing the selection.
    pub fn insert_text(&mut self, text: &str) {
        let (start, end) = self.sel.range();
        self.replace_range(start, end, text);
    }

    /// Starts a new page at the caret (Ctrl+Enter): the text after it moves to the next page.
    pub fn insert_page_break(&mut self) {
        let (start, end) = self.sel.range();
        self.replace_range_flagged(
            start, end, "
", true,
        );
    }

    /// A page with nothing on it: a break before and a break after an empty paragraph.
    pub fn insert_blank_page(&mut self) {
        let (start, end) = self.sel.range();
        self.replace_range_flagged(
            start, end, "

", true,
        );
        let blank = self.sel.head.para;
        // The paragraph holding the caret starts the page after the blank one.
        self.edit_para_styles(blank, blank, |s| s.page_break = true);
    }

    pub fn enter(&mut self) {
        let at = self.sel.head;
        if self.sel.is_collapsed() && self.doc.paragraphs[at.para].style.image.is_some() {
            let caret = Position::new(at.para + 1, 0);
            self.edit_paragraphs(
                at.para + 1,
                0,
                vec![Paragraph::default()],
                Selection::caret(caret),
            );
            return;
        }
        let p = &self.doc.paragraphs[at.para];
        // Enter on an empty list item ends the list instead of adding another item.
        if self.sel.is_collapsed() && p.is_empty() && p.style.list.is_some() {
            self.edit_para_styles(at.para, at.para, |s| {
                s.list = None;
                s.indent = 0;
            });
            return;
        }
        self.insert_text("\n");
    }

    pub fn delete_selection(&mut self) -> bool {
        if self.sel.is_collapsed() {
            return false;
        }
        let (start, end) = self.sel.range();
        self.replace_range(start, end, "");
        true
    }

    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.remove_image_at_caret() {
            return;
        }
        let at = self.sel.head;
        // Backspace at the start of a paragraph that begins a page first takes the break away.
        if at.offset == 0 && self.doc.paragraphs[at.para].style.page_break {
            self.edit_para_styles(at.para, at.para, |s| s.page_break = false);
            return;
        }
        // Backspace at the start of a list item first removes the bullet/number.
        if at.offset == 0 && self.doc.paragraphs[at.para].style.list.is_some() {
            self.edit_para_styles(at.para, at.para, |s| s.list = None);
            return;
        }
        if let Some(prev) = self.prev_position(at) {
            self.replace_range(prev, at, "");
            if self.track_changes {
                self.sel = Selection::caret(prev);
            }
        }
    }

    pub fn delete_forward(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.remove_image_at_caret() {
            return;
        }
        let at = self.sel.head;
        if let Some(next) = self.next_position(at) {
            self.replace_range(at, next, "");
        }
    }

    /// Selected text (paragraphs joined by `\n`).
    pub fn selected_text(&self) -> String {
        let (s, e) = self.sel.range();
        (s.para..=e.para)
            .map(|i| {
                let t = self.doc.paragraphs[i].text();
                let from = if i == s.para { s.offset } else { 0 };
                let to = if i == e.para { e.offset } else { t.len() };
                t[from..to].to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Grapheme-aware position helpers (shared with movement).
    pub(super) fn prev_position(&self, at: Position) -> Option<Position> {
        if at.offset > 0 {
            let text = self.doc.paragraphs[at.para].text();
            let prev = text[..at.offset]
                .grapheme_indices(true)
                .next_back()
                .map_or(0, |(i, _)| i);
            return Some(Position::new(at.para, prev));
        }
        (at.para > 0).then(|| Position::new(at.para - 1, self.doc.paragraphs[at.para - 1].len()))
    }

    pub(super) fn next_position(&self, at: Position) -> Option<Position> {
        let text = self.doc.paragraphs[at.para].text();
        if at.offset < text.len() {
            let next = text[at.offset..].graphemes(true).next().map_or(0, str::len);
            return Some(Position::new(at.para, at.offset + next));
        }
        (at.para + 1 < self.doc.paragraphs.len()).then(|| Position::new(at.para + 1, 0))
    }
}
