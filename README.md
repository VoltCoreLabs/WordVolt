<div align="center">

# ⚡ WordVolt

**A fast, native word processor written in pure Rust.**
GPU-rendered with [GPUI](https://www.gpui.rs/), built around a clean block-based interface.

![Rust](https://img.shields.io/badge/Rust-2024_edition-orange?logo=rust)
![Status](https://img.shields.io/badge/status-early_development-blue)
![Platforms](https://img.shields.io/badge/platforms-Windows_%C2%B7_Linux_%C2%B7_macOS-lightgrey)
![License](https://img.shields.io/badge/license-Apache--2.0-green)

</div>

---

WordVolt is an attempt to build a modern alternative to Microsoft Word: a document editor that starts instantly,
scrolls smoothly, and does not carry decades of legacy UI. No Electron, no web view — just Rust drawing straight to the GPU.

> **Status: early development.** The editing core and the interface foundation work today; pagination, file formats
> and rich objects are on the roadmap below. Expect rough edges and breaking changes.

## ✨ What works today

| Area | Details |
| --- | --- |
| **Editing** | Typing (Latin, Cyrillic), Enter, Backspace/Delete by whole grapheme, multi-paragraph selection, clipboard (cut/copy/paste) |
| **Navigation** | Arrow keys (also line up/down with column memory), Home/End, Ctrl+Home/End, click, drag, double-click (word), triple-click (paragraph), Shift-select |
| **Polish** | Animated tooltips on every ribbon button (with shortcuts; gpui-kit's tooltip system: appear with a slide and fade, glide between neighbouring buttons), recent colours saved between sessions, *Change case* menu (UPPER / lower / Title / Sentence, real text changes), a ribbon that wraps its groups on narrow windows |
| **Telegram-style text** | Bold, italic, underline, ~~strikethrough~~, `inline code`, **spoiler** (hidden until the caret enters it), block quote, hyperlinks (Ctrl+click opens), superscript / subscript, ALL CAPS, text colour and highlight with a full colour picker (standard palette, recent colours, free picker, hex) |
| **Formatting** | Bold, italic, underline, font family (type a name or pick from the installed fonts), font size (A− / A+), paragraph alignment (left / center / right / justify) — from the ribbon or with <kbd>Ctrl</kbd>+<kbd>B</kbd> / <kbd>I</kbd> / <kbd>U</kbd> |
| **Files** | New / Open / Save / Save as (<kbd>Ctrl</kbd>+<kbd>N</kbd> / <kbd>O</kbd> / <kbd>S</kbd> / <kbd>Shift</kbd>+<kbd>S</kbd>); the Save dialog lets you pick the format: native `.wvd`, Word `.docx`, plain `.txt`, Markdown `.md`; asks before discarding unsaved changes |
| **History** | Unlimited undo / redo (<kbd>Ctrl</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd>) |
| **Pages** | Paged A4 view with real pagination: text flows onto new pages, mixed font sizes in one line, justified text, caret and selection across pages |
| **Interface** | Custom title bar with window controls, ribbon with tabs, collapsible ribbon and side panel, status bar with live word count |
| **Feel** | Smooth eased scrolling, animated caret (fade-in + soft blink), animated tabs and panels |
| **Languages** | UI strings in Russian (default) and English; a language switch is coming |
| **Dev experience** | Edit code or theme colors → the running app updates (see [Development](#-development)) |

## 🎨 Design

The UI is a **block system**: a light window background with white panels, and filled buttons with a thin outline on top.
Everything is built from a small set of reusable components, and every color, radius and spacing comes from design tokens —
which is what will make the upcoming dark theme a drop-in replacement rather than a rewrite.

- Accent: blue `#2563EB` · radii 8 / 12 / 16 px · comfortable 32 px controls
- Icons: [Phosphor](https://phosphoricons.com/) · UI font: SF Pro Text (if installed) → Inter → Segoe UI

Details live in [`docs/design.md`](docs/design.md).

## 🚀 Getting started

**Requirements**

- Rust (stable, edition 2024) — install via [rustup](https://rustup.rs/)
- Windows: Visual Studio Build Tools with the *Desktop development with C++* workload
- Linux / macOS: the system libraries GPUI needs (see the [Zed build docs](https://zed.dev/docs/development))

```bash
git clone https://github.com/VoltCoreLabs/WordVolt.git
cd WordVolt
cargo run --release
```

The first build compiles GPUI and takes a few minutes; later builds are incremental.

### Keyboard shortcuts

| Action | Shortcut |
| --- | --- |
| Select all | <kbd>Ctrl</kbd>+<kbd>A</kbd> |
| Cut / Copy / Paste | <kbd>Ctrl</kbd>+<kbd>X</kbd> / <kbd>C</kbd> / <kbd>V</kbd> |
| Undo / Redo | <kbd>Ctrl</kbd>+<kbd>Z</kbd> / <kbd>Ctrl</kbd>+<kbd>Y</kbd> (or <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>) |
| Bold / Italic / Underline | <kbd>Ctrl</kbd>+<kbd>B</kbd> / <kbd>I</kbd> / <kbd>U</kbd> |
| Strikethrough / Code / Spoiler | <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>X</kbd> / <kbd>M</kbd> / <kbd>P</kbd> |
| Insert / edit a link | <kbd>Ctrl</kbd>+<kbd>K</kbd> |
| Clear formatting | <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>N</kbd> |
| Open a link | <kbd>Ctrl</kbd>+click |
| Start / end of line | <kbd>Home</kbd> / <kbd>End</kbd> |
| Start / end of document | <kbd>Ctrl</kbd>+<kbd>Home</kbd> / <kbd>End</kbd> |

(On macOS, <kbd>Cmd</kbd> replaces <kbd>Ctrl</kbd>.)

## 🛠 Development

```bash
cargo dev        # rebuild and restart the app every time you save a file (needs cargo-watch)
cargo test       # run all tests
cargo clippy --workspace --all-targets -- -D warnings
```

Install the watcher once with `cargo install cargo-watch`.

**Live theme editing.** In debug builds the app watches [`assets/theme/light.toml`](assets/theme/light.toml):
change a color, save, and the open window updates instantly — no rebuild.

## 🧱 Architecture

A Cargo workspace with strictly one-way dependencies, so the document logic never depends on the UI:

```
app/                 binary entry point (a few lines)
crates/
├─ wv-core/          document model, selection, editing commands, undo/redo   (no UI)
├─ wv-layout/        text layout and pagination                               (planned)
├─ wv-io/            file formats: wvd, docx, txt, md today; rtf, html, odt, pdf planned
├─ wv-i18n/          UI strings (en, ru)
└─ wv-ui/            GPUI interface: theme tokens, ui_kit components, editor, ribbon
```

- **Document model:** `Document → Paragraph → Run (text + style)`; positions are UTF-8 byte offsets on grapheme boundaries.
- **Undo/redo:** every change is a reversible paragraph splice, so any operation is undoable by construction.
- **Small files:** source files stay under ~300 lines; new features are new modules, not bigger files.

See [`docs/structure.md`](docs/structure.md) for the full file tree and [`docs/decisions.md`](docs/decisions.md) for why things are the way they are.

## 🗺 Roadmap

**Long-term goal:** a full-featured Word alternative — every tab of the ribbon — plus features of its own
(command palette, document tabs, scripting instead of VBA, version history, …). The full map is in [`plan.md`](plan.md#7-видение-полный-клон-word--свои-функции).

Next up:

- [x] Workspace, theme tokens, reusable block UI kit
- [x] Document model with undo/redo
- [x] Live editor: typing, caret, selection, clipboard, formatting
- [x] Custom title bar, ribbon, animations, smooth scrolling
- [x] Layout engine and paged view: pagination, font sizes, justified text
- [ ] Colour pickers, tab stops, indents, line spacing controls, page setup (size, margins)
- [x] Open / save: native `.wvd`, Word `.docx` (basic), `.txt`, `.md`
- [ ] Full `.docx` (lists, tables, images, headers) and `.rtf`, `.html`, `.odt`
- [ ] IME input, lists, styles (Heading 1–3), find & replace, spell check
- [ ] Tables, images, headers & footers, page numbers
- [ ] PDF export and printing
- [ ] Dark theme
- [ ] Footnotes, table of contents, comments, track changes

The living plan with task status is in [`plan.md`](plan.md).

## 🤝 Contributing

Work happens on short-lived branches (`feat/…`, `fix/…`, `docs/…`) merged through pull requests.
Before opening one, make sure `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` pass.
Project conventions (file size limits, design tokens, documentation of decisions) are written down in [`CLAUDE.md`](CLAUDE.md).

## 📦 Third-party assets

- **GPUI** and **gpui-component** via [gpui-kit](https://crates.io/crates/gpui-kit) — Apache-2.0
- **[Phosphor Icons](https://phosphoricons.com/)** — MIT (`assets/icons/phosphor/LICENSE`)
- **[Inter](https://rsms.me/inter/)** — SIL Open Font License 1.1 (`assets/fonts/LICENSE-Inter.txt`)
- **SF Pro Text** is Apple's proprietary font and is **not** distributed with this repository. If it is installed on your
  machine (or placed in `assets/fonts/` locally) WordVolt uses it; otherwise it falls back to Inter.

## 📄 License

Licensed under the [Apache License 2.0](LICENSE).
