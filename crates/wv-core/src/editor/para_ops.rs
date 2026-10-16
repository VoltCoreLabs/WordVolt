use super::Editor;
use crate::position::Selection;
use crate::style::{ListKind, NamedStyle, ParaStyle};

impl Editor {
    /// Applies `f` to the style of paragraphs `first..=last`; the selection stays as it is.
    pub(super) fn edit_para_styles(
        &mut self,
        first: usize,
        last: usize,
        f: impl Fn(&mut ParaStyle),
    ) {
        let mut paras = self.doc.paragraphs[first..=last].to_vec();
        paras.iter_mut().for_each(|p| f(&mut p.style));
        let sel: Selection = self.sel;
        self.edit_paragraphs(first, paras.len(), paras, sel);
    }

    /// First and last paragraph touched by the selection.
    fn selected_paras(&self) -> (usize, usize) {
        let (s, e) = self.sel.range();
        (s.para, e.para)
    }

    /// Makes the selected paragraphs list items of `kind`, or removes the list if they all already are.
    pub fn toggle_list(&mut self, kind: ListKind) {
        let (a, b) = self.selected_paras();
        let on = !self.doc.paragraphs[a..=b]
            .iter()
            .all(|p| p.style.list == Some(kind));
        self.edit_para_styles(a, b, |s| s.list = on.then_some(kind));
    }

    /// Moves the selected paragraphs `delta` steps (0.5 inch) in or out; list items change level.
    pub fn change_indent(&mut self, delta: i8) {
        let (a, b) = self.selected_paras();
        self.edit_para_styles(a, b, |s| {
            s.indent = (i16::from(s.indent) + i16::from(delta)).clamp(0, 8) as u8
        });
    }

    pub fn set_line_spacing(&mut self, spacing: Option<f32>) {
        let (a, b) = self.selected_paras();
        self.edit_para_styles(a, b, |s| s.line_spacing = spacing);
    }

    /// Applies a built-in style to the selected paragraphs.
    pub fn set_named_style(&mut self, named: NamedStyle) {
        let (a, b) = self.selected_paras();
        self.edit_para_styles(a, b, |s| s.named = named);
    }

    pub fn named_style(&self) -> NamedStyle {
        self.doc.paragraphs[self.sel.head.para].style.named
    }

    pub fn list_kind(&self) -> Option<ListKind> {
        self.doc.paragraphs[self.sel.head.para].style.list
    }

    pub fn line_spacing(&self) -> Option<f32> {
        self.doc.paragraphs[self.sel.head.para].style.line_spacing
    }

    /// Tab indents list items and multi-paragraph selections (Shift+Tab outdents); elsewhere it types a tab.
    pub fn tab_key(&mut self, shift: bool) {
        let (a, b) = self.selected_paras();
        let in_list = self.doc.paragraphs[a..=b]
            .iter()
            .any(|p| p.style.list.is_some());
        if in_list || a != b {
            self.change_indent(if shift { -1 } else { 1 });
        } else if !shift {
            self.insert_text("\t");
        }
    }
}
