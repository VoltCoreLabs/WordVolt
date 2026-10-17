use gpui_kit::*;
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius, space};
use crate::ui_kit::{
    ButtonKind, Icon, LineEdit, Outcome, Tip, button, icon_button, pop_in, track_bounds,
};

pub enum TablePickerEvent {
    /// Rows, columns.
    Insert(u16, u16),
}

/// Grid size offered for picking with the mouse.
const GRID_ROWS: u16 = 8;
const GRID_COLS: u16 = 10;
const CELL: f32 = 20.0;
const MAX_ROWS: u16 = 50;
const MAX_COLS: u16 = 20;

/// Insert-table button with a popover: hover a grid and click, or type the number of rows and columns.
pub struct TablePicker {
    lang: Lang,
    open: bool,
    /// Bounds of the open panel (see `track_bounds`).
    panel_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    /// Counts openings so the pop-in animation restarts each time.
    epoch: u64,
    /// Grid cell under the pointer (1-based rows, columns); (0, 0) = none.
    hover: (u16, u16),
    rows: LineEdit,
    cols: LineEdit,
    rows_focus: FocusHandle,
    cols_focus: FocusHandle,
}

impl EventEmitter<TablePickerEvent> for TablePicker {}

impl TablePicker {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            open: false,
            panel_bounds: Default::default(),
            epoch: 0,
            hover: (0, 0),
            rows: LineEdit::default(),
            cols: LineEdit::default(),
            rows_focus: cx.focus_handle(),
            cols_focus: cx.focus_handle(),
        }
    }

    fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = !self.open;
        if self.open {
            self.epoch += 1;
            self.hover = (0, 0);
            self.rows.set("3", true);
            self.cols.set("3", true);
            window.focus(&self.rows_focus, cx);
        }
        cx.notify();
    }

    fn insert(&mut self, rows: u16, cols: u16, cx: &mut Context<Self>) {
        self.open = false;
        cx.emit(TablePickerEvent::Insert(
            rows.clamp(1, MAX_ROWS),
            cols.clamp(1, MAX_COLS),
        ));
        cx.notify();
    }

    /// Inserts the table whose size was typed into the two fields.
    fn insert_typed(&mut self, cx: &mut Context<Self>) {
        if let (Ok(rows), Ok(cols)) = (
            self.rows.text().parse::<u16>(),
            self.cols.text().parse::<u16>(),
        ) {
            self.insert(rows, cols, cx);
        }
    }

    fn switch_field(&mut self, in_rows: bool, window: &mut Window, cx: &mut Context<Self>) {
        let target = if in_rows {
            &self.cols_focus
        } else {
            &self.rows_focus
        };
        window.focus(target, cx);
        cx.notify();
    }

    fn on_key(
        &mut self,
        in_rows: bool,
        ev: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = ev.keystroke.key.as_str();
        if key == "enter" {
            self.insert_typed(cx);
        } else {
            let field = if in_rows {
                &mut self.rows
            } else {
                &mut self.cols
            };
            match field.key(ev, cx) {
                Outcome::Cancel => self.open = false,
                Outcome::Changed => {
                    // Numbers only, at most two digits.
                    let digits: String = field
                        .text()
                        .chars()
                        .filter(char::is_ascii_digit)
                        .take(2)
                        .collect();
                    field.set(&digits, false);
                }
                _ => {}
            }
            cx.notify();
        }
        cx.stop_propagation();
    }

    fn panel(&mut self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
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

impl Render for TablePicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.lang;
        let button = icon_button("table-btn", Icon::Table, self.open, cx)
            .tip(tr(lang, Key::TipTable))
            .on_click(cx.listener(|this, _, window, cx| this.toggle(window, cx)));
        let pop_key = format!("table-pop-{}", self.epoch);
        let menu = self.open.then(|| {
            deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .offset(point(px(0.), px(36.)))
                    .child(pop_in(self.panel(window, cx), pop_key)),
            )
            .with_priority(10)
        });
        div()
            .relative()
            .on_mouse_down_out(cx.listener(|this, ev: &MouseDownEvent, _, cx| {
                if this.open && !this.panel_bounds.get().contains(&ev.position) {
                    this.open = false;
                    cx.notify();
                }
            }))
            .child(button)
            .children(menu)
    }
}
