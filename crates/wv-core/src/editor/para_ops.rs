use super::Editor;
use crate::position::Selection;
use crate::style::{ListKind, MAX_TABS, NamedStyle, ParaMetrics, ParaStyle};

/// One indent step (0.5 inch) in points.
const INDENT_PT: u16 = 36;

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

    /// Moves the first line of the selected paragraphs `delta` points in or out (0..=255).
    pub fn change_first_line(&mut self, delta: i16) {
        let (a, b) = self.selected_paras();
        self.edit_para_styles(a, b, |s| {
            s.first_line = (i16::from(s.first_line) + delta).clamp(0, 255) as u8
        });
    }

    /// Spacing and indents of the paragraph at the caret (what the paragraph dialog shows).
    pub fn para_metrics(&self) -> ParaMetrics {
        let s = &self.doc.paragraphs[self.sel.head.para].style;
        ParaMetrics {
            left_pt: if s.list.is_some() {
                u16::from(s.left_extra)
            } else {
                u16::from(s.indent) * INDENT_PT + u16::from(s.left_extra)
            },
            first_line: s.first_line,
            space_before: s.space_before,
            space_after: s.space_after,
            line_spacing: s.line_spacing,
            tabs: s.tabs.clone(),
        }
    }

    /// Applies the values of the paragraph dialog to the selected paragraphs.
    pub fn set_para_metrics(&mut self, m: &ParaMetrics) {
        let (a, b) = self.selected_paras();
        let mut tabs = m.tabs.clone();
        tabs.sort_unstable();
        tabs.dedup();
        tabs.truncate(MAX_TABS);
        self.edit_para_styles(a, b, |s| {
            if s.list.is_some() {
                s.left_extra = m.left_pt.min(255) as u8;
            } else {
                // Whole 0.5 inch steps are the indent level, the rest is kept in points.
                s.indent = (m.left_pt / INDENT_PT).min(8) as u8;
                s.left_extra = if m.left_pt / INDENT_PT >= 8 {
                    0
                } else {
                    (m.left_pt % INDENT_PT) as u8
                };
            }
            s.first_line = m.first_line;
            s.space_before = m.space_before;
            s.space_after = m.space_after;
            s.line_spacing = m.line_spacing;
            s.tabs = tabs.clone();
        });
    }

    /// Sets the gap above (`after = false`) or below the selected paragraphs, in points.
    pub fn set_para_gap(&mut self, after: bool, points: Option<u8>) {
        let (a, b) = self.selected_paras();
        self.edit_para_styles(a, b, |s| {
            if after {
                s.space_after = points;
            } else {
                s.space_before = points;
            }
        });
    }

    /// The gap above / below the first selected paragraph in points (`None` = the default).
    pub fn para_gap(&self, after: bool) -> Option<u8> {
        let s = &self.doc.paragraphs[self.sel.head.para].style;
        if after { s.space_after } else { s.space_before }
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
        if self.table_info().is_some() {
            return self.next_cell(!shift);
        }
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
