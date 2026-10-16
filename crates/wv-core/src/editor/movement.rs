use super::Editor;
use crate::position::Position;

impl Editor {
    pub fn move_left(&mut self, extend: bool) {
        if !extend && !self.sel.is_collapsed() {
            let (start, _) = self.sel.range();
            return self.set_head(start, false);
        }
        if let Some(p) = self.prev_position(self.sel.head) {
            self.set_head(p, extend);
        }
    }

    pub fn move_right(&mut self, extend: bool) {
        if !extend && !self.sel.is_collapsed() {
            let (_, end) = self.sel.range();
            return self.set_head(end, false);
        }
        if let Some(p) = self.next_position(self.sel.head) {
            self.set_head(p, extend);
        }
    }

    pub fn move_line_start(&mut self, extend: bool) {
        self.set_head(Position::new(self.sel.head.para, 0), extend);
    }

    pub fn move_line_end(&mut self, extend: bool) {
        let para = self.sel.head.para;
        self.set_head(Position::new(para, self.doc.paragraphs[para].len()), extend);
    }

    pub fn move_doc_start(&mut self, extend: bool) {
        self.set_head(Position::default(), extend);
    }

    pub fn move_doc_end(&mut self, extend: bool) {
        self.set_head(self.doc.end(), extend);
    }

    pub fn select_all(&mut self) {
        self.sel.anchor = Position::default();
        self.sel.head = self.doc.end();
    }
}
