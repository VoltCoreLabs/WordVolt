use std::collections::HashSet;

use wv_core::{Cell, Paragraph, TableLook};

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

/// What the file says about the size of one table (twips).
#[derive(Default)]
pub struct RawTable {
    pub grid: Vec<u32>,
    pub indent: u32,
    /// `w:trHeight` of every row (0 = none).
    pub rows: Vec<u32>,
}

/// The look of a table from what the file says; `None` when it is the default one (equal
/// columns over the text width, rows as tall as their text).
pub fn look_of(raw: &RawTable, text_width: f32, rows: usize, cols: usize) -> Option<TableLook> {
    if text_width <= 0.0 || cols == 0 {
        return None;
    }
    let widths: Vec<f32> = if raw.grid.len() == cols && raw.grid.iter().sum::<u32>() > 0 {
        raw.grid.iter().map(|w| *w as f32 / text_width).collect()
    } else {
        vec![1.0 / cols as f32; cols]
    };
    let heights: Vec<f32> = (0..rows)
        .map(|r| raw.rows.get(r).copied().unwrap_or(0) as f32 / 15.0)
        .collect();
    let look = TableLook {
        widths,
        indent: raw.indent as f32 / text_width,
        rows: heights,
    }
    .normalized();
    let equal = 1.0 / cols as f32;
    let default = look.indent < 0.001
        && look.rows.iter().all(|h| *h == 0.0)
        && look.widths.iter().all(|w| (w - equal).abs() < 0.01);
    (!default).then_some(look)
}

/// `w:fill` of a `w:shd` ("auto" is no colour).
pub fn parse_fill(value: &str) -> Option<u32> {
    (value.len() == 6)
        .then(|| u32::from_str_radix(value, 16).ok())
        .flatten()
}
