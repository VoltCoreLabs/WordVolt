use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::*;
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius, space};
use crate::ui_kit::color_math::{hex, hsv_to_rgb, mix, parse_hex, rgb_to_hsv};
use crate::ui_kit::{Icon, LineEdit, Outcome, RecentColors, Tip, icon_button};

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

    fn panel(&mut self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        let (surface, border, muted, text, hover) =
            (t.surface, t.border, t.text_muted, t.text, t.surface_alt);
        let lang = self.lang;
        let id = self.id;
        let section = |label: &'static str| {
            div()
                .text_xs()
                .text_color(muted)
                .pt(space::sm())
                .child(label)
        };

        let none = div()
            .id(SharedString::from(format!("{id}-none")))
            .h(px(28.))
            .px(space::md())
            .flex()
            .items_center()
            .rounded(radius::sm())
            .text_sm()
            .cursor_pointer()
            .hover(move |s| s.bg(hover))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.pick(None, cx)),
            )
            .child(tr(lang, Key::NoColor));

        let mut counter = 0;
        let grid: Vec<_> = palette()
            .into_iter()
            .map(|row| {
                let swatches: Vec<_> = row
                    .into_iter()
                    .map(|c| {
                        counter += 1;
                        self.swatch((SharedString::from(format!("{id}-sw")), counter), c, cx)
                    })
                    .collect();
                div().flex().gap(px(4.)).children(swatches)
            })
            .collect();

        let recent = cx.global::<RecentColors>().list();
        let recent_row = (!recent.is_empty()).then(|| {
            let swatches: Vec<_> = recent
                .into_iter()
                .enumerate()
                .map(|(i, c)| self.swatch((SharedString::from(format!("{id}-recent")), i), c, cx))
                .collect();
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(section(tr(lang, Key::RecentColors)))
                .child(div().flex().gap(px(4.)).children(swatches))
        });

        let (hue, sat, val) = self.hsv;
        let (square_cell, hue_cell) = (self.square_bounds.clone(), self.hue_bounds.clone());
        let square = div()
            .id(SharedString::from(format!("{id}-square")))
            .w(px(SQUARE.0))
            .h(px(SQUARE.1))
            .rounded(px(6.))
            .overflow_hidden()
            .cursor_crosshair()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, ev: &MouseDownEvent, _, cx| {
                    this.drag = Some(Drag::Square);
                    this.drag_to(ev.position, cx);
                }),
            )
            .child(
                canvas(
                    move |b, _, _| square_cell.set(b),
                    move |b, (), window, _| {
                        window.paint_quad(fill(b, rgb(hsv_to_rgb(hue, 1.0, 1.0))));
                        let white = Hsla::from(rgb(0xFFFFFF));
                        window.paint_quad(fill(
                            b,
                            linear_gradient(
                                90.,
                                linear_color_stop(white, 0.),
                                linear_color_stop(Hsla { a: 0., ..white }, 1.),
                            ),
                        ));
                        let black = Hsla::from(rgb(0x000000));
                        window.paint_quad(fill(
                            b,
                            linear_gradient(
                                180.,
                                linear_color_stop(Hsla { a: 0., ..black }, 0.),
                                linear_color_stop(black, 1.),
                            ),
                        ));
                        let at = point(
                            b.origin.x + b.size.width * sat,
                            b.origin.y + b.size.height * (1.0 - val),
                        );
                        let mut ring = fill(
                            Bounds::new(
                                point(at.x - px(6.), at.y - px(6.)),
                                size(px(12.), px(12.)),
                            ),
                            rgba(0),
                        );
                        ring.corner_radii = Corners::all(px(6.));
                        ring.border_widths = Edges::all(px(2.));
                        ring.border_color = rgb(0xFFFFFF).into();
                        window.paint_quad(ring);
                    },
                )
                .size_full(),
            );
        let hue_bar = div()
            .id(SharedString::from(format!("{id}-hue")))
            .w(px(SQUARE.0))
            .h(px(HUE_HEIGHT))
            .rounded(px(7.))
            .overflow_hidden()
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, ev: &MouseDownEvent, _, cx| {
                    this.drag = Some(Drag::Hue);
                    this.drag_to(ev.position, cx);
                }),
            )
            .child(
                canvas(
                    move |b, _, _| hue_cell.set(b),
                    move |b, (), window, _| {
                        let step = b.size.width / 6.0;
                        for i in 0..6 {
                            // A little overlap hides the hairline gap between the segments.
                            let part = Bounds::new(
                                point(b.origin.x + step * i as f32, b.origin.y),
                                size(step + px(1.), b.size.height),
                            );
                            let (from, to) = (
                                hsv_to_rgb(60.0 * i as f32, 1.0, 1.0),
                                hsv_to_rgb(60.0 * (i + 1) as f32 % 360.0, 1.0, 1.0),
                            );
                            window.paint_quad(fill(
                                part,
                                linear_gradient(
                                    90.,
                                    linear_color_stop(rgb(from), 0.),
                                    linear_color_stop(rgb(to), 1.),
                                ),
                            ));
                        }
                        let x = b.origin.x + b.size.width * (hue / 360.0);
                        window.paint_quad(fill(
                            Bounds::new(
                                point(x - px(1.5), b.origin.y),
                                size(px(3.), b.size.height),
                            ),
                            rgb(0xFFFFFF),
                        ));
                    },
                )
                .size_full(),
            );
        let preview = rgb(self.rgb());
        let hex_row = div()
            .flex()
            .items_center()
            .gap(space::sm())
            .child(
                div()
                    .size(px(28.))
                    .flex_none()
                    .rounded(px(6.))
                    .border_1()
                    .border_color(border)
                    .bg(preview),
            )
            .child(
                div()
                    .id(SharedString::from(format!("{id}-hex")))
                    .track_focus(&self.focus)
                    .on_key_down(cx.listener(Self::on_hex_key))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| window.focus(&this.focus, cx)),
                    )
                    .flex_1()
                    .h(px(28.))
                    .px(space::sm())
                    .flex()
                    .items_center()
                    .rounded(radius::sm())
                    .border_1()
                    .border_color(border)
                    .text_sm()
                    .text_color(text)
                    .cursor_text()
                    .child(self.hex.view(150.0, window, cx)),
            );

        div()
            .id(SharedString::from(format!("{id}-panel")))
            .occlude()
            .w(px(SQUARE.0 + 24.))
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(6.))
            .bg(surface)
            .border_1()
            .border_color(border)
            .rounded(radius::md())
            .shadow_lg()
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, _, cx| {
                if ev.pressed_button == Some(MouseButton::Left) {
                    this.drag_to(ev.position, cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.end_drag(cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.end_drag(cx)),
            )
            .child(none)
            .child(div().flex().flex_col().gap(px(4.)).children(grid))
            .children(recent_row)
            .child(section(tr(lang, Key::CustomColor)))
            .child(square)
            .child(hue_bar)
            .child(hex_row)
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
        let menu = self.open.then(|| {
            deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .offset(point(px(0.), px(36.)))
                    .child(self.panel(window, cx)),
            )
            .with_priority(10)
        });
        div()
            .relative()
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                if this.open {
                    this.open = false;
                    cx.notify();
                }
            }))
            .child(button)
            .children(menu)
    }
}
