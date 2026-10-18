use gpui_kit::*;

use super::page_paint::PageView;
use super::table_resize::{Grab, grab_at};

/// Paints the tables on this page: cell backgrounds, then the borders. The pointer on a column
/// border gets the resize cursor.
pub fn paint_tables(
    v: &PageView,
    at: &dyn Fn(f32, f32) -> Point<Pixels>,
    visible: &dyn Fn(f32, f32) -> bool,
    window: &mut Window,
) {
    let z = v.zoom;
    let (left, top) = (
        v.layout.setup_of(v.page).margin_left,
        v.layout.setup_of(v.page).margin_top,
    );
    let mouse = window.mouse_position();
    for table in &v.layout.pages[v.page].tables {
        let (first, last) = (table.row_y[0], table.row_y[table.row_y.len() - 1]);
        if !visible(top + first, last - first) {
            continue;
        }
        for shade in &table.shades {
            window.paint_quad(fill(
                Bounds::new(
                    at(left + shade.x, top + shade.y),
                    size(px(shade.width * z), px(shade.height * z)),
                ),
                rgb(shade.color),
            ));
        }
        // Cells picked with the mouse (a block between two cells) are highlighted as cells.
        if let Some(sel) = v.cell_selection.filter(|s| s.table == table.table) {
            for c in &table.cells {
                let (row, col) = (c.row as u16, c.col as u16);
                if row >= sel.r0 && row <= sel.r1 && col >= sel.c0 && col <= sel.c1 {
                    window.paint_quad(fill(
                        Bounds::new(
                            at(left + c.x, top + c.y),
                            size(px(c.width * z), px(c.height * z)),
                        ),
                        v.selection_color,
                    ));
                }
            }
        }
        // Borders go around every cell, so merged cells show no line inside. They stay a pixel
        // thick when zoomed out (a thinner line would vanish).
        let line = (z).max(1.0);
        for c in &table.cells {
            let (x, y, w, h) = (left + c.x, top + c.y, c.width, c.height);
            let edges = [
                (x, y - 0.5, w * z, line),
                (x, y + h - 0.5, w * z, line),
                (x - 0.5, y, line, h * z),
                (x + w - 0.5, y, line, h * z),
            ];
            for (lx, ly, width, height) in edges {
                window.paint_quad(fill(
                    Bounds::new(at(lx, ly), size(px(width), px(height))),
                    v.table_color,
                ));
            }
        }
        // A border under the pointer can be dragged.
        let origin = (f32::from(at(0.0, 0.0).x), f32::from(at(0.0, 0.0).y));
        let pointer = (f32::from(mouse.x), f32::from(mouse.y));
        match grab_at(table, v.layout.setup_of(v.page), origin, z, pointer) {
            Some((Grab::Column(_), _)) => {
                window.set_window_cursor_style(CursorStyle::ResizeLeftRight)
            }
            Some((Grab::Row(_), _)) => window.set_window_cursor_style(CursorStyle::ResizeUpDown),
            None => {}
        }
    }
}
