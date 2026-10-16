use crate::position::Position;

/// Search settings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SearchOptions {
    pub match_case: bool,
    /// Only whole words: letters, digits and `_` must not touch the match.
    pub whole_word: bool,
}

fn fold(c: char, opts: SearchOptions) -> char {
    if opts.match_case {
        c
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Non-overlapping matches of `query` in `text`, as byte ranges. Case folding is per character
/// (simple lowercase), so offsets always stay valid. A query with a line break never matches.
pub fn find_in_text(text: &str, query: &str, opts: SearchOptions) -> Vec<(usize, usize)> {
    if query.is_empty() || query.contains('\n') {
        return Vec::new();
    }
    let needle: Vec<char> = query.chars().map(|c| fold(c, opts)).collect();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut found = Vec::new();
    let mut i = 0;
    while i + needle.len() <= chars.len() {
        let hit = (0..needle.len()).all(|k| fold(chars[i + k].1, opts) == needle[k]);
        if hit {
            let start = chars[i].0;
            let end = chars.get(i + needle.len()).map_or(text.len(), |c| c.0);
            let before_ok = text[..start]
                .chars()
                .next_back()
                .is_none_or(|c| !is_word(c));
            let after_ok = text[end..].chars().next().is_none_or(|c| !is_word(c));
            if !opts.whole_word || (before_ok && after_ok) {
                found.push((start, end));
                i += needle.len();
                continue;
            }
        }
        i += 1;
    }
    found
}

/// All matches in the document (in document order); every match lies inside one paragraph.
pub fn find_all(
    paragraph_texts: impl Iterator<Item = String>,
    query: &str,
    opts: SearchOptions,
) -> Vec<(Position, Position)> {
    let mut all = Vec::new();
    for (para, text) in paragraph_texts.enumerate() {
        for (start, end) in find_in_text(&text, query, opts) {
            all.push((Position::new(para, start), Position::new(para, end)));
        }
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANY: SearchOptions = SearchOptions {
        match_case: false,
        whole_word: false,
    };

    #[test]
    fn case_insensitive_by_default_and_exact_on_request() {
        assert_eq!(
            find_in_text("Cat cat CAT", "cat", ANY),
            [(0, 3), (4, 7), (8, 11)]
        );
        let exact = SearchOptions {
            match_case: true,
            ..ANY
        };
        assert_eq!(find_in_text("Cat cat CAT", "cat", exact), [(4, 7)]);
    }

    #[test]
    fn whole_words_only() {
        let words = SearchOptions {
            whole_word: true,
            ..ANY
        };
        assert_eq!(
            find_in_text("cat concat cat_x cat.", "cat", words),
            [(0, 3), (17, 20)]
        );
    }

    #[test]
    fn cyrillic_offsets_are_bytes_and_matches_do_not_overlap() {
        assert_eq!(
            find_in_text("Привет, привет", "ПРИВЕТ", ANY),
            [(0, 12), (14, 26)]
        );
        assert_eq!(find_in_text("aaaa", "aa", ANY), [(0, 2), (2, 4)]);
    }

    #[test]
    fn empty_and_multiline_queries_find_nothing() {
        assert!(find_in_text("abc", "", ANY).is_empty());
        assert!(
            find_in_text(
                "abc", "a
b", ANY
            )
            .is_empty()
        );
    }
}
