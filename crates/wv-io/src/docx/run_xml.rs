use wv_core::{CharStyle, Revision, Run, Script};

use super::writer::{escape, escape_attr};

pub(super) fn write_run(xml: &mut String, run: &Run, revisions: &[Revision]) {
    // A footnote mark is a reference to the note (Word draws the number).
    if let Some(id) = run.style.note {
        xml.push_str(&format!(
            "<w:r><w:rPr><w:rStyle w:val=\"FootnoteReference\"/></w:rPr><w:footnoteReference w:id=\"{id}\"/></w:r>"
        ));
        return;
    }
    // A tracked change wraps the run in `w:ins` / `w:del` (deleted text is `w:delText`).
    let change = run.style.change;
    if let Some(c) = change {
        let rev = revisions.iter().find(|r| r.id == c.id);
        let (author, date) = rev.map_or(("", ""), |r| (r.author.as_str(), r.date.as_str()));
        let tag = if c.deleted { "del" } else { "ins" };
        xml.push_str(&format!(
            "<w:{tag} w:id=\"{}\" w:author=\"{}\" w:date=\"{}\">",
            c.id,
            escape_attr(author),
            escape_attr(date)
        ));
    }
    let text_tag = if change.is_some_and(|c| c.deleted) {
        "delText"
    } else {
        "t"
    };
    xml.push_str("<w:r>");
    write_properties(xml, &run.style);
    for (i, piece) in run.text.split('\t').enumerate() {
        if i > 0 {
            xml.push_str("<w:tab/>");
        }
        if !piece.is_empty() {
            xml.push_str(&format!(
                "<w:{text_tag} xml:space=\"preserve\">{}</w:{text_tag}>",
                escape(piece)
            ));
        }
    }
    xml.push_str("</w:r>");
    if let Some(c) = change {
        xml.push_str(if c.deleted { "</w:del>" } else { "</w:ins>" });
    }
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
