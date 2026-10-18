use super::*;

impl TablePicker {
    pub(super) fn panel(&mut self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = cx.global::<Theme>();
        let (surface, border, accent, accent_subtle, muted) = (
            th.surface,
            th.border,
            th.accent,
            th.accent_subtle,
            th.text_muted,
        );
        let lang = self.lang;
        let hover = self.hover;
        let rows_grid: Vec<_> = (1..=GRID_ROWS)
            .map(|r| {
                let cells: Vec<_> = (1..=GRID_COLS)
                    .map(|c| {
                        let lit = r <= hover.0 && c <= hover.1;
                        div()
                            .id(("table-cell", usize::from(r) * 100 + usize::from(c)))
                            .size(px(CELL))
                            .rounded(px(4.))
                            .border_1()
                            .border_color(if lit { accent } else { border })
                            .bg(if lit { accent_subtle } else { surface })
                            .cursor_pointer()
                            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                                if *hovered && this.hover != (r, c) {
                                    this.hover = (r, c);
                                    cx.notify();
                                }
                            }))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| this.insert(r, c, cx)),
                            )
                    })
                    .collect();
                div().flex().gap(px(3.)).children(cells)
            })
            .collect();
        let title = if hover == (0, 0) {
            tr(lang, Key::TipTable).to_string()
        } else {
            format!("{} \u{00D7} {}", hover.0, hover.1)
        };
        let focused_rows = self.rows_focus.is_focused(window);
        let focused_cols = self.cols_focus.is_focused(window);
        let field = |id: &'static str,
                     edit: &LineEdit,
                     focus: &FocusHandle,
                     focused: bool,
                     in_rows: bool,
                     cx: &mut Context<Self>| {
            div()
                .id(id)
                .track_focus(focus)
                .key_context("TabField")
                .on_action(
                    cx.listener(move |this, _: &crate::actions::FieldNext, window, cx| {
                        this.switch_field(in_rows, window, cx)
                    }),
                )
                .on_key_down(cx.listener(move |this, ev: &KeyDownEvent, window, cx| {
                    this.on_key(in_rows, ev, window, cx)
                }))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| {
                        window.focus(
                            if in_rows {
                                &this.rows_focus
                            } else {
                                &this.cols_focus
                            },
                            cx,
                        )
                    }),
                )
                .w(px(54.))
                .h(px(32.))
                .px(space::md())
                .flex()
                .items_center()
                .rounded(radius::md())
                .border_1()
                .border_color(if focused { accent } else { border })
                .text_sm()
                .cursor_text()
                .child(edit.view(30.0, window, cx))
        };
        let rows_field = field(
            "table-rows",
            &self.rows,
            &self.rows_focus,
            focused_rows,
            true,
            cx,
        );
        let cols_field = field(
            "table-cols",
            &self.cols,
            &self.cols_focus,
            focused_cols,
            false,
            cx,
        );
        let custom = div()
            .flex()
            .items_center()
            .gap(space::sm())
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(tr(lang, Key::TableRows)),
            )
            .child(rows_field)
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(tr(lang, Key::TableCols)),
            )
            .child(cols_field)
            .child(
                button(
                    "table-insert",
                    tr(lang, Key::InsertTable),
                    ButtonKind::Primary,
                    cx,
                )
                .on_click(cx.listener(|this, _, _, cx| this.insert_typed(cx))),
            );
        div()
            .id("table-panel")
            .occlude()
            .relative()
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .bg(surface)
            .border_1()
            .border_color(border)
            .rounded(radius::md())
            .shadow_lg()
            .child(track_bounds(self.panel_bounds.clone()))
            .child(div().text_sm().child(title))
            .child(div().flex().flex_col().gap(px(3.)).children(rows_grid))
            .child(custom)
    }
}
