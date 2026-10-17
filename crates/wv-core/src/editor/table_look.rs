use super::Editor;
use crate::document::TableLook;
use crate::position::Selection;

impl Editor {
    /// (rows, columns) of a table.
    pub fn table_dims(&self, table: u32) -> (usize, usize) {
        self.doc
            .paragraphs
            .iter()
            .filter_map(|p| p.style.cell)
            .filter(|c| c.table == table)
            .fold((0, 0), |(r, c), cell| {
                (
                    r.max(usize::from(cell.row_end())),
                    c.max(usize::from(cell.col_end())),
                )
            })
    }

    /// How `table` is sized (defaults filled in).
    pub fn table_look(&self, table: u32) -> TableLook {
        let (rows, cols) = self.table_dims(table);
        TableLook::resolved(self.doc.table_looks.get(&table), rows, cols)
    }

    /// Sets how a table is sized (one undo step).
    pub fn set_table_look(&mut self, table: u32, look: TableLook) {
        let mut settings = self.doc.settings();
        settings.table_looks.insert(table, look.normalized());
        self.change_settings(settings);
    }

    /// Like `set_table_look` for a drag: the first call (`first`) starts an undo step, the
    /// following ones change that step.
    pub fn drag_table_look(&mut self, table: u32, look: TableLook, first: bool) {
        if first {
            return self.set_table_look(table, look);
        }
        let look = look.normalized();
        self.amend_last_settings(|s| {
            s.table_looks.insert(table, look);
        });
    }

    /// Fills the cells touched by the selection (or the caret's cell) with `shade`.
    pub fn set_cell_shade(&mut self, shade: Option<u32>) {
        let Some(info) = self.table_info() else {
            return;
        };
        let Some((_, rect)) = self.selected_rect(&info) else {
            return;
        };
        let mut paras = self.doc.paragraphs[info.first..=info.last].to_vec();
        for cell in paras.iter_mut().filter_map(|p| p.style.cell.as_mut()) {
            if cell.row >= rect.r0
                && cell.row <= rect.r1
                && cell.col >= rect.c0
                && cell.col <= rect.c1
            {
                cell.shade = shade;
            }
        }
        let sel: Selection = self.sel;
        self.edit_paragraphs(info.first, paras.len(), paras, sel);
    }
}

#[cfg(test)]
mod tests {
    use crate::document::{Document, TableLook};
    use crate::editor::Editor;

    fn table(rows: u16, cols: u16) -> Editor {
        let mut e = Editor::new(Document::from_text("x"));
        e.insert_table(rows, cols);
        e
    }

    fn look(widths: Vec<f32>, indent: f32, rows: Vec<f32>) -> TableLook {
        TableLook {
            widths,
            indent,
            rows,
        }
    }

    #[test]
    fn a_table_is_equal_until_sized_and_dragging_is_one_undo_step() {
        let mut e = table(2, 3);
        let t = e.table_info().unwrap().table;
        assert_eq!(e.table_look(t).widths, vec![1.0 / 3.0; 3]);
        for (i, w) in [0.3, 0.35, 0.4].into_iter().enumerate() {
            e.drag_table_look(t, look(vec![w, 0.3, 0.2], 0.1, vec![0.0, 50.0]), i == 0);
        }
        let now = e.table_look(t);
        assert!((now.widths[0] - 0.4).abs() < 1e-5);
        assert_eq!((now.indent, now.rows[1]), (0.1, 50.0));
        assert!(e.undo());
        assert_eq!(e.table_look(t), TableLook::resolved(None, 2, 3));
        assert!(e.redo());
        assert_eq!(e.table_look(t).rows[1], 50.0);
    }

    #[test]
    fn inserting_and_deleting_rows_and_columns_keeps_the_look_in_step() {
        let mut e = table(2, 2);
        let t = e.table_info().unwrap().table;
        e.set_table_look(t, look(vec![0.5, 0.2], 0.0, vec![30.0, 60.0]));
        e.insert_col(true);
        let w = e.table_look(t).widths;
        assert_eq!(w.len(), 3);
        assert!(w[0] > w[1], "{w:?}");
        assert!(e.undo());
        assert_eq!(e.table_look(t).widths, vec![0.5, 0.2]);
        assert!(e.redo());
        e.delete_col();
        assert_eq!(e.table_look(t).widths.len(), 2);
        // A row inserted above the second row: its height slot comes with it.
        e.next_cell(true);
        e.insert_row(false);
        assert_eq!(e.table_look(t).rows.len(), 3);
        e.delete_row();
        assert_eq!(e.table_look(t).rows.len(), 2);
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
