//! Word `.docx` (Office Open XML): paragraphs, bold/italic/underline, size, colour, alignment.
//! Tables: rows/columns are kept (equal column widths; merged cells become empty cells).
//! Header/footer: one text line each (PAGE/NUMPAGES fields <-> `{page}`/`{pages}`).
//! Not supported yet: images, per-section or first/even-page headers, styles defined in styles.xml (only direct formatting and Heading styles).

mod band_parts;
mod band_write;
mod bands;
mod document_xml;
mod package;
mod picture;
mod read_util;
mod run_xml;
mod table_read;
mod table_xml;
mod writer;

use std::collections::HashMap;

use wv_core::{Document, ImageData, ImageRef};

use crate::IoError;

pub fn decode(bytes: &[u8]) -> Result<Document, IoError> {
    let links = package::read_links(bytes);
    let (mut doc, rels) = document_xml::parse(
        &package::read_document_xml(bytes)?,
        &links,
        &package::read_numbering(bytes),
    )?;
    attach_images(&mut doc, &rels, &links, bytes);
    (doc.header, doc.footer, doc.variants) = bands::read(bytes);
    Ok(doc)
}

/// Loads the pictures the body refers to (`rels[i]` = relationship id of picture `i`) into
/// `doc.images`; pictures whose file is missing or unreadable are dropped.
fn attach_images(
    doc: &mut Document,
    rels: &[String],
    links: &HashMap<String, String>,
    bytes: &[u8],
) {
    let mut ids: Vec<Option<u32>> = Vec::new();
    for rel in rels {
        let loaded = links
            .get(rel)
            .and_then(|target| {
                package::read_part(
                    bytes,
                    &format!("word/{}", target.trim_start_matches("/word/")),
                )
            })
            .and_then(|data| crate::probe_image(&data).map(|(format, _, _)| (format, data)));
        ids.push(loaded.map(|(format, data)| {
            doc.images.push(ImageData {
                format,
                bytes: std::sync::Arc::new(data),
            });
            (doc.images.len() - 1) as u32
        }));
    }
    for p in &mut doc.paragraphs {
        if let Some(image) = p.style.image {
            p.style.image = ids
                .get(image.id as usize)
                .copied()
                .flatten()
                .map(|id| ImageRef { id, ..image });
        }
    }
}

pub fn encode(doc: &Document) -> Result<Vec<u8>, IoError> {
    let (xml, links, number_lists) = writer::document_xml(doc);
    package::write(
        &xml,
        &links,
        number_lists,
        &band_parts::parts(doc),
        doc.variants.odd_even,
        &doc.images,
    )
}

#[cfg(test)]
mod tests;
