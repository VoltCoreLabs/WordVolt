use wv_core::{Align, BAND_SIZE_PT, Band, CharStyle};

/// word/header1.xml or footer1.xml (`root` = "hdr" / "ftr"): one paragraph with the band's
/// formatted runs; `{page}` and `{pages}` become PAGE / NUMPAGES fields.
pub fn part_xml(root: &str, band: &Band) -> String {
    let runs: String = band
        .runs()
        .iter()
        .map(|r| run_xml(&r.text, &r.style))
        .collect();
    let jc = match band.align {
        Align::Left | Align::Justify => "left",
        Align::Center => "center",
        Align::Right => "right",
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:{root} xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:p><w:pPr><w:jc w:val=\"{jc}\"/></w:pPr>{runs}</w:p></w:{root}>"
    )
}

/// The text of one run as `w:r` elements; the tokens turn into fields.
fn run_xml(text: &str, style: &CharStyle) -> String {
    let rpr = run_props(style);
    let mut runs = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        let next = ["{pages}", "{page}"]
            .iter()
            .filter_map(|t| rest.find(t).map(|i| (i, *t)))
            .min_by_key(|(i, t)| (*i, std::cmp::Reverse(t.len())));
        let (literal, field) = match next {
            Some((i, token)) => (&rest[..i], Some(token)),
            None => (rest, None),
        };
        if !literal.is_empty() {
            runs.push_str(&format!(
                "<w:r>{rpr}<w:t xml:space=\"preserve\">{}</w:t></w:r>",
                escape(literal)
            ));
        }
        let Some(token) = field else { break };
        let instr = if token == "{page}" {
            "PAGE"
        } else {
            "NUMPAGES"
        };
        runs.push_str(&format!(
            "<w:fldSimple w:instr=\" {instr} \"><w:r>{rpr}<w:t>1</w:t></w:r></w:fldSimple>"
        ));
        rest = &rest[rest.find(token).unwrap_or(0) + token.len()..];
    }
    runs
}

/// `<w:rPr>` of the formatting (the size is always written: Word's default is 11 pt).
fn run_props(style: &CharStyle) -> String {
    let mut xml = String::from("<w:rPr>");
    if style.bold {
        xml.push_str("<w:b/>");
    }
    if style.italic {
        xml.push_str("<w:i/>");
    }
    if style.underline {
        xml.push_str("<w:u w:val=\"single\"/>");
    }
    if style.strike {
        xml.push_str("<w:strike/>");
    }
    if let Some(color) = style.color {
        xml.push_str(&format!("<w:color w:val=\"{color:06X}\"/>"));
    }
    let half_points = u32::from(style.size_pt.unwrap_or(BAND_SIZE_PT)) * 2;
    xml.push_str(&format!(
        "<w:sz w:val=\"{half_points}\"/><w:szCs w:val=\"{half_points}\"/></w:rPr>"
    ));
    xml
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
