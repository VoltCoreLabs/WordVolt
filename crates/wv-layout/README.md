# wv-layout
Turns a `wv-core` document into lines and pages. No UI dependency: text is measured through the
`Measurer` trait (the UI implements it with GPUI's text system; tests use a fixed-width fake).

- `measure.rs` — `Measurer` trait, font size helpers.
- `line.rs` — `Line`/`Fragment`: positions inside a line (`x_at`, `index_at`).
- `paragraph.rs` — line breaking (UAX #14), per-run font sizes, alignment incl. justify, line spacing.
- `page.rs` — `PageSetup` (A4 by default) and pagination of laid-out paragraphs.
- `document.rs` — `DocLayout`: whole-document layout, caret rectangles, hit-testing, selection rectangles.

All lengths are logical pixels (96 per inch). Not yet: tabs stops, widow/orphan control, bidi, hyphenation.
