use std::collections::{BTreeMap, HashMap};

use wv_core::Document;

/// Space between a cell's border and its text (px).
pub const CELL_PADDING: f32 = 6.0;

/// Smallest text height of a row (about one line).
pub const MIN_ROW_TEXT: f32 = 18.0;

/// Gap below a table.
pub const TABLE_GAP: f32 = 8.0;

/// A cell of a table: where it sits and how much it covers, its paragraphs (document order) and
/// background.
pub struct CellBlock {
    pub row: usize,
    pub col: usize,
    pub rowspan: usize,
    pub colspan: usize,
    pub paras: Vec<usize>,
    pub shade: Option<u32>,
}

/// A table of the document: its cells (merged ones once, at their top-left position).
pub struct TableBlock {
    pub table: u32,
    /// Column widths as fractions of the text width.
    pub fractions: Vec<f32>,
    pub rows: usize,
    pub cells: Vec<CellBlock>,
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
        *entry = (*entry).max(usize::from(cell.col_end()));
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

/// Text width inside a cell that starts at column `col` and covers `colspan` columns.
pub fn inner_width(content_width: f32, fractions: &[f32], col: u16, colspan: u8) -> f32 {
    let from = usize::from(col);
    let to = (from + usize::from(colspan.max(1))).min(fractions.len());
    let fraction: f32 = fractions.get(from..to).map_or(1.0, |f| f.iter().sum());
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
        let mut found: BTreeMap<(u16, u16), CellBlock> = BTreeMap::new();
        let mut j = i;
        while j < n && cell(j).is_some_and(|c| c.table == first.table) {
            let c = cell(j).expect("checked above");
            found
                .entry((c.row, c.col))
                .or_insert_with(|| CellBlock {
                    row: usize::from(c.row),
                    col: usize::from(c.col),
                    rowspan: usize::from(c.rowspan.max(1)),
                    colspan: usize::from(c.colspan.max(1)),
                    paras: Vec::new(),
                    shade: c.shade,
                })
                .paras
                .push(j);
            j += 1;
        }
        let rows = found.values().map(|c| c.row + c.rowspan).max().unwrap_or(0);
        let cols = found.values().map(|c| c.col + c.colspan).max().unwrap_or(0);
        let fractions = match doc.table_widths.get(&first.table) {
            Some(w) if w.len() == cols => w.clone(),
            _ => vec![1.0 / cols.max(1) as f32; cols],
        };
        out.push(Block::Table(TableBlock {
            table: first.table,
            fractions,
            rows,
            cells: found.into_values().collect(),
        }));
        i = j;
    }
    out
}

/// The area of a cell, relative to the page's text area.
#[derive(Clone, Debug, PartialEq)]
pub struct CellRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
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
    /// Every cell of this part of the table (merged ones once); borders go around them.
    pub cells: Vec<CellRect>,
    /// Filled cells of this part of the table.
    pub shades: Vec<Shade>,
    /// Column boundaries, `cols + 1` values starting at 0.
    pub col_x: Vec<f32>,
    /// Row boundaries on this page, `rows + 1` values starting at the table's top.
    pub row_y: Vec<f32>,
}
