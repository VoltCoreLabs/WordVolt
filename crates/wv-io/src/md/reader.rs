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
    if let Some(rest) = block.strip_prefix("> ").or_else(|| block.strip_prefix('>')) {
        let mut p = paragraph(rest.trim_start());
        p.style.quote = true;
        return p;
    }
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
    let (mut strike_on, mut spoiler_on) = (false, false);
    let (mut runs, mut buf) = (Vec::new(), String::new());
    let mut i = 0;
    while i < chars.len() {
        let rest: String = chars[i..chars.len().min(i + 4)].iter().collect();
        if chars[i] == '\\'
            && i + 1 < chars.len()
            && matches!(chars[i + 1], '\\' | '*' | '<' | '~' | '[' | ']' | '`' | '|')
        {
            buf.push(chars[i + 1]);
            i += 2;
        } else if rest.starts_with("<u>") || rest.starts_with("</u>") {
            flush(
                &mut runs, &mut buf, heading, bold_on, italic_on, underline, strike_on, spoiler_on,
            );
            underline = rest.starts_with("<u>");
            i += if underline { 3 } else { 4 };
        } else if chars[i] == '`' {
            // Inline code: everything up to the closing backtick is literal.
            if let Some(len) = chars[i + 1..]
                .iter()
                .position(|c| *c == '`')
                .filter(|n| *n > 0)
            {
                flush(
                    &mut runs, &mut buf, heading, bold_on, italic_on, underline, strike_on,
                    spoiler_on,
                );
                let style = CharStyle {
                    bold: heading || bold_on,
                    italic: italic_on,
                    underline,
                    strike: strike_on,
                    spoiler: spoiler_on,
                    mono: true,
                    ..Default::default()
                };
                runs.push(Run::new(
                    chars[i + 1..i + 1 + len].iter().collect::<String>(),
                    style,
                ));
                i += len + 2;
            } else {
                buf.push('`');
                i += 1;
            }
        } else if chars[i] == '|' && chars.get(i + 1) == Some(&'|') {
            let before = i.checked_sub(1).map(|j| chars[j]);
            let after = chars.get(i + 2).copied();
            let valid = if spoiler_on {
                before.is_some_and(|c| !c.is_whitespace())
            } else {
                after.is_some_and(|c| !c.is_whitespace())
            };
            if valid {
                flush(
                    &mut runs, &mut buf, heading, bold_on, italic_on, underline, strike_on,
                    spoiler_on,
                );
                spoiler_on = !spoiler_on;
            } else {
                buf.push_str("||");
            }
            i += 2;
        } else if chars[i] == '['
            && let Some((label, url, used)) = link_at(&chars[i..])
        {
            flush(
                &mut runs, &mut buf, heading, bold_on, italic_on, underline, strike_on, spoiler_on,
            );
            for mut run in inline(&label, heading) {
                run.style.link = Some(url.clone());
                runs.push(run);
            }
            i += used;
        } else if chars[i] == '~' && chars.get(i + 1) == Some(&'~') {
            let before = i.checked_sub(1).map(|j| chars[j]);
            let after = chars.get(i + 2).copied();
            let valid = if strike_on {
                before.is_some_and(|c| !c.is_whitespace())
            } else {
                after.is_some_and(|c| !c.is_whitespace())
            };
            if valid {
                flush(
                    &mut runs, &mut buf, heading, bold_on, italic_on, underline, strike_on,
                    spoiler_on,
                );
                strike_on = !strike_on;
            } else {
                buf.push_str("~~");
            }
            i += 2;
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
                flush(
                    &mut runs, &mut buf, heading, bold_on, italic_on, underline, strike_on,
                    spoiler_on,
                );
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
    flush(
        &mut runs, &mut buf, heading, bold_on, italic_on, underline, strike_on, spoiler_on,
    );
    runs
}

/// `[label](url)` at the start of `chars`: (label, url, chars used).
fn link_at(chars: &[char]) -> Option<(String, String, usize)> {
    let mut depth = 0;
    let mut close = None;
    for (n, c) in chars.iter().enumerate() {
        match c {
            '\\' => continue,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(n);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close?;
    if chars.get(close + 1) != Some(&'(') {
        return None;
    }
    let end = chars[close + 2..].iter().position(|c| *c == ')')? + close + 2;
    let label: String = chars[1..close].iter().collect();
    let url: String = chars[close + 2..end].iter().collect();
    (!url.is_empty()).then_some((label, url, end + 1))
}

/// Moves the buffered text into a run with the current inline styles.
#[allow(clippy::too_many_arguments)] // the inline parser's marker state, kept as plain flags
fn flush(
    runs: &mut Vec<Run>,
    buf: &mut String,
    heading: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
    spoiler: bool,
) {
    if !buf.is_empty() {
        let style = CharStyle {
            bold: heading || bold,
            italic,
            underline,
            strike,
            spoiler,
            ..Default::default()
        };
        runs.push(Run::new(std::mem::take(buf), style));
    }
}
