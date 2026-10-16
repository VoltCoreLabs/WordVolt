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
    TipBold,
    TipItalic,
    TipUnderline,
    TipStrike,
    TipSuper,
    TipSub,
    TipCase,
    TipCode,
    TipSpoiler,
    TipLink,
    TipTextColor,
    TipHighlight,
    TipAlignLeft,
    TipAlignCenter,
    TipAlignRight,
    TipAlignJustify,
    TipQuote,
    TipBullets,
    TipNumbers,
    TipUndo,
    TipRedo,
    TipNew,
    TipOpen,
    TipSizeDown,
    TipSizeUp,
    TipPanel,
    TipRibbon,
    CaseUpper,
    CaseLower,
    CaseTitle,
    CaseSentence,
    CaseDisplay,
    TipIndent,
    TipOutdent,
    TipLineSpacing,
    GroupStyles,
    StyleNormal,
    StyleTitle,
    StyleSubtitle,
    StyleHeading1,
    StyleHeading2,
    StyleHeading3,
    FindPlaceholder,
    ReplacePlaceholder,
    ReplaceOne,
    ReplaceAll,
    NoMatches,
    TipMatchCase,
    TipWholeWord,
    TipFindPrev,
    TipFindNext,
    TipFindClose,
    GroupEditing,
    TipFind,
    TipReplace,
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
        Key::TipBold => "Жирный (Ctrl+B)",
        Key::TipItalic => "Курсив (Ctrl+I)",
        Key::TipUnderline => "Подчёркнутый (Ctrl+U)",
        Key::TipStrike => "Зачёркнутый (Ctrl+Shift+X)",
        Key::TipSuper => "Верхний индекс",
        Key::TipSub => "Нижний индекс",
        Key::TipCase => "Регистр",
        Key::TipCode => "Код (Ctrl+Shift+M)",
        Key::TipSpoiler => "Спойлер (Ctrl+Shift+P)",
        Key::TipLink => "Ссылка (Ctrl+K)",
        Key::TipTextColor => "Цвет текста",
        Key::TipHighlight => "Цвет фона текста (маркер)",
        Key::TipAlignLeft => "По левому краю",
        Key::TipAlignCenter => "По центру",
        Key::TipAlignRight => "По правому краю",
        Key::TipAlignJustify => "По ширине",
        Key::TipQuote => "Цитата",
        Key::TipBullets => "Маркированный список",
        Key::TipNumbers => "Нумерованный список",
        Key::TipUndo => "Отменить (Ctrl+Z)",
        Key::TipRedo => "Повторить (Ctrl+Y)",
        Key::TipNew => "Новый документ (Ctrl+N)",
        Key::TipOpen => "Открыть (Ctrl+O)",
        Key::TipSizeDown => "Уменьшить размер шрифта",
        Key::TipSizeUp => "Увеличить размер шрифта",
        Key::TipPanel => "Панель стилей",
        Key::TipRibbon => "Свернуть или развернуть ленту",
        Key::CaseUpper => "ВСЕ ПРОПИСНЫЕ",
        Key::CaseLower => "все строчные",
        Key::CaseTitle => "Каждое Слово С Прописной",
        Key::CaseSentence => "Как в предложениях",
        Key::CaseDisplay => "Только показывать прописными",
        Key::TipIndent => "Увеличить отступ (Tab в списке)",
        Key::TipOutdent => "Уменьшить отступ (Shift+Tab)",
        Key::TipLineSpacing => "Межстрочный интервал",
        Key::GroupStyles => "Стили",
        Key::StyleNormal => "Обычный",
        Key::StyleTitle => "Название",
        Key::StyleSubtitle => "Подзаголовок",
        Key::StyleHeading1 => "Заголовок 1",
        Key::StyleHeading2 => "Заголовок 2",
        Key::StyleHeading3 => "Заголовок 3",
        Key::FindPlaceholder => "Найти",
        Key::ReplacePlaceholder => "Заменить на",
        Key::ReplaceOne => "Заменить",
        Key::ReplaceAll => "Заменить все",
        Key::NoMatches => "Нет совпадений",
        Key::TipMatchCase => "С учётом регистра",
        Key::TipWholeWord => "Слово целиком",
        Key::TipFindPrev => "Предыдущее (Shift+Enter, Shift+F3)",
        Key::TipFindNext => "Следующее (Enter, F3)",
        Key::TipFindClose => "Закрыть (Esc)",
        Key::GroupEditing => "Правка",
        Key::TipFind => "Найти (Ctrl+F)",
        Key::TipReplace => "Заменить (Ctrl+H)",
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
        Key::TipBold => "Bold (Ctrl+B)",
        Key::TipItalic => "Italic (Ctrl+I)",
        Key::TipUnderline => "Underline (Ctrl+U)",
        Key::TipStrike => "Strikethrough (Ctrl+Shift+X)",
        Key::TipSuper => "Superscript",
        Key::TipSub => "Subscript",
        Key::TipCase => "Change case",
        Key::TipCode => "Code (Ctrl+Shift+M)",
        Key::TipSpoiler => "Spoiler (Ctrl+Shift+P)",
        Key::TipLink => "Link (Ctrl+K)",
        Key::TipTextColor => "Text colour",
        Key::TipHighlight => "Highlight colour (marker)",
        Key::TipAlignLeft => "Align left",
        Key::TipAlignCenter => "Centre",
        Key::TipAlignRight => "Align right",
        Key::TipAlignJustify => "Justify",
        Key::TipQuote => "Block quote",
        Key::TipBullets => "Bulleted list",
        Key::TipNumbers => "Numbered list",
        Key::TipUndo => "Undo (Ctrl+Z)",
        Key::TipRedo => "Redo (Ctrl+Y)",
        Key::TipNew => "New document (Ctrl+N)",
        Key::TipOpen => "Open (Ctrl+O)",
        Key::TipSizeDown => "Decrease font size",
        Key::TipSizeUp => "Increase font size",
        Key::TipPanel => "Styles panel",
        Key::TipRibbon => "Collapse or expand the ribbon",
        Key::CaseUpper => "UPPERCASE",
        Key::CaseLower => "lowercase",
        Key::CaseTitle => "Capitalize Each Word",
        Key::CaseSentence => "Sentence case",
        Key::CaseDisplay => "Display as capitals only",
        Key::TipIndent => "Increase indent (Tab in a list)",
        Key::TipOutdent => "Decrease indent (Shift+Tab)",
        Key::TipLineSpacing => "Line spacing",
        Key::GroupStyles => "Styles",
        Key::StyleNormal => "Normal",
        Key::StyleTitle => "Title",
        Key::StyleSubtitle => "Subtitle",
        Key::StyleHeading1 => "Heading 1",
        Key::StyleHeading2 => "Heading 2",
        Key::StyleHeading3 => "Heading 3",
        Key::FindPlaceholder => "Find",
        Key::ReplacePlaceholder => "Replace with",
        Key::ReplaceOne => "Replace",
        Key::ReplaceAll => "Replace all",
        Key::NoMatches => "No matches",
        Key::TipMatchCase => "Match case",
        Key::TipWholeWord => "Whole word",
        Key::TipFindPrev => "Previous (Shift+Enter, Shift+F3)",
        Key::TipFindNext => "Next (Enter, F3)",
        Key::TipFindClose => "Close (Esc)",
        Key::GroupEditing => "Editing",
        Key::TipFind => "Find (Ctrl+F)",
        Key::TipReplace => "Replace (Ctrl+H)",
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
