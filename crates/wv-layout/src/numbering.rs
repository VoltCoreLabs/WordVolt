use wv_core::{ListKind, ParaStyle};

/// Item counters per nesting level (0..=8); turns list paragraphs into markers ("1.", "a.", "•").
#[derive(Default)]
pub struct Counters([u32; 9]);

impl Counters {
    /// Marker text of a paragraph with this style, or `None` if it is not a list item.
    /// Must be called for every paragraph in document order.
    pub fn marker(&mut self, style: &ParaStyle) -> Option<String> {
        let level = usize::from(style.indent).min(8);
        match style.list {
            Some(ListKind::Number) => {
                self.0[level] += 1;
                self.0[level + 1..].fill(0);
                Some(number_label(self.0[level], level))
            }
            Some(ListKind::Bullet) => {
                self.0[level..].fill(0);
                Some(["\u{2022}", "\u{25E6}", "\u{25AA}"][level % 3].to_string())
            }
            None => {
                self.0 = [0; 9];
                None
            }
        }
    }
}

/// 1. / a. / i. by nesting level, repeating every three levels.
pub fn number_label(n: u32, level: usize) -> String {
    match level % 3 {
        0 => format!("{n}."),
        1 => format!("{}.", alpha(n)),
        _ => format!("{}.", roman(n)),
    }
}

/// a, b, ... z, aa, ab, ...
fn alpha(mut n: u32) -> String {
    let mut out = Vec::new();
    while n > 0 {
        n -= 1;
        out.push(b'a' + (n % 26) as u8);
        n /= 26;
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

fn roman(n: u32) -> String {
    if n == 0 || n >= 4000 {
        return n.to_string();
    }
    let table = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut n = n;
    let mut out = String::new();
    for (value, text) in table {
        while n >= value {
            out.push_str(text);
            n -= value;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use wv_core::{ListKind, ParaStyle};

    use super::*;

    fn item(kind: Option<ListKind>, indent: u8) -> ParaStyle {
        ParaStyle {
            list: kind,
            indent,
            ..Default::default()
        }
    }

    #[test]
    fn labels_by_level() {
        assert_eq!(number_label(3, 0), "3.");
        assert_eq!(number_label(1, 1), "a.");
        assert_eq!(number_label(27, 1), "aa.");
        assert_eq!(number_label(4, 2), "iv.");
        assert_eq!(number_label(1994, 2), "mcmxciv.");
        assert_eq!(number_label(2, 3), "2.");
    }

    #[test]
    fn numbering_counts_nests_and_restarts() {
        let mut c = Counters::default();
        let n = Some(ListKind::Number);
        let marks: Vec<_> = [
            item(n, 0),
            item(n, 0),
            item(n, 1),
            item(n, 1),
            item(n, 0),
            item(None, 0),
            item(n, 0),
        ]
        .iter()
        .map(|s| c.marker(s))
        .collect();
        let marks: Vec<_> = marks.iter().map(|m| m.as_deref()).collect();
        assert_eq!(
            marks,
            [
                Some("1."),
                Some("2."),
                Some("a."),
                Some("b."),
                Some("3."),
                None,
                Some("1.")
            ]
        );
    }

    #[test]
    fn bullets_change_glyph_by_level() {
        let mut c = Counters::default();
        let b = Some(ListKind::Bullet);
        assert_eq!(c.marker(&item(b, 0)).as_deref(), Some("\u{2022}"));
        assert_eq!(c.marker(&item(b, 1)).as_deref(), Some("\u{25E6}"));
        assert_eq!(c.marker(&item(b, 2)).as_deref(), Some("\u{25AA}"));
    }
}
