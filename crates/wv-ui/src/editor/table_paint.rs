use gpui_kit::*;

use super::page_paint::PageView;

/// Pointer distance (px) from a column border at which it can be grabbed.
pub const GRAB: f32 = 4.0;

/// Paints the tables on this page: cell backgrounds, then the borders. The pointer on a column
/// border gets the resize cursor.
pub fn paint_tables(
    v: &PageView,
    at: &dyn Fn(f32, f32) -> Point<Pixels>,
    visible: &dyn Fn(f32, f32) -> bool,
    window: &mut Window,
) {
    let z = v.zoom;
    let (left, top) = (v.layout.setup.margin_left, v.layout.setup.margin_top);
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
        // Borders go around every cell, so merged cells show no line inside.
        for c in &table.cells {
            let (x, y, w, h) = (left + c.x, top + c.y, c.width, c.height);
            for (lx, ly, lw, lh) in [
                (x, y - 0.5, w, 1.0),
                (x, y + h - 0.5, w, 1.0),
                (x - 0.5, y, 1.0, h),
                (x + w - 0.5, y, 1.0, h),
            ] {
                window.paint_quad(fill(
                    Bounds::new(at(lx, ly), size(px(lw * z), px(lh * z))),
                    v.table_color,
                ));
            }
        }
        for (i, x) in table.col_x.iter().enumerate() {
            // Inner borders can be dragged.
            let inner = i > 0 && i + 1 < table.col_x.len();
            let (bx, top_y) = (at(left + x, top + first), at(left + x, top + last));
            if inner
                && (f32::from(mouse.x) - f32::from(bx.x)).abs() <= GRAB
                && mouse.y >= bx.y
                && mouse.y <= top_y.y
            {
                window.set_window_cursor_style(CursorStyle::ResizeLeftRight);
            }
        }
    }
}
