use std::ops::Range;

use gpui_kit::*;
use wv_core::Position;

use super::measure::GpuiMeasurer;
use super::view::EditorView;

/// Byte offset in `text` of a UTF-16 offset (what the platform text input speaks).
fn byte_of(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (i, c) in text.char_indices() {
        if units >= utf16 {
            return i;
        }
        units += c.len_utf16();
    }
    text.len()
}

fn utf16_of(text: &str, byte: usize) -> usize {
    let byte = byte.min(text.len());
    text[..byte].encode_utf16().count()
}

/// Typed and composed text arrives here (the platform text input: layouts, dead keys, IME of
/// Chinese/Japanese/Korean, emoji picker). Offsets are UTF-16 inside the paragraph of the caret.
impl EntityInputHandler for EditorView {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let text = self.core.doc.paragraphs[self.core.sel.head.para].text();
        let (a, b) = (byte_of(&text, range.start), byte_of(&text, range.end));
        *adjusted = Some(utf16_of(&text, a)..utf16_of(&text, b));
        Some(text[a..b.max(a)].to_string())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let sel = self.core.sel;
        let text = self.core.doc.paragraphs[sel.head.para].text();
        let anchor = if sel.anchor.para == sel.head.para {
            sel.anchor.offset
        } else {
            sel.head.offset
        };
        let (a, b) = (utf16_of(&text, anchor), utf16_of(&text, sel.head.offset));
        Some(UTF16Selection {
            range: a.min(b)..a.max(b),
            reversed: b < a,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        let (para, range) = self.marked.clone()?;
        let text = self.core.doc.paragraphs.get(para)?.text();
        Some(utf16_of(&text, range.start)..utf16_of(&text, range.end))
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked = None;
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace(range, text, cx);
        self.marked = None;
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        _: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (start, end) = self.ime_target(range);
        let text: String = new_text.chars().filter(|c| !c.is_control()).collect();
        self.apply(cx, |c| c.replace_range(start, end, &text));
        self.marked =
            (!text.is_empty()).then(|| (start.para, start.offset..start.offset + text.len()));
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        window: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let para = self.core.sel.head.para;
        let text = self.core.doc.paragraphs[para].text();
        let at = Position::new(para, byte_of(&text, range.start));
        let layout = self.layout.clone()?;
        let m = GpuiMeasurer::new(window, self.measure_cache.clone());
        let caret = layout.caret(at, &m)?;
        let page = self.page_bounds.borrow().get(caret.page).copied()?;
        Some(Bounds::new(
            point(
                page.origin.x + px(caret.x * self.zoom),
                page.origin.y + px(caret.y * self.zoom),
            ),
            size(px(1.), px(caret.height * self.zoom)),
        ))
    }

    fn character_index_for_point(
        &mut self,
        _: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }
}

impl EditorView {
    /// What a text input edit replaces: the given UTF-16 range of the caret's paragraph, else the
    /// composition being typed, else the selection.
    fn ime_target(&self, range: Option<Range<usize>>) -> (Position, Position) {
        let para = self.core.sel.head.para;
        if let Some(r) = range {
            let text = self.core.doc.paragraphs[para].text();
            return (
                Position::new(para, byte_of(&text, r.start)),
                Position::new(para, byte_of(&text, r.end)),
            );
        }
        match &self.marked {
            Some((p, r)) if *p < self.core.doc.paragraphs.len() => (
                Position::new(*p, r.start),
                Position::new(*p, r.end.min(self.core.doc.paragraphs[*p].len())),
            ),
            _ => self.core.sel.range(),
        }
    }

    /// Types `text` over the target of the edit.
    fn replace(&mut self, range: Option<Range<usize>>, text: &str, cx: &mut Context<Self>) {
        let (start, end) = self.ime_target(range);
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        if text.is_empty() && start == end {
            return;
        }
        self.apply(cx, |c| c.replace_range(start, end, &text));
    }
}
