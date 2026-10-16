//! UI strings (English + Russian). Add a `Key` variant and BOTH translations.

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
    TabHome,
    TabInsert,
    TabLayout,
    TabView,
    GroupFont,
    GroupParagraph,
    GroupHistory,
    GroupFile,
    GroupObjects,
    GroupPage,
    GroupZoom,
    Save,
    Styles,
    StylesHint,
    UnsavedTitle,
    DontSave,
    Cancel,
    ErrorTitle,
    FormatWvd,
    FormatDocx,
    FormatTxt,
    FormatMd,
    AllSupported,
    GroupStyle,
    NoColor,
    RecentColors,
    CustomColor,
    LinkAddress,
    RemoveLink,
}

pub fn t(lang: Lang, key: Key) -> &'static str {
    match lang {
        Lang::Ru => ru(key),
        Lang::En => en(key),
    }
}

fn ru(key: Key) -> &'static str {
    match key {
        Key::UntitledDocument => "Новый документ",
        Key::TabHome => "Главная",
        Key::TabInsert => "Вставка",
        Key::TabLayout => "Макет",
        Key::TabView => "Вид",
        Key::GroupFont => "Шрифт",
        Key::GroupParagraph => "Абзац",
        Key::GroupHistory => "История",
        Key::GroupFile => "Файл",
        Key::GroupObjects => "Объекты",
        Key::GroupPage => "Страница",
        Key::GroupZoom => "Масштаб",
        Key::Save => "Сохранить",
        Key::Styles => "Стили",
        Key::StylesHint => "Заголовок 1 · Заголовок 2 · Обычный",
        Key::UnsavedTitle => "Сохранить изменения?",
        Key::DontSave => "Не сохранять",
        Key::Cancel => "Отмена",
        Key::ErrorTitle => "Не удалось выполнить операцию",
        Key::FormatWvd => "Документ WordVolt (.wvd)",
        Key::FormatDocx => "Документ Word (.docx)",
        Key::FormatTxt => "Текст (.txt)",
        Key::FormatMd => "Markdown (.md)",
        Key::AllSupported => "Все поддерживаемые документы",
        Key::GroupStyle => "Начертание",
        Key::NoColor => "Без цвета",
        Key::RecentColors => "Последние",
        Key::CustomColor => "Свой цвет",
        Key::LinkAddress => "Адрес ссылки",
        Key::RemoveLink => "Убрать ссылку",
    }
}

fn en(key: Key) -> &'static str {
    match key {
        Key::UntitledDocument => "New document",
        Key::TabHome => "Home",
        Key::TabInsert => "Insert",
        Key::TabLayout => "Layout",
        Key::TabView => "View",
        Key::GroupFont => "Font",
        Key::GroupParagraph => "Paragraph",
        Key::GroupHistory => "History",
        Key::GroupFile => "File",
        Key::GroupObjects => "Objects",
        Key::GroupPage => "Page",
        Key::GroupZoom => "Zoom",
        Key::Save => "Save",
        Key::Styles => "Styles",
        Key::StylesHint => "Heading 1 · Heading 2 · Normal",
        Key::UnsavedTitle => "Save changes?",
        Key::DontSave => "Don't save",
        Key::Cancel => "Cancel",
        Key::ErrorTitle => "The operation failed",
        Key::FormatWvd => "WordVolt document (.wvd)",
        Key::FormatDocx => "Word document (.docx)",
        Key::FormatTxt => "Plain text (.txt)",
        Key::FormatMd => "Markdown (.md)",
        Key::AllSupported => "All supported documents",
        Key::GroupStyle => "Style",
        Key::NoColor => "No colour",
        Key::RecentColors => "Recent",
        Key::CustomColor => "Custom colour",
        Key::LinkAddress => "Link address",
        Key::RemoveLink => "Remove link",
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
