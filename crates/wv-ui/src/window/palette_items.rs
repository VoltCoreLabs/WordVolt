use wv_core::NamedStyle;
use wv_i18n::Key;

use super::menu_bar::Command;

/// What a command of the palette does (run by `MainWindow::run_palette`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Menu(Command),
    Bold,
    Italic,
    Underline,
    Strike,
    Superscript,
    Subscript,
    Code,
    ClearFormat,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignJustify,
    Bullets,
    Numbered,
    IndentMore,
    IndentLess,
    Quote,
    Style(NamedStyle),
    PageBreak,
    BlankPage,
    SectionBreak,
    Footnote,
    PageNumber,
    Image,
    Table,
    UpdateFields,
    TrackChanges,
    AcceptAll,
    RejectAll,
    NextChange,
    PrevChange,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ZoomFit,
    Marks,
    Ruler,
    MultiPage,
    NavPanel,
    StylesPanel,
    CommentsPanel,
    Portrait,
    Landscape,
    Columns(u8),
}

/// A command of the palette: its name, what it does and its shortcut (if it has one).
pub struct Item {
    pub name: Key,
    pub action: Action,
    pub shortcut: &'static str,
}

const fn item(name: Key, action: Action, shortcut: &'static str) -> Item {
    Item {
        name,
        action,
        shortcut,
    }
}

/// Every command the palette finds, in the order they are listed when nothing is typed.
pub const ITEMS: &[Item] = &[
    item(Key::PalNew, Action::Menu(Command::New), "Ctrl+N"),
    item(Key::PalOpen, Action::Menu(Command::Open), "Ctrl+O"),
    item(Key::PalSave, Action::Menu(Command::Save), "Ctrl+S"),
    item(
        Key::PalSaveAs,
        Action::Menu(Command::SaveAs),
        "Ctrl+Shift+S",
    ),
    item(Key::PalExportPdf, Action::Menu(Command::ExportPdf), ""),
    item(Key::PalPrint, Action::Menu(Command::Print), "Ctrl+P"),
    item(Key::PalUndo, Action::Menu(Command::Undo), "Ctrl+Z"),
    item(Key::PalRedo, Action::Menu(Command::Redo), "Ctrl+Y"),
    item(Key::PalFind, Action::Menu(Command::Find), "Ctrl+F"),
    item(Key::PalReplace, Action::Menu(Command::Replace), "Ctrl+H"),
    item(Key::PalBold, Action::Bold, "Ctrl+B"),
    item(Key::PalItalic, Action::Italic, "Ctrl+I"),
    item(Key::PalUnderline, Action::Underline, "Ctrl+U"),
    item(Key::PalStrike, Action::Strike, ""),
    item(Key::PalSuperscript, Action::Superscript, ""),
    item(Key::PalSubscript, Action::Subscript, ""),
    item(Key::PalCode, Action::Code, ""),
    item(Key::PalClearFormat, Action::ClearFormat, ""),
    item(Key::PalAlignLeft, Action::AlignLeft, ""),
    item(Key::PalAlignCenter, Action::AlignCenter, ""),
    item(Key::PalAlignRight, Action::AlignRight, ""),
    item(Key::PalAlignJustify, Action::AlignJustify, ""),
    item(Key::PalBullets, Action::Bullets, ""),
    item(Key::PalNumbered, Action::Numbered, ""),
    item(Key::PalIndentMore, Action::IndentMore, ""),
    item(Key::PalIndentLess, Action::IndentLess, ""),
    item(Key::PalQuote, Action::Quote, ""),
    item(Key::StyleNormal, Action::Style(NamedStyle::Normal), ""),
    item(Key::StyleTitle, Action::Style(NamedStyle::Title), ""),
    item(Key::StyleSubtitle, Action::Style(NamedStyle::Subtitle), ""),
    item(Key::StyleHeading1, Action::Style(NamedStyle::Heading1), ""),
    item(Key::StyleHeading2, Action::Style(NamedStyle::Heading2), ""),
    item(Key::StyleHeading3, Action::Style(NamedStyle::Heading3), ""),
    item(Key::PalPageBreak, Action::PageBreak, "Ctrl+Enter"),
    item(Key::PalBlankPage, Action::BlankPage, ""),
    item(Key::SectionBreak, Action::SectionBreak, ""),
    item(Key::PalFootnote, Action::Footnote, ""),
    item(Key::PalPageNumber, Action::PageNumber, ""),
    item(Key::PalImage, Action::Image, ""),
    item(Key::PalTable, Action::Table, ""),
    item(Key::PalUpdateFields, Action::UpdateFields, ""),
    item(Key::TrackChanges, Action::TrackChanges, ""),
    item(Key::AcceptAll, Action::AcceptAll, ""),
    item(Key::RejectAll, Action::RejectAll, ""),
    item(Key::TipNextChange, Action::NextChange, ""),
    item(Key::TipPrevChange, Action::PrevChange, ""),
    item(Key::PalZoomIn, Action::ZoomIn, ""),
    item(Key::PalZoomOut, Action::ZoomOut, ""),
    item(Key::PalZoomReset, Action::ZoomReset, ""),
    item(Key::ZoomFit, Action::ZoomFit, ""),
    item(Key::PalMarks, Action::Marks, ""),
    item(Key::Ruler, Action::Ruler, ""),
    item(Key::PalMultiPage, Action::MultiPage, ""),
    item(Key::PalNavPanel, Action::NavPanel, ""),
    item(Key::PalStylesPanel, Action::StylesPanel, ""),
    item(Key::PalCommentsPanel, Action::CommentsPanel, ""),
    item(Key::PalPortrait, Action::Portrait, ""),
    item(Key::PalLandscape, Action::Landscape, ""),
    item(Key::PalOneColumn, Action::Columns(1), ""),
    item(Key::PalTwoColumns, Action::Columns(2), ""),
    item(Key::PalThreeColumns, Action::Columns(3), ""),
    item(Key::MenuAbout, Action::Menu(Command::About), ""),
];

/// Commands whose name matches `query` (the letters in order, any case), best first; all of them
/// for an empty query. `name` gives the text of an item's name.
pub fn matching(query: &str, name: &dyn Fn(Key) -> String) -> Vec<&'static Item> {
    let query: Vec<char> = query
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let mut scored: Vec<(usize, i32, &Item)> = ITEMS
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            score(&query, &name(item.name).to_lowercase()).map(|s| (i, s, item))
        })
        .collect();
    scored.sort_by_key(|(i, s, _)| (-s, *i));
    scored.into_iter().map(|(_, _, item)| item).collect()
}

/// `None` when `query` is not in `text` as a subsequence; otherwise a higher score for matches
/// that start a word and run together.
fn score(query: &[char], text: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    let chars: Vec<char> = text.chars().collect();
    let (mut at, mut score, mut last) = (0, 0, None::<usize>);
    for q in query {
        let found = (at..chars.len()).find(|i| chars[*i] == *q)?;
        let word_start = found == 0 || !chars[found - 1].is_alphanumeric();
        score += 1
            + if word_start { 3 } else { 0 }
            + if last == Some(found.wrapping_sub(1)) {
                2
            } else {
                0
            };
        last = Some(found);
        at = found + 1;
    }
    Some(score)
}

#[cfg(test)]
mod tests {
    use super::{Action, matching, score};
    use wv_i18n::{Key, Lang, t};

    fn names(lang: Lang) -> impl Fn(Key) -> String {
        move |k| t(lang, k).to_string()
    }

    #[test]
    fn letters_must_come_in_order_and_word_starts_win() {
        let q = |s: &str| s.chars().collect::<Vec<_>>();
        assert_eq!(score(&q("zz"), "abc"), None);
        assert!(score(&q("sv"), "save as").unwrap() > score(&q("sv"), "xsxv").unwrap());
        assert_eq!(score(&[], "anything"), Some(0));
    }

    #[test]
    fn the_palette_finds_commands_by_part_of_the_name() {
        let found = matching("bol", &names(Lang::En));
        assert_eq!(found[0].action, Action::Bold);
        let found = matching("жирн", &names(Lang::Ru));
        assert!(found.iter().any(|i| i.action == Action::Bold));
        assert_eq!(matching("", &names(Lang::En)).len(), super::ITEMS.len());
        assert!(matching("qqqq", &names(Lang::En)).is_empty());
    }
}
