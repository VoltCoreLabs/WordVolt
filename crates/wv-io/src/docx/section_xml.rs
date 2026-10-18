use wv_core::{NumberFormat, PageSpec};

fn twips(px: f32) -> i64 {
    (px * 15.0).round() as i64
}

/// `w:sectPr` for a page setup. `refs` are header/footer references and `extra` more children
/// (`w:titlePg`); a section that ends inside the body also gets `w:type` (it starts a new page).
pub(super) fn sect_pr(page: &PageSpec, refs: &str, extra: &str, in_paragraph: bool) -> String {
    let cols = if page.columns > 1 {
        format!("<w:cols w:num=\"{}\" w:space=\"720\"/>", page.columns)
    } else {
        String::new()
    };
    let numbering = if page.page_start.is_some() || page.number_format != NumberFormat::Decimal {
        format!(
            "<w:pgNumType{}{}/>",
            page.page_start
                .map_or(String::new(), |n| format!(" w:start=\"{n}\"")),
            if page.number_format == NumberFormat::Decimal {
                String::new()
            } else {
                format!(" w:fmt=\"{}\"", page.number_format.id())
            }
        )
    } else {
        String::new()
    };
    let kind = if in_paragraph {
        "<w:type w:val=\"nextPage\"/>"
    } else {
        ""
    };
    format!(
        "<w:sectPr>{refs}{kind}<w:pgSz w:w=\"{}\" w:h=\"{}\"{}/><w:pgMar w:top=\"{}\" w:right=\"{}\" w:bottom=\"{}\" w:left=\"{}\" w:header=\"{}\" w:footer=\"{}\" w:gutter=\"{}\"/>{numbering}{cols}{extra}</w:sectPr>",
        twips(page.width),
        twips(page.height),
        if page.is_landscape() {
            " w:orient=\"landscape\""
        } else {
            ""
        },
        twips(page.margin_top),
        twips(page.margin_right),
        twips(page.margin_bottom),
        twips(page.margin_left),
        twips(page.header_distance),
        twips(page.footer_distance),
        twips(page.gutter),
    )
}
