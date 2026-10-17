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

/// Invisible probe that stores the bounds of the drop-down panel it is placed in (the panel must be
/// `relative()`). A panel is drawn as a separate layer, so GPUI's "mouse down outside" would also fire
/// for clicks inside it; callers compare the click position with these bounds instead.
pub fn track_bounds(bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>) -> impl IntoElement {
    canvas(move |b, _, _| bounds.set(b), |_, _, _, _| {})
        .absolute()
        .top_0()
        .left_0()
        .size_full()
}
