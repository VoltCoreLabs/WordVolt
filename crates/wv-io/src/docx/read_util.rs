use quick_xml::events::BytesStart;
use wv_core::NamedStyle;

pub(super) fn attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == name)
        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
}

/// Word's fixed highlight palette.
pub(super) fn named_highlight(name: &str) -> Option<u32> {
    Some(match name {
        "yellow" => 0xFFFF00,
        "green" => 0x00FF00,
        "cyan" => 0x00FFFF,
        "magenta" => 0xFF00FF,
        "blue" => 0x0000FF,
        "red" => 0xFF0000,
        "darkBlue" => 0x000080,
        "darkCyan" => 0x008080,
        "darkGreen" => 0x008000,
        "darkMagenta" => 0x800080,
        "darkRed" => 0x800000,
        "darkYellow" => 0x808000,
        "darkGray" => 0x808080,
        "lightGray" => 0xC0C0C0,
        "black" => 0x000000,
        _ => return None,
    })
}

/// `<w:b/>` is on; `<w:b w:val="0"/>` / `false` is off.
pub(super) fn flag(e: &BytesStart) -> bool {
    !matches!(
        attr(e, b"val").as_deref(),
        Some("0" | "false" | "off" | "none")
    )
}

/// Word's built-in style ids ("Heading1", "heading 2", "Title", ...) -> our styles; deeper headings map to Heading 3.
pub(super) fn named_from_docx(id: &str) -> NamedStyle {
    let key: String = id
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase();
    match key.as_str() {
        "title" => NamedStyle::Title,
        "subtitle" => NamedStyle::Subtitle,
        "heading1" => NamedStyle::Heading1,
        "heading2" => NamedStyle::Heading2,
        k if k.starts_with("heading") => NamedStyle::Heading3,
        _ => NamedStyle::Normal,
    }
}
