use gpui_kit::*;
use wv_core::Position;

use super::view::EditorView;

impl EditorView {
    /// Document position under a window point (nearest page and line when outside any text).
    fn hit(&self, point: Point<Pixels>, window: &Window) -> Option<Position> {
        let layout = self.layout.as_ref()?;
        let bounds = self.page_bounds.borrow();
        // Only rendered pages have bounds. The row under the pointer (pages side by side share a
        // bottom edge), then the page of that row nearest horizontally.
        let rendered: Vec<usize> = (0..bounds.len())
            .filter(|i| bounds[*i].size.height > px(0.))
            .collect();
        let row_page = rendered
            .iter()
            .copied()
            .find(|i| point.y <= bounds[*i].bottom())
            .or_else(|| rendered.last().copied())?;
        let bottom = f32::from(bounds[row_page].bottom());
        let distance = |b: &Bounds<Pixels>| {
            let (left, right) = (f32::from(b.left()), f32::from(b.right()));
            (left - f32::from(point.x))
                .max(f32::from(point.x) - right)
                .max(0.0)
        };
        let page = rendered
            .iter()
            .copied()
            .filter(|i| (f32::from(bounds[*i].bottom()) - bottom).abs() < 1.0)
            .min_by(|a, b| distance(&bounds[*a]).total_cmp(&distance(&bounds[*b])))
            .unwrap_or(row_page);
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
        if self.start_image_drag(ev.position, cx) || self.start_border_drag(ev.position) {
            return;
        }
        // A click away from the picture ends cropping.
        self.cropping = false;
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
        if let Some(picture) = self.image_at(ev.position) {
            // A right click on a picture selects it (it has no text to put the caret in).
            if self.core.sel.head.para != picture.para {
                self.apply(cx, |core| {
                    core.set_head(Position::new(picture.para, 0), false)
                });
            }
        } else if let Some(at) = self.hit(ev.position, window) {
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
                self.drag_image(ev.position, cx);
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

    pub(super) fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.selecting = false;
        self.end_image_drag();
        self.border_drag = None;
        if !self.core.sel.is_collapsed()
            && let Some(style) = self.painter.take()
        {
            self.apply(cx, |c| c.paint_char_style(&style));
        }
    }
}
