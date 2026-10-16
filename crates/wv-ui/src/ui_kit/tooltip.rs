use gpui_kit::*;

use crate::theme::Theme;

struct TipView {
    text: SharedString,
}

impl Render for TipView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        div()
            .px(px(8.))
            .py(px(4.))
            .rounded(px(6.))
            .bg(t.text)
            .text_color(t.surface)
            .text_xs()
            .child(self.text.clone())
    }
}

/// `.tip("text")` on a button shows a tooltip on hover.
pub trait Tip {
    fn tip(self, text: impl Into<SharedString>) -> Self;
}

impl Tip for Stateful<Div> {
    fn tip(self, text: impl Into<SharedString>) -> Self {
        let text = text.into();
        self.tooltip(move |_, cx| cx.new(|_| TipView { text: text.clone() }).into())
    }
}
