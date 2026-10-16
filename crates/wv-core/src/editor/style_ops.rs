use super::Editor;
use crate::position::Selection;
use crate::style::{Align, CharStyle};

impl Editor {
    /// Applies `f` to the style of the selected text; with a caret it changes the style of the next typed text.
    fn change_char_style(
        &mut self,
        all_set: impl Fn(&CharStyle) -> bool,
        f: impl Fn(&mut CharStyle, bool),
    ) {
        let (start, end) = self.sel.range();
        if start == end {
            let mut style = self
                .pending_style
                .clone()
                .unwrap_or_else(|| self.doc.paragraphs[start.para].style_at(start.offset));
            let on = !all_set(&style);
            f(&mut style, on);
            self.pending_style = Some(style);
            return;
        }
        let on = !self.doc.all_chars(start, end, &all_set);
        let mut paras = self.doc.paragraphs[start.para..=end.para].to_vec();
        let last = paras.len() - 1;
        for (i, p) in paras.iter_mut().enumerate() {
            let from = if i == 0 { start.offset } else { 0 };
            let to = if i == last { end.offset } else { p.len() };
            p.map_style(from, to, |s| f(s, on));
        }
        let sel = self.sel;
        self.edit_paragraphs(start.para, last + 1, paras, sel);
    }

    pub fn toggle_bold(&mut self) {
        self.change_char_style(|s| s.bold, |s, on| s.bold = on);
    }

    pub fn toggle_italic(&mut self) {
        self.change_char_style(|s| s.italic, |s, on| s.italic = on);
    }

    pub fn toggle_underline(&mut self) {
        self.change_char_style(|s| s.underline, |s, on| s.underline = on);
    }

    /// Sets alignment of every paragraph touched by the selection.
    pub fn set_align(&mut self, align: Align) {
        let (start, end) = self.sel.range();
        let mut paras = self.doc.paragraphs[start.para..=end.para].to_vec();
        paras.iter_mut().for_each(|p| p.style.align = align);
        let sel: Selection = self.sel;
        self.edit_paragraphs(start.para, paras.len(), paras, sel);
    }

    /// Alignment of the paragraph with the caret.
    pub fn align(&self) -> Align {
        self.doc.paragraphs[self.sel.head.para].style.align
    }
}
