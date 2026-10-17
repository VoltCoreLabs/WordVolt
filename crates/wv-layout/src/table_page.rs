use crate::page::{EPS, Page, PageSetup, Slice};
use crate::paragraph::ParagraphLayout;
use crate::table::{CELL_PADDING, CellBlock, CellRect, MIN_ROW_TEXT, Shade, TableBlock, TableBox};

/// Flows a table onto the pages. A row never splits; rows tied together by a merged cell move as
/// one block.
pub(crate) fn paginate_table(
    paragraphs: &[ParagraphLayout],
    table: &TableBlock,
    setup: &PageSetup,
    pages: &mut Vec<Page>,
    y: &mut f32,
) {
    let limit = setup.content_height();
    let content = setup.content_width();
    let mut col_x = vec![0.0];
    for f in &table.fractions {
        col_x.push(col_x[col_x.len() - 1] + f * content);
    }
    let gap = |p: &ParagraphLayout| p.space_after.unwrap_or(setup.paragraph_spacing);
    let need = |cell: &CellBlock| {
        let total: f32 = cell.paras.iter().map(|i| paragraphs[*i].height).sum();
        let gaps: f32 = cell
            .paras
            .iter()
            .take(cell.paras.len().saturating_sub(1))
            .map(|i| gap(&paragraphs[*i]))
            .sum();
        total.max(MIN_ROW_TEXT) + gaps + 2.0 * CELL_PADDING
    };

    // Row heights: single-row cells decide; a taller merged cell stretches its last row.
    let mut heights = vec![MIN_ROW_TEXT + 2.0 * CELL_PADDING; table.rows];
    for cell in table.cells.iter().filter(|c| c.rowspan == 1) {
        heights[cell.row] = heights[cell.row].max(need(cell));
    }
    for cell in table.cells.iter().filter(|c| c.rowspan > 1) {
        let last = (cell.row + cell.rowspan).min(table.rows) - 1;
        let have: f32 = heights[cell.row..=last].iter().sum();
        heights[last] += (need(cell) - have).max(0.0);
    }

    let mut open_box: Option<usize> = None;
    let mut row = 0;
    while row < table.rows {
        // The block of rows that must stay together: no merged cell may cross its lower edge.
        let mut end = row + 1;
        while end < table.rows
            && table
                .cells
                .iter()
                .any(|c| c.row < end && c.row + c.rowspan > end)
        {
            end += 1;
        }
        let block_height: f32 = heights[row..end].iter().sum();
        if *y + block_height > limit + EPS && !pages.last().is_some_and(Page::is_empty) {
            pages.push(Page::default());
            *y = 0.0;
            open_box = None;
        }
        let page = pages.last_mut().expect("at least one page");
        let box_ix = *open_box.get_or_insert_with(|| {
            page.tables.push(TableBox {
                table: table.table,
                cells: Vec::new(),
                shades: Vec::new(),
                col_x: col_x.clone(),
                row_y: vec![*y],
            });
            page.tables.len() - 1
        });
        let mut tops = Vec::with_capacity(end - row + 1);
        let mut at = *y;
        for h in &heights[row..end] {
            tops.push(at);
            at += h;
            page.tables[box_ix].row_y.push(at);
        }
        tops.push(at);
        for cell in table.cells.iter().filter(|c| (row..end).contains(&c.row)) {
            let last = (cell.row + cell.rowspan).min(table.rows);
            let (x0, x1) = (
                col_x[cell.col],
                col_x[(cell.col + cell.colspan).min(col_x.len() - 1)],
            );
            let (y0, y1) = (tops[cell.row - row], tops[last.min(end) - row]);
            page.tables[box_ix].cells.push(CellRect {
                x: x0,
                y: y0,
                width: x1 - x0,
                height: y1 - y0,
            });
            if let Some(color) = cell.shade {
                page.tables[box_ix].shades.push(Shade {
                    x: x0,
                    y: y0,
                    width: x1 - x0,
                    height: y1 - y0,
                    color,
                });
            }
            let mut cell_y = y0 + CELL_PADDING;
            for (k, &para_ix) in cell.paras.iter().enumerate() {
                let para = &paragraphs[para_ix];
                page.slices.push(Slice {
                    para: para_ix,
                    lines: 0..para.lines.len(),
                    y: cell_y,
                    height: para.height,
                    x: x0 + CELL_PADDING,
                    width: (x1 - x0 - 2.0 * CELL_PADDING).max(20.0),
                });
                cell_y += para.height
                    + if k + 1 < cell.paras.len() {
                        gap(para)
                    } else {
                        0.0
                    };
            }
        }
        *y = at;
        row = end;
    }
}
