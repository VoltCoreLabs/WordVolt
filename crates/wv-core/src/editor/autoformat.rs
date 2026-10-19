use std::ops::Range;

use super::Editor;
use crate::position::{Position, Selection};
use crate::style::{ListKind, NamedStyle};

/// What a Markdown marker typed at the start of a paragraph turns it into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Block {
    Heading(NamedStyle),
    List(ListKind),
    Quote,
}

/// The block the text before the caret asks for: only a marker and the space just typed.
/// `# `..`###### ` headings, `- `/`* `/`+ ` bullets, `1. `/`1) ` numbers, `> ` quote.
pub(super) fn block_marker(before: &str) -> Option<Block> {
    let marker = before.strip_suffix(' ')?;
    if marker.is_empty() || marker.contains(char::is_whitespace) {
        return None;
    }
    if marker.chars().all(|c| c == '#') {
        return Some(Block::Heading(match marker.len() {
            1 => NamedStyle::Heading1,
            2 => NamedStyle::Heading2,
            3 => NamedStyle::Heading3,
            4 => NamedStyle::Heading4,
            5 => NamedStyle::Heading5,
            6 => NamedStyle::Heading6,
            _ => return None,
        }));
    }
    match marker {
        "-" | "*" | "+" => return Some(Block::List(ListKind::Bullet)),
        ">" => return Some(Block::Quote),
        _ => {}
    }
    let digits = marker.strip_suffix(['.', ')'])?;
    (!digits.is_empty() && digits.len() <= 9 && digits.chars().all(|c| c.is_ascii_digit()))
        .then_some(Block::List(ListKind::Number))
}

/// Inline formatting closed by the last typed character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Inline {
    Bold,
    Italic,
    Strike,
    Mono,
}

/// A `**bold**`-like span found at the end of the text before the caret: the byte ranges of its
/// opening and closing delimiters (the text between them is the content).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Span {
    pub kind: Inline,
    pub open: Range<usize>,
    pub close: Range<usize>,
}

const DELIMITERS: [(&str, Inline); 5] = [
    ("**", Inline::Bold),
    ("~~", Inline::Strike),
    ("`", Inline::Mono),
    ("*", Inline::Italic),
    ("_", Inline::Italic),
];

/// The span `before` ends with, if the character typed last closed one. Content must be
/// non-empty, without spaces at its edges; `*`/`_` open only after a space or punctuation (so
/// `snake_case_name` is left alone).
pub(super) fn inline_span(before: &str) -> Option<Span> {
    for (delim, kind) in DELIMITERS {
        let Some(head) = before.strip_suffix(delim) else {
            continue;
        };
        let single = delim.len() == 1 && delim != "`";
        // A single `*` right after another `*` belongs to a `**` that is not finished.
        if single && head.ends_with(delim) {
            continue;
        }
        let Some(open) = head.rfind(delim) else {
            continue;
        };
        let content = &head[open + delim.len()..];
        if content.is_empty()
            || content.starts_with(char::is_whitespace)
            || content.ends_with(char::is_whitespace)
            || (single && content.contains(delim))
        {
            continue;
        }
        let before_open = head[..open].chars().next_back();
        if single
            && before_open
                .is_some_and(|c| c.is_alphanumeric() || c == delim.chars().next().unwrap_or(' '))
        {
            continue;
        }
        return Some(Span {
            kind,
            open: open..open + delim.len(),
            close: head.len()..before.len(),
        });
    }
    None
}

impl Editor {
    /// Markdown shortcuts while typing: call it after the character `typed` was inserted at the
    /// caret. A marker at the start of a paragraph followed by a space becomes a heading, list or
    /// quote; a closed `**bold**`, `*italic*`, `` `code` `` or `~~strike~~` loses its delimiters
    /// and gets the formatting. Returns whether the text was changed.
    pub fn autoformat_typed(&mut self, typed: char) -> bool {
        if !self.sel.is_collapsed() {
            return false;
        }
        let at = self.sel.head;
        let para = &self.doc.paragraphs[at.para];
        if para.style.image.is_some() || at.offset > para.len() {
            return false;
        }
        let text = para.text();
        let Some(before) = text.get(..at.offset) else {
            return false;
        };
        if typed == ' ' {
            self.autoformat_block(at, before)
        } else if matches!(typed, '*' | '_' | '`' | '~') {
            self.autoformat_inline(at, before)
        } else {
            false
        }
    }

    fn autoformat_block(&mut self, at: Position, before: &str) -> bool {
        let style = &self.doc.paragraphs[at.para].style;
        let Some(block) = block_marker(before) else {
            return false;
        };
        let fresh = style.list.is_none() && !style.quote && style.named == NamedStyle::Normal;
        if !fresh {
            return false;
        }
        self.replace_range(Position::new(at.para, 0), at, "");
        match block {
            Block::Heading(named) => self.set_named_style(named),
            Block::List(kind) => self.toggle_list(kind),
            Block::Quote => self.toggle_quote(),
        }
        true
    }

    fn autoformat_inline(&mut self, at: Position, before: &str) -> bool {
        let Some(span) = inline_span(before) else {
            return false;
        };
        let para = at.para;
        // The closing delimiter first, so the earlier offsets stay valid.
        self.replace_range(
            Position::new(para, span.close.start),
            Position::new(para, span.close.end),
            "",
        );
        self.replace_range(
            Position::new(para, span.open.start),
            Position::new(para, span.open.end),
            "",
        );
        let content_end = span.close.start - span.open.len();
        let (start, end) = (
            Position::new(para, span.open.start),
            Position::new(para, content_end),
        );
        self.sel = Selection {
            anchor: start,
            head: end,
        };
        match span.kind {
            Inline::Bold => self.toggle_bold(),
            Inline::Italic => self.toggle_italic(),
            Inline::Strike => self.toggle_strike(),
            Inline::Mono => self.toggle_mono(),
        }
        self.sel = Selection::caret(end);
        // What is typed after the span goes on without its formatting.
        let mut next = self.doc.paragraphs[para].style_at(end.offset);
        match span.kind {
            Inline::Bold => next.bold = false,
            Inline::Italic => next.italic = false,
            Inline::Strike => next.strike = false,
            Inline::Mono => next.mono = false,
        }
        self.pending_style = Some(next);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{Block, Inline, block_marker, inline_span};
    use crate::editor::Editor;
    use crate::style::{ListKind, NamedStyle};

    fn type_str(e: &mut Editor, text: &str) {
        for ch in text.chars() {
            e.insert_text(&ch.to_string());
            e.autoformat_typed(ch);
        }
    }

    #[test]
    fn markers_at_the_start_make_blocks() {
        assert_eq!(
            block_marker("# "),
            Some(Block::Heading(NamedStyle::Heading1))
        );
        assert_eq!(
            block_marker("### "),
            Some(Block::Heading(NamedStyle::Heading3))
        );
        assert_eq!(block_marker("####### "), None);
        assert_eq!(block_marker("- "), Some(Block::List(ListKind::Bullet)));
        assert_eq!(block_marker("12. "), Some(Block::List(ListKind::Number)));
        assert_eq!(block_marker("3) "), Some(Block::List(ListKind::Number)));
        assert_eq!(block_marker("> "), Some(Block::Quote));
        assert_eq!(block_marker("слово - "), None);
        assert_eq!(block_marker("-"), None);
        assert_eq!(block_marker(". "), None);
    }

    #[test]
    fn closed_spans_are_found() {
        let s = inline_span("a **жирный**").unwrap();
        assert_eq!((s.kind, s.open, s.close), (Inline::Bold, 2..4, 16..18));
        assert_eq!(inline_span("x *курсив*").unwrap().kind, Inline::Italic);
        assert_eq!(inline_span("`code`").unwrap().kind, Inline::Mono);
        assert_eq!(inline_span("~~старое~~").unwrap().kind, Inline::Strike);
        assert!(inline_span("snake_case_").is_none());
        assert!(inline_span("2 * 3 *").is_none());
        assert!(inline_span("**пусто**").is_some());
        assert!(inline_span("** x**").is_none());
        assert!(inline_span("**").is_none());
        // `**bold*` is not finished: the single star must not close an italic span.
        assert!(inline_span("**bold*").is_none());
    }

    #[test]
    fn typing_a_heading_marker_styles_the_paragraph() {
        let mut e = Editor::default();
        type_str(&mut e, "## Заголовок");
        assert_eq!(e.doc.text(), "Заголовок");
        assert_eq!(e.named_style(), NamedStyle::Heading2);
    }

    #[test]
    fn typing_list_and_quote_markers() {
        let mut e = Editor::default();
        type_str(&mut e, "- пункт");
        assert_eq!(e.doc.text(), "пункт");
        assert_eq!(e.list_kind(), Some(ListKind::Bullet));
        let mut n = Editor::default();
        type_str(&mut n, "1. раз");
        assert_eq!(n.list_kind(), Some(ListKind::Number));
        let mut q = Editor::default();
        type_str(&mut q, "> цитата");
        assert!(q.quote_active());
        assert_eq!(q.doc.text(), "цитата");
    }

    #[test]
    fn a_marker_inside_text_or_a_list_stays() {
        let mut e = Editor::default();
        type_str(&mut e, "a - b");
        assert_eq!(e.doc.text(), "a - b");
        let mut l = Editor::default();
        type_str(&mut l, "- x");
        l.enter();
        type_str(&mut l, "- y");
        assert_eq!(l.doc.text(), "x\n- y");
    }

    #[test]
    fn typing_bold_and_italic_spans() {
        let mut e = Editor::default();
        type_str(&mut e, "Это **жирный** текст");
        assert_eq!(e.doc.text(), "Это жирный текст");
        let bold_at = |e: &Editor, i: usize| e.doc.paragraphs[0].style_at(i).bold;
        assert!(bold_at(&e, "Это ж".len()));
        assert!(!bold_at(&e, "Это жирный т".len()));
        let mut i = Editor::default();
        type_str(&mut i, "*мир*!");
        assert_eq!(i.doc.text(), "мир!");
        assert!(i.doc.paragraphs[0].style_at("м".len()).italic);
        assert!(!i.doc.paragraphs[0].style_at("мир!".len()).italic);
    }

    #[test]
    fn typing_code_and_strike_and_undo() {
        let mut e = Editor::default();
        type_str(&mut e, "`x`");
        assert_eq!(e.doc.text(), "x");
        assert!(e.doc.paragraphs[0].style_at(1).mono);
        let mut s = Editor::default();
        type_str(&mut s, "~~да~~");
        assert_eq!(s.doc.text(), "да");
        assert!(s.doc.paragraphs[0].style_at(1).strike);
        assert!(s.undo());
        assert!(!s.doc.paragraphs[0].style_at("д".len()).strike);
    }
}
