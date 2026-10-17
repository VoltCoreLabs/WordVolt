use std::collections::HashSet;

use wv_core::{Cell, Paragraph};

/// Orders the paragraphs of one parsed table by cell and fills cells the file left out (merged or
/// missing), so that every row has every column.
pub fn normalize(mut paras: Vec<Paragraph>, table: u32) -> Vec<Paragraph> {
    let mut seen = HashSet::new();
    let (mut rows, mut cols) = (0, 0);
    for c in paras.iter().filter_map(|p| p.style.cell) {
        seen.insert((c.row, c.col));
        rows = rows.max(c.row + 1);
        cols = cols.max(c.col + 1);
    }
    for row in 0..rows {
        for col in 0..cols {
            if !seen.contains(&(row, col)) {
                let mut p = Paragraph::default();
                p.style.cell = Some(Cell { table, row, col });
                paras.push(p);
            }
        }
    }
    paras.sort_by_key(|p| p.style.cell.map(|c| (c.row, c.col)));
    paras
}
