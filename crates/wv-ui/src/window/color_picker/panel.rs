use super::*;

impl ColorPicker {
    pub(super) fn panel(&mut self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            .relative()
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
            .child(track_bounds(self.panel_bounds.clone()))
            .child(none)
            .child(div().flex().flex_col().gap(px(4.)).children(grid))
            .children(recent_row)
            .child(section(tr(lang, Key::CustomColor)))
            .child(square)
            .child(hue_bar)
            .child(hex_row)
    }
}
