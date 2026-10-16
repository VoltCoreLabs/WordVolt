use wv_core::{CharStyle, Document, Paragraph, Run};

/// Blocks are separated by blank lines; lines inside a block are joined with a space.
pub fn decode(text: &str) -> Document {
    let mut paragraphs = Vec::new();
    let mut block: Vec<&str> = Vec::new();
    for line in text.lines().chain(std::iter::once("")) {
        if line.trim().is_empty() {
            if !block.is_empty() {
                paragraphs.push(paragraph(&block.join(" ")));
                block.clear();
            }
        } else {
            block.push(line.trim_end());
        }
    }
    if paragraphs.is_empty() {
        Document::default()
    } else {
        Document { paragraphs }
    }
}

fn paragraph(block: &str) -> Paragraph {
    if block.trim() == "&nbsp;" {
        return Paragraph::default();
    }
    let (text, heading) = match block.trim_start_matches('#') {
        rest if rest.len() < block.len()
            && block.len() - rest.len() <= 6
            && rest.starts_with(' ') =>
        {
            (rest.trim_start(), true)
        }
        _ => (block, false),
    };
    let mut p = Paragraph {
        runs: inline(text, heading),
        ..Default::default()
    };
    p.runs.retain(|r| !r.text.is_empty());
    p
}

fn inline(text: &str, heading: bool) -> Vec<Run> {
    let chars: Vec<char> = text.chars().collect();
    let (mut bold_on, mut italic_on, mut underline) = (false, false, false);
    let (mut runs, mut buf) = (Vec::new(), String::new());
    let mut i = 0;
    let mut flush = |buf: &mut String, bold_on: bool, italic_on: bool, underline: bool| {
        if !buf.is_empty() {
            let style = CharStyle {
                bold: heading || bold_on,
                italic: italic_on,
                underline,
                ..Default::default()
            };
            runs.push(Run::new(std::mem::take(buf), style));
        }
    };
    while i < chars.len() {
        let rest: String = chars[i..chars.len().min(i + 4)].iter().collect();
        if chars[i] == '\\' && i + 1 < chars.len() && matches!(chars[i + 1], '\\' | '*' | '<') {
            buf.push(chars[i + 1]);
            i += 2;
        } else if rest.starts_with("<u>") || rest.starts_with("</u>") {
            flush(&mut buf, bold_on, italic_on, underline);
            underline = rest.starts_with("<u>");
            i += if underline { 3 } else { 4 };
        } else if chars[i] == '*' {
            let n = chars[i..].iter().take(3).take_while(|c| **c == '*').count();
            let closing = match n {
                1 => italic_on,
                2 => bold_on,
                _ => bold_on && italic_on,
            };
            let before = i.checked_sub(1).map(|j| chars[j]);
            let after = chars.get(i + n).copied();
            let valid = if closing {
                before.is_some_and(|c| !c.is_whitespace())
            } else {
                after.is_some_and(|c| !c.is_whitespace())
            };
            if valid {
                flush(&mut buf, bold_on, italic_on, underline);
                match n {
                    1 => italic_on = !closing,
                    2 => bold_on = !closing,
                    _ => (bold_on, italic_on) = (!closing, !closing),
                }
            } else {
                buf.extend(std::iter::repeat_n('*', n));
            }
            i += n;
        } else {
            buf.push(chars[i]);
            i += 1;
        }
    }
    flush(&mut buf, bold_on, italic_on, underline);
    runs
}
