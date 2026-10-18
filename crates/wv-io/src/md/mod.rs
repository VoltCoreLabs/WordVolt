//! Markdown subset: paragraphs, `**bold**`, `*italic*`, `<u>underline</u>`, `# headings` (read as bold).
//! Tables are GFM pipe tables (first row = header; several paragraphs in a cell joined by `<br>`).
//! Paragraph alignment and font size/colour are not representable and are dropped.

mod list;
mod reader;
mod table;
mod writer;

pub use reader::decode;
pub use writer::encode;

#[cfg(test)]
mod tests {
    use wv_core::{CharStyle, Document, Paragraph, Run};

    use super::*;

    fn styled(text: &str, bold: bool, italic: bool, underline: bool) -> Run {
        Run::new(
            text,
            CharStyle {
                bold,
                italic,
                underline,
                ..Default::default()
            },
        )
    }

    #[test]
    fn round_trip_keeps_inline_styles_and_empty_paragraphs() {
        let mut doc = Document::from_text("");
        doc.paragraphs = vec![
            Paragraph {
                runs: vec![
                    styled("plain ", false, false, false),
                    styled("bold", true, false, false),
                    styled(" and ", false, false, false),
                    styled("both", true, true, false),
                    styled(" ", false, false, false),
                    styled("under", false, false, true),
                    styled(" ", false, false, false),
                    Run::new(
                        "gone",
                        CharStyle {
                            strike: true,
                            ..Default::default()
                        },
                    ),
                ],
                ..Default::default()
            },
            Paragraph::default(),
            Paragraph {
                runs: vec![
                    Run::new(
                        "code",
                        CharStyle {
                            mono: true,
                            ..Default::default()
                        },
                    ),
                    styled(" ", false, false, false),
                    Run::new(
                        "secret",
                        CharStyle {
                            spoiler: true,
                            ..Default::default()
                        },
                    ),
                    styled(" ", false, false, false),
                    Run::new(
                        "link",
                        CharStyle {
                            link: Some("https://example.com/a_(b)".replace("_(b)", "")),
                            bold: true,
                            ..Default::default()
                        },
                    ),
                ],
                style: wv_core::ParaStyle {
                    quote: true,
                    ..Default::default()
                },
            },
            Paragraph {
                runs: vec![styled("2 * 3 = 6, a_b, <tag>, \\", false, false, false)],
                ..Default::default()
            },
        ];
        assert_eq!(decode(&encode(&doc)), doc);
    }

    #[test]
    fn reads_headings_lists_and_soft_wrapped_lines() {
        let doc = decode("# Title\n\nline one\nline two\n\n- item");
        assert_eq!(doc.text(), "Title\nline one line two\nitem");
        assert_eq!(doc.paragraphs[0].style.named, wv_core::NamedStyle::Heading1);
    }

    #[test]
    fn spaces_stay_outside_markers() {
        let doc = Document {
            paragraphs: vec![Paragraph {
                runs: vec![styled(" bold ", true, false, false)],
                ..Default::default()
            }],
            ..Document::default()
        };
        assert_eq!(encode(&doc), " **bold** ");
    }

    #[test]
    fn stray_asterisks_are_literal() {
        assert_eq!(decode("a * b ** c").text(), "a * b ** c");
    }

    #[test]
    fn lists_round_trip_with_nesting_and_numbering() {
        use wv_core::ListKind;
        let mut doc = Document::from_text("intro\nfirst\nnested\nsecond\nbullet\nafter");
        for (i, kind, indent) in [
            (1, ListKind::Number, 0),
            (2, ListKind::Number, 1),
            (3, ListKind::Number, 0),
            (4, ListKind::Bullet, 0),
        ] {
            doc.paragraphs[i].style.list = Some(kind);
            doc.paragraphs[i].style.indent = indent;
        }
        let text = encode(&doc);
        assert!(
            text.contains("1. first\n  1. nested\n2. second\n- bullet"),
            "{text}"
        );
        assert_eq!(decode(&text), doc);
    }

    #[test]
    fn reads_foreign_lists_and_continuation_lines() {
        use wv_core::ListKind;
        let doc = decode("Shopping:\n* milk\n* bread\n  and butter\n3) tea\n\nlast");
        assert_eq!(doc.text(), "Shopping:\nmilk\nbread and butter\ntea\nlast");
        assert_eq!(doc.paragraphs[1].style.list, Some(ListKind::Bullet));
        assert_eq!(doc.paragraphs[3].style.list, Some(ListKind::Number));
        assert_eq!(doc.paragraphs[4].style.list, None);
    }

    #[test]
    fn tables_round_trip_and_foreign_tables_are_read() {
        use wv_core::Editor;
        let mut e = Editor::default();
        e.insert_text("before");
        e.insert_table(2, 2);
        e.insert_text("a | b");
        e.next_cell(true);
        e.next_cell(true);
        e.insert_text("c");
        let text = encode(&e.doc);
        assert!(
            text.contains("| a \\| b |  |\n| --- | --- |\n| c |  |"),
            "{text}"
        );
        assert_eq!(decode(&text), e.doc);

        let doc = decode("x\n\nName | Qty\n:-- | --:\nTea | 2\n\ny");
        let cells: Vec<_> = doc
            .paragraphs
            .iter()
            .map(|p| p.style.cell.map(|c| (c.row, c.col)))
            .collect();
        assert_eq!(doc.text(), "x\nName\nQty\nTea\n2\ny");
        assert_eq!(
            cells,
            [
                None,
                Some((0, 0)),
                Some((0, 1)),
                Some((1, 0)),
                Some((1, 1)),
                None
            ]
        );
    }
}
