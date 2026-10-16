use gpui_kit::*;

actions!(
    wordvolt,
    [
        NewDocument,
        OpenDocument,
        SaveDocument,
        SaveDocumentAs,
        InsertLink
    ]
);

/// Shortcuts: Ctrl on Windows/Linux, Cmd on macOS.
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("secondary-n", NewDocument, None),
        KeyBinding::new("secondary-o", OpenDocument, None),
        KeyBinding::new("secondary-s", SaveDocument, None),
        KeyBinding::new("secondary-shift-s", SaveDocumentAs, None),
        KeyBinding::new("secondary-k", InsertLink, None),
    ]);
}
