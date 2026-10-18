use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_core::{Float, Wrap};
use wv_i18n::{Key, Lang, t as tr};

use crate::editor::ImageSettings;
use crate::theme::{Theme, radius, space};
use crate::ui_kit::{ButtonKind, LineEdit, Outcome, button, pop_in};

const PX_PER_CM: f32 = 96.0 / 2.54;
const FIELD_WIDTH: f32 = 110.0;

pub enum ImageDialogEvent {
    Apply(ImageSettings),
}

/// How the picture sits in the text, in the order of the buttons.
const MODES: [(Key, Option<Wrap>); 4] = [
    (Key::WrapInline, None),
    (Key::WrapTopBottom, Some(Wrap::TopBottom)),
    (Key::WrapFront, Some(Wrap::Front)),
    (Key::WrapBehind, Some(Wrap::Behind)),
];

fn cm(px: f32) -> String {
    let text = format!("{:.2}", px / PX_PER_CM);
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn parse(text: &str) -> Option<f32> {
    let v: f32 = text.trim().replace(',', ".").parse().ok()?;
    v.is_finite().then_some(v * PX_PER_CM)
}

/// "Size and position" of a picture: a popup opened at a point (from the right-click menu).
pub struct ImageDialog {
    lang: Lang,
    open_at: Option<Point<Pixels>>,
    epoch: u64,
    start: ImageSettings,
    mode: Option<Wrap>,
    keep_proportions: bool,
    /// Width, height, left, top (cm).
    edits: [LineEdit; 4],
    focus: [FocusHandle; 4],
}

impl EventEmitter<ImageDialogEvent> for ImageDialog {}

impl ImageDialog {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            open_at: None,
            epoch: 0,
            start: ImageSettings {
                width: 1.0,
                height: 1.0,
                float: None,
            },
            mode: None,
            keep_proportions: true,
            edits: Default::default(),
            focus: std::array::from_fn(|_| cx.focus_handle()),
        }
    }

    pub fn open(
        &mut self,
        at: Point<Pixels>,
        settings: ImageSettings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_at = Some(at);
        self.epoch += 1;
        self.start = settings;
        self.mode = settings.float.map(|f| f.wrap);
        let float = settings.float.unwrap_or(Float {
            x: 0.0,
            y: 0.0,
            wrap: Wrap::TopBottom,
        });
        let values = [settings.width, settings.height, float.x, float.y];
        for (edit, v) in self.edits.iter_mut().zip(values) {
            edit.set(&cm(v), false);
        }
        window.focus(&self.focus[0], cx);
        cx.notify();
    }

    pub fn is_open(&self) -> bool {
        self.open_at.is_some()
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        self.open_at = None;
        cx.notify();
    }

    fn apply(&mut self, cx: &mut Context<Self>) {
        let s = self.start;
        let value = |i: usize, old: f32| parse(self.edits[i].text()).unwrap_or(old);
        let (mut width, mut height) = (value(0, s.width).max(16.0), value(1, s.height).max(16.0));
        if self.keep_proportions {
            let aspect = s.width / s.height.max(1.0);
            let width_changed = (width - s.width).abs() > 0.5;
            let height_changed = (height - s.height).abs() > 0.5;
            if width_changed || !height_changed {
                height = width / aspect;
            } else {
                width = height * aspect;
            }
        }
        let old = s.float.unwrap_or(Float {
            x: 0.0,
            y: 0.0,
            wrap: Wrap::TopBottom,
        });
        let float = self.mode.map(|wrap| Float {
            x: value(2, old.x),
            y: value(3, old.y),
            wrap,
        });
        self.open_at = None;
        cx.emit(ImageDialogEvent::Apply(ImageSettings {
            width,
            height,
            float,
        }));
        cx.notify();
    }

    fn on_key(&mut self, index: usize, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        match self.edits[index].key(ev, cx) {
            Outcome::Submit => self.apply(cx),
            Outcome::Cancel => self.close(cx),
            _ => cx.notify(),
        }
        cx.stop_propagation();
    }
}

impl Render for ImageDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(at) = self.open_at else {
            return div().into_any_element();
        };
        let panel = self.panel(window, cx);
        let key = format!("image-dialog-pop-{}", self.epoch);
        deferred(
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .occlude()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| this.close(cx)),
                )
                .child(
                    anchored()
                        .position(at)
                        .anchor(Anchor::TopLeft)
                        .snap_to_window_with_margin(px(8.))
                        .child(pop_in(panel, key)),
                ),
        )
        .with_priority(30)
        .into_any_element()
    }
}

mod view;
