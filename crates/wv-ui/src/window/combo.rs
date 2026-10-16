use gpui_kit::*;

use crate::theme::{Theme, radius, space};
use crate::ui_kit::{Icon, LineEdit, Outcome, Weight, icon};

pub enum ComboEvent {
    /// The typed or chosen text; `None` = the "default" entry.
    Picked(Option<String>),
}

/// How a combo gets its entries and looks.
pub struct ComboConfig {
    pub id: &'static str,
    pub width: f32,
    /// Label of the first entry that resets to the default (fonts only).
    pub default_label: Option<String>,
    /// Show every entry in its own font (font list).
    pub preview_font: bool,
    pub items: fn(&App) -> Vec<String>,
}

/// Editable drop-down: type a value by hand (Enter applies) or pick from the list, which filters
/// while typing. Used for the font family and the font size.
pub struct Combo {
    config: ComboConfig,
    edit: LineEdit,
    focus: FocusHandle,
    list_scroll: ScrollHandle,
    open: bool,
    shown: String,
    items: Vec<String>,
    /// The user typed something, so the list is filtered by it.
    typed: bool,
}

impl EventEmitter<ComboEvent> for Combo {}

impl Combo {
    pub fn new(config: ComboConfig, shown: String, cx: &mut Context<Self>) -> Self {
        Self {
            config,
            edit: LineEdit::default(),
            focus: cx.focus_handle(),
            list_scroll: ScrollHandle::new(),
            open: false,
            shown,
            items: Vec::new(),
            typed: false,
        }
    }

    /// Value of the caret/selection, shown while the menu is closed.
    pub fn set_current(&mut self, text: &str) {
        if !self.open && self.shown != text {
            self.shown = text.to_string();
        }
    }

    fn open_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            self.items = (self.config.items)(cx);
        }
        self.open = true;
        self.typed = false;
        self.list_scroll.set_offset(point(px(0.), px(0.)));
        self.edit.set(&self.shown, true);
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        cx.notify();
    }

    fn pick(&mut self, value: Option<String>, cx: &mut Context<Self>) {
        self.open = false;
        if let Some(value) = &value {
            self.shown = value.clone();
        }
        cx.emit(ComboEvent::Picked(value));
        cx.notify();
    }

    fn on_key(&mut self, ev: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match self.edit.key(ev, cx) {
            Outcome::Submit => {
                let typed = self.edit.text().trim().to_string();
                self.pick((!typed.is_empty()).then_some(typed), cx);
            }
            Outcome::Cancel => self.close(cx),
            Outcome::Changed => {
                self.typed = true;
                self.list_scroll.set_offset(point(px(0.), px(0.)));
                cx.notify();
            }
            Outcome::Nothing => cx.notify(),
        }
        cx.stop_propagation();
    }

    fn list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        let (surface, border, hover) = (t.surface, t.border, t.surface_alt);
        let filter = if self.typed {
            self.edit.text().trim().to_lowercase()
        } else {
            String::new()
        };
        let names: Vec<String> = self
            .items
            .iter()
            .filter(|n| filter.is_empty() || n.to_lowercase().contains(&filter))
            .take(400)
            .cloned()
            .collect();
        let preview = self.config.preview_font;
        let id = self.config.id;
        let item = |index: usize, label: String, value: Option<String>, cx: &mut Context<Self>| {
            let picked = value.clone();
            let mut row = div()
                .id((SharedString::from(format!("{id}-item")), index))
                .h(px(30.))
                .px(px(10.))
                .flex()
                .items_center()
                .flex_none()
                .rounded(radius::sm())
                .text_sm()
                .cursor_pointer()
                .hover(move |s| s.bg(hover))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| this.pick(picked.clone(), cx)),
                )
                .child(label);
            if preview && let Some(family) = value {
                row = row.font_family(family);
            }
            row
        };
        let mut rows = Vec::new();
        if let Some(label) = &self.config.default_label {
            rows.push(item(0, label.clone(), None, cx));
        }
        rows.extend(
            names
                .into_iter()
                .enumerate()
                .map(|(i, name)| item(i + 1, name.clone(), Some(name), cx)),
        );
        div()
            .id(SharedString::from(format!("{id}-list")))
            .track_scroll(&self.list_scroll)
            .occlude()
            .w(px(self.config.width.max(160.)))
            .max_h(px(320.))
            .overflow_y_scroll()
            .p(px(4.))
            .flex()
            .flex_col()
            .bg(surface)
            .border_1()
            .border_color(border)
            .rounded(radius::md())
            .shadow_lg()
            .children(rows)
    }
}

impl Render for Combo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        let (surface, border, accent, muted) = (t.surface, t.border, t.accent, t.text_muted);
        let content = if self.open {
            self.edit
                .view(self.config.width - 50.0, window, cx)
                .into_any_element()
        } else {
            div()
                .truncate()
                .child(self.shown.clone())
                .into_any_element()
        };
        let field = div()
            .id(SharedString::from(format!("{}-field", self.config.id)))
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    if !this.open {
                        this.open_menu(window, cx);
                    }
                }),
            )
            .w(px(self.config.width))
            .h(px(32.))
            .px(space::md())
            .flex()
            .items_center()
            .justify_between()
            .gap(space::sm())
            .rounded(radius::md())
            .border_1()
            .border_color(if self.open { accent } else { border })
            .bg(surface)
            .text_sm()
            .cursor_text()
            .child(div().flex_1().overflow_hidden().child(content))
            .child(icon(Icon::CaretDown, Weight::Regular, px(14.), muted));
        let menu = self.open.then(|| {
            deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .offset(point(px(0.), px(36.)))
                    .child(self.list(cx)),
            )
            .with_priority(10)
        });
        div()
            .relative()
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                if this.open {
                    this.close(cx);
                }
            }))
            .child(field)
            .children(menu)
    }
}
