use gpui_kit::*;
use wv_core::Position;

use super::view::EditorView;

impl EditorView {
    /// Moves the caret one visual line up (`-1`) or down (`1`), keeping the column.
    pub(super) fn move_vertical(&mut self, dir: i32, extend: bool, cx: &mut Context<Self>) {
        let head = self.core.sel.head;
        let target = {
            let layouts = self.layouts.borrow();
            let Some(layout) = layouts.get(head.para) else {
                return;
            };
            let Some(caret) = layout.position_for_index(head.offset) else {
                return;
            };
            let line = layout.line_height();
            let x = self.goal_x.unwrap_or(caret.x);
            let y = caret.y + line * 0.5 + line * dir as f32;
            let bounds = layout.bounds();
            if y >= bounds.top() && y <= bounds.bottom() {
                Some((head.para, point(x, y), x))
            } else {
                let next = head.para as i32 + dir;
                layouts.get(next as usize).filter(|_| next >= 0).map(|n| {
                    let nb = n.bounds();
                    let y = if dir > 0 {
                        nb.top() + line * 0.5
                    } else {
                        nb.bottom() - line * 0.5
                    };
                    (next as usize, point(x, y), x)
                })
            }
        };
        match target {
            Some((para, pt, x)) => {
                let at = Position::new(para, self.offset_at(para, pt));
                self.apply(cx, |core| core.set_head(at, extend));
                self.goal_x = Some(x);
            }
            None if dir < 0 => self.apply(cx, |core| core.move_doc_start(extend)),
            None => self.apply(cx, |core| core.move_doc_end(extend)),
        }
    }
}
