use std::collections::BTreeSet;

use super::Editor;
use crate::position::Selection;

/// No column gets narrower than this share of the text width.
pub const MIN_COLUMN: f32 = 0.04;

/// Fractions that sum to 1, none below [`MIN_COLUMN`].
pub fn normalize(widths: Vec<f32>) -> Vec<f32> {
    if widths.is_empty() {
        return widths;
    }
    let clamped: Vec<f32> = widths
        .iter()
        .map(|w| {
            if w.is_finite() {
                w.max(MIN_COLUMN)
            } else {
                MIN_COLUMN
            }
        })
        .collect();
    let sum: f32 = clamped.iter().sum();
    clamped.into_iter().map(|w| w / sum).collect()
}

impl Editor {
    /// Column widths of `table` (`cols` columns) as fractions of the text width; equal unless set.
    pub fn col_fractions(&self, table: u32, cols: usize) -> Vec<f32> {
        match self.doc.table_widths.get(&table) {
            Some(w) if w.len() == cols => w.clone(),
            _ => vec![1.0 / cols.max(1) as f32; cols],
        }
    }

    /// Sets the column widths of a table (one undo step).
    pub fn set_table_widths(&mut self, table: u32, widths: Vec<f32>) {
        let mut settings = self.doc.settings();
        settings.table_widths.insert(table, normalize(widths));
        self.change_settings(settings);
    }

    /// Like `set_table_widths` for a drag: the first call (`first`) starts an undo step, the
    /// following ones change that step.
    pub fn drag_table_widths(&mut self, table: u32, widths: Vec<f32>, first: bool) {
        if first {
            return self.set_table_widths(table, widths);
        }
        let widths = normalize(widths);
        self.amend_last_settings(|s| {
            s.table_widths.insert(table, widths);
        });
    }

    /// Fills the cells touched by the selection (or the caret's cell) with `shade`.
    pub fn set_cell_shade(&mut self, shade: Option<u32>) {
        let Some(info) = self.table_info() else {
            return;
        };
        let (start, end) = self.sel.range();
        let cells: BTreeSet<(u16, u16)> = self.doc.paragraphs
            [start.para.max(info.first)..=end.para.min(info.last)]
            .iter()
            .filter_map(|p| p.style.cell)
            .map(|c| (c.row, c.col))
            .collect();
        let mut paras = self.doc.paragraphs[info.first..=info.last].to_vec();
        for cell in paras.iter_mut().filter_map(|p| p.style.cell.as_mut()) {
            if cells.contains(&(cell.row, cell.col)) {
                cell.shade = shade;
            }
        }
        let sel: Selection = self.sel;
        self.edit_paragraphs(info.first, paras.len(), paras, sel);
    }
}

#[cfg(test)]
mod tests {
    use crate::document::Document;
    use crate::editor::Editor;

    fn table(rows: u16, cols: u16) -> Editor {
        let mut e = Editor::new(Document::from_text("x"));
        e.insert_table(rows, cols);
        e
    }

    #[test]
    fn widths_are_equal_until_set_and_dragging_is_one_undo_step() {
        let mut e = table(2, 3);
        let info = e.table_info().unwrap();
        assert_eq!(e.col_fractions(info.table, 3), vec![1.0 / 3.0; 3]);
        for (i, w) in [0.4, 0.45, 0.5].into_iter().enumerate() {
            e.drag_table_widths(info.table, vec![w, 0.6 - w + 0.2, 0.2], i == 0);
        }
        let now = e.col_fractions(info.table, 3);
        assert!((now.iter().sum::<f32>() - 1.0).abs() < 1e-5);
        assert!(now[0] > 0.4);
        assert!(e.undo());
        assert_eq!(e.col_fractions(info.table, 3), vec![1.0 / 3.0; 3]);
    }

    #[test]
    fn inserting_and_deleting_a_column_keeps_the_widths_in_step() {
        let mut e = table(1, 2);
        let t = e.table_info().unwrap().table;
        e.set_table_widths(t, vec![0.7, 0.3]);
        e.insert_col(true);
        let w = e.col_fractions(t, 3);
        assert_eq!(w.len(), 3);
        assert!((w.iter().sum::<f32>() - 1.0).abs() < 1e-5);
        assert!(w[0] > w[1] && w[0] > w[2]);
        assert!(e.undo());
        assert_eq!(e.col_fractions(t, 2), vec![0.7, 0.3]);
        assert!(e.redo());
        e.delete_col();
        assert_eq!(e.col_fractions(t, 2).len(), 2);
    }

    #[test]
    fn shading_fills_the_selected_cells_only() {
        let mut e = table(2, 2);
        e.set_cell_shade(Some(0xFFEE99));
        let shaded: Vec<_> = e
            .doc
            .paragraphs
            .iter()
            .filter_map(|p| p.style.cell)
            .filter(|c| c.shade.is_some())
            .map(|c| (c.row, c.col))
            .collect();
        assert_eq!(shaded.len(), 1);
        assert!(e.undo());
        assert!(
            e.doc
                .paragraphs
                .iter()
                .all(|p| p.style.cell.is_none_or(|c| c.shade.is_none()))
        );
    }
}
