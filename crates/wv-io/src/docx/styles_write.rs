use wv_core::{Align, Document};

use super::writer::{docx_style_id, escape_attr};

/// Style id used in the file for user style `id`.
pub(super) fn style_id(id: u32) -> String {
    format!("WV{id}")
}

/// The `w:style` elements of the user styles (they go into word/styles.xml); empty without any.
pub(super) fn xml(doc: &Document) -> String {
    let mut out = String::new();
    for (id, s) in &doc.styles {
        out.push_str(&format!(
            "<w:style w:type=\"paragraph\" w:customStyle=\"1\" w:styleId=\"{}\"><w:name w:val=\"{}\"/>",
            style_id(*id),
            escape_attr(&s.name)
        ));
        out.push_str(&format!(
            "<w:basedOn w:val=\"{}\"/><w:qFormat/>",
            docx_style_id(s.base).unwrap_or("Normal")
        ));
        let mut ppr = String::new();
        if s.space_before.is_some() || s.space_after.is_some() || s.line_spacing.is_some() {
            ppr.push_str("<w:spacing");
            if let Some(pt) = s.space_before {
                ppr.push_str(&format!(" w:before=\"{}\"", u32::from(pt) * 20));
            }
            if let Some(pt) = s.space_after {
                ppr.push_str(&format!(" w:after=\"{}\"", u32::from(pt) * 20));
            }
            if let Some(line) = s.line_spacing {
                ppr.push_str(&format!(
                    " w:line=\"{}\" w:lineRule=\"auto\"",
                    (line * 240.0).round() as u32
                ));
            }
            ppr.push_str("/>");
        }
        if let Some(align) = s.align {
            let jc = match align {
                Align::Left => "left",
                Align::Center => "center",
                Align::Right => "right",
                Align::Justify => "both",
            };
            ppr.push_str(&format!("<w:jc w:val=\"{jc}\"/>"));
        }
        if !ppr.is_empty() {
            out.push_str(&format!("<w:pPr>{ppr}</w:pPr>"));
        }
        let mut rpr = String::new();
        if let Some(font) = &s.font {
            let font = escape_attr(font);
            rpr.push_str(&format!(
                "<w:rFonts w:ascii=\"{font}\" w:hAnsi=\"{font}\" w:cs=\"{font}\"/>"
            ));
        }
        if s.bold {
            rpr.push_str("<w:b/>");
        }
        if s.italic {
            rpr.push_str("<w:i/>");
        }
        if let Some(color) = s.color {
            rpr.push_str(&format!("<w:color w:val=\"{color:06X}\"/>"));
        }
        if let Some(pt) = s.size_pt {
            rpr.push_str(&format!("<w:sz w:val=\"{}\"/>", u32::from(pt) * 2));
        }
        if s.underline {
            rpr.push_str("<w:u w:val=\"single\"/>");
        }
        if !rpr.is_empty() {
            out.push_str(&format!("<w:rPr>{rpr}</w:rPr>"));
        }
        out.push_str("</w:style>");
    }
    out
}
