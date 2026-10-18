use super::*;

impl ImageDialog {
    pub(super) fn field(
        &self,
        index: usize,
        label: Key,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Div {
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
                    .child(tr(self.lang, label)),
            )
            .child(
                div()
                    .id(("image-field", index))
                    .track_focus(&self.focus[index])
                    .key_context("TabField")
                    .on_action(cx.listener(
                        move |this, _: &crate::actions::FieldNext, window, cx| {
                            window.focus(&this.focus[(index + 1) % 4], cx);
                            cx.notify();
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

    pub(super) fn mode_button(&self, i: usize, cx: &mut Context<Self>) -> Stateful<Div> {
        let th = cx.global::<Theme>();
        let (border, accent, subtle, hover) =
            (th.border, th.accent, th.accent_subtle, th.surface_alt);
        let (label, wrap) = MODES[i];
        let active = self.mode == wrap;
        div()
            .id(("image-wrap", i))
            .h(px(30.))
            .px(space::sm())
            .flex()
            .items_center()
            .rounded(radius::md())
            .border_1()
            .border_color(if active { accent } else { border })
            .when(active, |d| d.bg(subtle).text_color(accent))
            .when(!active, |d| d.hover(move |s| s.bg(hover)))
            .text_sm()
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.mode = wrap;
                    cx.notify();
                }),
            )
            .child(tr(self.lang, label))
    }

    pub(super) fn panel(&mut self, window: &Window, cx: &mut Context<Self>) -> Stateful<Div> {
        let th = cx.global::<Theme>();
        let (surface, border, muted, accent, subtle) = (
            th.surface,
            th.border,
            th.text_muted,
            th.accent,
            th.accent_subtle,
        );
        let lang = self.lang;
        let floating = self.mode.is_some();
        let keep = self.keep_proportions;
        let modes = div()
            .flex()
            .flex_wrap()
            .gap(px(6.))
            .children((0..MODES.len()).map(|i| self.mode_button(i, cx)));
        div()
            .id("image-dialog")
            .occlude()
            .w(px(2.0 * FIELD_WIDTH + 52.0))
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .bg(surface)
            .border_1()
            .border_color(border)
            .rounded(radius::md())
            .shadow_lg()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(tr(lang, Key::ImageDialogTitle)),
            )
            .child(
                div()
                    .flex()
                    .gap(space::md())
                    .child(self.field(0, Key::ImageWidth, window, cx))
                    .child(self.field(1, Key::ImageHeight, window, cx)),
            )
            .child(
                div()
                    .id("image-keep")
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .cursor_pointer()
                    .text_sm()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.keep_proportions = !this.keep_proportions;
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .size(px(16.))
                            .rounded(px(4.))
                            .border_1()
                            .border_color(if keep { accent } else { border })
                            .when(keep, |d| d.bg(subtle))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(accent)
                            .child(if keep { "✓" } else { "" }),
                    )
                    .child(tr(lang, Key::ImageKeepProportions)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(tr(lang, Key::ImageWrapTitle)),
            )
            .child(modes)
            .when(floating, |d| {
                d.child(
                    div()
                        .flex()
                        .gap(space::md())
                        .child(self.field(2, Key::ImageLeft, window, cx))
                        .child(self.field(3, Key::ImageTop, window, cx)),
                )
            })
            .child(
                button("image-apply", tr(lang, Key::Apply), ButtonKind::Primary, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.apply(cx))),
            )
    }
}
