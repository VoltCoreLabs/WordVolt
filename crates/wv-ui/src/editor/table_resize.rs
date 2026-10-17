use gpui_kit::*;
use wv_core::MIN_COLUMN;

use super::table_paint::GRAB;
use super::view::EditorView;

/// A column border being dragged.
pub struct ColDrag {
    table: u32,
    /// Index of the column to the right of the border.
    right: usize,
    start_x: f32,
    start: Vec<f32>,
    /// Width of the table in layout px.
    width: f32,
    /// No step recorded yet (the first change starts the undo step).
    first: bool,
}

impl EditorView {
    /// Starts dragging when the point is on an inner column border of a table.
    pub(super) fn start_col_drag(&mut self, at: Point<Pixels>) -> bool {
        let Some(layout) = self.layout.clone() else {
            return false;
        };
        let bounds = self.page_bounds.borrow().clone();
        let z = self.zoom;
        for (page, b) in bounds.iter().enumerate() {
            let Some(page_layout) = layout.pages.get(page) else {
                continue;
            };
            for table in &page_layout.tables {
                let top = f32::from(b.origin.y) + (layout.setup.margin_top + table.row_y[0]) * z;
                let bottom = f32::from(b.origin.y)
                    + (layout.setup.margin_top + table.row_y[table.row_y.len() - 1]) * z;
                if f32::from(at.y) < top || f32::from(at.y) > bottom {
                    continue;
                }
                let width = table.col_x[table.col_x.len() - 1];
                for (i, x) in table.col_x.iter().enumerate() {
                    if i == 0 || i + 1 == table.col_x.len() {
                        continue;
                    }
                    let screen = f32::from(b.origin.x) + (layout.setup.margin_left + x) * z;
                    if (f32::from(at.x) - screen).abs() <= GRAB {
                        let start = table
                            .col_x
                            .windows(2)
                            .map(|w| (w[1] - w[0]) / width)
                            .collect();
                        self.col_drag = Some(ColDrag {
                            table: table.table,
                            right: i,
                            start_x: f32::from(at.x),
                            start,
                            width,
                            first: true,
                        });
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Continues a column drag: the border moves, the two columns beside it trade width.
    pub(super) fn drag_col(&mut self, at: Point<Pixels>, cx: &mut Context<Self>) {
        let zoom = self.zoom;
        let Some(d) = self.col_drag.as_mut() else {
            return;
        };
        let shift = (f32::from(at.x) - d.start_x) / zoom / d.width;
        let (a, b) = (d.start[d.right - 1], d.start[d.right]);
        let shift = shift.clamp(MIN_COLUMN - a, b - MIN_COLUMN);
        let mut widths = d.start.clone();
        widths[d.right - 1] = a + shift;
        widths[d.right] = b - shift;
        let (table, first) = (d.table, std::mem::take(&mut d.first));
        self.apply(cx, |core| core.drag_table_widths(table, widths, first));
        self.reveal_caret.set(false);
    }
}
