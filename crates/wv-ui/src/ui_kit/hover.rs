use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use gpui_kit::*;

/// Hover state per element id: (hovered, generation). The generation changes on every enter/leave
/// so the fade animation restarts.
#[derive(Default)]
pub struct HoverMap(RefCell<HashMap<ElementId, (bool, u64)>>);

impl Global for HoverMap {}

impl HoverMap {
    fn get(&self, id: &ElementId) -> (bool, u64) {
        self.0.borrow().get(id).copied().unwrap_or((false, 0))
    }

    fn set(&self, id: &ElementId, hovered: bool) {
        let mut map = self.0.borrow_mut();
        let entry = map.entry(id.clone()).or_insert((false, 0));
        if entry.0 != hovered {
            *entry = (hovered, entry.1 + 1);
        }
    }
}

/// Look of the layer that fades in on hover.
pub struct HoverLook {
    pub bg: Rgba,
    pub border: Option<Rgba>,
    pub radius: Pixels,
    /// Grow by this much on every side (1px to cover the element's own border).
    pub bleed: Pixels,
}

/// Makes `el` report hover changes so [`hover_layer`] can fade.
pub fn track_hover(el: Stateful<Div>, id: &ElementId) -> Stateful<Div> {
    let id = id.clone();
    el.on_hover(move |hovered, window, cx| {
        cx.global::<HoverMap>().set(&id, *hovered);
        if !*hovered {
            super::tooltip::hide(window, cx);
        }
        window.refresh();
    })
}

/// A layer (put it as the FIRST child of a `relative()` element) that fades in/out with hover.
/// `content` is drawn inside the layer, so it fades together with the background.
pub fn hover_layer(
    id: &ElementId,
    look: HoverLook,
    content: Option<AnyElement>,
    cx: &App,
) -> AnyElement {
    let (hovered, generation) = cx.global::<HoverMap>().get(id);
    let mut layer = div()
        .absolute()
        .top(-look.bleed)
        .left(-look.bleed)
        .right(-look.bleed)
        .bottom(-look.bleed)
        .flex()
        .items_center()
        .justify_center()
        .rounded(look.radius)
        .bg(look.bg);
    if let Some(border) = look.border {
        layer = layer.border_1().border_color(border);
    }
    layer = layer.children(content);
    if generation == 0 {
        return layer.opacity(0.).into_any_element();
    }
    layer
        .with_animation(
            SharedString::from(format!("hover-{id:?}-{generation}")),
            Animation::new(Duration::from_millis(140)).with_easing(ease_out_quint()),
            move |el, delta| el.opacity(if hovered { delta } else { 1. - delta }),
        )
        .into_any_element()
}
