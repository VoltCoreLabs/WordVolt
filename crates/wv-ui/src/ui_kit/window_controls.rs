use gpui_kit::*;

use super::hover::{HoverLook, hover_layer, track_hover};
use super::icon::{Icon, Weight, icon};
use crate::theme::{Theme, radius, space};

/// Width of the three buttons plus gaps (3 * 32 + 2 * 4).
pub const CONTROLS_WIDTH: Pixels = px(104.);

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
    let id = ElementId::from(id);
    // Close turns red with a white icon; the icon is part of the fading layer so both change together.
    let (bg, hover_icon) = if is_close {
        (
            t.danger,
            Some(icon(glyph, Weight::Regular, px(16.), t.on_accent).into_any_element()),
        )
    } else {
        (t.border, None)
    };
    let look = HoverLook {
        bg,
        border: None,
        radius: radius::md(),
        bleed: px(0.),
    };
    let control = div()
        .id(id.clone())
        .relative()
        .window_control_area(area)
        .size(px(32.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(radius::md())
        .cursor_pointer()
        .on_click(move |_, window, _| on_click(window));
    let base_icon = icon(glyph, Weight::Regular, px(16.), t.text);
    let layer = hover_layer(&id, look, hover_icon, cx);
    let control = track_hover(control, &id);
    if is_close {
        // The red layer (with its white icon) must cover the dark icon.
        control.child(base_icon).child(layer)
    } else {
        control.child(layer).child(base_icon)
    }
}
