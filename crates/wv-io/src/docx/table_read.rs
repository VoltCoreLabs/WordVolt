use std::collections::HashSet;

use wv_core::{Cell, Paragraph};

/// Orders the paragraphs of one parsed table by cell and fills the positions the file left out
/// (the ones a merged cell covers need no cell), so that every row has every column.
pub fn normalize(mut paras: Vec<Paragraph>, table: u32) -> Vec<Paragraph> {
    let mut seen = HashSet::new();
    let (mut rows, mut cols) = (0, 0);
    let cells: Vec<Cell> = paras.iter().filter_map(|p| p.style.cell).collect();
    for c in &cells {
        seen.insert((c.row, c.col));
        rows = rows.max(c.row_end());
        cols = cols.max(c.col_end());
    }
    for row in 0..rows {
        for col in 0..cols {
            if !seen.contains(&(row, col)) && !cells.iter().any(|c| c.covers(row, col)) {
                let mut p = Paragraph::default();
                p.style.cell = Some(Cell::at(table, row, col));
                paras.push(p);
            }
        }
    }
    paras.sort_by_key(|p| p.style.cell.map(|c| (c.row, c.col)));
    paras
}

/// Column widths as fractions of the text width from the `w:tblGrid` widths (twips); `None` when
/// the grid does not match the table or the columns are equal anyway.
pub fn fractions(grid: &[u32], cols: usize) -> Option<Vec<f32>> {
    let sum: u32 = grid.iter().sum();
    if grid.len() != cols || cols == 0 || sum == 0 {
        return None;
    }
    let fractions: Vec<f32> = grid.iter().map(|w| *w as f32 / sum as f32).collect();
    let equal = 1.0 / cols as f32;
    (fractions.iter().any(|f| (f - equal).abs() > 0.01)).then_some(fractions)
}

/// `w:fill` of a `w:shd` ("auto" is no colour).
pub fn parse_fill(value: &str) -> Option<u32> {
    (value.len() == 6)
        .then(|| u32::from_str_radix(value, 16).ok())
        .flatten()
}
