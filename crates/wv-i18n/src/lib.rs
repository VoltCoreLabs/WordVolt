//! UI strings (English + Russian). Add a `Key` variant and BOTH translations.

mod en;
mod keys;
mod ru;

pub use keys::Key;

pub const APP_NAME: &str = "WordVolt";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    En,
    #[default]
    Ru,
}

pub fn t(lang: Lang, key: Key) -> &'static str {
    match lang {
        Lang::Ru => ru::text(key),
        Lang::En => en::text(key),
    }
}

/// "5 слов" / "1 word".
pub fn words(lang: Lang, n: usize) -> String {
    match lang {
        Lang::En => format!("{n} word{}", if n == 1 { "" } else { "s" }),
        Lang::Ru => {
            let form = match (n % 10, n % 100) {
                (1, r) if r != 11 => "слово",
                (2..=4, r) if !(12..=14).contains(&r) => "слова",
                _ => "слов",
            };
            format!("{n} {form}")
        }
    }
}

/// "Документ «name» изменён…" / "“name” has unsaved changes…".
pub fn unsaved_detail(lang: Lang, name: &str) -> String {
    match lang {
        Lang::Ru => format!("Документ «{name}» изменён. Сохранить изменения перед продолжением?"),
        Lang::En => {
            format!("\u{201C}{name}\u{201D} has unsaved changes. Save them before continuing?")
        }
    }
}

/// "3 из 12" / "3 of 12" (current match of all matches).
pub fn match_of(lang: Lang, current: usize, total: usize) -> String {
    match lang {
        Lang::Ru => format!("{current} из {total}"),
        Lang::En => format!("{current} of {total}"),
    }
}

/// "Заменено: 3" / "Replaced 3".
pub fn replaced_count(lang: Lang, n: usize) -> String {
    match lang {
        Lang::Ru => format!("Заменено: {n}"),
        Lang::En => format!("Replaced {n}"),
    }
}

/// "Страница 1 из 3" / "Page 1 of 3".
pub fn page_of(lang: Lang, current: usize, total: usize) -> String {
    match lang {
        Lang::Ru => format!("Страница {current} из {total}"),
        Lang::En => format!("Page {current} of {total}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untitled_document_in_both_languages() {
        assert_eq!(t(Lang::Ru, Key::UntitledDocument), "Новый документ");
        assert_eq!(t(Lang::En, Key::UntitledDocument), "New document");
    }

    #[test]
    fn russian_plurals() {
        let forms: Vec<_> = [0, 1, 2, 5, 11, 12, 21, 22, 25, 111, 112]
            .iter()
            .map(|n| words(Lang::Ru, *n))
            .collect();
        assert_eq!(
            forms,
            [
                "0 слов",
                "1 слово",
                "2 слова",
                "5 слов",
                "11 слов",
                "12 слов",
                "21 слово",
                "22 слова",
                "25 слов",
                "111 слов",
                "112 слов"
            ]
        );
    }

    #[test]
    fn english_plurals_and_pages() {
        assert_eq!(words(Lang::En, 1), "1 word");
        assert_eq!(words(Lang::En, 3), "3 words");
        assert_eq!(page_of(Lang::Ru, 1, 2), "Страница 1 из 2");
    }
}
