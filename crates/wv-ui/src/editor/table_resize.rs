use gpui_kit::*;
use wv_core::{MIN_COLUMN, TableLook};
use wv_layout::{PageSetup, TableBox};

use super::view::EditorView;

/// Pointer distance (px) from a table border at which it can be grabbed.
pub const GRAB: f32 = 4.0;

/// A table border under the pointer.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Grab {
    /// Vertical border `0..=cols` (0 = left edge, cols = right edge).
    Column(usize),
    /// The border below row `r` (the bottom edge of the table for the last row).
    Row(usize),
}

/// The border of `table` (part of a table on one page) at window point `at`; `origin` is the
/// page's top-left corner in the window. Also gives the height of the row above a row border.
pub fn grab_at(
    table: &TableBox,
    setup: &PageSetup,
    origin: (f32, f32),
    zoom: f32,
    at: (f32, f32),
) -> Option<(Grab, f32)> {
    let sx = |x: f32| origin.0 + (setup.margin_left + x) * zoom;
    let sy = |y: f32| origin.1 + (setup.margin_top + y) * zoom;
    let (left, right) = (sx(table.col_x[0]), sx(table.col_x[table.col_x.len() - 1]));
    let (top, bottom) = (sy(table.row_y[0]), sy(table.row_y[table.row_y.len() - 1]));
    if at.1 >= top - GRAB && at.1 <= bottom + GRAB {
        for (i, x) in table.col_x.iter().enumerate() {
            if (at.0 - sx(*x)).abs() <= GRAB {
                return Some((Grab::Column(i), 0.0));
            }
        }
    }
    if at.0 >= left - GRAB && at.0 <= right + GRAB {
        for k in 1..table.row_y.len() {
            if (at.1 - sy(table.row_y[k])).abs() <= GRAB {
                let height = table.row_y[k] - table.row_y[k - 1];
                return Some((Grab::Row(table.first_row + k - 1), height));
            }
        }
    }
    None
}

/// A table border being dragged.
pub struct BorderDrag {
    table: u32,
    grab: Grab,
    /// Pointer position (x for a column border, y for a row border) when the drag began.
    start: f32,
    /// How the table looked then.
    look: TableLook,
    /// Height of the row above the border then (px).
    row_height: f32,
    /// Width of the text area in layout px.
    content: f32,
    /// No step recorded yet (the first change starts the undo step).
    first: bool,
}

impl EditorView {
    /// Starts dragging when the point is on a table border.
    pub(super) fn start_border_drag(&mut self, at: Point<Pixels>) -> bool {
        let Some(layout) = self.layout.clone() else {
            return false;
        };
        let bounds = self.page_bounds.borrow().clone();
        let pointer = (f32::from(at.x), f32::from(at.y));
        for (page, b) in bounds.iter().enumerate() {
            let Some(page_layout) = layout.pages.get(page) else {
                continue;
            };
            let origin = (f32::from(b.origin.x), f32::from(b.origin.y));
            for table in &page_layout.tables {
                let Some((grab, row_height)) =
                    grab_at(table, &layout.setup, origin, self.zoom, pointer)
                else {
                    continue;
                };
                self.border_drag = Some(BorderDrag {
                    table: table.table,
                    grab,
                    start: if matches!(grab, Grab::Column(_)) {
                        pointer.0
                    } else {
                        pointer.1
                    },
                    look: self.core.table_look(table.table),
                    row_height,
                    content: layout.setup.content_width(),
                    first: true,
                });
                return true;
            }
        }
        false
    }

    /// Continues a border drag: an inner column border trades width between its neighbours, the
    /// left and right edges move the table's edge, a row border sets the row's height.
    pub(super) fn drag_border(&mut self, at: Point<Pixels>, cx: &mut Context<Self>) {
        let zoom = self.zoom;
        let Some(d) = self.border_drag.as_mut() else {
            return;
        };
        let mut look = d.look.clone();
        match d.grab {
            Grab::Column(i) => {
                let shift = (f32::from(at.x) - d.start) / zoom / d.content;
                let cols = look.widths.len();
                if cols == 0 {
                    return;
                }
                if i == 0 {
                    let shift = shift.clamp(-look.indent, look.widths[0] - MIN_COLUMN);
                    look.indent += shift;
                    look.widths[0] -= shift;
                } else if i >= cols {
                    let room = 1.0 - look.indent - look.widths.iter().sum::<f32>();
                    let last = look.widths[cols - 1];
                    look.widths[cols - 1] += shift.clamp(MIN_COLUMN - last, room.max(0.0));
                } else {
                    let (a, b) = (look.widths[i - 1], look.widths[i]);
                    let shift = shift.clamp(MIN_COLUMN - a, b - MIN_COLUMN);
                    look.widths[i - 1] = a + shift;
                    look.widths[i] = b - shift;
                }
            }
            Grab::Row(r) => {
                let height = (d.row_height + (f32::from(at.y) - d.start) / zoom).max(0.0);
                if let Some(slot) = look.rows.get_mut(r) {
                    *slot = height;
                }
            }
        }
        let (table, first) = (d.table, std::mem::take(&mut d.first));
        self.apply(cx, |core| core.drag_table_look(table, look, first));
        self.reveal_caret.set(false);
    }
}
