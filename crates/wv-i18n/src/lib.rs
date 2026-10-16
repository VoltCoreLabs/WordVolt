//! UI strings (English + Russian). Add a `Key` variant and both translations.

pub const APP_NAME: &str = "WordVolt";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    En,
    #[default]
    Ru,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    UntitledDocument,
}

pub fn t(lang: Lang, key: Key) -> &'static str {
    match (lang, key) {
        (Lang::En, Key::UntitledDocument) => "New document",
        (Lang::Ru, Key::UntitledDocument) => "Новый документ",
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
}
