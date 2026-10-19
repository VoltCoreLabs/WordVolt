use super::Document;
use crate::position::Position;

/// Counts for the statistics dialog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextStats {
    pub words: usize,
    /// Characters (graphemes are not merged: one per `char`), spaces included.
    pub chars: usize,
    pub chars_no_spaces: usize,
    /// Paragraphs that hold some text.
    pub paragraphs: usize,
}

impl TextStats {
    /// Counts the lines of `text` (paragraphs are separated by `\n`).
    pub fn of(text: &str) -> Self {
        text.split('\n').fold(Self::default(), |mut s, para| {
            s.words += para.split_whitespace().count();
            s.chars += para.chars().count();
            s.chars_no_spaces += para.chars().filter(|c| !c.is_whitespace()).count();
            s.paragraphs += usize::from(!para.trim().is_empty());
            s
        })
    }
}

impl Document {
    /// Counts of the whole document.
    pub fn stats(&self) -> TextStats {
        TextStats::of(&self.text())
    }

    /// Counts of the text between two positions.
    pub fn stats_in(&self, start: Position, end: Position) -> TextStats {
        let (s, e) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        let text = (s.para..=e.para)
            .map(|i| {
                let t = self.paragraphs[i].text();
                let from = if i == s.para { s.offset } else { 0 };
                let to = if i == e.para { e.offset } else { t.len() };
                t.get(from..to).unwrap_or_default().to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");
        TextStats::of(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::TextStats;
    use crate::document::Document;
    use crate::position::Position;

    #[test]
    fn counts_words_characters_and_paragraphs() {
        let s = TextStats::of("Привет, мир\n\nвторой абзац 😀");
        assert_eq!(s.words, 5);
        assert_eq!(s.paragraphs, 2);
        assert_eq!(
            s.chars,
            "Привет, мир".chars().count() + "второй абзац 😀".chars().count()
        );
        assert_eq!(s.chars_no_spaces, s.chars - 3);
        assert_eq!(TextStats::of(""), TextStats::default());
    }

    #[test]
    fn counts_a_range_of_the_document() {
        let doc = Document::from_text("раз два\nтри четыре\nпять");
        assert_eq!(doc.stats().words, 5);
        let part = doc.stats_in(
            Position::new(0, "раз ".len()),
            Position::new(1, "три".len()),
        );
        assert_eq!((part.words, part.paragraphs), (2, 2));
        let same = doc.stats_in(Position::new(1, 0), Position::new(1, 0));
        assert_eq!(same, TextStats::default());
    }
}
