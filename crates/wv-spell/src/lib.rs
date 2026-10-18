//! Spell checking with Hunspell dictionaries (Russian and English, bundled), picked by the script
//! of each word, plus a personal word list. No UI here.

use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::Range;
use std::sync::{Arc, Mutex, RwLock};

use spellbook::Dictionary;

const RU_AFF: &str = include_str!("../../../assets/dictionaries/ru/index.aff");
const RU_DIC: &str = include_str!("../../../assets/dictionaries/ru/index.dic");
const EN_AFF: &str = include_str!("../../../assets/dictionaries/en/index.aff");
const EN_DIC: &str = include_str!("../../../assets/dictionaries/en/index.dic");

/// Results are remembered per paragraph text; the memory is cleared when it gets this big.
const CACHE_LIMIT: usize = 50_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Script {
    Cyrillic,
    Latin,
}

/// The spell checker: two dictionaries and the words the user added.
pub struct Speller {
    ru: Option<Dictionary>,
    en: Option<Dictionary>,
    /// Lower-case words added by the user (or skipped for this session).
    user: RwLock<HashSet<String>>,
    /// Misspelled ranges of paragraph texts already checked (keyed by a hash of the text).
    cache: Mutex<HashMap<u64, Arc<Vec<Range<usize>>>>>,
}

impl Speller {
    /// Loads the bundled dictionaries (takes a moment: do it off the UI thread).
    pub fn load() -> Self {
        Self {
            ru: Dictionary::new(RU_AFF, RU_DIC).ok(),
            en: Dictionary::new(EN_AFF, EN_DIC).ok(),
            user: RwLock::default(),
            cache: Mutex::default(),
        }
    }

    /// A checker without dictionaries (everything is correct); for tests and as a fallback.
    pub fn empty() -> Self {
        Self {
            ru: None,
            en: None,
            user: RwLock::default(),
            cache: Mutex::default(),
        }
    }

    /// Adds a word to the personal list.
    pub fn add_word(&self, word: &str) {
        if let Ok(mut user) = self.user.write() {
            user.insert(word.to_lowercase());
        }
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Is `word` (letters only, one language) spelled right?
    pub fn is_correct(&self, word: &str) -> bool {
        let Some(script) = script_of(word) else {
            return true;
        };
        if self
            .user
            .read()
            .is_ok_and(|user| user.contains(&word.to_lowercase()))
        {
            return true;
        }
        match script {
            Script::Cyrillic => self.ru.as_ref().is_none_or(|d| accepted(d, word)),
            Script::Latin => self.en.as_ref().is_none_or(|d| accepted(d, word)),
        }
    }

    /// Up to `limit` corrections for a misspelled word.
    pub fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        let dict = match script_of(word) {
            Some(Script::Cyrillic) => self.ru.as_ref(),
            Some(Script::Latin) => self.en.as_ref(),
            None => None,
        };
        let mut out = Vec::new();
        if let Some(dict) = dict {
            dict.suggest(word, &mut out);
        }
        out.truncate(limit);
        out
    }

    /// Byte ranges of the misspelled words of `text`.
    pub fn misspelled(&self, text: &str) -> Arc<Vec<Range<usize>>> {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let key = hasher.finish();
        if let Some(found) = self.cache.lock().ok().and_then(|c| c.get(&key).cloned()) {
            return found;
        }
        let found: Vec<Range<usize>> = words(text)
            .into_iter()
            .filter(|range| !self.is_correct(&text[range.clone()]))
            .collect();
        let found = Arc::new(found);
        if let Ok(mut cache) = self.cache.lock() {
            if cache.len() >= CACHE_LIMIT {
                cache.clear();
            }
            cache.insert(key, found.clone());
        }
        found
    }
}

/// Hunspell accepts the word, also with the first letter capital (start of a sentence) and, for
/// short capitals, as an abbreviation.
fn accepted(dict: &Dictionary, word: &str) -> bool {
    dict.check(word)
}

/// The script all letters of the word are in; `None` for mixed or other scripts.
fn script_of(word: &str) -> Option<Script> {
    let mut found = None;
    for c in word.chars().filter(|c| c.is_alphabetic()) {
        let script = match c {
            '\u{0400}'..='\u{04FF}' => Script::Cyrillic,
            'a'..='z' | 'A'..='Z' => Script::Latin,
            _ => return None,
        };
        match found {
            None => found = Some(script),
            Some(s) if s == script => {}
            Some(_) => return None,
        }
    }
    found
}

/// Ranges of the words worth checking: letters (with `'`, `’` and `-` inside), no digits, and not
/// parts of web addresses or e-mail addresses.
pub fn words(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    // Whitespace-separated chunks: a chunk that looks like an address is skipped as a whole.
    let mut chunk_start = None;
    let mut push_chunk = |from: usize, to: usize| {
        let chunk = &text[from..to];
        if chunk.contains("://") || chunk.contains('@') || chunk.starts_with("www.") {
            return;
        }
        let mut word_start: Option<usize> = None;
        let flush = |start: usize, end: usize, out: &mut Vec<Range<usize>>| {
            let word = &text[start..end];
            let trimmed = word.trim_matches(['-', '\'', '’']);
            if trimmed.is_empty() || word.chars().any(|c| c.is_ascii_digit()) {
                return;
            }
            let lead = word.len() - word.trim_start_matches(['-', '\'', '’']).len();
            let (s, e) = (start + lead, start + lead + trimmed.len());
            // Single letters and all-capital short words (abbreviations) are not checked.
            let letters = trimmed.chars().count();
            let caps = trimmed.chars().all(|c| !c.is_lowercase());
            if letters >= 2 && !(caps && letters <= 5) {
                out.push(s..e);
            }
        };
        for (i, c) in chunk.char_indices() {
            let at = from + i;
            if c.is_alphanumeric() || matches!(c, '\'' | '’' | '-') {
                word_start.get_or_insert(at);
            } else if let Some(start) = word_start.take() {
                flush(start, at, &mut out);
            }
        }
        if let Some(start) = word_start {
            flush(start, to, &mut out);
        }
    };
    for (i, c) in text.char_indices() {
        if c.is_whitespace() {
            if let Some(start) = chunk_start.take() {
                push_chunk(start, i);
            }
        } else if chunk_start.is_none() {
            chunk_start = Some(i);
        }
    }
    if let Some(start) = chunk_start {
        push_chunk(start, text.len());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slices<'a>(text: &'a str, ranges: &[Range<usize>]) -> Vec<&'a str> {
        ranges.iter().map(|r| &text[r.clone()]).collect()
    }

    #[test]
    fn words_skip_numbers_addresses_and_single_letters() {
        let text =
            "Привет, мир! Это a test 123 abc1 http://site.ru/x me@mail.ru NASA (don't) -dash- я";
        let found = words(text);
        assert_eq!(
            slices(text, &found),
            ["Привет", "мир", "Это", "test", "don't", "dash"]
        );
    }

    #[test]
    fn script_picks_the_dictionary_and_mixed_words_are_left_alone() {
        assert_eq!(script_of("слово"), Some(Script::Cyrillic));
        assert_eq!(script_of("word"), Some(Script::Latin));
        assert_eq!(script_of("wordслово"), None);
        assert_eq!(script_of("日本"), None);
    }

    #[test]
    fn russian_and_english_words_are_checked_and_corrected() {
        let s = Speller::load();
        assert!(s.is_correct("документ") && s.is_correct("Документы"));
        assert!(!s.is_correct("докумнет"));
        assert!(s.is_correct("document") && !s.is_correct("documnet"));
        assert!(s.suggest("докумнет", 5).iter().any(|w| w == "документ"));
        let text = "Это докумнет и documnet тут";
        let bad = s.misspelled(text);
        assert_eq!(slices(text, &bad), ["докумнет", "documnet"]);
    }

    #[test]
    fn a_word_added_by_the_user_is_accepted() {
        let s = Speller::load();
        assert!(!s.is_correct("вольтворд"));
        let text = "вольтворд";
        assert_eq!(s.misspelled(text).len(), 1);
        s.add_word("ВольтВорд");
        assert!(s.is_correct("вольтворд"));
        assert!(s.misspelled(text).is_empty(), "the memory was cleared");
    }
}
