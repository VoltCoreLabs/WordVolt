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

    /// Formatting at the caret (or the common formatting of the selection) for toolbar state.
    pub fn current_style(&self) -> CharStyle {
        let (s, e) = self.sel.range();
        if s == e {
            return self
                .pending_style
                .clone()
                .unwrap_or_else(|| self.doc.paragraphs[s.para].style_at(s.offset));
        }
        CharStyle {
            bold: self.doc.all_chars(s, e, |c| c.bold),
            italic: self.doc.all_chars(s, e, |c| c.italic),
            underline: self.doc.all_chars(s, e, |c| c.underline),
            size_pt: {
                let first = self.doc.paragraphs[s.para].style_at(s.offset + 1).size_pt;
                self.doc
                    .all_chars(s, e, |c| c.size_pt == first)
                    .then_some(first)
                    .flatten()
            },
            ..CharStyle::default()
        }
    }

    /// Sets the font size (pt) of the selection; with a caret it applies to the next typed text.
    pub fn set_size_pt(&mut self, size: u16) {
        let (start, end) = self.sel.range();
        if start == end {
            let mut style = self
                .pending_style
                .clone()
                .unwrap_or_else(|| self.doc.paragraphs[start.para].style_at(start.offset));
            style.size_pt = Some(size);
            self.pending_style = Some(style);
            return;
        }
        let mut paras = self.doc.paragraphs[start.para..=end.para].to_vec();
        let last = paras.len() - 1;
        for (i, p) in paras.iter_mut().enumerate() {
            let from = if i == 0 { start.offset } else { 0 };
            let to = if i == last { end.offset } else { p.len() };
            p.map_style(from, to, |s| s.size_pt = Some(size));
        }
        let sel = self.sel;
        self.edit_paragraphs(start.para, last + 1, paras, sel);
    }

    /// Next bigger (`step > 0`) or smaller (`step < 0`) size from the usual Word size list.
    pub fn bump_size(&mut self, step: i32) {
        const SIZES: [u16; 21] = [
            8, 9, 10, 11, 12, 14, 16, 18, 20, 22, 24, 26, 28, 36, 48, 72, 96, 120, 144, 192, 288,
        ];
        let current = self.current_style().size_pt.unwrap_or(12);
        let next = if step > 0 {
            SIZES
                .iter()
                .copied()
                .find(|s| *s > current)
                .unwrap_or(current)
        } else {
            SIZES
                .iter()
                .rev()
                .copied()
                .find(|s| *s < current)
                .unwrap_or(current)
        };
        self.set_size_pt(next);
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
