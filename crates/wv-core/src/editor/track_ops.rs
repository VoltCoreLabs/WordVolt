use super::Editor;
use crate::document::{Paragraph, Revision};
use crate::position::{Position, Selection};
use crate::style::Change;

impl Editor {
    /// Change tracking on: typing is marked as inserted, deleting keeps the text, marked deleted.
    pub fn set_tracking(&mut self, on: bool) {
        self.track_changes = on;
    }

    pub fn tracking(&self) -> bool {
        self.track_changes
    }

    /// Name stored with the changes made from now on.
    pub fn set_author(&mut self, author: &str) {
        self.author = author.to_string();
    }

    fn new_revision(&mut self) -> u32 {
        let id = self.doc.next_revision_id();
        self.doc.revisions.push(Revision {
            id,
            author: self.author.clone(),
            date: crate::time::now_iso(),
        });
        id
    }

    fn is_own(&self, id: u32) -> bool {
        self.doc
            .revisions
            .iter()
            .any(|r| r.id == id && r.author == self.author)
    }

    /// The tracked version of `replace_range`: the old text is marked deleted (own insertions are
    /// simply removed) and the new text is inserted after it, marked as inserted. Returns false
    /// when tracking is off.
    pub(super) fn tracked_replace(
        &mut self,
        (start, end): (Position, Position),
        text: &str,
        page_break: bool,
    ) -> bool {
        if !self.track_changes {
            return false;
        }
        let mut at = end;
        if start != end {
            let id = self.new_revision();
            let own = |rev: u32| self.is_own(rev);
            let mut paras = Vec::new();
            for i in start.para..=end.para {
                let mut p = self.doc.paragraphs[i].clone();
                let from = if i == start.para { start.offset } else { 0 };
                let to = if i == end.para { end.offset } else { p.len() };
                let removed = p.track_delete((from, to), id, &own);
                if i == end.para {
                    at.offset = end.offset - removed;
                }
                paras.push(p);
            }
            let count = paras.len();
            self.edit_paragraphs(start.para, count, paras, Selection::caret(at));
        }
        if text.is_empty() {
            return true;
        }
        let mut style = self
            .pending_style
            .take()
            .unwrap_or_else(|| self.doc.paragraphs[at.para].style_at(at.offset));
        let id = self
            .continued_insertion(at)
            .unwrap_or_else(|| self.new_revision());
        style.change = Some(Change { id, deleted: false });
        self.pending_style = Some(style);
        self.track_changes = false;
        self.replace_range_flagged(at, at, text, page_break);
        self.track_changes = true;
        true
    }

    /// The own insertion that ends right before `at` (typing goes on in it).
    fn continued_insertion(&self, at: Position) -> Option<u32> {
        let p = &self.doc.paragraphs[at.para];
        let prev = p.text().get(..at.offset)?.chars().next_back()?;
        let change = p.style_at_raw(at.offset - prev.len_utf8())?.change?;
        (!change.deleted && self.is_own(change.id)).then_some(change.id)
    }

    /// Accepts (`accept`) or rejects the changes whose id passes `filter`: accepted insertions
    /// become plain text and accepted deletions go; rejecting does the opposite. One undo step.
    fn resolve_changes(&mut self, accept: bool, filter: impl Fn(u32) -> bool) {
        let touched = |p: &Paragraph| {
            p.runs
                .iter()
                .any(|r| r.style.change.is_some_and(|c| filter(c.id)))
        };
        let Some(first) = self.doc.paragraphs.iter().position(touched) else {
            return;
        };
        let last = self
            .doc
            .paragraphs
            .iter()
            .rposition(touched)
            .unwrap_or(first);
        let paras: Vec<_> = self.doc.paragraphs[first..=last]
            .iter()
            .map(|p| {
                let mut q = p.clone();
                q.runs = std::mem::take(&mut q.runs)
                    .into_iter()
                    .filter_map(|mut r| match r.style.change {
                        Some(c) if filter(c.id) => {
                            if c.deleted == accept {
                                None
                            } else {
                                r.style.change = None;
                                Some(r)
                            }
                        }
                        _ => Some(r),
                    })
                    .collect();
                q.normalize();
                q
            })
            .collect();
        let head = self.sel.head;
        let para = head.para.min(self.doc.paragraphs.len() - 1);
        let count = last - first + 1;
        self.edit_paragraphs(
            first,
            count,
            paras,
            Selection::caret(Position::new(para, 0)),
        );
        let len = self.doc.paragraphs[para].len();
        self.sel = Selection::caret(Position::new(para, head.offset.min(len)));
    }

    pub fn accept_all_changes(&mut self) {
        self.resolve_changes(true, |_| true);
    }

    pub fn reject_all_changes(&mut self) {
        self.resolve_changes(false, |_| true);
    }

    /// Accepts the changes in the selection (or the one at the caret).
    pub fn accept_changes_here(&mut self) {
        let ids = self.changes_here();
        self.resolve_changes(true, |id| ids.contains(&id));
    }

    pub fn reject_changes_here(&mut self) {
        let ids = self.changes_here();
        self.resolve_changes(false, |id| ids.contains(&id));
    }

    /// Ids of the changes the selection touches; with a caret, those of the characters next to it.
    pub fn changes_here(&self) -> Vec<u32> {
        let (s, e) = self.sel.range();
        let mut ids = Vec::new();
        for i in s.para..=e.para {
            let p = &self.doc.paragraphs[i];
            let from = if i == s.para { s.offset } else { 0 };
            let to = if i == e.para { e.offset } else { p.len() };
            let mut pos = 0;
            for run in &p.runs {
                let (a, b) = (pos, pos + run.text.len());
                pos = b;
                let overlaps = if from == to {
                    a <= from && from <= b
                } else {
                    a < to && from < b
                };
                if let (true, Some(c)) = (overlaps, run.style.change)
                    && !ids.contains(&c.id)
                {
                    ids.push(c.id);
                }
            }
        }
        ids
    }

    /// Selects the next (or previous) tracked change after the caret, wrapping around.
    pub fn goto_change(&mut self, forward: bool) -> bool {
        let mut spans: Vec<(Position, Position)> = Vec::new();
        for (i, p) in self.doc.paragraphs.iter().enumerate() {
            let mut pos = 0;
            for run in &p.runs {
                let end = pos + run.text.len();
                if run.style.change.is_some() {
                    spans.push((Position::new(i, pos), Position::new(i, end)));
                }
                pos = end;
            }
        }
        let (first, last) = self.sel.range();
        let next = if forward {
            spans.iter().find(|(s, _)| *s > last).or(spans.first())
        } else {
            spans
                .iter()
                .rev()
                .find(|(s, _)| *s < first)
                .or(spans.last())
        };
        let Some(&(s, e)) = next else {
            return false;
        };
        self.sel = Selection { anchor: s, head: e };
        true
    }
}
