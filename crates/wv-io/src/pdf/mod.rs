//! PDF export: the document is laid out with the real font metrics and drawn page by page; the
//! fonts are embedded.

mod draw;
mod embed;
mod fonts;
mod pic;

use std::collections::BTreeMap;

use pdf_writer::{Finish, Name, Pdf, Rect, Ref};
use wv_core::{CharStyle, Document};
use wv_layout::{DocLayout, PageSetup};

use crate::IoError;
use embed::Ids;
use fonts::FontSet;
pub use fonts::{FontRequest, FontSource, SystemFonts};

/// The document as a PDF file.
pub fn encode(doc: &Document, source: &dyn FontSource) -> Result<Vec<u8>, IoError> {
    let fonts = FontSet::new(source);
    if fonts.face_index(&CharStyle::default()).is_none() {
        return Err(IoError::NoFont);
    }
    let setup = PageSetup::from_spec(&doc.page);
    let layout = DocLayout::new(doc, setup, &fonts);
    let pages: Vec<_> = (0..layout.pages.len())
        .map(|page| draw::draw_page(doc, &layout, &fonts, page))
        .collect();
    let (width, height) = (layout.setup.width * 0.75, layout.setup.height * 0.75);
    let faces = fonts.into_faces();

    let mut pdf = Pdf::new();
    let mut ids = Ids::new();
    let (catalog, tree) = (ids.next(), ids.next());
    let font_refs: Vec<Ref> = faces
        .iter()
        .enumerate()
        .map(|(n, face)| embed::write_font(&mut pdf, &mut ids, face, n))
        .collect();
    let mut image_refs = BTreeMap::new();
    for id in pages.iter().flat_map(|p| p.images.iter().copied()) {
        if let Some(data) = doc.images.get(id as usize)
            && let std::collections::btree_map::Entry::Vacant(slot) = image_refs.entry(id)
            && let Some(r) = pic::write_image(&mut pdf, &mut ids, data)
        {
            slot.insert(r);
        }
    }

    let page_ids: Vec<Ref> = pages.iter().map(|_| ids.next()).collect();
    pdf.catalog(catalog).pages(tree);
    pdf.pages(tree)
        .kids(page_ids.iter().copied())
        .count(page_ids.len() as i32);
    for (page, id) in pages.iter().zip(&page_ids) {
        let content = ids.next();
        let mut p = pdf.page(*id);
        p.media_box(Rect::new(0.0, 0.0, width, height))
            .parent(tree)
            .contents(content);
        let mut resources = p.resources();
        let mut fonts_dict = resources.fonts();
        for (n, r) in font_refs.iter().enumerate() {
            fonts_dict.pair(Name(format!("F{n}").as_bytes()), *r);
        }
        fonts_dict.finish();
        let mut xobjects = resources.x_objects();
        for id in &page.images {
            if let Some(r) = image_refs.get(id) {
                xobjects.pair(Name(format!("I{id}").as_bytes()), *r);
            }
        }
        xobjects.finish();
        resources.finish();
        p.finish();
        pdf.stream(content, &page.data);
    }
    Ok(pdf.finish())
}

#[cfg(test)]
mod tests;
