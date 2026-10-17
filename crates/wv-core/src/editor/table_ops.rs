use super::Editor;
use crate::document::Paragraph;
use crate::position::{Position, Selection};
use crate::style::Cell;

/// Where the caret is inside a table (all counts and indexes are 0-based).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableInfo {
    pub table: u32,
    pub rows: u16,
    pub cols: u16,
    pub row: u16,
    pub col: u16,
    /// Paragraph range of the whole table.
    pub first: usize,
    pub last: usize,
}

impl Editor {
    /// The table the caret is in, if any.
    pub fn table_info(&self) -> Option<TableInfo> {
        let at = self.sel.head.para;
        let cell = self.doc.paragraphs[at].style.cell?;
        let same = |i: usize| {
            self.doc
                .paragraphs
                .get(i)
                .and_then(|p| p.style.cell)
                .is_some_and(|c| c.table == cell.table)
        };
        let (mut first, mut last) = (at, at);
        while first > 0 && same(first - 1) {
            first -= 1;
        }
        while same(last + 1) {
            last += 1;
        }
        let cells = self.doc.paragraphs[first..=last]
            .iter()
            .filter_map(|p| p.style.cell);
        let (rows, cols) = cells.fold((0, 0), |(r, c), cell| {
            (r.max(cell.row + 1), c.max(cell.col + 1))
        });
        Some(TableInfo {
            table: cell.table,
            rows,
            cols,
            row: cell.row,
            col: cell.col,
            first,
            last,
        })
    }

    /// Inserts an empty `rows` x `cols` table after the caret's paragraph (not inside another table).
    pub fn insert_table(&mut self, rows: u16, cols: u16) {
        if rows == 0 || cols == 0 || self.table_info().is_some() {
            return;
        }
        let table = self
            .doc
            .paragraphs
            .iter()
            .filter_map(|p| p.style.cell)
            .map(|c| c.table + 1)
            .max()
            .unwrap_or(0);
        let at = self.sel.head.para + 1;
        let mut paras: Vec<Paragraph> = (0..rows)
            .flat_map(|row| (0..cols).map(move |col| (row, col)))
            .map(|(row, col)| cell_paragraph(Cell { table, row, col }))
            .collect();
        // Typing must be possible after the table.
        if at >= self.doc.paragraphs.len() {
            paras.push(Paragraph::default());
        }
        self.edit_paragraphs(at, 0, paras, Selection::caret(Position::new(at, 0)));
    }

    /// Rebuilds the caret's table: `f` edits the table's paragraphs and returns the (row, col) of the
    /// cell that gets the caret. If no cells remain the table disappears.
    fn rebuild_table(&mut self, f: impl FnOnce(&mut Vec<Paragraph>, TableInfo) -> (u16, u16)) {
        let Some(info) = self.table_info() else {
            return;
        };
        let mut paras = self.doc.paragraphs[info.first..=info.last].to_vec();
        let (row, col) = f(&mut paras, info);
        paras.sort_by_key(|p| p.style.cell.map(|c| (c.row, c.col)));
        let caret_para = paras
            .iter()
            .position(|p| p.style.cell.is_some_and(|c| (c.row, c.col) == (row, col)))
            .map_or(0, |i| info.first + i);
        let removed = info.last - info.first + 1;
        let sel = Selection::caret(Position::new(caret_para, 0));
        self.edit_paragraphs(info.first, removed, paras, sel);
        // After deleting a whole table the caret may point past the end.
        let last = self.doc.paragraphs.len() - 1;
        if self.sel.head.para > last {
            self.sel = Selection::caret(Position::new(last, 0));
        }
    }

    /// New empty row above or below the caret's row.
    pub fn insert_row(&mut self, below: bool) {
        self.rebuild_table(|paras, info| {
            let at = if below { info.row + 1 } else { info.row };
            for c in paras.iter_mut().filter_map(|p| p.style.cell.as_mut()) {
                if c.row >= at {
                    c.row += 1;
                }
            }
            paras.extend((0..info.cols).map(|col| {
                cell_paragraph(Cell {
                    table: info.table,
                    row: at,
                    col,
                })
            }));
            (at, info.col)
        });
    }

    /// New empty column left or right of the caret's column.
    pub fn insert_col(&mut self, right: bool) {
        self.rebuild_table(|paras, info| {
            let at = if right { info.col + 1 } else { info.col };
            for c in paras.iter_mut().filter_map(|p| p.style.cell.as_mut()) {
                if c.col >= at {
                    c.col += 1;
                }
            }
            paras.extend((0..info.rows).map(|row| {
                cell_paragraph(Cell {
                    table: info.table,
                    row,
                    col: at,
                })
            }));
            (info.row, at)
        });
    }

    /// Removes the caret's row (the whole table if it was the last row).
    pub fn delete_row(&mut self) {
        self.rebuild_table(|paras, info| {
            paras.retain(|p| p.style.cell.is_none_or(|c| c.row != info.row));
            for c in paras.iter_mut().filter_map(|p| p.style.cell.as_mut()) {
                if c.row > info.row {
                    c.row -= 1;
                }
            }
            (info.row.min(info.rows.saturating_sub(2)), info.col)
        });
    }

    /// Removes the caret's column (the whole table if it was the last column).
    pub fn delete_col(&mut self) {
        self.rebuild_table(|paras, info| {
            paras.retain(|p| p.style.cell.is_none_or(|c| c.col != info.col));
            for c in paras.iter_mut().filter_map(|p| p.style.cell.as_mut()) {
                if c.col > info.col {
                    c.col -= 1;
                }
            }
            (info.row, info.col.min(info.cols.saturating_sub(2)))
        });
    }

    pub fn delete_table(&mut self) {
        self.rebuild_table(|paras, _| {
            paras.clear();
            (0, 0)
        });
    }

    /// Tab / Shift+Tab inside a table: next or previous cell (its text gets selected); Tab in the
    /// last cell adds a row.
    pub fn next_cell(&mut self, forward: bool) {
        let Some(info) = self.table_info() else {
            return;
        };
        let (mut row, mut col) = (
            i32::from(info.row),
            i32::from(info.col) + if forward { 1 } else { -1 },
        );
        if col >= i32::from(info.cols) {
            (col, row) = (0, row + 1);
        } else if col < 0 {
            (col, row) = (i32::from(info.cols) - 1, row - 1);
        }
        if row >= i32::from(info.rows) {
            return self.insert_row(true);
        }
        if row < 0 {
            return;
        }
        let target = self.doc.paragraphs[info.first..=info.last]
            .iter()
            .position(|p| {
                p.style
                    .cell
                    .is_some_and(|c| (i32::from(c.row), i32::from(c.col)) == (row, col))
            });
        if let Some(i) = target {
            let para = info.first + i;
            self.select_range(
                Position::new(para, 0),
                Position::new(para, self.doc.paragraphs[para].len()),
            );
        }
    }
}

fn cell_paragraph(cell: Cell) -> Paragraph {
    let mut p = Paragraph::default();
    p.style.cell = Some(cell);
    p
}
