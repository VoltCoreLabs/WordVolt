use super::Editor;
use crate::document::Document;
use crate::position::{Position, Selection};
use crate::style::Align;

fn typed(text: &str) -> Editor {
    let mut e = Editor::default();
    e.insert_text(text);
    e
}

#[test]
fn typing_and_enter() {
    let mut e = typed("Hello");
    e.enter();
    e.insert_text("мир");
    assert_eq!(e.doc.text(), "Hello\nмир");
    assert_eq!(e.sel.head, Position::new(1, "мир".len()));
}

#[test]
fn insert_multiline_text() {
    let e = typed("a\nb\nc");
    assert_eq!(e.doc.paragraphs.len(), 3);
    assert_eq!(e.sel.head, Position::new(2, 1));
}

#[test]
fn backspace_merges_paragraphs() {
    let mut e = typed("ab\ncd");
    e.sel = crate::Selection::caret(Position::new(1, 0));
    e.backspace();
    assert_eq!(e.doc.text(), "abcd");
    assert_eq!(e.sel.head, Position::new(0, 2));
}

#[test]
fn backspace_removes_whole_grapheme() {
    let mut e = typed("a👨‍👩‍👧b");
    e.move_left(false);
    e.backspace();
    assert_eq!(e.doc.text(), "ab");
}

#[test]
fn backspace_cyrillic() {
    let mut e = typed("Привет");
    e.backspace();
    assert_eq!(e.doc.text(), "Приве");
}

#[test]
fn delete_forward_at_end_merges_next() {
    let mut e = typed("ab\ncd");
    e.sel = crate::Selection::caret(Position::new(0, 2));
    e.delete_forward();
    assert_eq!(e.doc.text(), "abcd");
}

#[test]
fn delete_selection_across_paragraphs() {
    let mut e = typed("one\ntwo\nthree");
    e.sel.anchor = Position::new(0, 1);
    e.sel.head = Position::new(2, 2);
    assert_eq!(e.selected_text(), "ne\ntwo\nth");
    e.insert_text("X");
    assert_eq!(e.doc.text(), "oXree");
}

#[test]
fn typing_replaces_backwards_selection() {
    let mut e = typed("hello");
    e.sel.anchor = Position::new(0, 4);
    e.sel.head = Position::new(0, 1);
    e.insert_text("a");
    assert_eq!(e.doc.text(), "hao");
}

#[test]
fn undo_redo_roundtrip() {
    let mut e = typed("abc");
    e.enter();
    e.insert_text("d");
    let after = e.doc.clone();
    assert!(e.undo());
    assert!(e.undo());
    assert_eq!(e.doc.text(), "abc");
    assert!(e.undo());
    assert_eq!(e.doc.text(), "");
    assert!(!e.undo());
    e.redo();
    e.redo();
    e.redo();
    assert_eq!(e.doc, after);
    assert!(!e.redo());
}

#[test]
fn new_edit_clears_redo() {
    let mut e = typed("a");
    e.undo();
    assert!(e.can_redo());
    e.insert_text("b");
    assert!(!e.can_redo());
}

#[test]
fn bold_on_selection_and_toggle_back() {
    let mut e = typed("hello world");
    e.sel.anchor = Position::new(0, 0);
    e.sel.head = Position::new(0, 5);
    e.toggle_bold();
    let p = &e.doc.paragraphs[0];
    assert_eq!(p.runs.len(), 2);
    assert!(p.runs[0].style.bold && !p.runs[1].style.bold);
    e.toggle_bold();
    assert_eq!(e.doc.paragraphs[0].runs.len(), 1);
    e.undo();
    assert_eq!(e.doc.paragraphs[0].runs.len(), 2);
}

#[test]
fn bold_with_caret_applies_to_next_text() {
    let mut e = typed("a");
    e.toggle_bold();
    e.insert_text("b");
    e.toggle_bold();
    e.insert_text("c");
    let runs = &e.doc.paragraphs[0].runs;
    assert_eq!(runs.len(), 3);
    assert!(!runs[0].style.bold && runs[1].style.bold && !runs[2].style.bold);
}

#[test]
fn typing_inherits_style_before_caret() {
    let mut e = typed("ab");
    e.select_all();
    e.toggle_italic();
    e.move_doc_end(false);
    e.insert_text("c");
    assert_eq!(e.doc.paragraphs[0].runs.len(), 1);
    assert!(e.doc.paragraphs[0].runs[0].style.italic);
}

#[test]
fn align_applies_to_selected_paragraphs_and_enter_inherits() {
    let mut e = typed("a\nb");
    e.select_all();
    e.set_align(Align::Center);
    assert!(
        e.doc
            .paragraphs
            .iter()
            .all(|p| p.style.align == Align::Center)
    );
    e.move_doc_end(false);
    e.enter();
    assert_eq!(e.align(), Align::Center);
}

#[test]
fn caret_movement_crosses_paragraphs() {
    let mut e = Editor::new(Document::from_text("ab\ncd"));
    e.move_doc_end(false);
    e.move_left(false);
    e.move_left(false);
    assert_eq!(e.sel.head, Position::new(1, 0));
    e.move_left(false);
    assert_eq!(e.sel.head, Position::new(0, 2));
    e.move_line_start(true);
    assert_eq!(e.selected_text(), "ab");
}

#[test]
fn word_count() {
    assert_eq!(Document::from_text("one two\nthree").word_count(), 3);
}

#[test]
fn double_click_selects_word_and_triple_selects_paragraph() {
    let mut e = Editor::new(Document::from_text("hello big world"));
    e.select_word(Position::new(0, 7));
    assert_eq!(e.selected_text(), "big");
    e.select_paragraph(0);
    assert_eq!(e.selected_text(), "hello big world");
}

#[test]
fn revision_counts_changes_and_current_style_reflects_selection() {
    let mut e = typed("ab");
    assert_eq!(e.revision(), 1);
    e.select_all();
    e.toggle_bold();
    assert!(e.current_style().bold);
    e.undo();
    assert!(!e.current_style().bold);
    assert_eq!(e.revision(), 3);
}

#[test]
fn font_size_on_selection_caret_and_bump() {
    let mut e = typed("hello world");
    e.sel.anchor = Position::new(0, 0);
    e.sel.head = Position::new(0, 5);
    e.set_size_pt(20);
    assert_eq!(e.current_style().size_pt, Some(20));
    assert_eq!(e.doc.paragraphs[0].runs.len(), 2);
    e.bump_size(1);
    assert_eq!(e.current_style().size_pt, Some(22));
    e.bump_size(-1);
    e.bump_size(-1);
    assert_eq!(e.current_style().size_pt, Some(18));
    e.undo();
    e.undo();
    e.undo();
    e.undo();
    assert_eq!(e.doc.paragraphs[0].runs.len(), 1);
    e.sel = crate::Selection::caret(Position::new(0, 11));
    e.set_size_pt(30);
    e.insert_text("!");
    assert_eq!(
        e.doc.paragraphs[0].runs.last().unwrap().style.size_pt,
        Some(30)
    );
}

#[test]
fn bump_size_stops_at_the_ends() {
    let mut e = typed("x");
    e.select_all();
    e.set_size_pt(8);
    e.bump_size(-1);
    assert_eq!(e.current_style().size_pt, Some(8));
    e.set_size_pt(288);
    e.bump_size(1);
    assert_eq!(e.current_style().size_pt, Some(288));
}

#[test]
fn font_family_on_selection_caret_and_default() {
    let mut e = typed("hello world");
    e.sel.anchor = Position::new(0, 0);
    e.sel.head = Position::new(0, 5);
    e.set_font_family(Some("Georgia".into()));
    assert_eq!(e.current_style().font.as_deref(), Some("Georgia"));
    assert_eq!(e.doc.paragraphs[0].runs.len(), 2);
    e.select_all();
    assert_eq!(
        e.current_style().font,
        None,
        "mixed selection has no single font"
    );
    e.set_font_family(None);
    assert_eq!(e.doc.paragraphs[0].runs.len(), 1);
    e.sel = crate::Selection::caret(Position::new(0, 11));
    e.set_font_family(Some("Arial".into()));
    e.insert_text("!");
    assert_eq!(
        e.doc.paragraphs[0]
            .runs
            .last()
            .unwrap()
            .style
            .font
            .as_deref(),
        Some("Arial")
    );
}

#[test]
fn strike_colour_highlight_and_script() {
    use crate::Script;
    let mut e = typed("hello world");
    e.sel.anchor = Position::new(0, 0);
    e.sel.head = Position::new(0, 5);
    e.toggle_strike();
    e.set_color(Some(0xFF0000));
    e.set_highlight(Some(0xFFFF00));
    e.toggle_script(Script::Super);
    let s = e.current_style();
    assert!(
        s.strike
            && s.color == Some(0xFF0000)
            && s.highlight == Some(0xFFFF00)
            && s.script == Script::Super
    );
    e.toggle_script(Script::Super);
    assert_eq!(e.current_style().script, Script::Normal);
    e.toggle_script(Script::Sub);
    e.toggle_script(Script::Super);
    assert_eq!(
        e.current_style().script,
        Script::Super,
        "super replaces sub"
    );
    e.toggle_strike();
    assert!(!e.current_style().strike);
    e.set_color(None);
    e.set_highlight(None);
    assert_eq!(e.current_style().color, None);
    assert_eq!(e.doc.paragraphs[0].runs.len(), 2);
    e.select_all();
    assert_eq!(
        e.current_style().highlight,
        None,
        "mixed selection reports none"
    );
}

#[test]
fn caps_mono_spoiler_link_and_quote() {
    let mut e = typed("see docs here");
    e.sel.anchor = Position::new(0, 4);
    e.sel.head = Position::new(0, 8);
    e.toggle_caps();
    e.toggle_mono();
    e.toggle_spoiler();
    e.set_link(Some("https://example.com".into()));
    let s = e.current_style();
    assert!(s.caps && s.mono && s.spoiler && s.link.as_deref() == Some("https://example.com"));
    e.toggle_mono();
    assert!(!e.current_style().mono);
    e.set_link(None);
    assert_eq!(e.current_style().link, None);
    assert!(!e.quote_active());
    e.toggle_quote();
    assert!(e.quote_active());
    e.toggle_quote();
    assert!(!e.quote_active());
}

#[test]
fn typing_after_a_link_does_not_extend_it_but_typing_inside_does() {
    let mut e = typed("abc");
    e.select_all();
    e.set_link(Some("u".into()));
    e.sel = crate::Selection::caret(Position::new(0, 3));
    e.insert_text("!");
    assert_eq!(e.doc.paragraphs[0].runs.last().unwrap().style.link, None);
    e.sel = crate::Selection::caret(Position::new(0, 1));
    e.insert_text("X");
    assert_eq!(e.doc.paragraphs[0].runs[0].text, "aXbc");
    assert_eq!(e.doc.paragraphs[0].runs[0].style.link.as_deref(), Some("u"));
}

#[test]
fn clear_formatting_resets_everything() {
    let mut e = typed("hello");
    e.select_all();
    e.toggle_bold();
    e.set_color(Some(0xFF0000));
    e.set_link(Some("u".into()));
    e.clear_formatting();
    assert_eq!(e.current_style(), crate::CharStyle::default());
    assert_eq!(e.doc.paragraphs[0].runs.len(), 1);
}

#[test]
fn change_case_variants_keep_styles() {
    use crate::CaseKind;
    let mut e = typed("hello wORLD. second sentence");
    e.sel.anchor = Position::new(0, 0);
    e.sel.head = Position::new(0, 5);
    e.toggle_bold();
    e.select_all();
    e.change_case(CaseKind::Upper);
    assert_eq!(e.doc.text(), "HELLO WORLD. SECOND SENTENCE");
    assert!(
        e.doc.paragraphs[0].runs[0].style.bold,
        "bold survives the change"
    );
    e.change_case(CaseKind::Lower);
    assert_eq!(e.doc.text(), "hello world. second sentence");
    e.change_case(CaseKind::Title);
    assert_eq!(e.doc.text(), "Hello World. Second Sentence");
    e.change_case(CaseKind::Sentence);
    assert_eq!(e.doc.text(), "Hello world. Second sentence");
    assert_eq!(e.selected_text(), "Hello world. Second sentence");
    assert!(e.undo());
    assert_eq!(e.doc.text(), "Hello World. Second Sentence");
}

#[test]
fn change_case_with_a_caret_affects_the_word_and_handles_cyrillic() {
    use crate::CaseKind;
    let mut e = typed("привет мир. новый день");
    // Byte offsets: Cyrillic letters take two bytes, so 23 is inside "новый".
    e.sel = crate::Selection::caret(Position::new(0, 23));
    e.change_case(CaseKind::Upper);
    assert_eq!(e.doc.text(), "привет мир. НОВЫЙ день");
    e.select_all();
    e.change_case(CaseKind::Sentence);
    assert_eq!(e.doc.text(), "Привет мир. Новый день");
}

#[test]
fn lists_indent_enter_backspace_and_tab() {
    use crate::ListKind;
    let mut e = typed("one\ntwo");
    e.select_all();
    e.toggle_list(ListKind::Bullet);
    assert!(
        e.doc
            .paragraphs
            .iter()
            .all(|p| p.style.list == Some(ListKind::Bullet))
    );
    e.toggle_list(ListKind::Number);
    assert!(
        e.doc
            .paragraphs
            .iter()
            .all(|p| p.style.list == Some(ListKind::Number)),
        "switches kind"
    );
    e.toggle_list(ListKind::Number);
    assert!(
        e.doc.paragraphs.iter().all(|p| p.style.list.is_none()),
        "toggles off"
    );

    e.toggle_list(ListKind::Bullet);
    e.move_doc_end(false);
    e.tab_key(false);
    assert_eq!(e.doc.paragraphs[1].style.indent, 1, "Tab nests a list item");
    e.tab_key(true);
    assert_eq!(e.doc.paragraphs[1].style.indent, 0);
    e.enter();
    assert_eq!(e.doc.paragraphs.len(), 3);
    assert_eq!(
        e.doc.paragraphs[2].style.list,
        Some(ListKind::Bullet),
        "new item inherits the list"
    );
    e.enter();
    assert_eq!(
        e.doc.paragraphs.len(),
        3,
        "Enter on an empty item ends the list"
    );
    assert_eq!(e.doc.paragraphs[2].style.list, None);
    e.sel = crate::Selection::caret(Position::new(0, 0));
    e.backspace();
    assert_eq!(
        e.doc.paragraphs[0].style.list, None,
        "Backspace at the start removes the bullet"
    );
    assert_eq!(e.doc.text(), "one\ntwo\n");
}

#[test]
fn indent_clamps_and_line_spacing_round_trips() {
    let mut e = typed("x");
    for _ in 0..12 {
        e.change_indent(1);
    }
    assert_eq!(e.doc.paragraphs[0].style.indent, 8);
    for _ in 0..12 {
        e.change_indent(-1);
    }
    assert_eq!(e.doc.paragraphs[0].style.indent, 0);
    e.set_line_spacing(Some(1.5));
    assert_eq!(e.line_spacing(), Some(1.5));
    e.undo();
    assert_eq!(e.line_spacing(), None);
}

#[test]
fn named_styles_apply_enter_at_end_returns_to_normal_and_size_bumps_from_the_style() {
    use crate::NamedStyle;
    let mut e = typed("Chapter");
    e.set_named_style(NamedStyle::Heading1);
    assert_eq!(e.named_style(), NamedStyle::Heading1);
    e.enter();
    assert_eq!(
        e.doc.paragraphs[1].style.named,
        NamedStyle::Normal,
        "Enter at the end of a heading"
    );
    e.sel = crate::Selection::caret(Position::new(0, 3));
    e.enter();
    assert_eq!(
        e.doc.paragraphs[1].style.named,
        NamedStyle::Heading1,
        "splitting keeps the style"
    );
    e.sel = crate::Selection::caret(Position::new(0, 1));
    e.bump_size(1);
    assert_eq!(
        e.current_style().size_pt,
        Some(22),
        "bumps from the heading size (20), not from 12"
    );
    let resolved = e.doc.paragraphs[1].with_named_style().unwrap();
    assert!(resolved.runs[0].style.bold && resolved.runs[0].style.size_pt == Some(20));
    assert!(e.doc.paragraphs[2].with_named_style().is_none());
}

#[test]
fn replace_match_keeps_the_formatting_of_the_matched_text() {
    let mut e = typed("one two three");
    e.sel.anchor = Position::new(0, 4);
    e.sel.head = Position::new(0, 7);
    e.toggle_bold();
    let m = e.find_matches("two", crate::SearchOptions::default());
    assert_eq!(m.len(), 1);
    e.replace_match(m[0].0, m[0].1, "2");
    assert_eq!(e.doc.text(), "one 2 three");
    let bold: Vec<_> = e.doc.paragraphs[0]
        .runs
        .iter()
        .filter(|r| r.style.bold)
        .map(|r| r.text.as_str())
        .collect();
    assert_eq!(bold, ["2"]);
}

#[test]
fn replace_all_is_one_undo_step_and_handles_line_breaks_and_unicode() {
    let mut e = typed(
        "Привет мир
привет всем
ничего",
    );
    let before = e.doc.clone();
    let opts = crate::SearchOptions::default();
    assert_eq!(e.replace_all("привет", "hi", opts), 2);
    assert_eq!(
        e.doc.text(),
        "hi мир
hi всем
ничего"
    );
    assert!(e.undo());
    assert_eq!(e.doc, before, "one undo restores everything");
    assert!(e.redo());
    assert_eq!(
        e.replace_all(
            "мир", "a
b", opts
        ),
        1
    );
    assert_eq!(
        e.doc.text(),
        "hi a
b
hi всем
ничего"
    );
    assert_eq!(e.replace_all("нет такого", "x", opts), 0);
}

#[test]
fn tables_insert_navigate_grow_shrink_and_protect_cell_borders() {
    let mut e = typed("intro");
    e.insert_table(2, 3);
    assert_eq!(
        e.doc.paragraphs.len(),
        1 + 6 + 1,
        "intro + 6 cells + a paragraph to type after the table"
    );
    let info = e.table_info().unwrap();
    assert_eq!((info.rows, info.cols, info.row, info.col), (2, 3, 0, 0));
    e.insert_text("A");
    e.tab_key(false);
    e.insert_text("B");
    assert_eq!(e.table_info().map(|i| (i.row, i.col)), Some((0, 1)));
    e.tab_key(true);
    assert_eq!(
        e.selected_text(),
        "A",
        "Shift+Tab goes back and selects the cell text"
    );
    // Backspace at the start of a cell must not merge it with the previous cell or the text above.
    let before = e.doc.clone();
    e.sel = crate::Selection::caret(Position::new(2, 0));
    e.backspace();
    assert_eq!(e.doc, before);
    e.sel = crate::Selection::caret(Position::new(1, 0));
    e.backspace();
    assert_eq!(e.doc, before);
    // Rows and columns.
    e.insert_row(true);
    assert_eq!(
        e.table_info().map(|i| (i.rows, i.cols, i.row)),
        Some((3, 3, 1))
    );
    e.insert_col(false);
    assert_eq!(
        e.table_info().map(|i| (i.rows, i.cols, i.col)),
        Some((3, 4, 0))
    );
    e.delete_col();
    e.delete_row();
    assert_eq!(e.table_info().map(|i| (i.rows, i.cols)), Some((2, 3)));
    // Tab in the last cell adds a row.
    let last = e.table_info().unwrap().last;
    e.sel = crate::Selection::caret(Position::new(last, 0));
    e.tab_key(false);
    assert_eq!(e.table_info().map(|i| i.rows), Some(3));
    e.delete_table();
    assert!(e.doc.paragraphs.iter().all(|p| p.style.cell.is_none()));
    assert_eq!(e.doc.paragraphs[0].text(), "intro");
    assert!(e.undo());
    assert_eq!(
        e.table_info().map(|i| i.rows),
        Some(3),
        "deleting a table is one undo step"
    );
}

#[test]
fn page_number_toggles_in_footer() {
    let mut e = Editor::default();
    e.toggle_page_number();
    assert_eq!(e.doc.footer.text(), "{page}");
    e.set_header_footer(
        crate::Band::new("Head").aligned(crate::Align::Left),
        crate::Band::new("Стр. {page}"),
    );
    e.toggle_page_number();
    assert_eq!(e.doc.footer.text(), "Стр.");
    assert_eq!(e.doc.header.text(), "Head");
    assert_eq!(e.doc.header.align, crate::Align::Left);
    assert!(e.is_modified());
}

#[test]
fn pictures_are_block_paragraphs() {
    use crate::ImageFormat;
    let mut e = Editor::default();
    e.insert_text("intro");
    e.insert_image(ImageFormat::Png, vec![1, 2, 3], 1200.0, 600.0);
    // Inserted after "intro", scaled to the maximum width, caret on the paragraph after it.
    assert_eq!(e.doc.paragraphs.len(), 3);
    let image = e.doc.paragraphs[1].style.image.unwrap();
    assert_eq!((image.width, image.height), (600.0, 300.0));
    assert_eq!(e.sel.head.para, 2);
    // Same bytes are stored once.
    e.insert_image(ImageFormat::Png, vec![1, 2, 3], 10.0, 10.0);
    assert_eq!(e.doc.images.len(), 1);
    // No typing into a picture paragraph; Enter adds a paragraph after it.
    e.sel = crate::Selection::caret(Position::new(1, 0));
    e.insert_text("x");
    assert_eq!(e.doc.paragraphs[1].text(), "");
    let before = e.doc.paragraphs.len();
    e.enter();
    assert_eq!(e.doc.paragraphs.len(), before + 1);
    assert_eq!(e.sel.head, Position::new(2, 0));
    // Backspace on the picture removes it; resizing keeps proportions.
    e.sel = crate::Selection::caret(Position::new(1, 0));
    e.set_image_width(300.0);
    assert_eq!(e.doc.paragraphs[1].style.image.unwrap().height, 150.0);
    e.backspace();
    assert!(e.doc.paragraphs[1].style.image.is_none());
    assert!(e.undo());
    assert!(e.doc.paragraphs[1].style.image.is_some());
}

#[test]
fn header_footer_and_page_setup_are_undoable() {
    use crate::{Band, Margins, PageSpec};
    let mut e = Editor::default();
    e.insert_text("x");
    e.set_header_footer(Band::new("Head"), Band::new("{page}"));
    e.set_page(PageSpec::a4().with_paper(2).with_margins(Margins::Wide));
    assert_eq!(e.doc.header.text(), "Head");
    assert!(e.undo());
    assert_eq!(e.doc.page, PageSpec::a4());
    assert_eq!(e.doc.header.text(), "Head");
    assert!(e.undo());
    assert!(e.doc.header.is_empty() && e.doc.footer.is_empty());
    assert_eq!(e.doc.text(), "x");
    assert!(e.redo() && e.redo());
    assert_eq!(e.doc.page.width, 816.0);
    assert_eq!(e.doc.footer.text(), "{page}");
    // No change = no history entry.
    let depth = e.can_undo();
    e.set_page(e.doc.page);
    assert_eq!(depth, e.can_undo());
}

#[test]
fn table_of_contents_is_inserted_and_replaced_in_one_step() {
    use crate::{NamedStyle, TocEntry};
    let mut e = Editor::default();
    e.insert_text("Intro\nBody");
    e.sel = crate::Selection::caret(Position::new(0, 0));
    e.set_named_style(NamedStyle::Heading1);
    assert_eq!(e.headings().len(), 1);
    let entry = |page| TocEntry {
        level: 1,
        text: "Intro".into(),
        page,
    };
    e.sel = crate::Selection::caret(Position::new(1, 0));
    e.set_toc("Contents", &[entry(1)]);
    let (first, last) = e.toc_range().unwrap();
    assert_eq!((first, last), (2, 3));
    assert!(e.doc.paragraphs[3].text().ends_with("1"));
    e.set_toc("Contents", &[entry(2)]);
    assert_eq!(e.toc_range(), Some((2, 3)));
    assert!(e.doc.paragraphs[3].text().ends_with("2"));
    // The contents never count as headings, and undo takes the last update back.
    assert_eq!(e.headings().len(), 1);
    assert!(e.undo());
    assert!(e.doc.paragraphs[3].text().ends_with("1"));
}

#[test]
fn dragging_a_picture_size_is_one_undo_step() {
    let mut e = Editor::new(Document::default());
    e.insert_image(crate::ImageFormat::Png, vec![1, 2, 3], 200.0, 100.0);
    e.set_head(Position::new(0, 0), false);
    for (i, width) in [210.0, 250.0, 300.0].into_iter().enumerate() {
        e.drag_image_width(width, i == 0);
    }
    let image = e.image_at_caret().unwrap();
    assert_eq!((image.width, image.height), (300.0, 150.0));
    assert!(e.undo());
    assert_eq!(e.image_at_caret().unwrap().width, 200.0);
    assert!(e.redo());
    assert_eq!(e.image_at_caret().unwrap().width, 300.0);
}

#[test]
fn a_picture_never_gets_wider_than_the_text() {
    let mut e = Editor::new(Document::default());
    let text = e.doc.page.width - e.doc.page.margin_left - e.doc.page.margin_right;
    e.insert_image(crate::ImageFormat::Png, vec![1], 4000.0, 2000.0);
    assert!(e.doc.paragraphs[0].style.image.unwrap().width <= text);
    e.set_head(Position::new(0, 0), false);
    e.set_image_width(5000.0);
    assert_eq!(e.image_at_caret().unwrap().width, text);
}

#[test]
fn a_contents_entry_points_to_its_heading() {
    let mut e = Editor::new(Document::from_text("One\nbody\nTwo\nbody\nTwo"));
    for para in [0, 2, 4] {
        e.sel = crate::Selection::caret(Position::new(para, 0));
        e.set_named_style(crate::NamedStyle::Heading1);
    }
    e.sel = crate::Selection::caret(Position::new(1, 0));
    let entries: Vec<_> = e
        .headings()
        .iter()
        .map(|h| crate::TocEntry {
            level: h.level,
            text: h.text.clone(),
            page: 1,
        })
        .collect();
    e.set_toc("Contents", &entries);
    let (first, _) = e.toc_range().unwrap();
    assert_eq!(e.toc_target(first), None, "the title is not an entry");
    // Headings shifted down by the 4 contents paragraphs; the second "Two" has its own entry.
    assert_eq!(e.toc_target(first + 1), Some(0));
    assert_eq!(e.toc_target(first + 2), Some(first + 4));
    assert_eq!(e.toc_target(first + 3), Some(first + 6));
    assert_eq!(e.toc_target(0), None);
}

#[test]
fn band_variants_are_one_undo_step() {
    let mut e = Editor::new(Document::from_text("x"));
    let variants = crate::BandVariants {
        different_first: true,
        first: (crate::Band::new("Title page"), crate::Band::default()),
        ..Default::default()
    };
    e.set_bands(
        crate::Band::new("Main"),
        crate::Band::new("{page}"),
        variants,
    );
    assert_eq!(e.doc.bands_for(None, 0).0.text(), "Title page");
    assert_eq!(e.doc.bands_for(None, 1).0.text(), "Main");
    assert!(e.undo());
    assert!(e.doc.header.is_empty() && !e.doc.variants.different_first);
}

#[test]
fn copy_and_paste_keep_formatting() {
    let mut e = Editor::new(Document::from_text("Привет мир\nвторая"));
    e.sel = Selection {
        anchor: Position::new(0, 0),
        head: Position::new(0, 6),
    };
    e.toggle_bold();
    e.sel = Selection {
        anchor: Position::new(0, 0),
        head: Position::new(1, 8),
    };
    let frag = e.copy_fragment();
    assert_eq!(frag.len(), 2);
    assert!(frag[0].runs[0].style.bold);
    e.sel = Selection::caret(Position::new(1, 12));
    e.paste_fragment(&frag);
    assert_eq!(e.doc.paragraphs.len(), 3);
    assert_eq!(
        e.doc.paragraphs[1].text(),
        "вторая".to_owned() + "Привет мир"
    );
    assert!(e.doc.paragraphs[1].runs.iter().any(|r| r.style.bold));
    assert_eq!(e.doc.paragraphs[2].text(), "втор");
    assert_eq!(e.sel.head, Position::new(2, 8));
    assert!(e.undo());
    assert_eq!(e.doc.paragraphs.len(), 2);
}

#[test]
fn paragraph_dialog_values_round_trip() {
    let mut e = Editor::new(Document::from_text("one\ntwo"));
    let m = crate::ParaMetrics {
        left_pt: 100,
        first_line: 20,
        space_before: Some(6),
        space_after: Some(12),
        line_spacing: Some(1.5),
        tabs: vec![
            crate::TabStop {
                pos: 144,
                kind: crate::TabKind::Right,
            },
            crate::TabStop::left(36),
            crate::TabStop::left(36),
        ],
        right_pt: 18,
    };
    e.set_para_metrics(&m);
    let got = e.para_metrics();
    assert_eq!((got.left_pt, got.first_line), (100, 20));
    assert_eq!(
        got.tabs.iter().map(|t| (t.pos, t.kind)).collect::<Vec<_>>(),
        [(36, crate::TabKind::Left), (144, crate::TabKind::Right)],
        "sorted, no duplicates"
    );
    assert_eq!(got.right_pt, 18);
    assert_eq!(e.doc.paragraphs[0].style.indent, 2);
    assert_eq!(e.doc.paragraphs[0].style.left_extra, 28);
    assert!(e.undo());
    assert_eq!(e.para_metrics(), crate::ParaMetrics::default());
}

#[test]
fn page_break_splits_the_paragraph_and_backspace_takes_it_away() {
    let mut e = Editor::new(Document::from_text("onetwo"));
    e.sel = Selection::caret(Position::new(0, 3));
    e.insert_page_break();
    assert_eq!(e.doc.paragraphs.len(), 2);
    assert!(!e.doc.paragraphs[0].style.page_break);
    assert!(e.doc.paragraphs[1].style.page_break);
    assert_eq!(e.doc.paragraphs[1].text(), "two");
    // Enter in the flagged paragraph does not flag the paragraph it creates.
    e.sel = Selection::caret(Position::new(1, 1));
    e.enter();
    assert!(!e.doc.paragraphs[2].style.page_break);
    e.sel = Selection::caret(Position::new(1, 0));
    e.backspace();
    assert!(!e.doc.paragraphs[1].style.page_break);
    assert_eq!(
        e.doc.paragraphs.len(),
        3,
        "the first backspace only removes the break"
    );
}

#[test]
fn blank_page_has_a_break_before_and_after() {
    let mut e = Editor::new(Document::from_text("ab"));
    e.sel = Selection::caret(Position::new(0, 1));
    e.insert_blank_page();
    let flags: Vec<bool> = e
        .doc
        .paragraphs
        .iter()
        .map(|p| p.style.page_break)
        .collect();
    assert_eq!(flags, [false, true, true]);
    assert_eq!(e.doc.paragraphs[1].text(), "");
}

#[test]
fn ime_composition_is_one_undo_step_and_commits_multibyte_text() {
    let mut e = Editor::new(Document::from_text("ab"));
    e.sel = Selection::caret(Position::new(0, 2));
    e.ime_replace(None, "ni", true);
    assert_eq!(e.doc.paragraphs[0].text(), "abni");
    assert_eq!(e.ime_marked(), Some(2..4));
    // The input method rewrites the whole composition on every keystroke.
    e.ime_replace(None, "nihao", true);
    assert_eq!(e.doc.paragraphs[0].text(), "abnihao");
    assert_eq!(e.ime_marked(), Some(2..7));
    // Committing replaces it with the final text.
    e.ime_replace(None, "你好", false);
    assert_eq!(e.doc.paragraphs[0].text(), "ab你好");
    assert_eq!(e.ime_marked(), None);
    assert_eq!(e.sel.head, Position::new(0, 8));
    // UTF-16 offsets: "你好" are two units.
    assert_eq!(e.ime_text(2..4), ("你好".to_string(), 2..4));
    assert_eq!(e.ime_selection(), (4..4, false));
    // The whole composition is a single undo step.
    assert!(e.undo());
    assert_eq!(e.doc.paragraphs[0].text(), "ab");
}

#[test]
fn ime_composition_over_a_selection_and_cancel() {
    let mut e = Editor::new(Document::from_text("hello"));
    e.sel = Selection {
        anchor: Position::new(0, 1),
        head: Position::new(0, 4),
    };
    e.ime_replace(None, "x", true);
    assert_eq!(e.doc.paragraphs[0].text(), "hxo");
    e.ime_replace(None, "xy", true);
    assert_eq!(e.doc.paragraphs[0].text(), "hxyo");
    // Cancelling the composition (an empty commit) brings back what it replaced.
    e.ime_replace(None, "", false);
    assert_eq!(e.doc.paragraphs[0].text(), "hello");
    assert_eq!(e.ime_marked(), None);
}

#[test]
fn plain_typing_through_the_text_input_replaces_the_selection() {
    let mut e = Editor::new(Document::from_text("hello"));
    e.sel = Selection {
        anchor: Position::new(0, 1),
        head: Position::new(0, 4),
    };
    e.ime_replace(None, "i", false);
    assert_eq!(e.doc.paragraphs[0].text(), "hio");
}

fn doc_with_picture() -> Editor {
    let mut e = Editor::new(Document::from_text("one\ntwo\nthree"));
    let mut picture = crate::Paragraph::default();
    picture.style.image = Some(crate::ImageRef::inline(0, 100.0, 50.0));
    e.doc.paragraphs.insert(1, picture);
    e.sel = Selection::caret(Position::new(1, 0));
    e
}

#[test]
fn a_picture_floats_and_its_size_changes_in_one_undo_step() {
    let mut e = doc_with_picture();
    let float = crate::Float {
        x: 10.0,
        y: 20.0,
        wrap: crate::Wrap::TopBottom,
    };
    e.set_image_geometry(200.0, 80.0, Some(float));
    let image = e.image_at_caret().unwrap();
    assert_eq!(
        (image.width, image.height, image.float),
        (200.0, 80.0, Some(float))
    );
    // A drag changes the step that the first change started.
    e.drag_image_geometry(210.0, 90.0, Some(float), false);
    assert_eq!(e.image_at_caret().unwrap().width, 210.0);
    assert!(e.undo());
    assert_eq!(
        e.image_at_caret().unwrap(),
        crate::ImageRef::inline(0, 100.0, 50.0)
    );
}

#[test]
fn a_picture_moves_to_another_paragraph() {
    let mut e = doc_with_picture();
    let float = crate::Float {
        x: 0.0,
        y: 0.0,
        wrap: crate::Wrap::Front,
    };
    // Before "three" (index 3): the picture paragraph ends up between "two" and "three".
    assert!(e.move_image_to(3, Some(float)));
    let texts: Vec<String> = e.doc.paragraphs.iter().map(|p| p.text()).collect();
    assert_eq!(texts, ["one", "two", "", "three"]);
    assert_eq!(e.sel.head.para, 2);
    assert_eq!(e.image_at_caret().unwrap().float, Some(float));
    // Back to the front of the document.
    assert!(e.move_image_to(0, None));
    assert!(e.doc.paragraphs[0].style.image.is_some());
    assert_eq!(e.sel.head.para, 0);
    assert!(e.undo());
    assert!(e.doc.paragraphs[2].style.image.is_some());
}

#[test]
fn turning_a_picture_keeps_its_centre() {
    let mut e = doc_with_picture();
    let float = crate::Float {
        x: 100.0,
        y: 100.0,
        wrap: crate::Wrap::Square,
    };
    e.set_image_geometry(100.0, 50.0, Some(float));
    e.turn_image(90.0);
    let i = e.image_at_caret().unwrap();
    assert_eq!((i.width, i.height, i.angle), (100.0, 50.0, 90.0));
    // The box is 50 x 100 now: the centre (150, 125) stays.
    let f = i.float.unwrap();
    assert_eq!((f.x, f.y), (125.0, 75.0));
    e.turn_image(-90.0);
    let i = e.image_at_caret().unwrap();
    assert_eq!(i.angle, 0.0);
    assert_eq!((i.float.unwrap().x, i.float.unwrap().y), (100.0, 100.0));
    e.turn_image(400.0);
    assert_eq!(e.image_at_caret().unwrap().angle, 40.0);
}

#[test]
fn dragging_the_angle_is_one_undo_step() {
    let mut e = doc_with_picture();
    e.drag_image_angle(10.0, true);
    e.drag_image_angle(20.0, false);
    e.drag_image_angle(30.0, false);
    assert_eq!(e.image_at_caret().unwrap().angle, 30.0);
    assert!(e.undo());
    assert_eq!(e.image_at_caret().unwrap().angle, 0.0);
}

#[test]
fn resetting_the_crop_restores_the_full_picture() {
    let mut e = doc_with_picture();
    let crop = crate::Crop {
        left: 0.25,
        top: 0.0,
        right: 0.25,
        bottom: 0.0,
    };
    e.drag_image_crop(crop, 50.0, 50.0, None, true);
    e.reset_image_crop();
    let i = e.image_at_caret().unwrap();
    assert_eq!((i.width, i.height), (100.0, 50.0));
    assert!(i.crop.is_none());
}

#[test]
fn a_footnote_is_a_mark_in_the_text_and_a_note_at_the_end() {
    let mut e = Editor::new(Document::from_text("Hello world\nsecond"));
    e.sel = Selection::caret(Position::new(0, 5));
    e.insert_footnote();
    // The mark sits in the text, the empty note is the last paragraph and holds the caret.
    assert_eq!(e.doc.paragraphs.len(), 3);
    assert_eq!(e.doc.paragraphs[0].text(), "Hello\u{FFFC} world");
    let mark = e.doc.paragraphs[0]
        .runs
        .iter()
        .find(|r| r.style.note.is_some())
        .expect("a run with the mark");
    assert_eq!(mark.text, "\u{FFFC}");
    assert_eq!(e.doc.paragraphs[2].style.note, mark.style.note);
    assert_eq!(e.sel.head, Position::new(2, 0));
    assert_eq!(e.doc.note_numbers().len(), 1);
    // Text typed next to the mark is not part of it.
    e.sel = Selection::caret(Position::new(0, 8));
    e.insert_text("x");
    assert!(
        e.doc.paragraphs[0]
            .runs
            .iter()
            .filter(|r| r.style.note.is_some())
            .count()
            == 1
    );
    assert!(e.undo());
    assert!(e.undo());
    assert_eq!(
        e.doc.paragraphs.len(),
        2,
        "the whole footnote is one undo step"
    );
    assert_eq!(e.doc.paragraphs[0].text(), "Hello world");
}

#[test]
fn footnotes_are_numbered_in_text_order_and_jump_both_ways() {
    let mut e = Editor::new(Document::from_text("one two three"));
    e.sel = Selection::caret(Position::new(0, 7));
    e.insert_footnote();
    e.sel = Selection::caret(Position::new(0, 3));
    e.insert_footnote();
    let numbers = e.doc.note_numbers();
    let ids: Vec<u32> = e.doc.paragraphs[0]
        .runs
        .iter()
        .filter_map(|r| r.style.note)
        .collect();
    // The mark that stands first in the text is number 1, whenever it was made.
    assert_eq!((numbers[&ids[0]], numbers[&ids[1]]), (1, 2));
    // From the mark (the caret after it) to its note and back to just after the mark.
    let first_mark_end = e.doc.note_mark_position(ids[0]).unwrap();
    let mark_len = wv_core_mark_len();
    let note = e
        .note_target(Position::new(0, first_mark_end.offset - mark_len))
        .unwrap();
    // A click just after the mark finds it too.
    assert_eq!(e.note_target(first_mark_end), Some(note));
    assert_eq!(e.doc.paragraphs[note.para].style.note, Some(ids[0]));
    assert_eq!(e.note_target(note), Some(first_mark_end));
}

#[test]
fn pasting_text_with_a_footnote_drops_the_mark() {
    let mut e = Editor::new(Document::from_text("abc"));
    e.sel = Selection::caret(Position::new(0, 1));
    e.insert_footnote();
    e.sel = Selection {
        anchor: Position::new(0, 0),
        head: Position::new(0, e.doc.paragraphs[0].len()),
    };
    let fragment = e.copy_fragment();
    e.sel = Selection::caret(Position::new(0, e.doc.paragraphs[0].len()));
    e.paste_fragment(&fragment);
    let marks = e.doc.paragraphs[0]
        .runs
        .iter()
        .filter(|r| r.style.note.is_some())
        .count();
    assert_eq!(marks, 1, "the pasted copy has no mark");
}

fn wv_core_mark_len() -> usize {
    crate::NOTE_MARK.len_utf8()
}

#[test]
fn a_comment_is_anchored_to_the_selected_text() {
    let mut e = Editor::new(Document::from_text("one two three"));
    assert_eq!(e.add_comment("Ann", "x"), None, "nothing is selected");
    e.sel = Selection {
        anchor: Position::new(0, 4),
        head: Position::new(0, 7),
    };
    let id = e.add_comment("Ann", "Why two?").unwrap();
    assert_eq!(
        e.doc.comment_range(id),
        Some((Position::new(0, 4), Position::new(0, 7)))
    );
    assert_eq!(e.comment_at(Position::new(0, 5)), Some(id));
    assert_eq!(e.comment_at(Position::new(0, 2)), None);
    let comments = e.doc.visible_comments();
    assert_eq!(
        (comments[0].author.as_str(), comments[0].text.as_str()),
        ("Ann", "Why two?")
    );
    // Typing at the end of the commented text does not extend it, typing inside does.
    e.sel = Selection::caret(Position::new(0, 7));
    e.insert_text("!");
    assert_eq!(e.doc.comment_range(id).unwrap().1, Position::new(0, 7));
    e.sel = Selection::caret(Position::new(0, 5));
    e.insert_text("?");
    assert_eq!(e.doc.comment_range(id).unwrap().1, Position::new(0, 8));
}

#[test]
fn deleting_a_comment_takes_the_anchor_off_and_undo_brings_it_back() {
    let mut e = Editor::new(Document::from_text("one two three"));
    e.sel = Selection {
        anchor: Position::new(0, 0),
        head: Position::new(0, 3),
    };
    let id = e.add_comment("Ann", "note").unwrap();
    e.set_comment_text(id, "changed");
    e.delete_comment(id);
    assert!(e.doc.visible_comments().is_empty());
    assert!(e.undo());
    assert_eq!(e.doc.visible_comments()[0].text, "changed");
    // Clearing the formatting keeps the anchor.
    e.sel = Selection {
        anchor: Position::new(0, 0),
        head: Position::new(0, 13),
    };
    e.clear_formatting();
    assert_eq!(e.doc.visible_comments().len(), 1);
}

fn tracked(text: &str) -> Editor {
    let mut e = Editor::new(Document::from_text(text));
    e.set_author("Ann");
    e.set_tracking(true);
    e
}

fn select(e: &mut Editor, from: usize, to: usize) {
    e.sel = Selection {
        anchor: Position::new(0, from),
        head: Position::new(0, to),
    };
}

#[test]
fn tracked_typing_is_marked_as_inserted_and_can_be_rejected() {
    let mut e = tracked("Привет мир");
    let at = "Привет".len();
    e.sel = Selection::caret(Position::new(0, at));
    e.insert_text("!!");
    e.insert_text("?");
    assert_eq!(e.doc.paragraphs[0].text(), "Привет!!? мир");
    assert_eq!(e.doc.changes(), [(1, false)], "typing continues one change");
    e.reject_all_changes();
    assert_eq!(e.doc.paragraphs[0].text(), "Привет мир");
    assert!(e.doc.changes().is_empty());
    e.undo();
    assert_eq!(e.doc.paragraphs[0].text(), "Привет!!? мир");
}

#[test]
fn tracked_deletion_keeps_the_text_until_accepted() {
    let mut e = tracked("Hello");
    select(&mut e, 1, 4);
    e.delete_selection();
    assert_eq!(e.doc.paragraphs[0].text(), "Hello");
    assert_eq!(e.doc.changes(), [(1, true)]);
    let before = e.doc.clone();
    e.reject_all_changes();
    assert_eq!(e.doc.paragraphs[0].text(), "Hello");
    assert!(e.doc.changes().is_empty());
    e.undo();
    assert_eq!(e.doc, before);
    e.accept_all_changes();
    assert_eq!(e.doc.paragraphs[0].text(), "Ho");
}

#[test]
fn deleting_own_insertion_leaves_no_trace_and_replacing_marks_both() {
    let mut e = tracked("ab");
    e.sel = Selection::caret(Position::new(0, 1));
    e.insert_text("x");
    e.backspace();
    assert_eq!(e.doc.paragraphs[0].text(), "ab");
    assert!(e.doc.changes().is_empty());
    select(&mut e, 0, 1);
    e.insert_text("Z");
    assert_eq!(e.doc.paragraphs[0].text(), "aZb");
    assert_eq!(e.doc.changes().len(), 2);
    e.accept_all_changes();
    assert_eq!(e.doc.paragraphs[0].text(), "Zb");
}

#[test]
fn changes_here_and_navigation() {
    let mut e = tracked("one two");
    select(&mut e, 4, 7);
    e.delete_selection();
    e.sel = Selection::caret(Position::new(0, 0));
    e.insert_text("A");
    assert!(e.goto_change(true));
    assert_eq!(e.selected_text(), "two");
    assert!(e.goto_change(true), "wraps around");
    assert_eq!(e.selected_text(), "A");
    assert!(e.goto_change(false));
    assert_eq!(e.selected_text(), "two");
    assert_eq!(e.changes_here().len(), 1);
    e.reject_changes_here();
    assert_eq!(e.doc.changes().len(), 1, "only the insertion is left");
    assert_eq!(e.doc.paragraphs[0].text(), "Aone two");
}

#[test]
fn untracked_edits_are_plain() {
    let mut e = Editor::new(Document::from_text("abc"));
    e.sel = Selection::caret(Position::new(0, 1));
    e.insert_text("x");
    assert!(e.doc.changes().is_empty());
}

#[test]
fn a_section_break_splits_the_text_and_sections_have_their_own_page_setup() {
    let mut e = Editor::new(Document::from_text("Первый раздел второй"));
    e.sel = Selection::caret(Position::new(0, "Первый раздел".len()));
    e.insert_section_break();
    assert_eq!(e.doc.paragraphs.len(), 2);
    assert_eq!(e.section_count(), 2);
    assert_eq!(e.doc.paragraphs[0].text(), "Первый раздел");
    assert_eq!(e.doc.paragraphs[1].text(), " второй");
    // The caret is in the second section; make it landscape.
    let portrait = e.page_here();
    e.set_page(portrait.with_landscape(true));
    assert!(e.page_here().width > e.page_here().height);
    assert_eq!(e.doc.page_of(0), portrait, "the first section is unchanged");
    e.sel = Selection::caret(Position::new(0, 0));
    assert_eq!(e.page_here(), portrait);
    e.undo();
    e.undo();
    assert_eq!(e.section_count(), 1);
    assert!(e.doc.sections.is_empty());
}

#[test]
fn enter_in_a_break_paragraph_keeps_one_break_at_the_end() {
    let mut e = Editor::new(Document::from_text("ab"));
    e.sel = Selection::caret(Position::new(0, 2));
    e.insert_section_break();
    e.sel = Selection::caret(Position::new(0, 1));
    e.enter();
    assert_eq!(e.section_count(), 2);
    assert!(e.doc.paragraphs[0].style.section.is_none());
    assert!(e.doc.paragraphs[1].style.section.is_some());
    e.sel = Selection::caret(Position::new(0, 0));
    e.remove_section_break();
    assert_eq!(e.section_count(), 1);
}

#[test]
fn a_style_defined_from_the_caret_formatting_styles_other_paragraphs() {
    let mut e = Editor::new(Document::from_text("Заголовок\nПростой текст"));
    e.sel = Selection {
        anchor: Position::new(0, 0),
        head: Position::new(0, "Заголовок".len()),
    };
    e.set_size_pt(18);
    e.toggle_bold();
    e.set_align(wv_core_align());
    e.sel = Selection::caret(Position::new(0, "За".len()));
    let id = e.define_style("Мой стиль");
    assert_eq!(e.user_style_here(), Some(id));
    assert_eq!(e.doc.styles[&id].name, "Мой стиль");
    assert_eq!(e.doc.styles[&id].size_pt, Some(18));
    // A plain paragraph gets the look of the style once it has it.
    e.sel = Selection::caret(Position::new(1, 0));
    e.set_user_style(id);
    let resolved = e.doc.resolve(&e.doc.paragraphs[1]).unwrap();
    assert_eq!(resolved.runs[0].style.size_pt, Some(18));
    assert!(resolved.runs[0].style.bold);
    assert_eq!(resolved.style.align, wv_core_align());
    // Changing the style changes its paragraphs.
    e.sel = Selection::caret(Position::new(0, 0));
    e.set_size_pt(30);
    e.update_style(id);
    let resolved = e.doc.resolve(&e.doc.paragraphs[1]).unwrap();
    assert_eq!(resolved.runs[0].style.size_pt, Some(30));
    // Applying a built-in style takes the user style off; deleting the style frees its paragraphs.
    e.sel = Selection::caret(Position::new(1, 0));
    e.set_named_style(wv_core_named());
    assert_eq!(e.user_style_here(), None);
    e.delete_style(id);
    assert!(e.doc.styles.is_empty());
    assert!(e.doc.paragraphs.iter().all(|p| p.style.custom.is_none()));
    e.undo();
    assert_eq!(e.doc.styles.len(), 1);
    assert_eq!(e.doc.paragraphs[0].style.custom, Some(id));
}

fn wv_core_align() -> crate::style::Align {
    crate::style::Align::Center
}

fn wv_core_named() -> crate::style::NamedStyle {
    crate::style::NamedStyle::Heading2
}

#[test]
fn sections_have_their_own_headers_and_footers() {
    let mut e = Editor::new(Document::from_text("one two"));
    e.set_header_footer(wv_core_band("Шапка"), wv_core_band("{page}"));
    e.sel = Selection::caret(Position::new(0, 3));
    e.insert_section_break();
    // Both sections start with the same bands.
    assert_eq!(e.bands_here().0.text(), "Шапка");
    // Change the header of the second section only.
    e.set_header_footer(wv_core_band("Вторая"), wv_core_band("{page}"));
    assert_eq!(e.bands_here().0.text(), "Вторая");
    assert_eq!(e.doc.bands_for(Some(1), 0).0.text(), "Шапка");
    assert_eq!(e.doc.bands_for(None, 1).0.text(), "Вторая");
    // Back in the first section it is the first section's header that changes.
    e.sel = Selection::caret(Position::new(0, 0));
    e.toggle_page_number();
    assert!(!e.doc.bands_for(Some(1), 0).1.has_page_number());
    assert!(e.doc.bands_for(None, 1).1.has_page_number());
    e.remove_section_break();
    assert!(e.doc.section_bands.is_empty());
}

fn wv_core_band(text: &str) -> crate::document::Band {
    crate::document::Band::new(text)
}

#[test]
fn the_files_default_font_fills_what_nothing_else_sets() {
    let mut e = Editor::new(Document::from_text("Заголовок\nТекст"));
    e.doc.default_font = Some("Georgia".into());
    e.doc.default_size_pt = Some(11);
    e.sel = Selection::caret(Position::new(0, 0));
    e.set_named_style(wv_core_named());
    let plain = e.doc.resolve(&e.doc.paragraphs[1]).unwrap();
    assert_eq!(plain.runs[0].style.font.as_deref(), Some("Georgia"));
    assert_eq!(plain.runs[0].style.size_pt, Some(11));
    // The heading keeps the size of its style; the font still comes from the file.
    let heading = e.doc.resolve(&e.doc.paragraphs[0]).unwrap();
    assert_eq!(heading.runs[0].style.size_pt, Some(16));
    assert_eq!(heading.runs[0].style.font.as_deref(), Some("Georgia"));
}

#[test]
fn bookmarks_mark_text_and_cross_references_link_to_them() {
    let mut e = Editor::new(Document::from_text("Введение\nТекст про закладки"));
    e.sel = Selection {
        anchor: Position::new(1, "Текст ".len()),
        head: Position::new(1, "Текст про".len()),
    };
    let id = e.add_bookmark("моя закладка").unwrap();
    assert_eq!(
        e.bookmark_position("моя закладка"),
        Some(Position::new(1, "Текст ".len()))
    );
    assert_eq!(e.doc.visible_bookmarks().len(), 1);
    // Typing next to the bookmark does not extend it.
    e.sel = Selection::caret(Position::new(1, "Текст про".len()));
    e.insert_text("!");
    assert!(
        e.doc.paragraphs[1]
            .style_at(e.sel.head.offset)
            .bookmarks
            .is_empty()
    );
    // Adding it again moves it; a caret marks the character after it.
    e.sel = Selection::caret(Position::new(0, 0));
    let moved = e.add_bookmark("моя закладка").unwrap();
    assert_ne!(moved, id);
    assert_eq!(
        e.bookmark_position("моя закладка"),
        Some(Position::new(0, 0))
    );
    assert_eq!(e.doc.visible_bookmarks().len(), 1);
    e.delete_bookmark(moved);
    assert!(e.bookmark_position("моя закладка").is_none());
    // A reference to a heading is a link to a hidden bookmark on it.
    e.sel = Selection::caret(Position::new(1, e.doc.paragraphs[1].len()));
    e.insert_heading_reference(0);
    let linked = e.doc.paragraphs[1].runs.last().unwrap();
    assert_eq!(linked.text, "Введение");
    let name = linked.style.link.clone().unwrap();
    assert!(name.starts_with("#_Ref"));
    assert!(
        e.doc.visible_bookmarks().is_empty(),
        "hidden ones are not listed"
    );
}

#[test]
fn a_character_can_have_several_bookmarks() {
    let mut e = Editor::new(Document::from_text("Введение\nТекст"));
    e.sel = Selection {
        anchor: Position::new(0, 0),
        head: Position::new(0, "Введение".len()),
    };
    e.add_bookmark("первая").unwrap();
    e.add_bookmark("вторая").unwrap();
    assert_eq!(e.doc.visible_bookmarks().len(), 2);
    assert_eq!(e.bookmark_position("первая"), Some(Position::new(0, 0)));
    assert_eq!(e.bookmark_position("вторая"), Some(Position::new(0, 0)));
    // A reference to the heading gets a hidden bookmark of its own; the others stay.
    e.sel = Selection::caret(Position::new(1, "Текст".len()));
    e.insert_heading_reference(0);
    assert_eq!(e.doc.visible_bookmarks().len(), 2);
    let link = e.doc.paragraphs[1].runs.last().unwrap().style.link.clone();
    assert!(link.unwrap().starts_with("#_Ref"));
    e.delete_bookmark(e.doc.visible_bookmarks()[0].id);
    assert_eq!(e.doc.visible_bookmarks().len(), 1);
    assert!(e.bookmark_position("вторая").is_some());
}

#[test]
fn page_number_fields_follow_their_bookmark() {
    let mut e = Editor::new(Document::from_text("Цель\nСм. стр. "));
    e.sel = Selection {
        anchor: Position::new(0, 0),
        head: Position::new(0, "Цель".len()),
    };
    e.add_bookmark("цель").unwrap();
    e.sel = Selection::caret(Position::new(1, "См. стр. ".len()));
    e.insert_page_reference("цель", "1");
    assert_eq!(e.doc.paragraphs[1].text(), "См. стр. 1");
    let run = e.doc.paragraphs[1].runs.last().unwrap();
    assert_eq!(run.style.link.as_deref(), Some("#цель"));
    assert!(run.style.page_ref.is_some());
    // The bookmark moved to page 12: updating changes the number, once, in one undo step.
    let page = |p: Position| (p.para == 0).then(|| "12".to_string());
    assert_eq!(e.update_fields(&page), 1);
    assert_eq!(e.doc.paragraphs[1].text(), "См. стр. 12");
    assert_eq!(e.update_fields(&page), 0);
    e.undo();
    assert_eq!(e.doc.paragraphs[1].text(), "См. стр. 1");
    // Text typed after the field is not part of it.
    e.sel = Selection::caret(Position::new(1, e.doc.paragraphs[1].len()));
    e.insert_text("!");
    assert!(
        e.doc.paragraphs[1]
            .runs
            .last()
            .unwrap()
            .style
            .page_ref
            .is_none()
    );
}
