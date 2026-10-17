use gpui_kit::prelude::FluentBuilder;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::*;

use super::hover::{HoverLook, hover_layer, track_hover};
use super::pop::track_bounds;
use crate::theme::{Theme, space};

/// Geometry and animation state of a pill tab bar. Keep one per bar and pass it to every render.
pub struct PillState {
    bar: Rc<Cell<Bounds<Pixels>>>,
    tabs: Vec<Rc<Cell<Bounds<Pixels>>>>,
    shown: Cell<usize>,
    prev: Cell<usize>,
    epoch: Cell<u64>,
}

impl PillState {
    pub fn new(count: usize) -> Self {
        Self {
            bar: Default::default(),
            tabs: (0..count).map(|_| Default::default()).collect(),
            shown: Cell::new(0),
            prev: Cell::new(0),
            epoch: Cell::new(0),
        }
    }

    /// Notes a selection change (restarts the slide from the previously shown tab).
    fn sync(&self, selected: usize) {
        if self.shown.get() != selected {
            self.prev.set(self.shown.get());
            self.shown.set(selected);
            self.epoch.set(self.epoch.get() + 1);
        }
    }

    /// (left, top, width, height) of tab `i` relative to the bar; `None` before the first paint.
    fn rect(&self, i: usize) -> Option<(f32, f32, f32, f32)> {
        let (bar, tab) = (self.bar.get(), self.tabs.get(i)?.get());
        (bar.size.width > px(0.) && tab.size.width > px(0.)).then(|| {
            (
                f32::from(tab.origin.x - bar.origin.x),
                f32::from(tab.origin.y - bar.origin.y),
                f32::from(tab.size.width),
                f32::from(tab.size.height),
            )
        })
    }
}

/// Label of a pill tab; the capsule behind the active one is drawn by `pill_tabs`.
pub fn tab(label: &'static str, active: bool, cx: &App) -> Stateful<Div> {
    let t = cx.global::<Theme>();
    let (fg, hover) = (if active { t.on_accent } else { t.text_muted }, t.surface);
    let id = ElementId::from(SharedString::from(format!("tab-{label}")));
    let look = HoverLook {
        bg: hover,
        border: None,
        radius: px(999.),
        bleed: px(0.),
    };
    let tab = div()
        .id(id.clone())
        .relative()
        .px(space::lg())
        .py(px(6.))
        .rounded(px(999.))
        .text_color(fg)
        .text_sm()
        .when(active, |s| s.font_weight(FontWeight::SEMIBOLD))
        .cursor_pointer();
    // The active pill has no hover effect.
    track_hover(tab, &id)
        .when(!active, |t| t.child(hover_layer(&id, look, None, cx)))
        .child(div().relative().child(label))
}

/// Capsule that holds the pill tabs; a white capsule slides to the selected tab.
pub fn pill_tabs(
    state: &PillState,
    selected: usize,
    tabs: impl IntoIterator<Item = Stateful<Div>>,
    cx: &App,
) -> Div {
    state.sync(selected);
    let t = cx.global::<Theme>();
    let (bar_bg, accent) = (t.border, t.accent);
    let capsule = |left: f32, top: f32, width: f32, height: f32| {
        div()
            .absolute()
            .left(px(left))
            .top(px(top))
            .w(px(width))
            .h(px(height))
            .rounded(px(999.))
            .bg(accent)
    };
    let slide = match (state.rect(state.prev.get()), state.rect(selected)) {
        (Some(from), Some(to)) => {
            let lerp = |a: f32, b: f32, d: f32| a + (b - a) * d;
            Some(
                capsule(to.0, to.1, to.2, to.3)
                    .with_animation(
                        SharedString::from(format!("pill-slide-{}", state.epoch.get())),
                        Animation::new(Duration::from_millis(260)).with_easing(ease_out_quint()),
                        move |el, d| {
                            el.left(px(lerp(from.0, to.0, d)))
                                .w(px(lerp(from.2, to.2, d)))
                        },
                    )
                    .into_any_element(),
            )
        }
        _ => None,
    };
    let tabs: Vec<_> = tabs
        .into_iter()
        .enumerate()
        .map(|(i, tab)| match state.tabs.get(i) {
            Some(probe) => tab.child(track_bounds(probe.clone())),
            None => tab,
        })
        .collect();
    div()
        .relative()
        .flex()
        .items_center()
        .gap(px(2.))
        .p(px(3.))
        .rounded(px(999.))
        .bg(bar_bg)
        .child(track_bounds(state.bar.clone()))
        .children(slide)
        .children(tabs)
}
