use wv_core::{Align, CharStyle, Document, ListKind, NamedStyle, Paragraph, Run, Script};

const NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// word/document.xml for the document (A4, 2.5 cm margins). Also returns the hyperlink targets:
/// relationship `rIdL<n>` is the n-th entry.
pub fn document_xml(doc: &Document) -> (String, Vec<String>, usize) {
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>
<w:document xmlns:w=\"{NS}\" xmlns:r=\"{REL_NS}\"><w:body>"
    );
    let mut links: Vec<String> = Vec::new();
    let mut lists = Lists::default();
    let paras = &doc.paragraphs;
    // Text width in twips (tables are laid out in it).
    let page = &doc.page;
    let text_width = ((page.width - page.margin_left - page.margin_right - page.gutter) * 15.0)
        .max(1500.0) as usize;
    let mut i = 0;
    while i < paras.len() {
        let Some(first) = paras[i].style.cell else {
            write_paragraph(&mut xml, &paras[i], &mut links, &mut lists);
            i += 1;
            continue;
        };
        let end = paras[i..]
            .iter()
            .position(|p| p.style.cell.is_none_or(|c| c.table != first.table))
            .map_or(paras.len(), |n| i + n);
        let table = first.table;
        super::table_xml::write(
            &mut xml,
            &paras[i..end],
            doc.table_looks.get(&table),
            text_width,
            &mut links,
            &mut lists,
        );
        // Word merges adjacent tables and wants a paragraph before the section properties.
        if end == paras.len() || paras[end].style.cell.is_some() {
            xml.push_str("<w:p/>");
        }
        i = end;
    }
    // Header/footer parts are written by `package::write` (`band_parts`).
    let refs: String = super::band_parts::parts(doc)
        .iter()
        .map(|p| p.reference())
        .collect();
    let title_page = if doc.variants.different_first {
        "<w:titlePg/>"
    } else {
        ""
    };
    let twips = |px: f32| (px * 15.0).round() as i64;
    let page = doc.page;
    xml.push_str(&format!(
        "<w:sectPr>{refs}<w:pgSz w:w=\"{}\" w:h=\"{}\"{}/><w:pgMar w:top=\"{}\" w:right=\"{}\" w:bottom=\"{}\" w:left=\"{}\" w:header=\"{}\" w:footer=\"{}\" w:gutter=\"{}\"/>{title_page}</w:sectPr></w:body></w:document>",
        twips(page.width),
        twips(page.height),
        if page.is_landscape() { " w:orient=\"landscape\"" } else { "" },
        twips(page.margin_top),
        twips(page.margin_right),
        twips(page.margin_bottom),
        twips(page.margin_left),
        twips(page.header_distance),
        twips(page.footer_distance),
        twips(page.gutter),
    ));

    (xml, links, lists.number_lists)
}

/// Which `w:num` a list paragraph belongs to: bullets share num 1, every run of numbered paragraphs
/// gets its own num (2, 3, ...) so it restarts at 1.
#[derive(Default)]
pub struct Lists {
    pictures: u32,
    number_lists: usize,
    previous_was_number: bool,
}

impl Lists {
    fn num_id(&mut self, kind: Option<ListKind>) -> Option<usize> {
        let id = match kind {
            Some(ListKind::Bullet) => Some(1),
            Some(ListKind::Number) => {
                if !self.previous_was_number {
                    self.number_lists += 1;
                }
                Some(self.number_lists + 1)
            }
            None => None,
        };
        self.previous_was_number = kind == Some(ListKind::Number);
        id
    }
}

pub fn write_paragraph(
    xml: &mut String,
    p: &Paragraph,
    links: &mut Vec<String>,
    lists: &mut Lists,
) {
    xml.push_str("<w:p>");
    let jc = match p.style.align {
        Align::Left => None,
        Align::Center => Some("center"),
        Align::Right => Some("right"),
        Align::Justify => Some("both"),
    };
    let num_id = lists.num_id(p.style.list);
    let has_props = jc.is_some()
        || p.style.quote
        || num_id.is_some()
        || p.style.indent > 0
        || p.style.line_spacing.is_some()
        || p.style.first_line > 0
        || p.style.page_break
        || p.style.left_extra > 0
        || !p.style.tabs.is_empty()
        || p.style.space_before.is_some()
        || p.style.space_after.is_some()
        || p.style.named != NamedStyle::Normal;
    if has_props {
        // Child order follows the schema: pStyle, numPr, spacing, ind, jc.
        xml.push_str("<w:pPr>");
        if let Some(id) = docx_style_id(p.style.named) {
            xml.push_str(&format!("<w:pStyle w:val=\"{id}\"/>"));
        } else if p.style.quote {
            xml.push_str("<w:pStyle w:val=\"Quote\"/>");
        }
        if p.style.page_break {
            xml.push_str("<w:pageBreakBefore/>");
        }
        if let Some(num_id) = num_id {
            xml.push_str(&format!(
                "<w:numPr><w:ilvl w:val=\"{}\"/><w:numId w:val=\"{num_id}\"/></w:numPr>",
                p.style.indent
            ));
        }
        if !p.style.tabs.is_empty() {
            xml.push_str("<w:tabs>");
            for pos in &p.style.tabs {
                xml.push_str(&format!(
                    "<w:tab w:val=\"left\" w:pos=\"{}\"/>",
                    u32::from(*pos) * 20
                ));
            }
            xml.push_str("</w:tabs>");
        }
        let (line, before, after) = (
            p.style.line_spacing,
            p.style.space_before,
            p.style.space_after,
        );
        if line.is_some() || before.is_some() || after.is_some() {
            xml.push_str("<w:spacing");
            if let Some(pt) = before {
                xml.push_str(&format!(" w:before=\"{}\"", u32::from(pt) * 20));
            }
            if let Some(pt) = after {
                xml.push_str(&format!(" w:after=\"{}\"", u32::from(pt) * 20));
            }
            if let Some(spacing) = line {
                xml.push_str(&format!(
                    " w:line=\"{}\" w:lineRule=\"auto\"",
                    (spacing * 240.0).round() as u32
                ));
            }
            xml.push_str("/>");
        }
        let left = if num_id.is_none() {
            u32::from(p.style.indent) * 720 + u32::from(p.style.left_extra) * 20
        } else {
            0
        };
        let first = u32::from(p.style.first_line) * 20;
        if left > 0 || first > 0 {
            xml.push_str("<w:ind");
            if left > 0 {
                xml.push_str(&format!(" w:left=\"{left}\""));
            }
            if first > 0 {
                xml.push_str(&format!(" w:firstLine=\"{first}\""));
            }
            xml.push_str("/>");
        }
        if let Some(jc) = jc {
            xml.push_str(&format!("<w:jc w:val=\"{jc}\"/>"));
        }
        xml.push_str("</w:pPr>");
    }
    if let Some(image) = p.style.image {
        lists.pictures += 1;
        xml.push_str(&super::picture::drawing_xml(image, lists.pictures));
    }
    let mut open_link: Option<&str> = None;
    for run in &p.runs {
        let link = run.style.link.as_deref();
        if link != open_link {
            if open_link.is_some() {
                xml.push_str("</w:hyperlink>");
            }
            if let Some(url) = link {
                links.push(url.to_string());
                xml.push_str(&format!("<w:hyperlink r:id=\"rIdL{}\">", links.len()));
            }
            open_link = link;
        }
        write_run(xml, run);
    }
    if open_link.is_some() {
        xml.push_str("</w:hyperlink>");
    }
    xml.push_str("</w:p>");
}

fn write_run(xml: &mut String, run: &Run) {
    xml.push_str("<w:r>");
    write_properties(xml, &run.style);
    for (i, piece) in run.text.split('\t').enumerate() {
        if i > 0 {
            xml.push_str("<w:tab/>");
        }
        if !piece.is_empty() {
            xml.push_str(&format!(
                "<w:t xml:space=\"preserve\">{}</w:t>",
                escape(piece)
            ));
        }
    }
    xml.push_str("</w:r>");
}

/// Child order follows the schema: rStyle, rFonts, b, i, caps, strike, color, sz, u, shd, vertAlign.
fn write_properties(xml: &mut String, s: &CharStyle) {
    let font = s.font.as_deref().or(s.mono.then_some("Consolas"));
    if !(s.bold
        || s.italic
        || s.caps
        || s.strike
        || s.underline
        || s.color.is_some()
        || s.size_pt.is_some()
        || s.highlight.is_some()
        || s.script != Script::Normal
        || s.link.is_some()
        || font.is_some())
    {
        return;
    }
    xml.push_str("<w:rPr>");
    if s.link.is_some() {
        xml.push_str("<w:rStyle w:val=\"Hyperlink\"/>");
    }
    if let Some(font) = font {
        let name = escape_attr(font);
        xml.push_str(&format!(
            "<w:rFonts w:ascii=\"{name}\" w:hAnsi=\"{name}\" w:cs=\"{name}\"/>"
        ));
    }
    if s.bold {
        xml.push_str("<w:b/>");
    }
    if s.italic {
        xml.push_str("<w:i/>");
    }
    if s.caps {
        xml.push_str("<w:caps/>");
    }
    if s.strike {
        xml.push_str("<w:strike/>");
    }
    if let Some(color) = s.color {
        xml.push_str(&format!("<w:color w:val=\"{:06X}\"/>", color & 0xFF_FFFF));
    }
    if let Some(size) = s.size_pt {
        xml.push_str(&format!("<w:sz w:val=\"{}\"/>", u32::from(size) * 2));
    }
    if s.underline {
        xml.push_str("<w:u w:val=\"single\"/>");
    }
    if let Some(fill) = s.highlight {
        xml.push_str(&format!(
            "<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"{:06X}\"/>",
            fill & 0xFF_FFFF
        ));
    }
    match s.script {
        Script::Normal => {}
        Script::Super => xml.push_str("<w:vertAlign w:val=\"superscript\"/>"),
        Script::Sub => xml.push_str("<w:vertAlign w:val=\"subscript\"/>"),
    }
    xml.push_str("</w:rPr>");
}

/// XML-escapes text and drops characters XML 1.0 forbids.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            c => out.push(c),
        }
    }
    out
}

fn escape_attr(text: &str) -> String {
    escape(text).replace('"', "&quot;")
}

/// Word style id of a built-in style (`None` = Normal).
fn docx_style_id(style: NamedStyle) -> Option<&'static str> {
    match style {
        NamedStyle::Normal => None,
        NamedStyle::Title => Some("Title"),
        NamedStyle::Subtitle => Some("Subtitle"),
        NamedStyle::Heading1 => Some("Heading1"),
        NamedStyle::Heading2 => Some("Heading2"),
        NamedStyle::Heading3 => Some("Heading3"),
        NamedStyle::Heading4 => Some("Heading4"),
        NamedStyle::Heading5 => Some("Heading5"),
        NamedStyle::Heading6 => Some("Heading6"),
        NamedStyle::Heading7 => Some("Heading7"),
        NamedStyle::Heading8 => Some("Heading8"),
        NamedStyle::Heading9 => Some("Heading9"),
    }
}
