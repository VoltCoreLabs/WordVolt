use std::collections::BTreeMap;

use wv_core::Paragraph;

use super::writer::{Lists, write_paragraph};

const BORDERS: &str = "<w:tblBorders><w:top w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:left w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:right w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/></w:tblBorders>";

/// One `w:tbl` for the paragraphs of a single table. `widths` are the column fractions (equal when
/// absent or not matching) of `text_width` twips.
pub fn write(
    xml: &mut String,
    paras: &[Paragraph],
    widths: Option<&Vec<f32>>,
    text_width: usize,
    links: &mut Vec<String>,
    lists: &mut Lists,
) {
    let mut cells: BTreeMap<(u16, u16), Vec<&Paragraph>> = BTreeMap::new();
    for p in paras {
        if let Some(c) = p.style.cell {
            cells.entry((c.row, c.col)).or_default().push(p);
        }
    }
    let rows = cells.keys().map(|k| k.0 + 1).max().unwrap_or(0);
    let cols = cells.keys().map(|k| k.1 + 1).max().unwrap_or(0);
    let fractions = match widths {
        Some(w) if w.len() == usize::from(cols) => w.clone(),
        _ => vec![1.0 / f32::from(cols.max(1)); usize::from(cols)],
    };
    let widths: Vec<usize> = fractions
        .iter()
        .map(|f| (f * text_width as f32).round() as usize)
        .collect();
    xml.push_str("<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"pct\"/>");
    xml.push_str(BORDERS);
    xml.push_str("</w:tblPr><w:tblGrid>");
    for width in &widths {
        xml.push_str(&format!("<w:gridCol w:w=\"{width}\"/>"));
    }
    xml.push_str("</w:tblGrid>");
    for r in 0..rows {
        xml.push_str("<w:tr>");
        for c in 0..cols {
            let shade = cells
                .get(&(r, c))
                .and_then(|list| list.first())
                .and_then(|p| p.style.cell)
                .and_then(|cell| cell.shade)
                .map(|rgb| {
                    format!("<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"{rgb:06X}\"/>")
                })
                .unwrap_or_default();
            xml.push_str(&format!(
                "<w:tc><w:tcPr><w:tcW w:w=\"{}\" w:type=\"dxa\"/>{shade}</w:tcPr>",
                widths[usize::from(c)]
            ));
            match cells.get(&(r, c)) {
                Some(list) => {
                    for p in list {
                        write_paragraph(xml, p, links, lists);
                    }
                }
                None => xml.push_str("<w:p/>"),
            }
            xml.push_str("</w:tc>");
        }
        xml.push_str("</w:tr>");
    }
    xml.push_str("</w:tbl>");
}
