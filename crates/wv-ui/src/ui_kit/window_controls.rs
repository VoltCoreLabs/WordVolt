use gpui_kit::*;

use super::icon::{Icon, Weight, icon};
use crate::theme::{Theme, radius, space};

/// Minimise / maximise-restore / close buttons for the custom titlebar.
/// `window_control_area` lets Windows handle them natively (snap layouts, etc.).
pub fn window_controls(window: &Window, cx: &App) -> Div {
    let max_icon = if window.is_maximized() {
        Icon::Restore
    } else {
        Icon::Maximize
    };
    div()
        .flex()
        .gap(space::xs())
        .child(control(
            "win-min",
            Icon::Minimize,
            WindowControlArea::Min,
            false,
            cx,
            |w| w.minimize_window(),
        ))
        .child(control(
            "win-max",
            max_icon,
            WindowControlArea::Max,
            false,
            cx,
            |w| w.zoom_window(),
        ))
        .child(control(
            "win-close",
            Icon::Close,
            WindowControlArea::Close,
            true,
            cx,
            |w| w.remove_window(),
        ))
}

fn control(
    id: &'static str,
    glyph: Icon,
    area: WindowControlArea,
    is_close: bool,
    cx: &App,
    on_click: impl Fn(&mut Window) + 'static,
) -> Stateful<Div> {
    let t = cx.global::<Theme>();
    let (hover_bg, hover_fg) = if is_close {
        (t.danger, t.on_accent)
    } else {
        (t.surface, t.text)
    };
    let hover_bg = if is_close { hover_bg } else { t.border };
    let group = SharedString::from(id);
    div()
        .id(id)
        .group(group.clone())
        .window_control_area(area)
        .size(px(32.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(radius::md())
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .on_click(move |_, window, _| on_click(window))
        .child(
            icon(glyph, Weight::Regular, px(16.), t.text)
                .group_hover(group, move |s| s.text_color(hover_fg)),
        )
}
