use wv_core::{CharStyle, Document, Run};

/// Paragraphs are separated by a blank line; an empty paragraph is written as `&nbsp;`.
pub fn encode(doc: &Document) -> String {
    doc.paragraphs
        .iter()
        .map(|p| {
            let text: String = if p.is_empty() {
                "&nbsp;".to_string()
            } else {
                p.runs.iter().map(run).collect()
            };
            if p.style.quote {
                format!("> {text}")
            } else {
                text
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n")
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
