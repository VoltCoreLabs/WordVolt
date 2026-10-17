use gpui_kit::*;
use wv_i18n::{Key, t};

use crate::ui_kit::{ButtonKind, Tip, block_group, button};
use crate::window::state::MainWindow;

/// Zoom steps of the − / + buttons.
pub(crate) const STEP: f32 = 1.25;

/// View tab: zoom of the page view.
pub fn groups(this: &MainWindow, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let zoom = this.editor.read(cx).zoom;
    let group = block_group(
        t(this.lang, Key::GroupZoom),
        vec![
            button("zoom-out", "−", ButtonKind::Default, cx)
                .tip(t(this.lang, Key::TipZoomOut))
                .on_click(zoom_to(cx, |z| z / STEP))
                .into_any_element(),
            button(
                "zoom-reset",
                format!("{}%", (zoom * 100.0).round() as u32),
                ButtonKind::Default,
                cx,
            )
            .tip(t(this.lang, Key::TipZoomReset))
            .on_click(zoom_to(cx, |_| 1.0))
            .into_any_element(),
            button("zoom-in", "+", ButtonKind::Default, cx)
                .tip(t(this.lang, Key::TipZoomIn))
                .on_click(zoom_to(cx, |z| z * STEP))
                .into_any_element(),
            button(
                "zoom-fit",
                t(this.lang, Key::ZoomFit),
                ButtonKind::Default,
                cx,
            )
            .on_click(cx.listener(|this, _, _, cx| {
                this.editor.update(cx, |view, cx| {
                    let fit = view.fit_width_zoom();
                    view.set_zoom(fit, cx);
                });
                cx.notify();
            }))
            .into_any_element(),
        ],
        cx,
    );
    vec![group.into_any_element()]
}

pub(crate) fn zoom_to(
    cx: &mut Context<MainWindow>,
    f: fn(f32) -> f32,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    cx.listener(move |this, _, _, cx| {
        this.editor
            .update(cx, |view, cx| view.set_zoom(f(view.zoom), cx));
        cx.notify();
    })
}
