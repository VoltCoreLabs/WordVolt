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
    pub cols: usize,
    pub rows: Vec<Vec<Vec<usize>>>,
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

/// Text width inside a cell when the table has `cols` equal columns.
pub fn inner_width(content_width: f32, cols: usize) -> f32 {
    (content_width / cols.max(1) as f32 - 2.0 * CELL_PADDING).max(20.0)
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
        for ((r, c), paragraphs) in cells {
            grid[usize::from(r)][usize::from(c)] = paragraphs;
        }
        out.push(Block::Table(TableBlock { cols, rows: grid }));
        i = j;
    }
    out
}

/// Geometry of (a page's part of) a table, relative to the page's text area.
#[derive(Clone, Debug, PartialEq)]
pub struct TableBox {
    /// Column boundaries, `cols + 1` values starting at 0.
    pub col_x: Vec<f32>,
    /// Row boundaries on this page, `rows + 1` values starting at the table's top.
    pub row_y: Vec<f32>,
}
