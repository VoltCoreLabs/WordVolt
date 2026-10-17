use std::collections::BTreeMap;

use wv_core::{Cell, Paragraph, TableLook};

use super::writer::{Lists, write_paragraph};

const BORDERS: &str = "<w:tblBorders><w:top w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:left w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:right w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/><w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"808080\"/></w:tblBorders>";

/// `<w:tcPr>` of a cell `width` twips wide: size, column span, vertical merge (`restart` starts
/// one, `continue` is a cell covered by the one above) and background.
fn cell_props(width: usize, cell: &Cell, vertical: Option<bool>, shade: bool) -> String {
    let mut xml = format!("<w:tcPr><w:tcW w:w=\"{width}\" w:type=\"dxa\"/>");
    if cell.colspan > 1 {
        xml.push_str(&format!("<w:gridSpan w:val=\"{}\"/>", cell.colspan));
    }
    match vertical {
        Some(true) => xml.push_str("<w:vMerge w:val=\"restart\"/>"),
        Some(false) => xml.push_str("<w:vMerge/>"),
        None => {}
    }
    if let (true, Some(rgb)) = (shade, cell.shade) {
        xml.push_str(&format!(
            "<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"{rgb:06X}\"/>"
        ));
    }
    xml.push_str("</w:tcPr>");
    xml
}

/// One `w:tbl` for the paragraphs of a single table. `look` is how it is sized (equal columns over
/// the text width when absent); `text_width` is the text width in twips.
pub fn write(
    xml: &mut String,
    paras: &[Paragraph],
    look: Option<&TableLook>,
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
    let infos: Vec<Cell> = cells
        .values()
        .filter_map(|list| list[0].style.cell)
        .collect();
    let rows = infos.iter().map(Cell::row_end).max().unwrap_or(0);
    let cols = infos.iter().map(Cell::col_end).max().unwrap_or(0);
    let look = TableLook::resolved(look, usize::from(rows), usize::from(cols));
    let widths: Vec<usize> = look
        .widths
        .iter()
        .map(|f| (f * text_width as f32).round() as usize)
        .collect();
    let full = look.indent == 0.0 && (look.widths.iter().sum::<f32>() - 1.0).abs() < 0.001;
    let span_width = |from: u16, span: u8| -> usize {
        widths
            .iter()
            .skip(usize::from(from))
            .take(usize::from(span.max(1)))
            .sum()
    };
    if full {
        xml.push_str("<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"pct\"/>");
    } else {
        let total: usize = widths.iter().sum();
        xml.push_str(&format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"{total}\" w:type=\"dxa\"/><w:tblInd w:w=\"{}\" w:type=\"dxa\"/>",
            (look.indent * text_width as f32).round() as usize
        ));
    }
    xml.push_str(BORDERS);
    xml.push_str("</w:tblPr><w:tblGrid>");
    for width in &widths {
        xml.push_str(&format!("<w:gridCol w:w=\"{width}\"/>"));
    }
    xml.push_str("</w:tblGrid>");
    for r in 0..rows {
        xml.push_str("<w:tr>");
        let least = look.rows.get(usize::from(r)).copied().unwrap_or(0.0);
        if least > 0.0 {
            xml.push_str(&format!(
                "<w:trPr><w:trHeight w:val=\"{}\" w:hRule=\"atLeast\"/></w:trPr>",
                (least * 15.0).round() as usize
            ));
        }
        let mut c = 0;
        while c < cols {
            if let Some(list) = cells.get(&(r, c)) {
                let cell = list[0].style.cell.unwrap_or(Cell::at(0, r, c));
                let vertical = (cell.rowspan > 1).then_some(true);
                xml.push_str("<w:tc>");
                xml.push_str(&cell_props(
                    span_width(c, cell.colspan),
                    &cell,
                    vertical,
                    true,
                ));
                for p in list {
                    write_paragraph(xml, p, links, lists);
                }
                xml.push_str("</w:tc>");
                c += u16::from(cell.colspan.max(1));
            } else if let Some(above) = infos.iter().find(|i| i.covers(r, c) && i.row < r) {
                // Covered by a cell that started in an earlier row.
                xml.push_str("<w:tc>");
                xml.push_str(&cell_props(
                    span_width(c, above.colspan),
                    above,
                    Some(false),
                    false,
                ));
                xml.push_str("<w:p/></w:tc>");
                c += u16::from(above.colspan.max(1));
            } else {
                xml.push_str(&format!(
                    "<w:tc><w:tcPr><w:tcW w:w=\"{}\" w:type=\"dxa\"/></w:tcPr><w:p/></w:tc>",
                    span_width(c, 1)
                ));
                c += 1;
            }
        }
        xml.push_str("</w:tr>");
    }
    xml.push_str("</w:tbl>");
}
