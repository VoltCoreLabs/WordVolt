//! Colour helpers on 0xRRGGBB values.

fn channels(c: u32) -> (f32, f32, f32) {
    (
        ((c >> 16) & 0xFF) as f32 / 255.0,
        ((c >> 8) & 0xFF) as f32 / 255.0,
        (c & 0xFF) as f32 / 255.0,
    )
}

fn pack(r: f32, g: f32, b: f32) -> u32 {
    let byte = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u32;
    byte(r) << 16 | byte(g) << 8 | byte(b)
}

/// (hue 0..360, saturation 0..1, value 0..1)
pub fn rgb_to_hsv(c: u32) -> (f32, f32, f32) {
    let (r, g, b) = channels(c);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let delta = max - min;
    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    (
        hue.rem_euclid(360.0),
        if max == 0.0 { 0.0 } else { delta / max },
        max,
    )
}

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> u32 {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h.rem_euclid(360.0) / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    pack(r + m, g + m, b + m)
}

/// `a` mixed with `b`: t = 0 gives `a`, t = 1 gives `b`.
pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    let ((ar, ag, ab), (br, bg, bb)) = (channels(a), channels(b));
    pack(ar + (br - ar) * t, ag + (bg - ag) * t, ab + (bb - ab) * t)
}

/// `#RRGGBB`, `RRGGBB`, `#RGB` or `RGB`.
pub fn parse_hex(text: &str) -> Option<u32> {
    let digits = text.trim().trim_start_matches('#');
    match digits.len() {
        6 => u32::from_str_radix(digits, 16).ok(),
        3 => {
            let expanded: String = digits.chars().flat_map(|c| [c, c]).collect();
            u32::from_str_radix(&expanded, 16).ok()
        }
        _ => None,
    }
}

pub fn hex(c: u32) -> String {
    format!("#{:06X}", c & 0xFF_FFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_round_trip() {
        for c in [
            0x000000, 0xFFFFFF, 0xFF0000, 0x00FF00, 0x0000FF, 0x2563EB, 0xFFCC00, 0x808080,
        ] {
            let (h, s, v) = rgb_to_hsv(c);
            assert_eq!(hsv_to_rgb(h, s, v), c, "{c:06X}");
        }
    }

    #[test]
    fn hex_parsing_and_mixing() {
        assert_eq!(parse_hex("#2563eb"), Some(0x2563EB));
        assert_eq!(parse_hex("f00"), Some(0xFF0000));
        assert_eq!(parse_hex("#12"), None);
        assert_eq!(parse_hex("zzzzzz"), None);
        assert_eq!(hex(0x0000FF), "#0000FF");
        assert_eq!(mix(0x000000, 0xFFFFFF, 0.0), 0x000000);
        assert_eq!(mix(0x000000, 0xFFFFFF, 1.0), 0xFFFFFF);
    }
}
