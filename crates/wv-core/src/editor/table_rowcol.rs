use std::collections::BTreeSet;

use super::Editor;
use super::table_ops::cell_paragraph;
use crate::style::Cell;

impl Editor {
    /// New empty row above or below the caret's cell. Merged cells that cross the new row grow
    /// instead of getting a cell of their own.
    pub fn insert_row(&mut self, below: bool) {
        self.rebuild_table(|paras, info| {
            let at = if below {
                info.row + info.rowspan
            } else {
                info.row
            };
            let mut covered: BTreeSet<u16> = BTreeSet::new();
            for c in paras.iter_mut().filter_map(|p| p.style.cell.as_mut()) {
                if c.row >= at {
                    c.row += 1;
                } else if c.row_end() > at {
                    c.rowspan = c.rowspan.saturating_add(1);
                    covered.extend(c.col..c.col_end());
                }
            }
            let mut first_new = None;
            for col in (0..info.cols).filter(|c| !covered.contains(c)) {
                first_new.get_or_insert(col);
                paras.push(cell_paragraph(Cell::at(info.table, at, col)));
            }
            (at, first_new.unwrap_or(info.col))
        });
    }

    /// New empty column left or right of the caret's cell (merged cells that cross it grow).
    pub fn insert_col(&mut self, right: bool) {
        self.rebuild_table_with(
            |paras, info| {
                let at = if right {
                    info.col + info.colspan
                } else {
                    info.col
                };
                let mut covered: BTreeSet<u16> = BTreeSet::new();
                for c in paras.iter_mut().filter_map(|p| p.style.cell.as_mut()) {
                    if c.col >= at {
                        c.col += 1;
                    } else if c.col_end() > at {
                        c.colspan = c.colspan.saturating_add(1);
                        covered.extend(c.row..c.row_end());
                    }
                }
                let mut first_new = None;
                for row in (0..info.rows).filter(|r| !covered.contains(r)) {
                    first_new.get_or_insert(row);
                    paras.push(cell_paragraph(Cell::at(info.table, row, at)));
                }
                (first_new.unwrap_or(info.row), at)
            },
            |info, mut widths| {
                // The new column takes an equal share; the others give way proportionally.
                let at = usize::from(if right {
                    info.col + info.colspan
                } else {
                    info.col
                });
                let share = 1.0 / (widths.len() + 1) as f32;
                widths.iter_mut().for_each(|w| *w *= 1.0 - share);
                widths.insert(at.min(widths.len()), share);
                Some(widths)
            },
        );
    }

    /// Removes the caret's row (the whole table if it was the last row). Merged cells that cross
    /// it get shorter; one that starts in it moves to the next row.
    pub fn delete_row(&mut self) {
        self.rebuild_table(|paras, info| {
            let row = info.row;
            paras.retain_mut(|p| {
                let Some(c) = p.style.cell.as_mut() else {
                    return true;
                };
                if c.row == row {
                    if c.rowspan > 1 {
                        c.rowspan -= 1;
                        true
                    } else {
                        false
                    }
                } else {
                    if c.row > row {
                        c.row -= 1;
                    } else if c.row_end() > row {
                        c.rowspan -= 1;
                    }
                    true
                }
            });
            (row.min(info.rows.saturating_sub(2)), info.col)
        });
    }

    /// Removes the caret's column (the whole table if it was the last column).
    pub fn delete_col(&mut self) {
        self.rebuild_table_with(
            |paras, info| {
                let col = info.col;
                paras.retain_mut(|p| {
                    let Some(c) = p.style.cell.as_mut() else {
                        return true;
                    };
                    if c.col == col {
                        if c.colspan > 1 {
                            c.colspan -= 1;
                            true
                        } else {
                            false
                        }
                    } else {
                        if c.col > col {
                            c.col -= 1;
                        } else if c.col_end() > col {
                            c.colspan -= 1;
                        }
                        true
                    }
                });
                (info.row, col.min(info.cols.saturating_sub(2)))
            },
            |info, mut widths| {
                widths.remove(usize::from(info.col).min(widths.len().saturating_sub(1)));
                Some(super::table_widths::normalize(widths))
            },
        );
    }
}
