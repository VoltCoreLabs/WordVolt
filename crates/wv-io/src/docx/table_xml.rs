use std::collections::BTreeMap;

use wv_core::Paragraph;

use super::writer::{Lists, write_paragraph};

/// Usable text width of the A4 page in twips (see `document_xml`).
const TEXT_WIDTH: usize = 9070;

const BORDERS: &str = "<w:tblBorders><w:top w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:left w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:right w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/></w:tblBorders>";

/// One `w:tbl` for the paragraphs of a single table (equal columns, full text width).
pub fn write(xml: &mut String, paras: &[Paragraph], links: &mut Vec<String>, lists: &mut Lists) {
    let mut cells: BTreeMap<(u16, u16), Vec<&Paragraph>> = BTreeMap::new();
    for p in paras {
        if let Some(c) = p.style.cell {
            cells.entry((c.row, c.col)).or_default().push(p);
        }
    }
    let rows = cells.keys().map(|k| k.0 + 1).max().unwrap_or(0);
    let cols = cells.keys().map(|k| k.1 + 1).max().unwrap_or(0);
    let width = TEXT_WIDTH / usize::from(cols.max(1));
    xml.push_str("<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"pct\"/>");
    xml.push_str(BORDERS);
    xml.push_str("</w:tblPr><w:tblGrid>");
    for _ in 0..cols {
        xml.push_str(&format!("<w:gridCol w:w=\"{width}\"/>"));
    }
    xml.push_str("</w:tblGrid>");
    for r in 0..rows {
        xml.push_str("<w:tr>");
        for c in 0..cols {
            xml.push_str(&format!(
                "<w:tc><w:tcPr><w:tcW w:w=\"{width}\" w:type=\"dxa\"/></w:tcPr>"
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
