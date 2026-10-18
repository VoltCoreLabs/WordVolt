use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::*;
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius, space};
use crate::ui_kit::color_math::{hex, hsv_to_rgb, mix, parse_hex, rgb_to_hsv};
use crate::ui_kit::{
    Icon, LineEdit, Outcome, RecentColors, Tip, icon_button, pop_in, track_bounds,
};

pub enum ColorEvent {
    /// `None` = no colour (automatic text colour / no highlight).
    Picked(Option<u32>),
}

#[derive(Clone, Copy, PartialEq)]
enum Drag {
    Square,
    Hue,
}

const SQUARE: (f32, f32) = (224.0, 120.0);
const HUE_HEIGHT: f32 = 14.0;

/// Colour button with a drop-down: standard palette, recently used colours, a free picker
/// (saturation/brightness square + hue bar) and a hex field.
pub struct ColorPicker {
    id: &'static str,
    glyph: Icon,
    tip: Key,
    lang: Lang,
    /// Colour shown in the bar under the icon.
    current: Option<u32>,
    open: bool,
    /// Bounds of the open panel (see `track_bounds`).
    panel_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    /// Counts openings so the pop-in animation restarts each time.
    epoch: u64,
    hsv: (f32, f32, f32),
    hex: LineEdit,
    focus: FocusHandle,
    drag: Option<Drag>,
    square_bounds: Rc<Cell<Bounds<Pixels>>>,
    hue_bounds: Rc<Cell<Bounds<Pixels>>>,
}

impl EventEmitter<ColorEvent> for ColorPicker {}

/// Standard palette: greys, vivid colours, then lighter and darker variants of the vivid row.
fn palette() -> Vec<Vec<u32>> {
    let greys = vec![
        0x000000, 0x434343, 0x666666, 0x999999, 0xB7B7B7, 0xCCCCCC, 0xD9D9D9, 0xEFEFEF, 0xF3F3F3,
        0xFFFFFF,
    ];
    let vivid = vec![
        0x980000, 0xFF0000, 0xFF9900, 0xFFFF00, 0x00FF00, 0x00FFFF, 0x4A86E8, 0x0000FF, 0x9900FF,
        0xFF00FF,
    ];
    let shade = |toward: u32, amount: f32| {
        vivid
            .iter()
            .map(|c| mix(*c, toward, amount))
            .collect::<Vec<_>>()
    };
    vec![
        greys,
        vivid.clone(),
        shade(0xFFFFFF, 0.8),
        shade(0xFFFFFF, 0.6),
        shade(0xFFFFFF, 0.4),
        shade(0x000000, 0.25),
        shade(0x000000, 0.5),
    ]
}

impl ColorPicker {
    pub fn new(
        id: &'static str,
        glyph: Icon,
        tip: Key,
        lang: Lang,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            id,
            glyph,
            tip,
            lang,
            current: None,
            open: false,
            panel_bounds: Default::default(),
            epoch: 0,
            hsv: (210.0, 0.8, 0.9),
            hex: LineEdit::default(),
            focus: cx.focus_handle(),
            drag: None,
            square_bounds: Rc::default(),
            hue_bounds: Rc::default(),
        }
    }

    /// Colour of the caret/selection (for the bar under the icon).
    pub fn set_current(&mut self, color: Option<u32>) {
        self.current = color;
    }

    fn toggle(&mut self, cx: &mut Context<Self>) {
        self.open = !self.open;
        if self.open {
            self.epoch += 1;
            let start = self.current.unwrap_or(0x2563EB);
            self.hsv = rgb_to_hsv(start);
            self.hex.set(&hex(start), false);
        }
        cx.notify();
    }

    fn pick(&mut self, color: Option<u32>, cx: &mut Context<Self>) {
        self.open = false;
        self.drag = None;
        if let Some(color) = color {
            cx.global::<RecentColors>().push(color);
            self.current = Some(color);
        } else {
            self.current = None;
        }
        cx.emit(ColorEvent::Picked(color));
        cx.notify();
    }

    fn rgb(&self) -> u32 {
        hsv_to_rgb(self.hsv.0, self.hsv.1, self.hsv.2)
    }

    /// Updates the hue or the saturation/brightness from a window position while dragging.
    fn drag_to(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let unit = |b: Bounds<Pixels>, p: Point<Pixels>| {
            (
                f32::from(p.x - b.origin.x) / f32::from(b.size.width).max(1.0),
                f32::from(p.y - b.origin.y) / f32::from(b.size.height).max(1.0),
            )
        };
        match self.drag {
            Some(Drag::Square) => {
                let (x, y) = unit(self.square_bounds.get(), position);
                self.hsv.1 = x.clamp(0.0, 1.0);
                self.hsv.2 = 1.0 - y.clamp(0.0, 1.0);
            }
            Some(Drag::Hue) => {
                let (x, _) = unit(self.hue_bounds.get(), position);
                self.hsv.0 = x.clamp(0.0, 1.0) * 359.9;
            }
            None => return,
        }
        self.hex.set(&hex(self.rgb()), false);
        cx.notify();
    }

    fn end_drag(&mut self, cx: &mut Context<Self>) {
        if self.drag.take().is_some() {
            let color = self.rgb();
            self.pick(Some(color), cx);
        }
    }

    fn on_hex_key(&mut self, ev: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match self.hex.key(ev, cx) {
            Outcome::Submit => {
                if let Some(color) = parse_hex(self.hex.text()) {
                    self.pick(Some(color), cx);
                }
            }
            Outcome::Cancel => {
                self.open = false;
                cx.notify();
            }
            Outcome::Changed => {
                if let Some(color) = parse_hex(self.hex.text()) {
                    self.hsv = rgb_to_hsv(color);
                }
                cx.notify();
            }
            Outcome::Nothing => cx.notify(),
        }
        cx.stop_propagation();
    }

    fn swatch(&self, id: (SharedString, usize), color: u32, cx: &mut Context<Self>) -> AnyElement {
        let (border, accent) = (cx.global::<Theme>().border, cx.global::<Theme>().accent);
        div()
            .id(id)
            .size(px(18.))
            .flex_none()
            .rounded(px(4.))
            .bg(rgb(color))
            .border_1()
            .border_color(border)
            .cursor_pointer()
            .hover(move |s| s.border_color(accent))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| this.pick(Some(color), cx)),
            )
            .into_any_element()
    }
}

impl Render for ColorPicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        let (bar, border) = (self.current.map_or(t.border_strong, rgb), t.border);
        let button = icon_button(
            SharedString::from(format!("{}-btn", self.id)),
            self.glyph,
            self.open,
            cx,
        )
        .tip(tr(self.lang, self.tip))
        .on_click(cx.listener(|this, _, _, cx| this.toggle(cx)))
        .child(
            div()
                .absolute()
                .bottom(px(3.))
                .left(px(8.))
                .right(px(8.))
                .h(px(3.))
                .rounded(px(1.5))
                .bg(bar)
                .border_1()
                .border_color(border),
        );
        let pop_key = format!("{}-pop-{}", self.id, self.epoch);
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
