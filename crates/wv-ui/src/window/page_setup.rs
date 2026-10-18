use gpui_kit::*;
use wv_core::PageSpec;
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius, space};
use crate::ui_kit::{
    ButtonKind, Icon, LineEdit, Outcome, Tip, button, icon_button, pop_in, track_bounds,
};

pub enum PageSetupEvent {
    Set(PageSpec),
}

const PX_PER_CM: f32 = 96.0 / 2.54;
const FIELD_WIDTH: f32 = 110.0;

/// Fields in tab order: width, height, top, bottom, left, right, gutter, header and footer
/// distance (cm).
const LABELS: [Key; 9] = [
    Key::PageWidth,
    Key::PageHeight,
    Key::MarginTop,
    Key::MarginBottom,
    Key::MarginLeft,
    Key::MarginRight,
    Key::Gutter,
    Key::HeaderDistance,
    Key::FooterDistance,
];

fn to_cm(px: f32) -> String {
    format!("{:.2}", px / PX_PER_CM)
}

/// "2,5" or "2.5" -> pixels; `None` when it is not a positive number.
fn parse_cm(text: &str) -> Option<f32> {
    let value: f32 = text.trim().replace(',', ".").parse().ok()?;
    (value.is_finite() && value >= 0.0).then_some(value * PX_PER_CM)
}

/// Button with a popover for the exact paper size and margins.
pub struct PageSetupEditor {
    lang: Lang,
    open: bool,
    panel_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    epoch: u64,
    current: PageSpec,
    edits: [LineEdit; 9],
    focus: [FocusHandle; 9],
}

impl EventEmitter<PageSetupEvent> for PageSetupEditor {}

impl PageSetupEditor {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            open: false,
            panel_bounds: Default::default(),
            epoch: 0,
            current: PageSpec::default(),
            edits: Default::default(),
            focus: std::array::from_fn(|_| cx.focus_handle()),
        }
    }

    /// The document's page (pre-fills the fields when the popover opens).
    pub fn set_current(&mut self, page: PageSpec) {
        if !self.open {
            self.current = page;
        }
    }

    fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = !self.open;
        if self.open {
            self.epoch += 1;
            let p = self.current;
            let values = [
                p.width,
                p.height,
                p.margin_top,
                p.margin_bottom,
                p.margin_left,
                p.margin_right,
                p.gutter,
                p.header_distance,
                p.footer_distance,
            ];
            for (edit, v) in self.edits.iter_mut().zip(values) {
                edit.set(&to_cm(v), false);
            }
            window.focus(&self.focus[0], cx);
        }
        cx.notify();
    }

    fn apply(&mut self, cx: &mut Context<Self>) {
        let values: Vec<Option<f32>> = self.edits.iter().map(|e| parse_cm(e.text())).collect();
        // A field that is not a number keeps the value the page has now.
        let c = self.current;
        let old = [
            c.width,
            c.height,
            c.margin_top,
            c.margin_bottom,
            c.margin_left,
            c.margin_right,
            c.gutter,
            c.header_distance,
            c.footer_distance,
        ];
        let v: Vec<f32> = values
            .iter()
            .zip(old)
            .map(|(n, o)| n.unwrap_or(o))
            .collect();
        let page = PageSpec::custom(v[0], v[1], [v[2], v[5], v[3], v[4]], [v[6], v[7], v[8]])
            .with_columns(c.columns);
        self.open = false;
        self.current = page;
        cx.emit(PageSetupEvent::Set(page));
        cx.notify();
    }

    fn switch_field(&mut self, from: usize, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus[(from + 1) % self.focus.len()], cx);
        cx.notify();
    }

    fn on_key(&mut self, index: usize, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        match self.edits[index].key(ev, cx) {
            Outcome::Submit => self.apply(cx),
            Outcome::Cancel => {
                self.open = false;
                cx.notify();
            }
            _ => cx.notify(),
        }
        cx.stop_propagation();
    }

    fn field(&self, index: usize, window: &Window, cx: &mut Context<Self>) -> Div {
        let th = cx.global::<Theme>();
        let (border, accent, muted) = (th.border, th.accent, th.text_muted);
        let focused = self.focus[index].is_focused(window);
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(tr(self.lang, LABELS[index])),
            )
            .child(
                div()
                    .id(("page-field", index))
                    .track_focus(&self.focus[index])
                    .key_context("TabField")
                    .on_action(cx.listener(
                        move |this, _: &crate::actions::FieldNext, window, cx| {
                            this.switch_field(index, window, cx)
                        },
                    ))
                    .on_key_down(
                        cx.listener(move |this, ev: &KeyDownEvent, _, cx| {
                            this.on_key(index, ev, cx)
                        }),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            window.focus(&this.focus[index], cx)
                        }),
                    )
                    .w(px(FIELD_WIDTH))
                    .h(px(32.))
                    .px(space::md())
                    .flex()
                    .items_center()
                    .rounded(radius::md())
                    .border_1()
                    .border_color(if focused { accent } else { border })
                    .text_sm()
                    .cursor_text()
                    .child(self.edits[index].view(FIELD_WIDTH - 24.0, window, cx)),
            )
    }

    fn panel(&mut self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = cx.global::<Theme>();
        let (surface, border, muted) = (th.surface, th.border, th.text_muted);
        let lang = self.lang;
        let row = |a: usize, b: usize, this: &Self, cx: &mut Context<Self>| {
            div()
                .flex()
                .gap(space::md())
                .child(this.field(a, window, cx))
                .child(this.field(b, window, cx))
        };
        div()
            .id("page-setup-panel")
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
            .child(row(0, 1, self, cx))
            .child(row(2, 3, self, cx))
            .child(row(4, 5, self, cx))
            .child(row(6, 7, self, cx))
            .child(div().flex().child(self.field(8, window, cx)))
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(tr(lang, Key::PageSetupHint)),
            )
            .child(
                button("page-apply", tr(lang, Key::Apply), ButtonKind::Primary, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.apply(cx))),
            )
    }
}

impl Render for PageSetupEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.lang;
        let button = icon_button("page-setup-btn", Icon::PageSetup, self.open, cx)
            .tip(tr(lang, Key::TipPageSetup))
            .on_click(cx.listener(|this, _, window, cx| this.toggle(window, cx)));
        let pop_key = format!("page-setup-pop-{}", self.epoch);
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
