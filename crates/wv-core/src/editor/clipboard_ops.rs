use super::Editor;
use crate::document::Paragraph;
use crate::position::{Position, Selection};

impl Editor {
    /// The selected part of the document with its formatting (what the clipboard keeps next to
    /// the plain text). The first paragraph starts at the selection, the last ends at it.
    pub fn copy_fragment(&self) -> Vec<Paragraph> {
        let (s, e) = self.sel.range();
        (s.para..=e.para)
            .map(|i| {
                let mut p = self.doc.paragraphs[i].clone();
                if i == e.para {
                    p.delete(e.offset, p.len());
                }
                if i == s.para {
                    p = p.split_off(s.offset);
                }
                p
            })
            .collect()
    }

    /// Pastes a fragment from `copy_fragment`, replacing the selection: character formatting
    /// is kept, and paragraphs after the first keep their paragraph style (list, heading, ...).
    pub fn paste_fragment(&mut self, fragment: &[Paragraph]) {
        let (start, end) = self.sel.range();
        let cell = self.doc.paragraphs[start.para].style.cell;
        if cell != self.doc.paragraphs[end.para].style.cell
            || self.doc.paragraphs[start.para].style.image.is_some()
        {
            return;
        }
        // Pictures are not carried over, and neither are footnotes (a copy of a mark would share
        // the note with the original): marks go, a pasted note becomes a plain paragraph.
        let mut frag: Vec<Paragraph> = fragment
            .iter()
            .filter(|p| p.style.image.is_none())
            .cloned()
            .map(|mut p| {
                p.runs.retain(|r| r.style.note.is_none());
                p.style.note = None;
                p
            })
            .collect();
        if frag.is_empty() {
            return;
        }
        self.pending_style = None;
        let mut first = self.doc.paragraphs[start.para].clone();
        let mut last = self.doc.paragraphs[end.para].clone();
        let tail = last.split_off(end.offset);
        first.delete(start.offset, first.len());

        let head = frag.remove(0);
        let head_len = head.len();
        first.insert_runs(start.offset, head.runs);
        let mut paras = vec![first];
        for mut p in frag {
            p.style.cell = cell;
            p.style.toc = false;
            paras.push(p);
        }
        let new_last = paras.len() - 1;
        let caret_offset = if new_last == 0 {
            start.offset + head_len
        } else {
            paras[new_last].len()
        };
        paras[new_last].append(tail);
        let caret = Position::new(start.para + new_last, caret_offset);
        self.edit_paragraphs(
            start.para,
            end.para - start.para + 1,
            paras,
            Selection::caret(caret),
        );
    }
}
