use std::collections::BTreeMap;

use wv_core::{Cell, Paragraph};

use super::reader::paragraph;
use super::writer::run;

/// GFM pipe table for the paragraphs of one table; the first row is the header.
/// Paragraphs of one cell are joined with `<br>`.
pub fn encode(paras: &[Paragraph]) -> String {
    let mut cells: BTreeMap<(u16, u16), Vec<String>> = BTreeMap::new();
    for p in paras {
        if let Some(c) = p.style.cell {
            let text: String = p.runs.iter().map(run).collect();
            cells.entry((c.row, c.col)).or_default().push(text);
        }
    }
    let rows = cells.keys().map(|k| k.0 + 1).max().unwrap_or(0);
    let cols = cells.keys().map(|k| k.1 + 1).max().unwrap_or(0);
    let mut lines = Vec::new();
    for r in 0..rows {
        let row: Vec<String> = (0..cols)
            .map(|c| cells.get(&(r, c)).map_or(String::new(), |v| v.join("<br>")))
            .collect();
        lines.push(format!("| {} |", row.join(" | ")));
        if r == 0 {
            lines.push(format!("|{}", " --- |".repeat(usize::from(cols))));
        }
    }
    lines.join("\n")
}

/// Splits a table row on unescaped pipes; the outer pipes are optional.
fn split_row(line: &str) -> Vec<String> {
    let line = line.trim();
    let line = line.strip_prefix('|').unwrap_or(line);
    let mut cells = vec![String::new()];
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' if chars.peek() == Some(&'|') => {
                cells.last_mut().map(|c| c.push_str("\\|"));
                chars.next();
            }
            '|' => cells.push(String::new()),
            _ => {
                if let Some(c) = cells.last_mut() {
                    c.push(ch);
                }
            }
        }
    }
    if cells.last().is_some_and(|c| c.trim().is_empty()) && line.trim_end().ends_with('|') {
        cells.pop();
    }
    cells.into_iter().map(|c| c.trim().to_string()).collect()
}

fn is_separator(line: &str) -> bool {
    let cells = split_row(line);
    !cells.is_empty()
        && cells.iter().all(|c| {
            let c = c.trim_matches(':');
            !c.is_empty() && c.chars().all(|ch| ch == '-')
        })
}

/// True when the block starts with a header row followed by a `---` separator row.
pub fn is_table(block: &[&str]) -> bool {
    block.len() >= 2 && block[0].contains('|') && is_separator(block[1])
}

/// Appends the paragraphs of the pipe table in `block` (header, separator, body rows).
pub fn decode(block: &[&str], out: &mut Vec<Paragraph>) {
    let table = out
        .iter()
        .filter_map(|p| p.style.cell)
        .map(|c| c.table + 1)
        .max()
        .unwrap_or(0);
    let rows: Vec<Vec<String>> = std::iter::once(block[0])
        .chain(block[2..].iter().copied())
        .map(split_row)
        .collect();
    let cols = rows[0].len().max(1);
    for (r, row) in rows.iter().enumerate() {
        for c in 0..cols {
            let text = row.get(c).map_or("", String::as_str);
            for part in text.split("<br>") {
                let mut p = paragraph(part);
                p.style.cell = Some(Cell {
                    table,
                    row: r as u16,
                    col: c as u16,
                });
                out.push(p);
            }
        }
    }
}
