use std::collections::{BTreeMap, HashMap};

use wv_core::Document;

/// Space between a cell's border and its text (px).
pub const CELL_PADDING: f32 = 6.0;

/// Smallest text height of a row (about one line).
pub const MIN_ROW_TEXT: f32 = 18.0;

/// Gap below a table.
pub const TABLE_GAP: f32 = 8.0;

/// A table of the document: `rows[row][col]` lists the paragraphs of that cell (document order).
pub struct TableBlock {
    pub table: u32,
    /// Column widths as fractions of the text width.
    pub fractions: Vec<f32>,
    pub rows: Vec<Vec<Vec<usize>>>,
    /// Background of every cell, `shades[row][col]`.
    pub shades: Vec<Vec<Option<u32>>>,
}

/// The document as a flow of body paragraphs and tables.
pub enum Block {
    Para(usize),
    Table(TableBlock),
}

/// Number of columns of every table (by table id).
pub fn column_counts(doc: &Document) -> HashMap<u32, usize> {
    let mut counts: HashMap<u32, usize> = HashMap::new();
    for cell in doc.paragraphs.iter().filter_map(|p| p.style.cell) {
        let entry = counts.entry(cell.table).or_default();
        *entry = (*entry).max(usize::from(cell.col) + 1);
    }
    counts
}

/// Column widths of every table (by table id) as fractions of the text width.
pub fn column_fractions(doc: &Document) -> HashMap<u32, Vec<f32>> {
    column_counts(doc)
        .into_iter()
        .map(|(table, cols)| {
            let fractions = match doc.table_widths.get(&table) {
                Some(w) if w.len() == cols => w.clone(),
                _ => vec![1.0 / cols.max(1) as f32; cols],
            };
            (table, fractions)
        })
        .collect()
}

/// Text width inside a cell of column `col`.
pub fn inner_width(content_width: f32, fractions: &[f32], col: u16) -> f32 {
    let fraction = fractions.get(usize::from(col)).copied().unwrap_or(1.0);
    (content_width * fraction - 2.0 * CELL_PADDING).max(20.0)
}

/// Splits the document into blocks; consecutive paragraphs with the same table id form one table.
pub fn blocks(doc: &Document) -> Vec<Block> {
    let n = doc.paragraphs.len();
    let cell = |i: usize| doc.paragraphs[i].style.cell;
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let Some(first) = cell(i) else {
            out.push(Block::Para(i));
            i += 1;
            continue;
        };
        let mut cells: BTreeMap<(u16, u16), Vec<usize>> = BTreeMap::new();
        let mut j = i;
        while j < n && cell(j).is_some_and(|c| c.table == first.table) {
            let c = cell(j).expect("checked above");
            cells.entry((c.row, c.col)).or_default().push(j);
            j += 1;
        }
        let rows = cells
            .keys()
            .map(|(r, _)| usize::from(*r) + 1)
            .max()
            .unwrap_or(0);
        let cols = cells
            .keys()
            .map(|(_, c)| usize::from(*c) + 1)
            .max()
            .unwrap_or(0);
        let mut grid = vec![vec![Vec::new(); cols]; rows];
        let mut shades = vec![vec![None; cols]; rows];
        for ((r, c), paragraphs) in cells {
            shades[usize::from(r)][usize::from(c)] = doc.paragraphs[paragraphs[0]]
                .style
                .cell
                .and_then(|cell| cell.shade);
            grid[usize::from(r)][usize::from(c)] = paragraphs;
        }
        let fractions = match doc.table_widths.get(&first.table) {
            Some(w) if w.len() == cols => w.clone(),
            _ => vec![1.0 / cols.max(1) as f32; cols],
        };
        out.push(Block::Table(TableBlock {
            table: first.table,
            fractions,
            rows: grid,
            shades,
        }));
        i = j;
    }
    out
}

/// A cell background, relative to the page's text area.
#[derive(Clone, Debug, PartialEq)]
pub struct Shade {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// 0xRRGGBB.
    pub color: u32,
}

/// Geometry of (a page's part of) a table, relative to the page's text area.
#[derive(Clone, Debug, PartialEq)]
pub struct TableBox {
    /// Table id (`Cell::table`).
    pub table: u32,
    /// Filled cells of this part of the table.
    pub shades: Vec<Shade>,
    /// Column boundaries, `cols + 1` values starting at 0.
    pub col_x: Vec<f32>,
    /// Row boundaries on this page, `rows + 1` values starting at the table's top.
    pub row_y: Vec<f32>,
}
