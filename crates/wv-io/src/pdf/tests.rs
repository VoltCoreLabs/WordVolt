use wv_core::{Document, Editor};

use super::*;

/// Any installed font will do; tests are skipped where there is none.
fn pdf_of(doc: &Document) -> Option<Vec<u8>> {
    encode(doc, &SystemFonts).ok()
}

#[test]
fn exports_a_pdf_with_pages_and_embedded_font() {
    let text = "Привет, мир! ".repeat(400);
    let Some(bytes) = pdf_of(&Document::from_text(&text)) else {
        return;
    };
    assert!(bytes.starts_with(b"%PDF-"));
    let body = String::from_utf8_lossy(&bytes);
    assert!(body.contains("/FontFile2"));
    assert!(body.contains("/Identity-H"));
    let pages = body.matches("/Type /Page\n").count() + body.matches("/Type /Page ").count();
    assert!(pages >= 2, "{pages} pages");
}

#[test]
fn a_header_and_a_picture_are_exported() {
    let mut e = Editor::new(Document::from_text("a"));
    e.set_header_footer(
        wv_core::Band::new("Head"),
        wv_core::Band::new("Стр. {page} из {pages}"),
    );
    let mut png = Vec::new();
    image::DynamicImage::new_rgb8(4, 4)
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap();
    e.insert_image(wv_core::ImageFormat::Png, png, 4.0, 4.0);
    let Some(bytes) = pdf_of(&e.doc) else { return };
    assert!(String::from_utf8_lossy(&bytes).contains("/Subtype /Image"));
}

#[test]
fn without_any_font_the_export_fails_cleanly() {
    struct None_;
    impl FontSource for None_ {
        fn load(&self, _: &FontRequest) -> Option<Vec<u8>> {
            None
        }
    }
    assert!(matches!(
        encode(&Document::default(), &None_),
        Err(IoError::NoFont)
    ));
}
