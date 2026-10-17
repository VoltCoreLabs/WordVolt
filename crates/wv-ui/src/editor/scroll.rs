use std::cell::Cell;
use std::time::Instant;

use gpui_kit::*;

/// Smooth vertical scrolling: the wheel moves `target`, every frame `current` eases towards it.
/// Cells, because it is shared with paint closures.
pub struct ScrollState {
    target: Cell<f32>,
    current: Cell<f32>,
    viewport_top: Cell<f32>,
    viewport_h: Cell<f32>,
    viewport_w: Cell<f32>,
    content_h: Cell<f32>,
    last_frame: Cell<Option<Instant>>,
    last_activity: Cell<Instant>,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self {
            target: Cell::new(0.),
            current: Cell::new(0.),
            viewport_top: Cell::new(0.),
            viewport_h: Cell::new(0.),
            viewport_w: Cell::new(0.),
            content_h: Cell::new(0.),
            last_frame: Cell::new(None),
            last_activity: Cell::new(Instant::now()),
        }
    }
}

impl ScrollState {
    fn max(&self) -> f32 {
        (self.content_h.get() - self.viewport_h.get()).max(0.)
    }

    pub fn set_viewport(&self, bounds: Bounds<Pixels>) {
        self.viewport_top.set(f32::from(bounds.top()));
        self.viewport_h.set(f32::from(bounds.size.height));
        self.viewport_w.set(f32::from(bounds.size.width));
    }

    pub fn set_content_height(&self, height: Pixels) {
        self.content_h.set(f32::from(height));
    }

    pub fn viewport_width(&self) -> f32 {
        self.viewport_w.get()
    }

    pub fn viewport_height(&self) -> f32 {
        self.viewport_h.get()
    }

    /// Scrolls just enough to bring something at `y` (from the top of the content, height `h`)
    /// into view.
    pub fn reveal_at(&self, y: f32, h: f32) {
        const MARGIN: f32 = 48.;
        let current = self.current.get();
        let (top, bottom) = (current + MARGIN, current + self.viewport_h.get() - MARGIN);
        let target = if y < top {
            y - MARGIN
        } else if y + h > bottom {
            y + h - self.viewport_h.get() + MARGIN
        } else {
            return;
        };
        self.target.set(target.clamp(0., self.max()));
        self.last_activity.set(Instant::now());
    }

    /// Keeps the same part of the document in view after the content height changed by `factor`.
    pub fn rescale(&self, factor: f32) {
        self.target.set(self.target.get() * factor);
        self.current.set(self.current.get() * factor);
    }

    pub fn reset(&self) {
        self.target.set(0.);
        self.current.set(0.);
    }

    /// Wheel/trackpad: positive `dy` scrolls down.
    pub fn scroll_by(&self, dy: f32) {
        self.target
            .set((self.target.get() + dy).clamp(0., self.max()));
        self.last_activity.set(Instant::now());
    }

    /// Advances the animation one frame; returns the current offset.
    pub fn tick(&self, window: &Window) -> f32 {
        let (target, current) = (self.target.get().clamp(0., self.max()), self.current.get());
        if (target - current).abs() < 0.3 {
            self.current.set(target);
            self.last_frame.set(None);
            return target;
        }
        let now = Instant::now();
        let dt = self
            .last_frame
            .get()
            .map_or(1. / 60., |t| (now - t).as_secs_f32())
            .min(0.05);
        self.last_frame.set(Some(now));
        let next = current + (target - current) * (1. - (-dt * 16.).exp());
        self.current.set(next);
        window.request_animation_frame();
        next
    }

    /// Thin scrollbar thumb that fades out when idle.
    pub fn paint_scrollbar(&self, bounds: Bounds<Pixels>, color: Rgba, window: &mut Window) {
        let max = self.max();
        let idle = self.last_activity.get().elapsed().as_secs_f32();
        let alpha = if idle < 0.8 {
            1.
        } else {
            1. - (idle - 0.8) / 0.4
        };
        if max <= 0. || alpha <= 0. {
            return;
        }
        if alpha < 1. {
            window.request_animation_frame();
        }
        let (vh, ch) = (self.viewport_h.get(), self.content_h.get());
        let thumb = (vh * vh / ch).max(32.);
        let y = bounds.top() + px((vh - thumb) * self.current.get() / max);
        let rect = Bounds::new(point(bounds.right() - px(8.), y), size(px(4.), px(thumb)));
        let mut quad = fill(
            rect,
            Rgba {
                a: alpha * 0.8,
                ..color
            },
        );
        quad.corner_radii = Corners::all(px(2.));
        window.paint_quad(quad);
    }
}
