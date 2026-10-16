# wv-core
Document model and editing logic. No UI dependencies.

- `document/` Document -> Paragraph -> Run (text + CharStyle). Offsets are UTF-8 byte offsets on char boundaries.
- `style/` CharStyle (bold/italic/underline/size/colour), ParaStyle (alignment).
- `position/` Position (paragraph, offset), Selection (anchor, head).
- `history/` Edit = one reversible change (paragraph splice); undo/redo stacks.
- `editor/` Editor = document + selection + history. All edits go through `Editor::replace_range`
  or `Editor::edit_paragraphs`, so undo/redo always works.

Not here: layout, rendering, file formats, up/down caret movement (needs layout).
