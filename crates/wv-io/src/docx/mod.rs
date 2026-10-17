//! Word `.docx` (Office Open XML): paragraphs, bold/italic/underline, size, colour, alignment.
//! Tables: rows/columns are kept (equal column widths; merged cells become empty cells).
//! Header/footer: one text line each (PAGE/NUMPAGES fields <-> `{page}`/`{pages}`).
//! Not supported yet: images, per-section or first/even-page headers, styles defined in styles.xml (only direct formatting and Heading styles).

mod bands;
mod document_xml;
mod package;
mod table_read;
mod table_xml;
mod writer;

use wv_core::Document;

use crate::IoError;

pub fn decode(bytes: &[u8]) -> Result<Document, IoError> {
    let mut doc = document_xml::parse(
        &package::read_document_xml(bytes)?,
        &package::read_links(bytes),
        &package::read_numbering(bytes),
    )?;
    (doc.header, doc.footer) = bands::read(bytes);
    Ok(doc)
}

pub fn encode(doc: &Document) -> Result<Vec<u8>, IoError> {
    let (xml, links, number_lists) = writer::document_xml(doc);
    package::write(&xml, &links, number_lists, (&doc.header, &doc.footer))
}

#[cfg(test)]
mod tests;
