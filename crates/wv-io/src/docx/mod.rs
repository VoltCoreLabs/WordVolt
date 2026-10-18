//! Word `.docx` (Office Open XML): paragraphs, bold/italic/underline, size, colour, alignment.
//! Tables: rows/columns are kept (equal column widths; merged cells become empty cells).
//! Header/footer: one text line each (PAGE/NUMPAGES fields <-> `{page}`/`{pages}`).
//! Not supported yet: images, per-section or first/even-page headers, styles defined in styles.xml (only direct formatting and Heading styles).

mod band_parts;
mod band_write;
mod bands;
mod comments;
mod document_xml;
mod footnotes;
mod package;
mod parts;
mod picture;
mod read_util;
mod run_xml;
mod section_xml;
mod styles_read;
mod styles_write;
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
        &package::read_part(bytes, "word/styles.xml")
            .map(|xml| styles_read::read(&xml))
            .unwrap_or_default(),
    )?;
    attach_images(&mut doc, &rels, &links, bytes);
    attach_footnotes(&mut doc, &links, bytes);
    if let Some(xml) = package::read_part(bytes, "word/comments.xml") {
        doc.comments = comments::read(&xml);
    }
    (doc.header, doc.footer, doc.variants) = bands::read(bytes);
    Ok(doc)
}

/// Reads word/footnotes.xml: the paragraphs of the notes go to the end of the document, as
/// footnote paragraphs (the marks in the text have been read with the body).
fn attach_footnotes(doc: &mut Document, links: &HashMap<String, String>, bytes: &[u8]) {
    let Some(xml) = package::read_part(bytes, "word/footnotes.xml") else {
        return;
    };
    let Ok((notes, _)) = document_xml::parse(&xml, links, &HashMap::new(), &HashMap::new()) else {
        return;
    };
    let mut previous = None;
    for mut p in notes
        .paragraphs
        .into_iter()
        .filter(|p| p.style.note.is_some())
    {
        if previous != p.style.note {
            footnotes::trim_start(&mut p);
        }
        previous = p.style.note;
        doc.paragraphs.push(p);
    }
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
        (
            footnotes::xml(doc).as_deref(),
            comments::xml(doc).as_deref(),
            &styles_write::xml(doc),
        ),
    )
}

#[cfg(test)]
mod tests;
