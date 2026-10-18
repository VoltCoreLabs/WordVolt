use super::{Comment, Document, PageSpec};
use crate::position::Position;

/// Comments, tracked changes and sections: lookups and fresh ids.
impl Document {
    /// An id no comment has.
    pub fn next_comment_id(&self) -> u32 {
        let anchored = self
            .paragraphs
            .iter()
            .flat_map(|p| p.runs.iter().filter_map(|r| r.style.comment));
        let known = self.comments.iter().map(|c| c.id);
        anchored.chain(known).max().map_or(1, |m| m + 1)
    }

    /// An id no tracked change has.
    pub fn next_revision_id(&self) -> u32 {
        let used = self
            .paragraphs
            .iter()
            .flat_map(|p| p.runs.iter().filter_map(|r| r.style.change.map(|c| c.id)));
        let known = self.revisions.iter().map(|r| r.id);
        used.chain(known).max().map_or(1, |m| m + 1)
    }

    /// The tracked changes that still have text, as (id, deleted), in document order.
    pub fn changes(&self) -> Vec<(u32, bool)> {
        let mut out: Vec<(u32, bool)> = Vec::new();
        for c in self
            .paragraphs
            .iter()
            .flat_map(|p| p.runs.iter().filter_map(|r| r.style.change))
        {
            if !out.contains(&(c.id, c.deleted)) {
                out.push((c.id, c.deleted));
            }
        }
        out
    }

    /// Comments that have text anchored to them, in the order of their anchors in the document.
    pub fn visible_comments(&self) -> Vec<&Comment> {
        let mut ids: Vec<u32> = Vec::new();
        for p in &self.paragraphs {
            for id in p.runs.iter().filter_map(|r| r.style.comment) {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        ids.iter()
            .filter_map(|id| self.comments.iter().find(|c| c.id == *id))
            .collect()
    }

    /// Where the text of comment `id` starts and ends (the first and last run that carry it).
    pub fn comment_range(&self, id: u32) -> Option<(Position, Position)> {
        let mut start = None;
        let mut end = None;
        for (i, p) in self.paragraphs.iter().enumerate() {
            let mut offset = 0;
            for run in &p.runs {
                let from = offset;
                offset += run.text.len();
                if run.style.comment == Some(id) {
                    start.get_or_insert(Position::new(i, from));
                    end = Some(Position::new(i, offset));
                }
            }
        }
        Some((start?, end?))
    }

    /// The section break (id) that ends the section paragraph `para` is in; `None` in the last one.
    pub fn section_end(&self, para: usize) -> Option<u32> {
        self.paragraphs[para..]
            .iter()
            .find_map(|p| p.style.section)
            .filter(|id| self.sections.contains_key(id))
    }

    /// Page setup of the section paragraph `para` is in.
    pub fn page_of(&self, para: usize) -> PageSpec {
        self.section_end(para)
            .and_then(|id| self.sections.get(&id).copied())
            .unwrap_or(self.page)
    }

    /// An id no section break has.
    pub fn next_section_id(&self) -> u32 {
        let used = self.paragraphs.iter().filter_map(|p| p.style.section);
        let known = self.sections.keys().copied();
        used.chain(known).max().map_or(1, |m| m + 1)
    }
}
