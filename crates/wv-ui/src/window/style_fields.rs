use wv_core::{Align, UserStyle};

/// Fields of the style dialog in tab order: name, font, size (pt), colour (hex), line spacing,
/// space before and after (pt).
pub const COUNT: usize = 7;

/// Text of the fields for a style (what `apply` reads back).
pub fn texts(s: &UserStyle) -> [String; COUNT] {
    [
        s.name.clone(),
        s.font.clone().unwrap_or_default(),
        s.size_pt.map_or(String::new(), |v| v.to_string()),
        s.color.map_or(String::new(), |c| format!("{c:06X}")),
        s.line_spacing.map_or(String::new(), |v| v.to_string()),
        s.space_before.map_or(String::new(), |v| v.to_string()),
        s.space_after.map_or(String::new(), |v| v.to_string()),
    ]
}

fn number(text: &str) -> Option<f32> {
    let v: f32 = text.trim().replace(',', ".").parse().ok()?;
    (v.is_finite() && v >= 0.0).then_some(v)
}

/// `base` with the values typed in the fields; an empty or invalid field means "not set".
pub fn apply(base: &UserStyle, t: &[String; COUNT]) -> UserStyle {
    let hex = t[3].trim().trim_start_matches('#');
    UserStyle {
        name: t[0].trim().to_string(),
        font: Some(t[1].trim().to_string()).filter(|f| !f.is_empty()),
        size_pt: number(&t[2]).map(|v| v.round().clamp(1.0, 400.0) as u16),
        color: u32::from_str_radix(hex, 16).ok().filter(|_| hex.len() == 6),
        line_spacing: number(&t[4]).map(|v| v.clamp(0.5, 5.0)),
        space_before: number(&t[5]).map(|v| v.round().min(255.0) as u8),
        space_after: number(&t[6]).map(|v| v.round().min(255.0) as u8),
        ..base.clone()
    }
}

/// The next alignment (none -> left -> centre -> right -> justify -> none).
pub fn next_align(a: Option<Align>) -> Option<Align> {
    match a {
        None => Some(Align::Left),
        Some(Align::Left) => Some(Align::Center),
        Some(Align::Center) => Some(Align::Right),
        Some(Align::Right) => Some(Align::Justify),
        Some(Align::Justify) => None,
    }
}

#[cfg(test)]
mod tests {
    use wv_core::{Align, UserStyle};

    use super::{COUNT, apply, next_align, texts};

    #[test]
    fn fields_round_trip_and_bad_values_unset() {
        let style = UserStyle {
            name: "Цитата".into(),
            font: Some("Georgia".into()),
            size_pt: Some(14),
            color: Some(0x1F4E79),
            line_spacing: Some(1.5),
            space_before: Some(6),
            space_after: Some(0),
            bold: true,
            ..UserStyle::default()
        };
        let t = texts(&style);
        assert_eq!(t[3], "1F4E79");
        assert_eq!(apply(&style, &t), style);
        let mut bad: [String; COUNT] = t.clone();
        bad[2] = "много".into();
        bad[3] = "#12".into();
        bad[4] = "99".into();
        bad[1] = "  ".into();
        let s = apply(&style, &bad);
        assert_eq!((s.size_pt, s.color, s.font), (None, None, None));
        assert_eq!(s.line_spacing, Some(5.0));
        assert!(s.bold);
        bad[3] = "#00ff7f".into();
        assert_eq!(apply(&style, &bad).color, Some(0x00FF7F));
    }

    #[test]
    fn align_cycles_through_unset() {
        let mut a = None;
        for _ in 0..5 {
            a = next_align(a);
        }
        assert_eq!(a, None);
        assert_eq!(next_align(None), Some(Align::Left));
    }
}
