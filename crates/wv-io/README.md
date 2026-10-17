# wv-io
File formats. One folder per format, each with `decode`/`encode`.

| Format | Extension | Notes |
| --- | --- | --- |
| WordVolt native | `.wvd` | JSON with a version header; keeps all formatting |
| Word | `.docx` | paragraphs, bold/italic/underline, size, colour, alignment, Heading styles (as bold); no tables/images/lists yet |
| Plain text | `.txt` | UTF-8 / UTF-16 (BOM) in, UTF-8 out; formatting dropped |
| Markdown | `.md` | `**bold**`, `*italic*`, `<u>underline</u>`, `# headings` (read as bold); alignment is not kept |

Entry points: `load_file`, `save_file`, `Format::from_path`. Saving is atomic (temp file + rename).
PDF: `encode_pdf` (export only). Planned: rtf, html, odt (see plan.md).
