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
