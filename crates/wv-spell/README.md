# wv-spell
Spell checking on `spellbook` (Hunspell dictionaries, embedded ru + en).

API: `Speller::{load, empty, add_word, is_correct, suggest, misspelled}`, `words(text)`.
The dictionary is chosen by the script of the word. Results are cached per text.
Not here: grammar, per-paragraph language, UI (see `wv-ui/src/spell.rs`).
