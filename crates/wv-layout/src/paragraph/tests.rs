use wv_core::{Align, CharStyle, Paragraph, Run};

use super::*;
use crate::test_support::{CHAR, Fixed};

fn opts(chars: f32) -> Options {
    Options {
        width: chars * CHAR,
        line_spacing: 1.0,
    }
}

fn plain(text: &str) -> Paragraph {
    Paragraph::plain(text)
}

fn lines_of(p: &Paragraph, chars: f32) -> Vec<String> {
    let l = layout_paragraph(p, &opts(chars), &Fixed);
    l.lines
        .iter()
        .map(|line| l.text[line.range.clone()].to_string())
        .collect()
}

#[test]
fn short_text_is_one_line() {
    assert_eq!(lines_of(&plain("hello world"), 20.0), ["hello world"]);
}

#[test]
fn wraps_at_spaces_and_keeps_trailing_space_on_the_line() {
    assert_eq!(
        lines_of(&plain("aaa bbb ccc"), 6.0),
        ["aaa ", "bbb ", "ccc"]
    );
    assert_eq!(lines_of(&plain("aaa bbb ccc"), 7.0), ["aaa bbb ", "ccc"]);
    assert_eq!(lines_of(&plain("aaa bbb ccc"), 11.0), ["aaa bbb ccc"]);
}

#[test]
fn trailing_spaces_hang_and_do_not_count_as_width() {
    let l = layout_paragraph(&plain("ab    "), &opts(3.0), &Fixed);
    assert_eq!(l.lines.len(), 1);
    assert_eq!(l.lines[0].width, 2.0 * CHAR);
    assert_eq!(l.lines[0].visible_end, 2);
}

#[test]
fn long_word_is_broken_by_graphemes() {
    assert_eq!(lines_of(&plain("abcdefghij"), 4.0), ["abcd", "efgh", "ij"]);
    assert_eq!(lines_of(&plain("Привет"), 4.0), ["Прив", "ет"]);
    assert_eq!(lines_of(&plain("a👨‍👩‍👧b"), 2.0).concat(), "a👨‍👩‍👧b");
}

#[test]
fn empty_paragraph_has_one_line_with_height() {
    let l = layout_paragraph(&plain(""), &opts(10.0), &Fixed);
    assert_eq!(l.lines.len(), 1);
    assert_eq!(l.height, 16.0);
    assert!(l.lines[0].fragments.is_empty());
}

#[test]
fn line_height_follows_the_biggest_run() {
    let big = CharStyle {
        size_pt: Some(24),
        ..Default::default()
    };
    let p = Paragraph {
        runs: vec![
            Run::new("small ", CharStyle::default()),
            Run::new("BIG", big),
        ],
        ..Default::default()
    };
    let l = layout_paragraph(&p, &opts(30.0), &Fixed);
    assert_eq!(l.lines.len(), 1);
    assert_eq!(l.lines[0].height, 32.0);
    assert_eq!(l.lines[0].fragments.len(), 2);
    assert_eq!(l.lines[0].fragments[1].x, 6.0 * CHAR);
}

#[test]
fn line_spacing_scales_height() {
    let l = layout_paragraph(
        &plain("a"),
        &Options {
            width: 100.0,
            line_spacing: 1.5,
        },
        &Fixed,
    );
    assert_eq!(l.height, 24.0);
}

#[test]
fn alignment_offsets() {
    let mut p = plain("abcd");
    p.style.align = Align::Center;
    assert_eq!(
        layout_paragraph(&p, &opts(10.0), &Fixed).lines[0].x,
        3.0 * CHAR
    );
    p.style.align = Align::Right;
    assert_eq!(
        layout_paragraph(&p, &opts(10.0), &Fixed).lines[0].x,
        6.0 * CHAR
    );
}

#[test]
fn justify_stretches_all_but_the_last_line() {
    let mut p = plain("aa bb cc dd");
    p.style.align = Align::Justify;
    let l = layout_paragraph(&p, &opts(7.0), &Fixed);
    assert_eq!(l.lines.len(), 2);
    let first = &l.lines[0];
    assert_eq!(
        first.space_extra,
        2.0 * CHAR,
        "one gap shares the 2 missing chars"
    );
    assert!((first.x_at(&l.text, first.visible_end, &Fixed) - 7.0 * CHAR).abs() < 0.01);
    assert_eq!(l.lines[1].space_extra, 0.0, "last line stays ragged");
}

#[test]
fn x_at_and_index_at_agree() {
    let big = CharStyle {
        size_pt: Some(24),
        ..Default::default()
    };
    let p = Paragraph {
        runs: vec![Run::new("ab", CharStyle::default()), Run::new("cd", big)],
        ..Default::default()
    };
    let l = layout_paragraph(&p, &opts(30.0), &Fixed);
    let line = &l.lines[0];
    for byte in 0..=4 {
        let x = line.x_at(&l.text, byte, &Fixed);
        assert_eq!(line.index_at(&l.text, x, &Fixed), byte, "byte {byte}");
    }
    assert_eq!(line.x_at(&l.text, 4, &Fixed), 2.0 * CHAR + 2.0 * 2.0 * CHAR);
}

#[test]
fn click_right_of_a_wrapped_line_stays_before_its_trailing_space() {
    let l = layout_paragraph(&plain("aaa bbb"), &opts(4.0), &Fixed);
    assert_eq!(l.lines.len(), 2);
    assert_eq!(l.lines[0].index_at(&l.text, 1000.0, &Fixed), 3);
    assert_eq!(l.lines[1].index_at(&l.text, 1000.0, &Fixed), 7);
}

#[test]
fn caret_at_wrap_point_belongs_to_the_next_line() {
    let l = layout_paragraph(&plain("aaa bbb"), &opts(4.0), &Fixed);
    assert_eq!(l.line_of(3), 0);
    assert_eq!(l.line_of(4), 1);
    assert_eq!(l.line_of(7), 1);
}

#[test]
fn quote_is_indented_and_wraps_narrower() {
    let mut p = plain("aaaa bbbb");
    p.style.quote = true;
    let l = layout_paragraph(&p, &opts(10.0), &Fixed);
    assert!(l.quote);
    assert_eq!(l.lines[0].x, QUOTE_INDENT);
    // 80 px - 24 px indent = 56 px = 7 chars: the second word moves to the next line.
    assert_eq!(l.lines.len(), 2);
}
