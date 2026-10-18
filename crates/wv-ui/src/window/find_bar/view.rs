use super::*;

impl Render for FindBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.open {
            return div().into_any_element();
        }
        let lang = self.lang;
        let th = cx.global::<Theme>();
        let (border, accent, muted) = (th.border, th.accent, th.text_muted);

        let field = |id: &'static str,
                     edit: &LineEdit,
                     placeholder: Key,
                     focus: &FocusHandle,
                     focused: bool,
                     window: &Window,
                     cx: &App| {
            let content = if edit.text().is_empty() && !focused {
                div()
                    .text_color(muted)
                    .child(tr(lang, placeholder))
                    .into_any_element()
            } else {
                edit.view(260.0, window, cx).into_any_element()
            };
            div()
                .id(id)
                .track_focus(focus)
                .flex_1()
                .h(px(32.))
                .px(space::md())
                .flex()
                .items_center()
                .overflow_hidden()
                .rounded(radius::md())
                .border_1()
                .border_color(if focused { accent } else { border })
                .text_sm()
                .cursor_text()
                .child(content)
        };

        let find_focused = self.find_focus.is_focused(window);
        let replace_focused = self.replace_focus.is_focused(window);
        let find_field = field(
            "find-field",
            &self.find,
            Key::FindPlaceholder,
            &self.find_focus,
            find_focused,
            window,
            cx,
        )
        .key_context("TabField")
        .on_action(
            cx.listener(|this, _: &crate::actions::FieldNext, window, cx| {
                if this.show_replace {
                    window.focus(&this.replace_focus, cx);
                }
            }),
        )
        .on_key_down(cx.listener(Self::on_find_key))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _, window, cx| window.focus(&this.find_focus, cx)),
        );
        let counter = self.counter();
        let top = div()
            .flex()
            .items_center()
            .gap(space::sm())
            .child(find_field)
            .child(
                div()
                    .min_w(px(90.))
                    .text_sm()
                    .text_color(muted)
                    .child(counter),
            )
            .child(
                icon_button("find-prev", Icon::CaretUp, false, cx)
                    .tip(tr(lang, Key::TipFindPrev))
                    .on_click(cx.listener(|this, _, _, cx| this.step(-1, cx))),
            )
            .child(
                icon_button("find-next", Icon::CaretDown, false, cx)
                    .tip(tr(lang, Key::TipFindNext))
                    .on_click(cx.listener(|this, _, _, cx| this.step(1, cx))),
            )
            .child(
                icon_button("find-case", Icon::TextAa, self.match_case, cx)
                    .tip(tr(lang, Key::TipMatchCase))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.match_case = !this.match_case;
                        this.recompute(true, cx);
                    })),
            )
            .child(
                icon_button("find-word", Icon::WholeWord, self.whole_word, cx)
                    .tip(tr(lang, Key::TipWholeWord))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.whole_word = !this.whole_word;
                        this.recompute(true, cx);
                    })),
            )
            .child(
                icon_button("find-close", Icon::Close, false, cx)
                    .tip(tr(lang, Key::TipFindClose))
                    .on_click(cx.listener(|this, _, _, cx| this.close(cx))),
            );
        let replace_row = self.show_replace.then(|| {
            let replace_field = field(
                "replace-field",
                &self.replace,
                Key::ReplacePlaceholder,
                &self.replace_focus,
                replace_focused,
                window,
                cx,
            )
            .key_context("TabField")
            .on_action(
                cx.listener(|this, _: &crate::actions::FieldNext, window, cx| {
                    window.focus(&this.find_focus, cx)
                }),
            )
            .on_key_down(cx.listener(Self::on_replace_key))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| window.focus(&this.replace_focus, cx)),
            );
            div()
                .flex()
                .items_center()
                .gap(space::sm())
                .child(replace_field)
                .child(
                    button(
                        "replace-one",
                        tr(lang, Key::ReplaceOne),
                        ButtonKind::Default,
                        cx,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.replace_one(cx))),
                )
                .child(
                    button(
                        "replace-all",
                        tr(lang, Key::ReplaceAll),
                        ButtonKind::Primary,
                        cx,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.replace_all(cx))),
                )
        });
        block(cx)
            .p(space::sm())
            .flex()
            .flex_col()
            .gap(space::sm())
            .child(top)
            .children(replace_row)
            .into_any_element()
    }
}
