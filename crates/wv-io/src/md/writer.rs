use wv_core::{CharStyle, Document, ListKind, NamedStyle, Run};

/// Paragraphs are separated by a blank line (list items by a single newline); an empty paragraph is
/// written as `&nbsp;`. List nesting is two spaces per level.
pub fn encode(doc: &Document) -> String {
    let mut out = String::new();
    let mut counters = [0u32; 9];
    let mut previous_was_item = false;
    for (i, p) in doc.paragraphs.iter().enumerate() {
        let text: String = if p.is_empty() {
            "&nbsp;".to_string()
        } else {
            p.runs.iter().map(run).collect()
        };
        let level = usize::from(p.style.indent).min(8);
        let line = match p.style.list {
            Some(ListKind::Bullet) => {
                counters[level..].fill(0);
                format!("{}- {text}", "  ".repeat(level))
            }
            Some(ListKind::Number) => {
                counters[level] += 1;
                counters[level + 1..].fill(0);
                format!("{}{}. {text}", "  ".repeat(level), counters[level])
            }
            None => {
                counters = [0; 9];
                match p.style.named {
                    NamedStyle::Title | NamedStyle::Heading1 => format!("# {text}"),
                    NamedStyle::Heading2 => format!("## {text}"),
                    NamedStyle::Heading3 => format!("### {text}"),
                    _ if p.style.quote => format!("> {text}"),
                    _ => text,
                }
            }
        };
        let is_item = p.style.list.is_some();
        if i > 0 {
            out.push_str(if previous_was_item && is_item {
                "\n"
            } else {
                "\n\n"
            });
        }
        out.push_str(&line);
        previous_was_item = is_item;
    }
    out
}

fn run(run: &Run) -> String {
    let text = &run.text;
    let core = text.trim();
    if core.is_empty() {
        return text.clone();
    }
    let lead = &text[..text.len() - text.trim_start().len()];
    let trail = &text[text.trim_end().len()..];
    let mark = emphasis(&run.style);
    let inner = if run.style.mono {
        format!("`{core}`")
    } else {
        escape(core)
    };
    let body = format!("{mark}{inner}{mark}");
    let body = if run.style.spoiler {
        format!("||{body}||")
    } else {
        body
    };
    let body = match &run.style.link {
        Some(url) => format!("[{body}]({url})"),
        None => body,
    };
    let body = if run.style.strike {
        format!("~~{body}~~")
    } else {
        body
    };
    let body = if run.style.underline {
        format!("<u>{body}</u>")
    } else {
        body
    };
    format!("{lead}{body}{trail}")
}

fn emphasis(style: &CharStyle) -> &'static str {
    match (style.bold, style.italic) {
        (true, true) => "***",
        (true, false) => "**",
        (false, true) => "*",
        (false, false) => "",
    }
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if matches!(ch, '\\' | '*' | '<' | '~' | '[' | ']' | '`' | '|') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}
