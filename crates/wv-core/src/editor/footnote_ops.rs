use super::Editor;
use crate::document::{NOTE_MARK, Paragraph, Run};
use crate::position::{Position, Selection};
use crate::style::CharStyle;

impl Editor {
    /// Puts a footnote mark at the caret and an empty note after the last paragraph of the document
    /// (it is drawn at the bottom of the page of the mark). The caret goes into the note. One undo
    /// step. Not inside a note or on a picture.
    pub fn insert_footnote(&mut self) {
        let at = self.sel.head;
        let p = &self.doc.paragraphs[at.para];
        if p.style.note.is_some() || p.style.image.is_some() {
            return;
        }
        let id = self.doc.next_note_id();
        let mut marked = p.clone();
        let mark = Run::new(
            NOTE_MARK.to_string(),
            CharStyle {
                note: Some(id),
                ..CharStyle::default()
            },
        );
        marked.insert_runs(at.offset, vec![mark]);
        let mut note = Paragraph::default();
        note.style.note = Some(id);
        // The edit covers everything from the marked paragraph to the end of the document.
        let count = self.doc.paragraphs.len() - at.para;
        let mut paras: Vec<Paragraph> = self.doc.paragraphs[at.para..].to_vec();
        paras[0] = marked;
        paras.push(note);
        let caret = Position::new(at.para + count, 0);
        self.edit_paragraphs(at.para, count, paras, Selection::caret(caret));
    }

    /// Where Ctrl+click on a footnote mark goes (the start of its note) or on a note goes (just
    /// after its mark); `None` where `pos` is neither.
    pub fn note_target(&self, pos: Position) -> Option<Position> {
        let p = self.doc.paragraphs.get(pos.para)?;
        if let Some(id) = p.style.note {
            return self.doc.note_mark_position(id);
        }
        // The mark right after the position, or right before it (a click lands on either side).
        let text = p.text();
        let before = text
            .get(..pos.offset)
            .and_then(|t| t.chars().next_back())
            .filter(|c| *c == NOTE_MARK)
            .and_then(|c| p.style_at_raw(pos.offset - c.len_utf8()))
            .and_then(|s| s.note);
        let after = p.style_at_raw(pos.offset).and_then(|s| s.note);
        let id = after.or(before)?;
        self.doc.note_paragraph(id).map(|i| Position::new(i, 0))
    }
}
