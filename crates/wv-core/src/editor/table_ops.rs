use super::Editor;
use crate::document::{Paragraph, TableLook};
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
    /// Rows and columns the caret's cell covers.
    pub rowspan: u16,
    pub colspan: u16,
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
            (r.max(cell.row_end()), c.max(cell.col_end()))
        });
        Some(TableInfo {
            table: cell.table,
            rows,
            cols,
            row: cell.row,
            col: cell.col,
            rowspan: u16::from(cell.rowspan.max(1)),
            colspan: u16::from(cell.colspan.max(1)),
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
            .map(|(row, col)| cell_paragraph(Cell::at(table, row, col)))
            .collect();
        // Typing must be possible after the table.
        if at >= self.doc.paragraphs.len() {
            paras.push(Paragraph::default());
        }
        self.edit_paragraphs(at, 0, paras, Selection::caret(Position::new(at, 0)));
    }

    /// Rebuilds the caret's table: `f` edits the table's paragraphs and returns the (row, col) of the
    /// cell that gets the caret. If no cells remain the table disappears.
    pub(super) fn rebuild_table(
        &mut self,
        f: impl FnOnce(&mut Vec<Paragraph>, TableInfo) -> (u16, u16),
    ) {
        self.rebuild_table_with(f, |_, _| None);
    }

    /// `rebuild_table` that also changes the table's look (in the same undo step):
    /// `look(info, current)` returns the new look, one without columns to forget it, or `None` to
    /// keep it.
    pub(super) fn rebuild_table_with(
        &mut self,
        f: impl FnOnce(&mut Vec<Paragraph>, TableInfo) -> (u16, u16),
        look: impl FnOnce(&TableInfo, TableLook) -> Option<TableLook>,
    ) {
        let Some(info) = self.table_info() else {
            return;
        };
        let current = self.table_look(info.table);
        let settings = look(&info, current).map(|new| {
            let mut s = self.doc.settings();
            if new.widths.is_empty() {
                s.table_looks.remove(&info.table);
            } else {
                s.table_looks.insert(info.table, new.normalized());
            }
            s
        });
        let mut paras = self.doc.paragraphs[info.first..=info.last].to_vec();
        let (row, col) = f(&mut paras, info);
        paras.sort_by_key(|p| p.style.cell.map(|c| (c.row, c.col)));
        let caret_para = paras
            .iter()
            .position(|p| p.style.cell.is_some_and(|c| c.covers(row, col)))
            .map_or(0, |i| info.first + i);
        let removed = info.last - info.first + 1;
        let sel = Selection::caret(Position::new(caret_para, 0));
        self.edit_paragraphs_with(info.first, removed, paras, sel, settings);
        // After deleting a whole table the caret may point past the end.
        let last = self.doc.paragraphs.len() - 1;
        if self.sel.head.para > last {
            self.sel = Selection::caret(Position::new(last, 0));
        }
    }

    pub fn delete_table(&mut self) {
        self.rebuild_table_with(
            |paras, _| {
                paras.clear();
                (0, 0)
            },
            |_, _| Some(TableLook::default()),
        );
    }

    /// Tab / Shift+Tab inside a table: next or previous cell in reading order (its text gets
    /// selected); Tab in the last cell adds a row.
    pub fn next_cell(&mut self, forward: bool) {
        let Some(info) = self.table_info() else {
            return;
        };
        // First paragraph of every cell, in document order.
        let mut cells: Vec<(usize, (u16, u16))> = Vec::new();
        for (i, p) in self.doc.paragraphs[info.first..=info.last]
            .iter()
            .enumerate()
        {
            if let Some(c) = p.style.cell {
                let key = (c.row, c.col);
                if cells.last().is_none_or(|(_, last)| *last != key) {
                    cells.push((info.first + i, key));
                }
            }
        }
        let Some(at) = cells.iter().position(|(_, k)| *k == (info.row, info.col)) else {
            return;
        };
        let target = if forward {
            if at + 1 == cells.len() {
                // The caret lands in the new row's first cell.
                self.insert_row(true);
                return;
            }
            at + 1
        } else if at == 0 {
            return;
        } else {
            at - 1
        };
        let para = cells[target].0;
        self.select_range(
            Position::new(para, 0),
            Position::new(para, self.doc.paragraphs[para].len()),
        );
    }
}

pub(super) fn cell_paragraph(cell: Cell) -> Paragraph {
    let mut p = Paragraph::default();
    p.style.cell = Some(cell);
    p
}
