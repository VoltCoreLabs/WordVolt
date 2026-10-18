use gpui_kit::*;
use wv_core::Paragraph;

use super::view::EditorView;

/// What was last copied inside the app: the plain text that went to the system clipboard and the
/// formatted paragraphs. A paste whose clipboard text is still that text keeps the formatting.
struct ClipFragment {
    plain: String,
    paras: Vec<Paragraph>,
}

impl Global for ClipFragment {}

impl EditorView {
    pub fn cut_selection(&mut self, cx: &mut Context<Self>) {
        self.copy_selection(cx);
        self.apply(cx, |c| _ = c.delete_selection());
    }

    pub fn copy_selection(&mut self, cx: &mut Context<Self>) {
        let text = self.core.selected_text();
        if !text.is_empty() {
            let paras = self.core.copy_fragment();
            // The footnote mark characters are not text.
            let text = text.replace(wv_core::NOTE_MARK, "");
            cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
            cx.set_global(ClipFragment { plain: text, paras });
        }
    }

    /// Pastes with the formatting when the clipboard still holds what the app copied.
    pub fn paste_clipboard(&mut self, cx: &mut Context<Self>) {
        self.paste(true, cx);
    }

    /// Ctrl+Shift+V: the text only.
    pub fn paste_plain(&mut self, cx: &mut Context<Self>) {
        self.paste(false, cx);
    }

    fn paste(&mut self, keep_format: bool, cx: &mut Context<Self>) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        let text = text.replace("\r\n", "\n").replace('\r', "\n");
        let rich = keep_format
            .then(|| cx.try_global::<ClipFragment>())
            .flatten()
            .filter(|c| c.plain == text)
            .map(|c| c.paras.clone());
        match rich {
            Some(paras) => self.apply(cx, |c| c.paste_fragment(&paras)),
            None => self.apply(cx, |c| c.insert_text(&text)),
        }
    }
}
