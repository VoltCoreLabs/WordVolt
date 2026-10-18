//! Plain text.

use wv_core::Document;

/// UTF-8 or UTF-16 (with BOM); invalid bytes become U+FFFD. `\r\n` is normalised to `\n`.
pub fn decode_text(bytes: &[u8]) -> String {
    let text = match bytes {
        [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8_lossy(rest).into_owned(),
        [0xFF, 0xFE, rest @ ..] => utf16(rest, u16::from_le_bytes),
        [0xFE, 0xFF, rest @ ..] => utf16(rest, u16::from_be_bytes),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    };
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn utf16(bytes: &[u8], read: fn([u8; 2]) -> u16) -> String {
    let (pairs, _) = bytes.as_chunks::<2>();
    let units: Vec<u16> = pairs.iter().map(|pair| read(*pair)).collect();
    String::from_utf16_lossy(&units)
}

pub fn decode(bytes: &[u8]) -> Document {
    Document::from_text(&decode_text(bytes))
}

pub fn encode(doc: &Document) -> Vec<u8> {
    match doc.with_inline_notes() {
        Some(plain) => plain.text().into_bytes(),
        None => doc.text().into_bytes(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footnotes_are_written_as_numbers_in_the_text() {
        let mut e = wv_core::Editor::new(Document::from_text("Hello world"));
        e.sel = wv_core::Selection::caret(wv_core::Position::new(0, 5));
        e.insert_footnote();
        e.insert_text("the note");
        assert_eq!(
            String::from_utf8(encode(&e.doc)).unwrap(),
            "Hello[1] world
[1] the note"
        );
    }

    #[test]
    fn decodes_utf8_bom_crlf_and_utf16() {
        assert_eq!(decode_text(b"\xEF\xBB\xBFa\r\nb"), "a\nb");
        let utf16le: Vec<u8> = [0xFF, 0xFE]
            .into_iter()
            .chain("Привет".encode_utf16().flat_map(u16::to_le_bytes))
            .collect();
        assert_eq!(decode_text(&utf16le), "Привет");
        assert_eq!(decode_text(&[0x61, 0xFF]), "a\u{FFFD}");
    }

    #[test]
    fn round_trip_keeps_paragraphs() {
        let doc = Document::from_text("one\n\nthree");
        assert_eq!(decode(&encode(&doc)), doc);
    }
}
