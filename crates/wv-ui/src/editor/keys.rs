use gpui_kit::*;

use super::view::EditorView;

impl EditorView {
    pub(super) fn on_key_down(
        &mut self,
        ev: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let k = &ev.keystroke;
        let m = k.modifiers;
        let shift = m.shift;
        // Ctrl on Windows/Linux, Cmd on macOS. AltGr (ctrl+alt) is just typing.
        let command = (m.control && !m.alt) || m.platform;
        match (k.key.as_str(), command) {
            ("enter", true) => self.apply(cx, |c| c.insert_page_break()),
            ("enter", _) => self.apply(cx, |c| c.enter()),
            ("backspace", _) => self.apply(cx, |c| c.backspace()),
            ("delete", _) => self.apply(cx, |c| c.delete_forward()),
            ("left", _) => self.apply(cx, |c| c.move_left(shift)),
            ("right", _) => self.apply(cx, |c| c.move_right(shift)),
            ("home", true) => self.apply(cx, |c| c.move_doc_start(shift)),
            ("end", true) => self.apply(cx, |c| c.move_doc_end(shift)),
            ("home", _) => self.move_line_edge(false, shift, cx),
            ("end", _) => self.move_line_edge(true, shift, cx),
            ("up", _) => self.move_vertical(-1, shift, window, cx),
            ("down", _) => self.move_vertical(1, shift, window, cx),
            // Telegram-style: Ctrl+Shift+X strike, +M code, +P spoiler, +N plain text.
            ("x", true) if shift => self.apply(cx, |c| c.toggle_strike()),
            ("m", true) if shift => self.apply(cx, |c| c.toggle_mono()),
            ("p", true) if shift => self.apply(cx, |c| c.toggle_spoiler()),
            ("n", true) if shift => self.apply(cx, |c| c.clear_formatting()),
            ("a", true) => self.apply(cx, |c| c.select_all()),
            ("z", true) if shift => self.apply(cx, |c| _ = c.redo()),
            ("z", true) => self.apply(cx, |c| _ = c.undo()),
            ("y", true) => self.apply(cx, |c| _ = c.redo()),
            ("b", true) => self.apply(cx, |c| c.toggle_bold()),
            ("i", true) => self.apply(cx, |c| c.toggle_italic()),
            ("u", true) => self.apply(cx, |c| c.toggle_underline()),
            ("c", true) => self.copy_selection(cx),
            ("x", true) => self.cut_selection(cx),
            ("v", true) if shift => self.paste_plain(cx),
            ("v", true) => self.paste_clipboard(cx),
            // Typed characters arrive through the text input (`ime.rs`).
            _ => {}
        }
    }
}
