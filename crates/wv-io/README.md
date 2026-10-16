# wv-io
File formats. One folder per format, each with `decode`/`encode`.

| Format | Extension | Notes |
| --- | --- | --- |
| WordVolt native | `.wvd` | JSON with a version header; keeps all formatting |
| Plain text | `.txt` | UTF-8 / UTF-16 (BOM) in, UTF-8 out; formatting dropped |
| Markdown | `.md` | `**bold**`, `*italic*`, `<u>underline</u>`, `# headings` (read as bold); alignment is not kept |

Entry points: `load_file`, `save_file`, `Format::from_path`. Saving is atomic (temp file + rename).
Planned: docx, rtf, html, odt, pdf (see plan.md).
