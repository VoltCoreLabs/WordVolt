use std::collections::BTreeMap;

use super::Editor;
use super::table_ops::{TableInfo, cell_paragraph};
use crate::document::Paragraph;
use crate::style::Cell;

/// The cells of a table by origin (row, col).
pub(super) type CellMap = BTreeMap<(u16, u16), Cell>;

/// Rectangle of grid positions (inclusive).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct Rect {
    pub(super) r0: u16,
    pub(super) c0: u16,
    pub(super) r1: u16,
    pub(super) c1: u16,
}

impl Rect {
    fn of(c: &Cell) -> Self {
        Self {
            r0: c.row,
            c0: c.col,
            r1: c.row_end() - 1,
            c1: c.col_end() - 1,
        }
    }

    fn join(self, o: Self) -> Self {
        Self {
            r0: self.r0.min(o.r0),
            c0: self.c0.min(o.c0),
            r1: self.r1.max(o.r1),
            c1: self.c1.max(o.c1),
        }
    }

    fn touches(&self, o: &Self) -> bool {
        self.r0 <= o.r1 && o.r0 <= self.r1 && self.c0 <= o.c1 && o.c0 <= self.c1
    }
}

/// The block of cells a mouse selection picked: from the cell where it started to the cell where
/// it ends (grown to whole merged cells).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellSelection {
    pub table: u32,
    pub r0: u16,
    pub c0: u16,
    pub r1: u16,
    pub c1: u16,
}

impl Editor {
    /// The selected block of cells, when the selection runs from one cell of a table to another.
    pub fn cell_selection(&self) -> Option<CellSelection> {
        let cell_of = |para: usize| self.doc.paragraphs.get(para).and_then(|p| p.style.cell);
        let (a, b) = (cell_of(self.sel.anchor.para)?, cell_of(self.sel.head.para)?);
        if a.table != b.table || (a.row, a.col) == (b.row, b.col) {
            return None;
        }
        let info = self.table_info()?;
        let (_, rect) = self.selected_rect(&info)?;
        Some(CellSelection {
            table: a.table,
            r0: rect.r0,
            c0: rect.c0,
            r1: rect.r1,
            c1: rect.c1,
        })
    }

    /// The cells (by origin) of the caret's table and the rectangle the selection spans, grown
    /// until it holds whole cells only.
    pub(super) fn selected_rect(&self, info: &TableInfo) -> Option<(CellMap, Rect)> {
        let mut cells: BTreeMap<(u16, u16), Cell> = BTreeMap::new();
        for p in &self.doc.paragraphs[info.first..=info.last] {
            if let Some(c) = p.style.cell {
                cells.entry((c.row, c.col)).or_insert(c);
            }
        }
        // The rectangle between the cell where the selection starts and the one where it ends
        // (a text selection runs row by row, but cells are picked as a block, like in Word).
        let (start, end) = self.sel.range();
        let cell_at = |para: usize| {
            self.doc.paragraphs[para.clamp(info.first, info.last)]
                .style
                .cell
        };
        let mut rect = Rect::of(&cell_at(start.para)?).join(Rect::of(&cell_at(end.para)?));
        loop {
            let grown = cells
                .values()
                .map(Rect::of)
                .filter(|r| r.touches(&rect))
                .fold(rect, Rect::join);
            if grown == rect {
                break;
            }
            rect = grown;
        }
        Some((cells, rect))
    }

    /// The selection covers several cells that can become one.
    pub fn can_merge(&self) -> bool {
        self.table_info()
            .and_then(|info| self.selected_rect(&info))
            .is_some_and(|(_, r)| r.r0 != r.r1 || r.c0 != r.c1)
    }

    /// The caret's cell is a merged one.
    pub fn can_split(&self) -> bool {
        self.table_info()
            .is_some_and(|i| i.rowspan > 1 || i.colspan > 1)
    }

    /// Merges the cells covered by the selection into one (the text of the others is appended).
    pub fn merge_cells(&mut self) {
        let Some(info) = self.table_info() else {
            return;
        };
        let Some((cells, rect)) = self.selected_rect(&info) else {
            return;
        };
        if rect.r0 == rect.r1 && rect.c0 == rect.c1 {
            return;
        }
        let Some(origin) = cells.get(&(rect.r0, rect.c0)).copied() else {
            return;
        };
        let merged = Cell {
            rowspan: (rect.r1 - rect.r0 + 1) as u8,
            colspan: (rect.c1 - rect.c0 + 1) as u8,
            ..origin
        };
        let inside =
            |c: &Cell| c.row >= rect.r0 && c.row <= rect.r1 && c.col >= rect.c0 && c.col <= rect.c1;
        self.rebuild_table(move |paras, _| {
            let mut kept = Vec::new();
            let mut moved: Vec<((u16, u16), Paragraph)> = Vec::new();
            for p in std::mem::take(paras) {
                match p.style.cell {
                    Some(c) if inside(&c) => moved.push(((c.row, c.col), p)),
                    _ => kept.push(p),
                }
            }
            // Row-major order of the original cells (a cell's own paragraphs keep their order);
            // empty paragraphs are dropped unless the whole merged cell is empty.
            moved.sort_by_key(|(key, _)| *key);
            let has_content = |p: &Paragraph| !p.is_empty() || p.style.image.is_some();
            let any = moved.iter().any(|(_, p)| has_content(p));
            let mut empty_kept = false;
            for (_, mut p) in moved {
                if has_content(&p) || (!any && !std::mem::replace(&mut empty_kept, true)) {
                    p.style.cell = Some(merged);
                    kept.push(p);
                }
            }
            *paras = kept;
            (merged.row, merged.col)
        });
    }

    /// Splits the caret's merged cell back into single cells.
    pub fn split_cell(&mut self) {
        self.rebuild_table(|paras, info| {
            let (row, col) = (info.row, info.col);
            let (rows, cols) = (info.rowspan, info.colspan);
            for c in paras.iter_mut().filter_map(|p| p.style.cell.as_mut()) {
                if (c.row, c.col) == (row, col) {
                    c.rowspan = 1;
                    c.colspan = 1;
                }
            }
            for r in row..row + rows {
                for k in col..col + cols {
                    if (r, k) != (row, col) {
                        paras.push(cell_paragraph(Cell::at(info.table, r, k)));
                    }
                }
            }
            (row, col)
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::document::{Document, Paragraph};
    use crate::editor::Editor;
    use crate::position::Position;

    /// A 3x3 table with "r.c" in every cell; paragraph 0 is the text before it.
    fn table() -> Editor {
        let mut e = Editor::new(Document::from_text("x"));
        e.insert_table(3, 3);
        let first = e.table_info().unwrap().first;
        for i in 0..9 {
            let cell = e.doc.paragraphs[first + i].style.cell;
            let mut p = Paragraph::plain(&format!("{}.{}", i / 3, i % 3));
            p.style.cell = cell;
            e.doc.paragraphs[first + i] = p;
        }
        e
    }

    fn cell_para(e: &Editor, row: u16, col: u16) -> usize {
        e.doc
            .paragraphs
            .iter()
            .position(|p| p.style.cell.is_some_and(|c| (c.row, c.col) == (row, col)))
            .unwrap()
    }

    fn select(e: &mut Editor, from: (u16, u16), to: (u16, u16)) {
        let (a, b) = (cell_para(e, from.0, from.1), cell_para(e, to.0, to.1));
        e.select_range(Position::new(a, 0), Position::new(b, 1));
    }

    #[test]
    fn merging_a_block_joins_the_text_and_splitting_gives_empty_cells_back() {
        let mut e = table();
        select(&mut e, (0, 0), (1, 1));
        assert!(e.can_merge());
        e.merge_cells();
        let info = e.table_info().unwrap();
        assert_eq!((info.rows, info.cols), (3, 3));
        assert_eq!((info.rowspan, info.colspan), (2, 2));
        let origin = cell_para(&e, 0, 0);
        let text: String = (0..4)
            .map(|k| e.doc.paragraphs[origin + k].text())
            .collect::<Vec<_>>()
            .join("|");
        assert_eq!(text, "0.0|0.1|1.0|1.1");
        // 3 cells of the block are gone, 4 paragraphs sit in the merged one.
        // (the text before the table and the paragraph after it are the extra 1 + 1)
        assert_eq!(e.doc.paragraphs.len(), 1 + 4 + 5 + 1);
        assert!(e.can_split());
        e.split_cell();
        assert_eq!(e.doc.paragraphs.len(), 1 + 4 + 5 + 3 + 1);
        assert!(!e.can_split());
        assert!(e.undo());
        assert!(e.can_split());
        assert!(e.undo());
        assert!(!e.can_split());
    }

    #[test]
    fn a_selection_that_cuts_a_merged_cell_grows_to_hold_it() {
        let mut e = table();
        select(&mut e, (0, 0), (0, 1));
        e.merge_cells();
        // (0,0) spans two columns: picking it with the cell below takes both columns along.
        select(&mut e, (0, 0), (1, 0));
        e.merge_cells();
        let info = e.table_info().unwrap();
        assert_eq!((info.rowspan, info.colspan), (2, 2));
    }

    #[test]
    fn rows_and_columns_next_to_merged_cells_adjust_the_spans() {
        let mut e = table();
        select(&mut e, (0, 0), (1, 1));
        e.merge_cells();
        // A row inserted inside the merged block makes it taller.
        e.select_range(
            Position::new(cell_para(&e, 2, 0), 0),
            Position::new(cell_para(&e, 2, 0), 0),
        );
        e.insert_row(false);
        let origin = e.doc.paragraphs[cell_para(&e, 0, 0)].style.cell.unwrap();
        assert_eq!(origin.rowspan, 2, "inserted below the block, not inside");
        e.select_range(
            Position::new(cell_para(&e, 0, 0), 0),
            Position::new(cell_para(&e, 0, 0), 0),
        );
        e.insert_row(true);
        let origin = e.doc.paragraphs[cell_para(&e, 0, 0)].style.cell.unwrap();
        assert_eq!(
            origin.rowspan, 2,
            "the caret's cell ends where the row goes in"
        );
        e.delete_col();
        let info = e.table_info().unwrap();
        assert_eq!(info.cols, 2, "the merged cell gave up one column");
        assert_eq!(info.colspan, 1);
    }

    #[test]
    fn dragging_from_cell_to_cell_picks_a_block() {
        let mut e = table();
        // Inside one cell: an ordinary text selection.
        let a = cell_para(&e, 1, 1);
        e.select_range(Position::new(a, 0), Position::new(a, 1));
        assert!(e.cell_selection().is_none());
        // From the top right to the bottom left cell: the block between them, not a row-wise run.
        let (from, to) = (cell_para(&e, 0, 2), cell_para(&e, 1, 0));
        e.select_range(Position::new(from, 0), Position::new(to, 1));
        let sel = e.cell_selection().unwrap();
        assert_eq!((sel.r0, sel.c0, sel.r1, sel.c1), (0, 0, 1, 2));
    }

    #[test]
    fn tab_walks_the_cells_in_reading_order() {
        let mut e = table();
        select(&mut e, (0, 0), (0, 1));
        e.merge_cells();
        e.next_cell(true);
        let info = e.table_info().unwrap();
        assert_eq!((info.row, info.col), (0, 2));
        e.next_cell(false);
        assert_eq!(e.table_info().unwrap().colspan, 2);
    }
}
