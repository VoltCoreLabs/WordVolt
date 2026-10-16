use gpui_kit::*;
use wv_core::Position;

use super::align::index_at;
use super::view::EditorView;

impl EditorView {
    /// Byte offset in paragraph `ix` under `point` (clamped to a char boundary).
    pub(super) fn offset_at(&self, ix: usize, point: Point<Pixels>) -> usize {
        let layouts = self.layouts.borrow();
        let align = self.core.doc.paragraphs[ix].style.align;
        let raw = layouts.get(ix).map_or(0, |l| index_at(l, point, align));
        let text = self.core.doc.paragraphs[ix].text();
        let mut offset = raw.min(text.len());
        while !text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    pub(super) fn on_mouse_down(
        &mut self,
        ix: usize,
        ev: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus, cx);
        let at = Position::new(ix, self.offset_at(ix, ev.position));
        self.apply(cx, |core| match ev.click_count {
            2 => core.select_word(at),
            3.. => core.select_paragraph(ix),
            _ => core.set_head(at, ev.modifiers.shift),
        });
        self.selecting = ev.click_count == 1;
        cx.stop_propagation();
    }

    pub(super) fn on_mouse_drag(&mut self, ix: usize, ev: &MouseMoveEvent, cx: &mut Context<Self>) {
        if !self.selecting || ev.pressed_button != Some(MouseButton::Left) {
            return;
        }
        let at = Position::new(ix, self.offset_at(ix, ev.position));
        self.apply(cx, |core| core.set_head(at, true));
    }

    /// Click on the page margin or below the text: caret goes to the nearest end.
    pub(super) fn on_sheet_mouse_down(
        &mut self,
        ev: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus, cx);
        let below = self
            .layouts
            .borrow()
            .last()
            .is_some_and(|l| ev.position.y > l.bounds().bottom());
        if below {
            self.apply(cx, |core| core.move_doc_end(ev.modifiers.shift));
        }
    }

    pub(super) fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.selecting = false;
    }
}
