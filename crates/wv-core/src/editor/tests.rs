use super::Editor;
use crate::document::Document;
use crate::position::Position;
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
    assert_eq!(e.doc.footer.text, "{page}");
    e.set_header_footer("Head", "Стр. {page}");
    e.toggle_page_number();
    assert_eq!(e.doc.footer.text, "Стр.");
    assert_eq!(e.doc.header.text, "Head");
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
