use super::Editor;
use crate::document::Bookmark;
use crate::position::{Position, Selection};

impl Editor {
    /// Anchors bookmark `name` to the selected text (with a caret: the character next to it). A
    /// bookmark of that name moves here. Returns its id; `None` for an empty name or no text.
    pub fn add_bookmark(&mut self, name: &str) -> Option<u32> {
        let name = name.trim();
        if name.is_empty() {
            return None;
        }
        let range = if self.sel.is_collapsed() {
            let at = self.sel.head;
            let next = self.next_position(at).filter(|n| n.para == at.para);
            let prev = self.prev_position(at).filter(|p| p.para == at.para);
            match (next, prev) {
                (Some(n), _) => (at, n),
                (None, Some(p)) => (p, at),
                _ => return None,
            }
        } else {
            self.sel.range()
        };
        if let Some(old) = self.anchored_id(name) {
            self.delete_bookmark(old);
        }
        let id = self.doc.next_bookmark_id();
        self.doc.bookmarks.push(Bookmark {
            id,
            name: name.to_string(),
        });
        self.anchor_bookmark(id, range);
        Some(id)
    }

    /// Gives the text of `range` to bookmark `id` (the selection stays where it was).
    fn anchor_bookmark(&mut self, id: u32, range: (Position, Position)) {
        let before = self.sel;
        self.sel = Selection {
            anchor: range.0,
            head: range.1,
        };
        self.apply_char_style(|s| s.bookmark = Some(id));
        self.sel = before;
    }

    /// The id of the bookmark `name` that has text (records of taken-off bookmarks stay for undo).
    fn anchored_id(&self, name: &str) -> Option<u32> {
        self.doc
            .bookmarks
            .iter()
            .rev()
            .filter(|b| b.name == name)
            .map(|b| b.id)
            .find(|id| {
                self.doc
                    .paragraphs
                    .iter()
                    .any(|p| p.runs.iter().any(|r| r.style.bookmark == Some(*id)))
            })
    }

    /// The text bookmark `name` marks.
    pub fn bookmark_text(&self, name: &str) -> Option<String> {
        let id = self.anchored_id(name)?;
        let text: String = self
            .doc
            .paragraphs
            .iter()
            .flat_map(|p| p.runs.iter())
            .filter(|r| r.style.bookmark == Some(id))
            .map(|r| r.text.as_str())
            .collect();
        Some(text)
    }

    /// Where bookmark `name` starts.
    pub fn bookmark_position(&self, name: &str) -> Option<Position> {
        let id = self.anchored_id(name)?;
        for (i, p) in self.doc.paragraphs.iter().enumerate() {
            let mut offset = 0;
            for run in &p.runs {
                if run.style.bookmark == Some(id) {
                    return Some(Position::new(i, offset));
                }
                offset += run.text.len();
            }
        }
        None
    }

    /// Takes bookmark `id` off its text (one undo step).
    pub fn delete_bookmark(&mut self, id: u32) {
        let hit: Vec<usize> = self
            .doc
            .paragraphs
            .iter()
            .enumerate()
            .filter(|(_, p)| p.runs.iter().any(|r| r.style.bookmark == Some(id)))
            .map(|(i, _)| i)
            .collect();
        let (Some(&first), Some(&last)) = (hit.first(), hit.last()) else {
            return;
        };
        let mut paras = self.doc.paragraphs[first..=last].to_vec();
        for p in &mut paras {
            for run in &mut p.runs {
                if run.style.bookmark == Some(id) {
                    run.style.bookmark = None;
                }
            }
            p.normalize();
        }
        let sel = self.sel;
        self.edit_paragraphs(first, paras.len(), paras, sel);
    }

    /// The name of a bookmark on the text of heading paragraph `para`; one that is not shown in
    /// the list (`_Ref..`) is made when the heading has none.
    pub fn heading_bookmark(&mut self, para: usize) -> Option<String> {
        let p = self.doc.paragraphs.get(para)?;
        if p.is_empty() {
            return None;
        }
        // A character has one bookmark: a heading that has one already is referred to by it.
        let existing = p
            .runs
            .iter()
            .find_map(|r| r.style.bookmark)
            .and_then(|id| self.doc.bookmarks.iter().find(|b| b.id == id));
        if let Some(b) = existing {
            return Some(b.name.clone());
        }
        let id = self.doc.next_bookmark_id();
        let name = format!("_Ref{id}");
        self.doc.bookmarks.push(Bookmark {
            id,
            name: name.clone(),
        });
        let end = Position::new(para, p.len());
        self.anchor_bookmark(id, (Position::new(para, 0), end));
        Some(name)
    }

    /// Types `text` as a link to bookmark `name` (Ctrl+click goes there). One undo step.
    pub fn insert_reference(&mut self, name: &str, text: &str) {
        let text = text.replace('\n', " ");
        let (start, end) = self.sel.range();
        let at = start.para;
        self.replace_range(start, end, &text);
        // The text is part of a link only now (text typed at a link's end is not), in the same
        // undo step.
        let from = start.offset;
        let mut para = self.doc.paragraphs[at].clone();
        para.map_style(from, from + text.len(), |s| {
            s.link = Some(format!("#{name}"))
        });
        self.amend_last_edit(at, para);
    }

    /// A cross-reference to heading paragraph `para`: its text as a link to the heading.
    pub fn insert_heading_reference(&mut self, para: usize) {
        let Some(name) = self.heading_bookmark(para) else {
            return;
        };
        let text = self.doc.paragraphs[para].text();
        self.insert_reference(&name, text.trim());
    }
}
