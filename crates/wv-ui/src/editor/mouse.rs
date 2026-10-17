use gpui_kit::*;
use wv_core::Position;

use super::view::EditorView;

impl EditorView {
    /// Document position under a window point (nearest page and line when outside any text).
    fn hit(&self, point: Point<Pixels>, window: &Window) -> Option<Position> {
        let layout = self.layout.as_ref()?;
        let bounds = self.page_bounds.borrow();
        // The row of pages under the pointer (pages side by side share a bottom edge), then the
        // page of that row nearest horizontally.
        let row = bounds
            .iter()
            .position(|b| point.y <= b.bottom())
            .unwrap_or(bounds.len().checked_sub(1)?);
        let bottom = f32::from(bounds[row].bottom());
        let distance = |b: &Bounds<Pixels>| {
            let (left, right) = (f32::from(b.left()), f32::from(b.right()));
            (left - f32::from(point.x))
                .max(f32::from(point.x) - right)
                .max(0.0)
        };
        let page = (0..bounds.len())
            .filter(|i| (f32::from(bounds[*i].bottom()) - bottom).abs() < 1.0)
            .min_by(|a, b| distance(&bounds[*a]).total_cmp(&distance(&bounds[*b])))
            .unwrap_or(row);
        let b = bounds[page];
        Some(layout.position_at(
            page,
            f32::from(point.x - b.origin.x) / self.zoom,
            f32::from(point.y - b.origin.y) / self.zoom,
            &self.measurer(window),
        ))
    }

    pub(super) fn on_mouse_down(
        &mut self,
        ev: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus, cx);
        if self.start_resize(ev.position) || self.start_border_drag(ev.position) {
            return;
        }
        let Some(at) = self.hit(ev.position, window) else {
            return;
        };
        // Ctrl/Cmd + click on a contents entry goes to its heading.
        if ev.modifiers.secondary()
            && let Some(heading) = self.core.toc_target(at.para)
        {
            self.apply(cx, |core| core.set_head(Position::new(heading, 0), false));
            return;
        }
        // Ctrl/Cmd + click on a link opens it.
        if ev.modifiers.secondary()
            && let Some(url) = self.core.doc.paragraphs[at.para]
                .style_after(at.offset)
                .and_then(|s| s.link)
        {
            cx.open_url(&url);
            return;
        }
        self.apply(cx, |core| match ev.click_count {
            2 => core.select_word(at),
            3.. => core.select_paragraph(at.para),
            _ => core.set_head(at, ev.modifiers.shift),
        });
        self.selecting = ev.click_count == 1;
        // The clicked point is already visible: do not scroll to the caret.
        self.reveal_caret.set(false);
    }

    /// Right click: the caret goes under the pointer unless it is inside the selection, then the
    /// window is asked for its context menu.
    pub(super) fn on_right_down(
        &mut self,
        ev: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus, cx);
        if let Some(at) = self.hit(ev.position, window) {
            let (start, end) = self.core.sel.range();
            if start == end || at < start || at > end {
                self.apply(cx, |core| core.set_head(at, false));
            }
        }
        self.reveal_caret.set(false);
        self.context_request = Some(ev.position);
        cx.notify();
    }

    /// Drag-selection, handled once for the whole viewport.
    pub(super) fn on_drag_move(
        &mut self,
        ev: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.border_drag.is_some() {
            if ev.pressed_button == Some(MouseButton::Left) {
                self.drag_border(ev.position, cx);
            } else {
                self.border_drag = None;
            }
            return;
        }
        if self.resizing.is_some() {
            if ev.pressed_button == Some(MouseButton::Left) {
                self.drag_resize(ev.position, cx);
            } else {
                self.resizing = None;
            }
            return;
        }
        if !self.selecting || ev.pressed_button != Some(MouseButton::Left) {
            return;
        }
        if let Some(at) = self.hit(ev.position, window) {
            self.apply(cx, |core| core.set_head(at, true));
        }
    }

    pub(super) fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.selecting = false;
        self.resizing = None;
        self.border_drag = None;
    }
}
