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

mod panel;
