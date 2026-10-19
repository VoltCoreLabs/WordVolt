/// Quick fade-in right after a keypress/click (the only phase that needs animation frames).
pub const FADE_IN: f32 = 0.12;
const SOLID: f32 = 0.55;
const HALF: f32 = 0.5;

/// Caret opacity `secs` after the last keypress/click: quick fade-in, solid for a moment,
/// then a stepped blink (visible/hidden every half second).
pub fn caret_alpha(secs: f32) -> f32 {
    if secs < FADE_IN {
        return smooth(secs / FADE_IN);
    }
    if secs < SOLID || (((secs - SOLID) / HALF) as u32).is_multiple_of(2) {
        return 1.0;
    }
    0.0
}

/// Seconds until `caret_alpha` next changes its step (for a timer instead of per-frame redraws).
pub fn next_change(secs: f32) -> f32 {
    let left = if secs < SOLID {
        SOLID - secs
    } else {
        HALF - (secs - SOLID) % HALF
    };
    left.max(0.02)
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[cfg(test)]
mod tests {
    use super::{caret_alpha, next_change};

    #[test]
    fn fades_in_then_blinks() {
        assert_eq!(caret_alpha(0.0), 0.0);
        assert!(caret_alpha(0.06) > 0.0 && caret_alpha(0.06) < 1.0);
        assert_eq!(caret_alpha(0.3), 1.0);
        assert_eq!(caret_alpha(0.55 + 0.6), 0.0);
        assert_eq!(caret_alpha(0.55 + 1.1), 1.0);
        assert!((0.0..=1.0).contains(&caret_alpha(7.77)));
    }

    #[test]
    fn next_change_hits_the_step() {
        assert!((next_change(0.3) - 0.25).abs() < 1e-4);
        assert!((next_change(0.55 + 0.1) - 0.4).abs() < 1e-4);
        assert!(next_change(0.55 + 0.5) >= 0.02);
    }
}
