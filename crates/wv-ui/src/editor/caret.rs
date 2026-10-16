/// Caret opacity `secs` after the last keypress/click: quick fade-in, solid for a moment,
/// then a smooth blink (fade out, hidden, fade in).
pub fn caret_alpha(secs: f32) -> f32 {
    const FADE_IN: f32 = 0.12;
    const SOLID: f32 = 0.55;
    const PERIOD: f32 = 1.0;
    if secs < FADE_IN {
        return smooth(secs / FADE_IN);
    }
    if secs < SOLID {
        return 1.0;
    }
    match (secs - SOLID) % PERIOD {
        t if t < 0.2 => 1.0 - smooth(t / 0.2),
        t if t < 0.5 => 0.0,
        t if t < 0.7 => smooth((t - 0.5) / 0.2),
        _ => 1.0,
    }
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[cfg(test)]
mod tests {
    use super::caret_alpha;

    #[test]
    fn fades_in_then_blinks() {
        assert_eq!(caret_alpha(0.0), 0.0);
        assert!(caret_alpha(0.06) > 0.0 && caret_alpha(0.06) < 1.0);
        assert_eq!(caret_alpha(0.3), 1.0);
        assert_eq!(caret_alpha(0.55 + 0.35), 0.0);
        assert_eq!(caret_alpha(0.55 + 0.9), 1.0);
        assert!((0.0..=1.0).contains(&caret_alpha(7.77)));
    }
}
