use wv_core::ListKind;

/// A line that starts a list item: (nesting level, kind, text after the marker).
pub(super) fn list_marker(line: &str) -> Option<(u8, ListKind, &str)> {
    let trimmed = line.trim_start();
    let lead: usize = line[..line.len() - trimmed.len()]
        .chars()
        .map(|c| if c == '\t' { 2 } else { 1 })
        .sum();
    let level = (lead / 2).min(8) as u8;
    for bullet in ["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(bullet) {
            return Some((level, ListKind::Bullet, rest));
        }
    }
    let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
    if (1..=9).contains(&digits) {
        let after = &trimmed[digits..];
        for sep in [". ", ") "] {
            if let Some(rest) = after.strip_prefix(sep) {
                return Some((level, ListKind::Number, rest));
            }
        }
    }
    None
}
