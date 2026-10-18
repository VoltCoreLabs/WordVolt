use std::rc::Rc;

use gpui_kit::*;

use super::page_paint::{PageView, paint_page};
use super::view::{EditorView, FIT_PADDING, PAGE_GAP};
use crate::theme::{Theme, space};

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focused_once {
            self.focused_once = true;
            window.focus(&self.focus, cx);
        }
        let layout = self.ensure_layout(window);
        let theme = cx.global::<Theme>();
        let (page_bg, border, thumb) = (theme.page, theme.border, theme.border_strong);
        let focused = self.focus.is_focused(window);
        let images = self.picture_sources();
        let doc = &self.core.doc;
        let bands = Rc::new((doc.header.clone(), doc.footer.clone(), doc.variants.clone()));
        // Only the rows of pages near the viewport become elements; the rest is two spacers of the
        // exact height (so a document of thousands of pages costs no more than a screenful).
        // Pages of different sections may differ in size: the grid is as big as the biggest.
        let (max_w, max_h) = layout.max_page_size();
        let (page_w, page_h) = (max_w * self.zoom, max_h * self.zoom);
        let pad = f32::from(space::xl());
        let viewport_w = self.scroll.viewport_width().max(page_w + 2.0 * FIT_PADDING);
        let viewport_h = if self.scroll.viewport_height() > 0.0 {
            self.scroll.viewport_height()
        } else {
            800.0
        };
        let cols = if self.multi_page {
            (((viewport_w + PAGE_GAP) / (page_w + PAGE_GAP)).floor() as usize).max(1)
        } else {
            1
        };
        let count = layout.pages.len();
        let total_rows = count.div_ceil(cols).max(1);
        let row_h = page_h + PAGE_GAP;
        // A move of the caret scrolls it into view (worked out from the layout, so it also works
        // when the caret's page is not among the rendered ones).
        if focused
            && self.reveal_caret.replace(false)
            && let Some(c) = layout.caret(self.core.sel.head, &self.measurer(window))
        {
            let y = pad + (c.page / cols) as f32 * row_h + c.y * self.zoom;
            self.scroll.reveal_at(y, c.height * self.zoom);
        }
        let offset = self.scroll.tick(window);
        let first_row = (((offset - pad) / row_h).floor().max(0.0) as usize).saturating_sub(1);
        let last_row = ((((offset + viewport_h - pad) / row_h).ceil().max(0.0)) as usize + 1)
            .min(total_rows - 1);
        {
            // Pages that are not rendered have no bounds (hit tests skip them).
            let mut bounds = self.page_bounds.borrow_mut();
            bounds.clear();
            bounds.resize(count, Bounds::default());
        }
        let section_ends: Rc<Vec<usize>> = Rc::new(
            doc.paragraphs
                .iter()
                .enumerate()
                .filter(|(_, p)| p.style.section.is_some())
                .map(|(i, _)| i)
                .collect(),
        );
        let cell_selection = self.core.cell_selection();
        let build_page = |ix: usize| {
            let view = PageView {
                layout: layout.clone(),
                measure: self.measure_cache.clone(),
                page: ix,
                selection: self.core.sel.range(),
                head: self.core.sel.head,
                focused,
                text: theme.text,
                selection_color: theme.accent_subtle,
                cell_selection,
                epoch: self.caret_epoch.clone(),
                code_bg: theme.code_bg,
                comment_bg: theme.comment_bg,
                spoiler_color: theme.spoiler,
                link_color: theme.accent,
                bar_color: theme.border_strong,
                find: self.find.clone(),
                find_current: self.find_current,
                find_color: theme.find_match,
                find_current_color: theme.find_current,
                table_color: theme.border_strong,
                misspell_color: theme.danger,
                cropping: self.cropping,
                bands: bands.clone(),
                images: images.clone(),
                band_color: theme.text_muted,
                zoom: self.zoom,
                show_marks: self.show_marks,
                section_ends: section_ends.clone(),
            };
            let bounds_store = self.page_bounds.clone();
            div()
                .relative()
                .flex_none()
                .w(px(layout.setup_of(ix).width * self.zoom))
                .h(px(layout.setup_of(ix).height * self.zoom))
                .bg(page_bg)
                .border_1()
                .border_color(border)
                .child(
                    canvas(
                        move |bounds, _, _| {
                            if let Some(slot) = bounds_store.borrow_mut().get_mut(ix) {
                                *slot = bounds;
                            }
                        },
                        move |bounds, (), window, cx| paint_page(&view, bounds, window, cx),
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full(),
                )
        };
        let spacer = |rows: usize| {
            div()
                .flex_none()
                .w_full()
                .h(px(rows as f32 * row_h - PAGE_GAP))
                .into_any_element()
        };
        let mut rows: Vec<AnyElement> = Vec::new();
        if first_row > 0 {
            rows.push(spacer(first_row));
        }
        for r in first_row..=last_row {
            let in_row = (r * cols..((r + 1) * cols).min(count)).map(build_page);
            rows.push(
                div()
                    .flex()
                    .flex_row()
                    .flex_none()
                    .justify_center()
                    .items_start()
                    .gap(px(PAGE_GAP))
                    .children(in_row)
                    .into_any_element(),
            );
        }
        if last_row + 1 < total_rows {
            rows.push(spacer(total_rows - 1 - last_row));
        }

        let (content_scroll, viewport_scroll) = (self.scroll.clone(), self.scroll.clone());
        let content_probe = canvas(
            move |b, _, _| content_scroll.set_content_height(b.size.height),
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        let (input_focus, input_view) = (self.focus.clone(), cx.entity());
        let viewport_probe = canvas(
            move |b, _, _| {
                viewport_scroll.set_viewport(b);
                (viewport_scroll, b)
            },
            move |_, (scroll, b), window, cx| {
                scroll.paint_scrollbar(b, thumb, window);
                window.handle_input(
                    &input_focus,
                    ElementInputHandler::new(b, input_view.clone()),
                    cx,
                );
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();

        div()
            .id("doc-scroll")
            .track_focus(&self.focus)
            .key_context("Editor")
            .on_key_down(cx.listener(Self::on_key_down))
            .on_action(cx.listener(|this, _: &crate::actions::EditorTab, _, cx| {
                this.apply(cx, |c| c.tab_key(false))
            }))
            .on_action(
                cx.listener(|this, _: &crate::actions::EditorTabBack, _, cx| {
                    this.apply(cx, |c| c.tab_key(true))
                }),
            )
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_down(MouseButton::Right, cx.listener(Self::on_right_down))
            .on_mouse_move(cx.listener(Self::on_drag_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, _, cx| {
                let delta = ev.delta.pixel_delta(px(48.));
                let (dx, dy) = (f32::from(delta.x), f32::from(delta.y));
                if ev.modifiers.secondary() {
                    // Ctrl/Cmd + wheel zooms.
                    let step = if dy > 0.0 { 1.1 } else { 1.0 / 1.1 };
                    this.set_zoom(this.zoom * step, cx);
                } else if ev.modifiers.shift || dx.abs() > dy.abs() {
                    let content = this.widest_page() * this.zoom + 2.0 * FIT_PADDING;
                    let max = (content - this.scroll.viewport_width()).max(0.0);
                    let step = if dx != 0.0 { dx } else { dy };
                    this.scroll_x = (this.scroll_x - step).clamp(0.0, max);
                    cx.notify();
                } else {
                    this.scroll.scroll_by(-dy);
                    cx.notify();
                }
            }))
            .cursor_text()
            .relative()
            .size_full()
            .overflow_hidden()
            .child(
                div()
                    .relative()
                    .w_full()
                    .min_w(px(max_w * self.zoom + 2.0 * FIT_PADDING))
                    .ml(px(-self.scroll_x))
                    // Rows of pages (several per row while they fit, like in Word, or one).
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(PAGE_GAP))
                    .py(px(pad))
                    .mt(px(-offset))
                    .children(rows)
                    .child(content_probe),
            )
            .child(viewport_probe)
    }
}
