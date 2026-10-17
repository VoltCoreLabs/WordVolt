use gpui_kit::*;

actions!(
    wordvolt,
    [
        NewDocument,
        OpenDocument,
        SaveDocument,
        SaveDocumentAs,
        InsertLink,
        FindInDocument,
        ReplaceInDocument,
        FindNext,
        FindPrevious,
        EditorTab,
        EditorTabBack,
        FieldNext,
        NewTab,
        PrintDocument,
        CloseTab,
        NextTab,
        PrevTab
    ]
);

/// Shortcuts: Ctrl on Windows/Linux, Cmd on macOS.
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("secondary-n", NewDocument, None),
        KeyBinding::new("secondary-o", OpenDocument, None),
        KeyBinding::new("secondary-s", SaveDocument, None),
        KeyBinding::new("secondary-shift-s", SaveDocumentAs, None),
        KeyBinding::new("secondary-t", NewTab, None),
        KeyBinding::new("secondary-p", PrintDocument, None),
        KeyBinding::new("secondary-w", CloseTab, None),
        KeyBinding::new("ctrl-tab", NextTab, None),
        KeyBinding::new("ctrl-shift-tab", PrevTab, None),
        KeyBinding::new("secondary-k", InsertLink, None),
        KeyBinding::new("secondary-f", FindInDocument, None),
        KeyBinding::new("secondary-h", ReplaceInDocument, None),
        KeyBinding::new("f3", FindNext, None),
        KeyBinding::new("shift-f3", FindPrevious, None),
        // Win over the root's focus traversal while the page has focus.
        KeyBinding::new("tab", FieldNext, Some("TabField")),
        KeyBinding::new("shift-tab", FieldNext, Some("TabField")),
        KeyBinding::new("tab", EditorTab, Some("Editor")),
        KeyBinding::new("shift-tab", EditorTabBack, Some("Editor")),
    ]);
}
