use std::time::Duration;

use gpui_kit::*;

/// Wraps a drop-down panel so it fades in while sliding down a few pixels. `key` must change on
/// every opening (e.g. include a counter), otherwise the animation only plays the first time.
pub fn pop_in(content: impl IntoElement, key: impl Into<SharedString>) -> AnimationElement<Div> {
    div().child(content).with_animation(
        key.into(),
        Animation::new(Duration::from_millis(140)).with_easing(ease_out_quint()),
        |el, delta| el.opacity(delta).mt(px(-6. * (1. - delta))),
    )
}
