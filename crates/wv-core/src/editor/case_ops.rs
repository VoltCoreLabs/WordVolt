use unicode_segmentation::UnicodeSegmentation;

use super::Editor;
use crate::position::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseKind {
    Upper,
    Lower,
    /// Every Word Starts With A Capital.
    Title,
    /// Only the first letter of each sentence is capital.
    Sentence,
}

/// Carries word/sentence context from one run to the next.
struct State {
    word_start: bool,
    sentence_start: bool,
}

fn convert(text: &str, kind: CaseKind, state: &mut State) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match kind {
            CaseKind::Upper => out.extend(c.to_uppercase()),
            CaseKind::Lower => out.extend(c.to_lowercase()),
            CaseKind::Title if c.is_alphanumeric() => {
                if state.word_start {
                    out.extend(c.to_uppercase());
                } else {
                    out.extend(c.to_lowercase());
                }
                state.word_start = false;
            }
            CaseKind::Title => {
                out.push(c);
                state.word_start = !matches!(c, '\'' | '\u{2019}');
            }
            CaseKind::Sentence if c.is_alphabetic() => {
                if state.sentence_start {
                    out.extend(c.to_uppercase());
                } else {
                    out.extend(c.to_lowercase());
                }
                state.sentence_start = false;
            }
            CaseKind::Sentence => {
                out.push(c);
                if matches!(c, '.' | '!' | '?' | '\u{2026}') {
                    state.sentence_start = true;
                }
            }
        }
    }
    out
}

impl Editor {
    /// Changes the case of the selected text (styles are kept). With a caret it changes the word around it.
    pub fn change_case(&mut self, kind: CaseKind) {
        let (mut start, mut end) = self.sel.range();
        if start == end {
            let text = self.doc.paragraphs[start.para].text();
            let word = text.split_word_bound_indices().find(|(i, w)| {
                *i <= start.offset
                    && start.offset <= i + w.len()
                    && w.chars().any(char::is_alphanumeric)
            });
            let Some((i, w)) = word else { return };
            (start, end) = (
                Position::new(start.para, i),
                Position::new(start.para, i + w.len()),
            );
        }
        let mut paras = self.doc.paragraphs[start.para..=end.para].to_vec();
        let old_last_len = paras.last().map_or(0, |p| p.len());
        let last = paras.len() - 1;
        let mut state = State {
            word_start: true,
            sentence_start: true,
        };
        for (i, p) in paras.iter_mut().enumerate() {
            let from = if i == 0 { start.offset } else { 0 };
            let to = if i == last { end.offset } else { p.len() };
            state.word_start = true;
            p.map_text(from, to, &mut |t| convert(t, kind, &mut state));
        }
        // Keep the selection around the same text even if a letter changed its length (e.g. ß -> SS).
        let delta = paras[last].len() as isize - old_last_len as isize;
        let mut sel = self.sel;
        if sel.head != sel.anchor {
            let moved = if sel.head >= sel.anchor {
                &mut sel.head
            } else {
                &mut sel.anchor
            };
            moved.offset = (moved.offset as isize + delta).max(0) as usize;
        }
        self.edit_paragraphs(start.para, last + 1, paras, sel);
    }
}
