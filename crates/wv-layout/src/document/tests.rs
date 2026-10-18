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
        columns: 1,
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
    doc.paragraphs[1].style.image = Some(ImageRef::inline(0, 40.0, 50.0));
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
    doc.table_looks.insert(
        0,
        wv_core::TableLook {
            widths: vec![0.25, 0.75],
            ..Default::default()
        },
    );
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

/// Counts how often text is measured.
struct Counting(std::cell::Cell<usize>);

impl crate::Measurer for Counting {
    fn width(&self, text: &str, style: &wv_core::CharStyle) -> f32 {
        self.0.set(self.0.get() + 1);
        Fixed.width(text, style)
    }

    fn ascent(&self, style: &wv_core::CharStyle) -> f32 {
        Fixed.ascent(style)
    }

    fn descent(&self, style: &wv_core::CharStyle) -> f32 {
        Fixed.descent(style)
    }
}

#[test]
fn unchanged_paragraphs_are_not_laid_out_again() {
    let mut doc = Document::from_text("first paragraph\nsecond one\nthird");
    let m = Counting(std::cell::Cell::new(0));
    let mut cache = crate::LayoutCache::default();
    let first = DocLayout::with_cache(&doc, small(), &m, &mut cache);
    let cold = m.0.replace(0);
    assert!(cold > 0);
    let again = DocLayout::with_cache(&doc, small(), &m, &mut cache);
    assert_eq!(m.0.replace(0), 0, "nothing changed, nothing measured");
    assert_eq!(first.pages, again.pages);
    // Editing the middle paragraph only re-measures that one.
    doc.paragraphs[1] = wv_core::Paragraph::plain("second one, longer");
    let edited = DocLayout::with_cache(&doc, small(), &m, &mut cache);
    let warm = m.0.replace(0);
    assert!(warm > 0 && warm < cold, "{warm} vs {cold}");
    assert_eq!(edited.paragraphs[1].text, "second one, longer");
}

#[test]
fn selection_rects_per_page_add_up_to_the_whole_selection() {
    let l = layout(&"line of text\n".repeat(12));
    assert!(l.pages.len() > 1);
    let (a, b) = (Position::new(0, 0), Position::new(11, 4));
    let all = l.selection_rects(a, b, &Fixed);
    let per_page: usize = (0..l.pages.len())
        .map(|p| l.selection_rects_on_page(a, b, p, &Fixed).len())
        .sum();
    assert_eq!(all.len(), per_page);
    assert!(all.iter().any(|r| r.page == 1));
}

#[test]
fn an_indented_table_with_a_tall_row() {
    let mut doc = Document::from_text(
        "a
b",
    );
    for (i, col) in [0u16, 1].into_iter().enumerate() {
        doc.paragraphs[i].style.cell = Some(wv_core::Cell::at(0, 0, col));
    }
    doc.table_looks.insert(
        0,
        wv_core::TableLook {
            widths: vec![0.25, 0.25],
            indent: 0.25,
            rows: vec![50.0],
        },
    );
    let layout = DocLayout::new(&doc, small(), &Fixed);
    let table = &layout.pages[0].tables[0];
    // 80 px text width: starts a quarter in, two columns of a quarter each (narrower table).
    assert_eq!(table.col_x, vec![20.0, 40.0, 60.0]);
    assert!(
        (table.row_y[1] - table.row_y[0] - 50.0).abs() < 0.01,
        "{:?}",
        table.row_y
    );
    assert_eq!(table.first_row, 0);
}

#[test]
fn a_page_break_starts_a_new_page() {
    let mut doc = Document::from_text("a\nb\nc");
    doc.paragraphs[1].style.page_break = true;
    let l = DocLayout::new(&doc, small(), &Fixed);
    assert_eq!(l.pages.len(), 2);
    assert_eq!(l.pages[0].slices.len(), 1);
    assert_eq!(l.pages[1].slices[0].para, 1);
    assert_eq!(l.pages[1].slices[0].y, 0.0);
}

#[test]
fn a_page_break_at_the_top_adds_no_blank_page() {
    let mut doc = Document::from_text("a\nb");
    doc.paragraphs[0].style.page_break = true;
    let l = DocLayout::new(&doc, small(), &Fixed);
    assert_eq!(l.pages.len(), 1);
}

fn two_columns() -> PageSetup {
    PageSetup {
        columns: 2,
        ..small()
    }
}

#[test]
fn text_fills_the_first_column_then_the_second() {
    // Content 80 x 80, gap 48, two columns of 16 px each: 2 chars per line, 5 lines per column.
    let doc = Document::from_text("a\nb\nc\nd\ne\nf\ng");
    let l = DocLayout::new(&doc, two_columns(), &Fixed);
    assert_eq!(l.pages.len(), 1, "both columns are on one page");
    let slices = &l.pages[0].slices;
    assert_eq!(slices.len(), 7);
    // First five paragraphs in column 1 (x = 0), the rest at the top of column 2.
    assert!(slices[..5].iter().all(|s| s.x == 0.0));
    let step = two_columns().column_width() + crate::page::COLUMN_GAP;
    assert_eq!((slices[5].x, slices[5].y), (step, 0.0));
    assert_eq!(slices[6].x, step);
}

#[test]
fn a_page_break_skips_to_the_next_page_not_the_next_column() {
    let mut doc = Document::from_text("a\nb");
    doc.paragraphs[1].style.page_break = true;
    let l = DocLayout::new(&doc, two_columns(), &Fixed);
    assert_eq!(l.pages.len(), 2);
    assert_eq!(l.pages[1].slices[0].x, 0.0);
}

#[test]
fn caret_in_the_second_column_is_offset() {
    let doc = Document::from_text("a\nb\nc\nd\ne\nf");
    let l = DocLayout::new(&doc, two_columns(), &Fixed);
    let c = l.caret(Position::new(5, 0), &Fixed).unwrap();
    let step = two_columns().column_width() + crate::page::COLUMN_GAP;
    assert_eq!(c.x, 10.0 + step);
}

fn with_float(wrap: wv_core::Wrap) -> DocLayout {
    let mut doc = Document::from_text("a\nb\nc");
    let mut picture = wv_core::Paragraph::default();
    picture.style.image = Some(wv_core::ImageRef {
        float: Some(wv_core::Float {
            x: 10.0,
            y: 32.0,
            wrap,
        }),
        ..wv_core::ImageRef::inline(0, 40.0, 16.0)
    });
    doc.paragraphs.insert(1, picture);
    DocLayout::new(&doc, small(), &Fixed)
}

#[test]
fn text_stops_above_and_goes_on_below_a_top_and_bottom_picture() {
    let l = with_float(wv_core::Wrap::TopBottom);
    let page = &l.pages[0];
    assert_eq!(page.floats.len(), 1);
    assert_eq!((page.floats[0].x, page.floats[0].y), (10.0, 32.0));
    // "a" fits above the picture (0..16); "b" would reach into the band 32..48 (+6 gap on both
    // sides) and starts below it instead.
    let ys: Vec<(usize, f32)> = page.slices.iter().map(|s| (s.para, s.y)).collect();
    assert_eq!(ys[0], (0, 0.0));
    let b = ys.iter().find(|(p, _)| *p == 2).unwrap();
    assert_eq!(b.1, 54.0);
    // The picture paragraph itself takes no room.
    let picture = page.slices.iter().find(|s| s.para == 1).unwrap();
    assert_eq!(picture.height, 0.0);
}

#[test]
fn text_runs_under_or_over_a_picture_that_does_not_push_it() {
    for wrap in [wv_core::Wrap::Front, wv_core::Wrap::Behind] {
        let l = with_float(wrap);
        let b = l.pages[0].slices.iter().find(|s| s.para == 2).unwrap();
        assert_eq!(b.y, 16.0, "{wrap:?} leaves the text where it is");
    }
}

#[test]
fn text_goes_along_the_side_of_a_square_picture() {
    let mut doc = Document::from_text("aaaa bbbb cccc dddd eeee");
    let mut picture = wv_core::Paragraph::default();
    picture.style.image = Some(wv_core::ImageRef {
        float: Some(wv_core::Float {
            x: 0.0,
            y: 0.0,
            wrap: wv_core::Wrap::Square,
        }),
        ..wv_core::ImageRef::inline(0, 24.0, 32.0)
    });
    doc.paragraphs.insert(0, picture);
    let l = DocLayout::new(&doc, small(), &Fixed);
    let text = &l.paragraphs[1];
    // The picture and its gap take 32 px at the left for the first lines; below it the text is
    // full width again.
    assert_eq!(text.lines[0].x, 32.0);
    assert_eq!(text.lines[1].x, 32.0);
    assert_eq!(text.lines[2].x, 32.0);
    assert_eq!(text.lines[3].x, 0.0);
    assert!(
        text.lines[0].range.len() <= 6,
        "narrow lines beside the picture"
    );
    // Nothing ran under the picture and the paragraph is still in one piece on the page.
    assert_eq!(l.pages.len(), 1);
}

#[test]
fn a_picture_with_no_room_beside_it_pushes_the_text_below() {
    let mut doc = Document::from_text("aaaa bbbb");
    let mut picture = wv_core::Paragraph::default();
    // 60 px of the 80 px wide text area plus the gap leaves less than a line is worth.
    picture.style.image = Some(wv_core::ImageRef {
        float: Some(wv_core::Float {
            x: 0.0,
            y: 0.0,
            wrap: wv_core::Wrap::Square,
        }),
        ..wv_core::ImageRef::inline(0, 60.0, 32.0)
    });
    doc.paragraphs.insert(0, picture);
    let l = DocLayout::new(&doc, small(), &Fixed);
    let first = &l.paragraphs[1].lines[0];
    assert_eq!(first.x, 0.0);
    assert!(first.gap > 0.0, "the line starts below the picture");
}

#[test]
fn text_that_flows_onto_a_page_with_a_picture_stays_out_of_its_band() {
    let mut doc = Document::from_text(
        "a
b
c
d
e
f
g
h
i
j",
    );
    let mut picture = wv_core::Paragraph::default();
    picture.style.image = Some(wv_core::ImageRef {
        float: Some(wv_core::Float {
            x: 10.0,
            y: 16.0,
            wrap: wv_core::Wrap::TopBottom,
        }),
        ..wv_core::ImageRef::inline(0, 40.0, 16.0)
    });
    doc.paragraphs.insert(7, picture);
    let l = DocLayout::new(&doc, small(), &Fixed);
    let page = &l.pages[1];
    assert_eq!(page.floats.len(), 1);
    // The picture is 16..32 on the page; text may not touch 10..38 (with its gap). The first
    // paragraph that flowed onto the page starts below that, not at the top.
    for s in page.slices.iter().filter(|s| s.height > 0.0) {
        assert!(
            s.y + s.height <= 10.0 + 0.01 || s.y >= 38.0 - 0.01,
            "paragraph {} at {} runs into the picture",
            s.para,
            s.y
        );
    }
}
