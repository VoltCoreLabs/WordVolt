use gpui_kit::*;
use wv_core::Position;

use super::view::EditorView;

impl EditorView {
    /// Moves the caret one visual line up (`-1`) or down (`1`), keeping the column; crosses pages.
    pub(super) fn move_vertical(
        &mut self,
        dir: i32,
        extend: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let Some(layout) = self.layout.clone() else {
            return;
        };
        let m = self.measurer(window);
        let head = self.core.sel.head;
        let Some(caret) = layout.caret(head, &m) else {
            return;
        };
        let setup = &layout.setup;
        let x = self.goal_x.unwrap_or(caret.x);
        let (mut page, mut y) = (
            caret.page,
            if dir > 0 {
                caret.y + caret.height + 1.0
            } else {
                caret.y - 1.0
            },
        );
        if dir > 0 && y >= setup.height - setup.margin_bottom {
            if page + 1 >= layout.pages.len() {
                return self.apply(cx, |c| c.move_doc_end(extend));
            }
            (page, y) = (page + 1, setup.margin_top + 1.0);
        } else if dir < 0 && y < setup.margin_top {
            if page == 0 {
                return self.apply(cx, |c| c.move_doc_start(extend));
            }
            (page, y) = (page - 1, setup.height - setup.margin_bottom - 1.0);
        }
        let target = layout.position_at(page, x, y, &m);
        if target == head {
            // Already on the first/last line of the document.
            return self.apply(cx, |c| {
                if dir > 0 {
                    c.move_doc_end(extend)
                } else {
                    c.move_doc_start(extend)
                }
            });
        }
        self.apply(cx, |c| c.set_head(target, extend));
        self.goal_x = Some(x);
    }

    /// Home / End: start or end of the visual line.
    pub(super) fn move_line_edge(&mut self, end: bool, extend: bool, cx: &mut Context<Self>) {
        let Some(layout) = self.layout.clone() else {
            return;
        };
        let head = self.core.sel.head;
        let (start_offset, end_offset) = layout.line_range(head);
        let target = Position::new(head.para, if end { end_offset } else { start_offset });
        self.apply(cx, |c| c.set_head(target, extend));
    }
}
