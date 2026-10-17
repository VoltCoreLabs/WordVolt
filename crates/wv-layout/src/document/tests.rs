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
        header_distance: 5.0,
        footer_distance: 5.0,
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

#[test]
fn table_cells_are_laid_out_side_by_side_and_hit_tested() {
    use wv_core::Cell;
    // 2 x 2 table: content 80 px -> columns of 40 px, text 28 px wide inside 6 px padding.
    let mut doc = Document::from_text("before\nA1\nB1\nA2\nB2\nafter");
    for (i, (row, col)) in [(0, 0), (0, 1), (1, 0), (1, 1)].into_iter().enumerate() {
        doc.paragraphs[i + 1].style.cell = Some(Cell::at(0, row, col));
    }
    let tall = PageSetup {
        height: 220.0,
        ..small()
    };
    let l = DocLayout::new(&doc, tall, &Fixed);
    let page = &l.pages[0];
    assert_eq!(page.tables.len(), 1);
    let t = &page.tables[0];
    assert_eq!(t.col_x, [0.0, 40.0, 80.0]);
    assert_eq!(t.row_y.len(), 3);
    let cell = |para: usize| page.slices.iter().find(|s| s.para == para).unwrap();
    assert_eq!(
        (cell(1).x, cell(2).x),
        (crate::CELL_PADDING, 40.0 + crate::CELL_PADDING)
    );
    assert_eq!(
        cell(1).y,
        cell(2).y,
        "cells of a row start at the same height"
    );
    assert!(cell(3).y > cell(1).y, "second row below the first");
    // Clicking inside the second column picks that cell, not the first one.
    let c = l.caret(Position::new(2, 1), &Fixed).unwrap();
    assert_eq!(
        l.position_at(c.page, c.x, c.y + 2.0, &Fixed),
        Position::new(2, 1)
    );
    assert_eq!(
        l.position_at(0, 10.0 + 41.0, 10.0 + t.row_y[1] + 3.0, &Fixed)
            .para,
        4
    );
    // Text after the table starts below it.
    let after = page.slices.iter().find(|s| s.para == 5).unwrap();
    assert!(after.y >= *t.row_y.last().unwrap());
}

#[test]
fn table_rows_move_to_the_next_page_instead_of_splitting() {
    use wv_core::Cell;
    // Rows are 16 + 12 = 28 px high; the page content is 80 px: 2 rows fit, the 3rd starts page 2.
    let mut doc = Document::from_text("a\nb\nc");
    for (i, row) in [0u16, 1, 2].into_iter().enumerate() {
        doc.paragraphs[i].style.cell = Some(Cell::at(0, row, 0));
    }
    let l = DocLayout::new(&doc, small(), &Fixed);
    assert_eq!(l.pages.len(), 2);
    assert_eq!(l.pages[0].tables[0].row_y.len(), 3);
    assert_eq!(
        l.pages[1].tables[0].row_y.len(),
        2,
        "the table continues on page 2"
    );
}

#[test]
fn picture_paragraph_takes_its_height_and_follows_alignment() {
    use wv_core::{Align, ImageRef};
    let mut doc = Document::from_text("a\n\nb");
    doc.paragraphs[1].style.image = Some(ImageRef {
        id: 0,
        width: 40.0,
        height: 50.0,
    });
    doc.paragraphs[1].style.align = Align::Center;
    let l = DocLayout::new(&doc, small(), &Fixed);
    let image = l.paragraphs[1].image.unwrap();
    assert_eq!((image.width, image.height), (40.0, 50.0));
    assert_eq!(l.paragraphs[1].height, 50.0);
    assert_eq!(l.paragraphs[1].lines[0].x, 20.0);
    // 16 + 50 + 16 = 82 > 80: "b" moves to the next page.
    assert_eq!(l.pages.len(), 2);
}

#[test]
fn column_widths_and_cell_shading_shape_the_table() {
    let mut doc = Document::from_text("a\nb\nc\nd");
    for (i, (row, col)) in [(0, 0), (0, 1), (1, 0), (1, 1)].into_iter().enumerate() {
        doc.paragraphs[i].style.cell = Some(wv_core::Cell::at(0, row, col));
    }
    doc.paragraphs[1].style.cell.as_mut().unwrap().shade = Some(0xFFEE99);
    doc.table_widths.insert(0, vec![0.25, 0.75]);
    let layout = DocLayout::new(&doc, small(), &Fixed);
    let table = &layout.pages[0].tables[0];
    assert_eq!(table.table, 0);
    assert!((table.col_x[1] - 20.0).abs() < 0.01 && (table.col_x[2] - 80.0).abs() < 0.01);
    assert_eq!(table.shades.len(), 1);
    assert_eq!(table.shades[0].color, 0xFFEE99);
    assert!((table.shades[0].x - 20.0).abs() < 0.01 && (table.shades[0].width - 60.0).abs() < 0.01);
}

fn merged_table(cells: &[(u16, u16, u8, u8)]) -> DocLayout {
    let text: Vec<String> = (0..cells.len()).map(|i| format!("c{i}")).collect();
    let mut doc = Document::from_text(&text.join(
        "
",
    ));
    for (i, (row, col, colspan, rowspan)) in cells.iter().copied().enumerate() {
        doc.paragraphs[i].style.cell = Some(wv_core::Cell {
            colspan,
            rowspan,
            ..wv_core::Cell::at(0, row, col)
        });
    }
    DocLayout::new(&doc, small(), &Fixed)
}

#[test]
fn a_merged_cell_spans_columns() {
    let layout = merged_table(&[(0, 0, 2, 1), (1, 0, 1, 1), (1, 1, 1, 1)]);
    let table = &layout.pages[0].tables[0];
    assert_eq!(table.cells.len(), 3);
    assert!(
        (table.cells[0].width - 80.0).abs() < 0.01,
        "covers both columns"
    );
    assert!((table.cells[1].width - 40.0).abs() < 0.01);
    assert_eq!(table.row_y.len(), 3, "two rows");
}

#[test]
fn a_merged_cell_spans_rows() {
    let layout = merged_table(&[(0, 0, 1, 2), (0, 1, 1, 1), (1, 1, 1, 1)]);
    let table = &layout.pages[0].tables[0];
    let tall = &table.cells[0];
    assert!((tall.height - (table.row_y[2] - table.row_y[0])).abs() < 0.01);
    assert_eq!(layout.pages[0].slices.len(), 3);
}
