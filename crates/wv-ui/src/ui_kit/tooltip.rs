use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::base::animation::{EffectTransition, ease_in_out_cubic, ease_out_cubic};
use gpui_kit::base::{
    ElementExt, TooltipDefaults, TooltipOverlay, TooltipRequest, TooltipTransition,
};
use gpui_kit::*;

use crate::theme::Theme;

/// Slide-and-fade when a tooltip first appears.
const ENTER: Duration = Duration::from_millis(150);
/// Glide from one button to the next when moving along a row of buttons.
const SLIDE: Duration = Duration::from_millis(200);

/// The window's tooltip overlay (gpui-kit's managed tooltip system: show delay, grace period between
/// neighbouring buttons, smart placement, animations).
struct Host(Entity<TooltipOverlay>);

impl Global for Host {}

/// Bounds of the trigger whose tooltip is shown (or was dismissed by a click) until the pointer leaves it.
#[derive(Default)]
struct Active(Cell<Option<Bounds<Pixels>>>);

impl Global for Active {}

struct TipView {
    text: SharedString,
}

impl Render for TipView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>();
        div()
            .m(px(6.))
            .px(px(8.))
            .py(px(4.))
            .rounded(px(6.))
            .bg(t.text)
            .text_color(t.surface)
            .text_xs()
            .shadow_md()
            .child(self.text.clone())
    }
}

fn render_tip(
    view: AnyView,
    transition: TooltipTransition,
    _: &mut Window,
    _: &mut App,
) -> AnyElement {
    let element = div().child(view);
    match transition {
        TooltipTransition::Enter { epoch } => EffectTransition::new(ENTER)
            .ease(ease_out_cubic)
            .slide_y(px(4.), px(0.))
            .fade(0.0, 1.0)
            .apply(
                element,
                ElementId::NamedInteger("tip-enter".into(), epoch as u64),
            )
            .into_any_element(),
        TooltipTransition::Switch {
            epoch,
            previous,
            current,
        } => {
            // Only glide along a row; across rows just appear.
            if (current.origin.y - previous.origin.y).abs() >= px(10.) {
                return element.into_any_element();
            }
            let dx = current.center().x - previous.center().x;
            EffectTransition::new(SLIDE)
                .ease(ease_in_out_cubic)
                .slide_x(-dx, px(0.))
                .apply(
                    element,
                    ElementId::NamedInteger("tip-slide".into(), epoch as u64),
                )
                .into_any_element()
        }
    }
}

/// Creates the overlay; call once at startup. Render [`layer`] once, at the top of the window.
pub fn init(cx: &mut App) {
    TooltipDefaults::new()
        .with_show_delay(Duration::from_millis(450))
        .install(cx);
    let overlay = cx.new(|_| TooltipOverlay::new().render_with(render_tip));
    cx.set_global(Host(overlay));
    cx.set_global(Active::default());
}

/// The overlay view: add it as the last child of the window's root element.
pub fn layer(cx: &App) -> AnyView {
    cx.global::<Host>().0.clone().into()
}

/// Hides the tooltip (the pointer left the button).
pub fn hide(window: &mut Window, cx: &mut App) {
    cx.global::<Active>().0.set(None);
    let overlay = cx.global::<Host>().0.clone();
    overlay.update(cx, |overlay, cx| overlay.request_hide(window, cx));
}

fn show(text: &SharedString, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    let active = cx.global::<Active>();
    if active.0.get() == Some(bounds) {
        return;
    }
    active.0.set(Some(bounds));
    let overlay = cx.global::<Host>().0.clone();
    let text = text.clone();
    let request = TooltipRequest::new(bounds, move |_, cx| {
        cx.new(|_| TipView { text: text.clone() }).into()
    });
    overlay.update(cx, |overlay, cx| overlay.request_show(request, window, cx));
}

/// A click dismisses the tooltip and keeps it away until the pointer leaves and comes back.
fn dismiss(bounds: Bounds<Pixels>, cx: &mut App) {
    cx.global::<Active>().0.set(Some(bounds));
    let overlay = cx.global::<Host>().0.clone();
    overlay.update(cx, |overlay, cx| overlay.hide(cx));
}

/// `.tip("text")` on a button shows an animated tooltip. The pointer leaving the button hides it
/// (`track_hover` calls [`hide`]).
pub trait Tip {
    fn tip(self, text: impl Into<SharedString>) -> Self;
}

impl Tip for Stateful<Div> {
    fn tip(self, text: impl Into<SharedString>) -> Self {
        let text: SharedString = text.into();
        let bounds = Rc::new(Cell::new(Bounds::<Pixels>::default()));
        let writer = bounds.clone();
        let shown = bounds.clone();
        self.on_prepaint(move |b, _, _| writer.set(b))
            .on_mouse_move(move |_, window, cx| show(&text, shown.get(), window, cx))
            .on_mouse_down(MouseButton::Left, move |_, _, cx| dismiss(bounds.get(), cx))
    }
}
