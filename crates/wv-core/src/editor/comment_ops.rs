use super::Editor;
use crate::document::Comment;
use crate::position::{Position, Selection};

impl Editor {
    /// Anchors a new comment to the selected text (one undo step for the anchor; the text of the
    /// comment is kept in the document). Nothing happens with no selection.
    pub fn add_comment(&mut self, author: &str, text: &str) -> Option<u32> {
        if self.sel.is_collapsed() {
            return None;
        }
        let id = self.doc.next_comment_id();
        self.doc.comments.push(Comment {
            id,
            author: author.to_string(),
            text: text.to_string(),
        });
        self.apply_char_style(|s| s.comment = Some(id));
        Some(id)
    }

    /// The comment whose text the position is in (or right after).
    pub fn comment_at(&self, pos: Position) -> Option<u32> {
        let p = self.doc.paragraphs.get(pos.para)?;
        let after = p.style_at_raw(pos.offset).and_then(|s| s.comment);
        let before = (pos.offset > 0)
            .then(|| {
                let text = p.text();
                let c = text.get(..pos.offset)?.chars().next_back()?;
                p.style_at_raw(pos.offset - c.len_utf8())
            })
            .flatten()
            .and_then(|s| s.comment);
        after.or(before)
    }

    /// Changes the text of comment `id`.
    pub fn set_comment_text(&mut self, id: u32, text: &str) {
        if let Some(c) = self.doc.comments.iter_mut().find(|c| c.id == id) {
            c.text = text.to_string();
            self.mark_modified();
        }
    }

    /// Takes comment `id` off its text (one undo step; its text is kept so that undo brings the
    /// whole comment back).
    pub fn delete_comment(&mut self, id: u32) {
        let hit: Vec<usize> = self
            .doc
            .paragraphs
            .iter()
            .enumerate()
            .filter(|(_, p)| p.runs.iter().any(|r| r.style.comment == Some(id)))
            .map(|(i, _)| i)
            .collect();
        let (Some(&first), Some(&last)) = (hit.first(), hit.last()) else {
            return;
        };
        let mut paras = self.doc.paragraphs[first..=last].to_vec();
        for p in &mut paras {
            for run in &mut p.runs {
                if run.style.comment == Some(id) {
                    run.style.comment = None;
                }
            }
            p.normalize();
        }
        let sel = Selection::caret(self.sel.head.min(Position::new(last, usize::MAX)));
        let sel = if self.sel.head.para >= first && self.sel.head.para <= last {
            self.sel
        } else {
            sel
        };
        self.edit_paragraphs(first, paras.len(), paras, sel);
    }
}
