use gpui_kit::*;
use wv_core::{ParaMetrics, TabLeader, TabStop};
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius, space};
use crate::ui_kit::{
    ButtonKind, Icon, LineEdit, Outcome, Tip, button, icon_button, pop_in, track_bounds,
};

pub enum ParagraphEvent {
    Set(ParaMetrics),
}

const PT_PER_CM: f32 = 72.0 / 2.54;
const FIELD_WIDTH: f32 = 140.0;

/// Fields in tab order: space before/after (pt), line spacing (times), left indent, first line
/// and tab stops (cm).
const LABELS: [Key; 6] = [
    Key::ParaSpaceBefore,
    Key::ParaSpaceAfter,
    Key::ParaLineSpacing,
    Key::ParaLeft,
    Key::ParaFirstLine,
    Key::ParaTabs,
];

fn cm(pt: f32) -> String {
    let value = format!("{:.2}", pt / PT_PER_CM);
    value
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn number(text: &str) -> Option<f32> {
    let value: f32 = text.trim().replace(',', ".").parse().ok()?;
    (value.is_finite() && value >= 0.0).then_some(value)
}

/// Centimetres to whole points; an empty or invalid field is 0.
fn cm_to_pt(text: &str) -> f32 {
    number(text).map_or(0.0, |v| (v * PT_PER_CM).round())
}

/// "1,5; 3.; 6_" -> tab stops in points with their leaders (a trailing `.`, `-` or `_` means
/// dots, hyphens or an underline).
fn parse_tabs(text: &str) -> Vec<(u16, TabLeader)> {
    text.split(';')
        .filter_map(|item| {
            let item = item.trim();
            let leader = match item.chars().last()? {
                '.' => TabLeader::Dots,
                '-' => TabLeader::Hyphens,
                '_' => TabLeader::Underscore,
                _ => TabLeader::None,
            };
            let digits = if leader == TabLeader::None {
                item
            } else {
                &item[..item.len() - 1]
            };
            let pt = (number(digits)? * PT_PER_CM).round().min(65535.0) as u16;
            Some((pt, leader))
        })
        .collect()
}

fn leader_mark(leader: TabLeader) -> &'static str {
    match leader {
        TabLeader::None => "",
        TabLeader::Dots => ".",
        TabLeader::Hyphens => "-",
        TabLeader::Underscore => "_",
    }
}

fn gap(text: &str) -> Option<u8> {
    number(text).map(|v| v.round().min(255.0) as u8)
}

/// Values of the fields for a paragraph (the same text `apply` reads back).
fn texts(m: &ParaMetrics) -> [String; 6] {
    [
        m.space_before.map_or(String::new(), |v| v.to_string()),
        m.space_after.map_or(String::new(), |v| v.to_string()),
        m.line_spacing.map_or(String::new(), |v| v.to_string()),
        cm(f32::from(m.left_pt)),
        cm(f32::from(m.first_line)),
        m.tabs
            .iter()
            .map(|t| format!("{}{}", cm(f32::from(t.pos)), leader_mark(t.leader)))
            .collect::<Vec<_>>()
            .join("; "),
    ]
}

/// Button with a popover for the exact indents, spacing and tab stops of the paragraph.
pub struct ParagraphEditor {
    lang: Lang,
    open: bool,
    panel_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    epoch: u64,
    current: ParaMetrics,
    edits: [LineEdit; 6],
    focus: [FocusHandle; 6],
}

impl EventEmitter<ParagraphEvent> for ParagraphEditor {}

impl ParagraphEditor {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            open: false,
            panel_bounds: Default::default(),
            epoch: 0,
            current: ParaMetrics::default(),
            edits: Default::default(),
            focus: std::array::from_fn(|_| cx.focus_handle()),
        }
    }

    /// The paragraph at the caret (pre-fills the fields when the popover opens).
    pub fn set_current(&mut self, metrics: ParaMetrics) {
        if !self.open {
            self.current = metrics;
        }
    }

    fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = !self.open;
        if self.open {
            self.epoch += 1;
            for (edit, text) in self.edits.iter_mut().zip(texts(&self.current)) {
                edit.set(&text, false);
            }
            window.focus(&self.focus[0], cx);
        }
        cx.notify();
    }

    fn apply(&mut self, cx: &mut Context<Self>) {
        let line = number(self.edits[2].text()).map(|v| v.clamp(0.5, 5.0));
        let metrics = ParaMetrics {
            space_before: gap(self.edits[0].text()),
            space_after: gap(self.edits[1].text()),
            line_spacing: line,
            left_pt: cm_to_pt(self.edits[3].text()).min(65535.0) as u16,
            first_line: cm_to_pt(self.edits[4].text()).min(255.0) as u8,
            // A stop that stays where it was keeps its kind (the dialog lists places and leaders).
            tabs: parse_tabs(self.edits[5].text())
                .into_iter()
                .map(|(pos, leader)| TabStop {
                    leader,
                    ..self
                        .current
                        .tabs
                        .iter()
                        .find(|t| t.pos == pos)
                        .copied()
                        .unwrap_or_else(|| TabStop::left(pos))
                })
                .collect(),
            right_pt: self.current.right_pt,
        };
        self.open = false;
        self.current = metrics.clone();
        cx.emit(ParagraphEvent::Set(metrics));
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
                    .id(("para-field", index))
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
            .id("paragraph-panel")
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
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(tr(lang, Key::ParaHint)),
            )
            .child(
                button("para-apply", tr(lang, Key::Apply), ButtonKind::Primary, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.apply(cx))),
            )
    }
}

impl Render for ParagraphEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.lang;
        let button = icon_button("paragraph-btn", Icon::Paragraph, self.open, cx)
            .tip(tr(lang, Key::TipParagraph))
            .on_click(cx.listener(|this, _, window, cx| this.toggle(window, cx)));
        let pop_key = format!("paragraph-pop-{}", self.epoch);
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

#[cfg(test)]
mod tests {
    use wv_core::TabLeader;

    use super::{leader_mark, parse_tabs};

    #[test]
    fn tab_field_has_places_and_leaders() {
        let tabs = parse_tabs("1,5; 3.; 6_; 8-; x");
        let leaders: Vec<_> = tabs.iter().map(|t| t.1).collect();
        assert_eq!(
            leaders,
            [
                TabLeader::None,
                TabLeader::Dots,
                TabLeader::Underscore,
                TabLeader::Hyphens
            ]
        );
        assert_eq!(tabs[1].0, 85);
        assert_eq!(leader_mark(TabLeader::Dots), ".");
    }
}
