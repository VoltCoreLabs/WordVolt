use wv_core::{Document, Position};

use super::*;
use crate::test_support::{CHAR, Fixed};

/// 80 x 80 content area: 10 chars per line, 5 lines of 16 px per page; no paragraph gap.
fn small() -> PageSetup {
    PageSetup {
        width: 100.0,
        height: 100.0,
        margin_top: 10.0,
        margin_right: 10.0,
        margin_bottom: 10.0,
        margin_left: 10.0,
        paragraph_spacing: 0.0,
        line_spacing: 1.0,
    }
}

fn layout(text: &str) -> DocLayout {
    DocLayout::new(&Document::from_text(text), small(), &Fixed)
}

#[test]
fn a4_content_area() {
    let a4 = PageSetup::a4();
    assert!((a4.content_width() - 601.7).abs() < 0.1);
    assert!((a4.content_height() - 930.5).abs() < 0.1);
}

#[test]
fn text_flows_onto_a_second_page() {
    // 6 paragraphs of one line each: 5 fit on page 1.
    let l = layout("a\nb\nc\nd\ne\nf");
    assert_eq!(l.pages.len(), 2);
    assert_eq!(l.pages[0].slices.len(), 5);
    assert_eq!(l.pages[1].slices[0].para, 5);
    assert_eq!(l.pages[1].slices[0].y, 0.0);
}

#[test]
fn a_long_paragraph_is_split_between_lines() {
    let l = layout(&"x".repeat(70)); // 7 lines of 10 chars
    assert_eq!(l.pages.len(), 2);
    assert_eq!(l.pages[0].slices[0].lines, 0..5);
    assert_eq!(l.pages[1].slices[0].lines, 5..7);
}

#[test]
fn paragraph_spacing_pushes_content_down() {
    let setup = PageSetup {
        paragraph_spacing: 8.0,
        ..small()
    };
    let l = DocLayout::new(&Document::from_text("a\nb"), setup, &Fixed);
    assert_eq!(l.pages[0].slices[1].y, 16.0 + 8.0);
}

#[test]
fn caret_and_hit_test_round_trip() {
    let l = layout("hello world\nsecond");
    for pos in [
        Position::new(0, 0),
        Position::new(0, 3),
        Position::new(0, 6),
        Position::new(1, 0),
        Position::new(1, 6),
    ] {
        let c = l.caret(pos, &Fixed).unwrap();
        let back = l.position_at(c.page, c.x, c.y + c.height / 2.0, &Fixed);
        assert_eq!(back, pos, "{pos:?}");
    }
}

#[test]
fn caret_on_the_second_page() {
    let l = layout("a\nb\nc\nd\ne\nf");
    let c = l.caret(Position::new(5, 1), &Fixed).unwrap();
    assert_eq!((c.page, c.y), (1, 10.0));
    assert_eq!(c.x, 10.0 + CHAR);
}

#[test]
fn clicking_below_the_text_picks_the_last_line() {
    let l = layout("one\ntwo");
    assert_eq!(l.position_at(0, 500.0, 500.0, &Fixed), Position::new(1, 3));
    assert_eq!(l.position_at(0, 0.0, 0.0, &Fixed), Position::new(0, 0));
}

#[test]
fn selection_rects_cover_partial_and_full_lines() {
    let l = layout("hello world foo\nbar");
    // "hello world foo" wraps at 10 chars: "hello " / "world foo".
    let rects = l.selection_rects(Position::new(0, 2), Position::new(1, 2), &Fixed);
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].x, 10.0 + 2.0 * CHAR);
    assert!(rects[1].width > 0.0 && rects[2].width == 2.0 * CHAR);
    assert!(
        l.selection_rects(Position::new(0, 1), Position::new(0, 1), &Fixed)
            .is_empty()
    );
}

#[test]
fn selected_empty_paragraph_shows_a_break_mark() {
    let l = layout("a\n\nb");
    let rects = l.selection_rects(Position::new(0, 0), Position::new(2, 1), &Fixed);
    assert!(rects.iter().any(|r| r.width == BREAK_MARK));
}

#[test]
fn line_range_for_home_and_end() {
    let l = layout("hello world foo"); // "hello " / "world foo"
    assert_eq!(l.line_range(Position::new(0, 2)), (0, 5));
    assert_eq!(l.line_range(Position::new(0, 8)), (6, 15));
}

#[test]
fn list_items_are_indented_numbered_and_tight() {
    use wv_core::{Align, ListKind};
    let mut doc = Document::from_text("first\nsecond\nplain");
    doc.paragraphs[0].style.list = Some(ListKind::Number);
    doc.paragraphs[1].style.list = Some(ListKind::Number);
    doc.paragraphs[1].style.indent = 1;
    doc.paragraphs[2].style.align = Align::Left;
    let l = DocLayout::new(&doc, small(), &Fixed);
    assert_eq!(l.paragraphs[0].marker.as_deref(), Some("1."));
    assert_eq!(l.paragraphs[1].marker.as_deref(), Some("a."));
    assert_eq!(l.paragraphs[2].marker, None);
    assert_eq!(l.paragraphs[0].indent, crate::LIST_HANGING);
    assert_eq!(
        l.paragraphs[1].indent,
        crate::INDENT_STEP + crate::LIST_HANGING
    );
    assert_eq!(l.paragraphs[0].lines[0].x, crate::LIST_HANGING);
    assert_eq!(l.paragraphs[0].space_after, Some(2.0));
    assert_eq!(l.paragraphs[2].space_after, None);
}

#[test]
fn indent_shrinks_the_text_width_and_line_spacing_scales_height() {
    // 80 px content: 10 chars; an indent of 48 px leaves 32 px = 4 chars.
    let mut doc = Document::from_text("aaaa bbbb");
    doc.paragraphs[0].style.indent = 1;
    doc.paragraphs[0].style.line_spacing = Some(2.0);
    let l = DocLayout::new(&doc, small(), &Fixed);
    assert_eq!(l.paragraphs[0].lines.len(), 2);
    assert_eq!(l.paragraphs[0].lines[0].height, 32.0);
}

#[test]
fn headings_are_bigger_have_space_before_and_it_is_dropped_at_the_page_top() {
    use wv_core::NamedStyle;
    let mut doc = Document::from_text("intro\nTitle");
    doc.paragraphs[1].style.named = NamedStyle::Heading1;
    let l = DocLayout::new(&doc, small(), &Fixed);
    // 20 pt = 26.67 px font; bold in Fixed is wider, the line is taller than a normal one.
    assert!(l.paragraphs[1].lines[0].height > 16.0);
    assert_eq!(l.paragraphs[1].space_before, 18.0);
    assert!(
        l.pages[0].slices[1].y > 16.0 + 18.0 - 0.01,
        "space before pushes the heading down"
    );
    // A heading that starts a page gets no gap above it.
    let mut doc = Document::from_text("a\nb\nc\nd\nHead");
    doc.paragraphs[4].style.named = NamedStyle::Heading1;
    let l = DocLayout::new(&doc, small(), &Fixed);
    if l.pages.len() > 1 {
        assert_eq!(l.pages[1].slices[0].y, 0.0);
    }
}
