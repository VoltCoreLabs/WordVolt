//! Word `.docx` (Office Open XML): paragraphs, bold/italic/underline, size, colour, alignment.
//! Not supported yet: tables as tables (their paragraphs are read as plain paragraphs), images,
//! headers/footers, styles defined in styles.xml (only direct formatting and Heading styles).

mod document_xml;
mod package;
mod writer;

use wv_core::Document;

use crate::IoError;

pub fn decode(bytes: &[u8]) -> Result<Document, IoError> {
    document_xml::parse(
        &package::read_document_xml(bytes)?,
        &package::read_links(bytes),
    )
}

pub fn encode(doc: &Document) -> Result<Vec<u8>, IoError> {
    let (xml, links) = writer::document_xml(doc);
    package::write(&xml, &links)
}

#[cfg(test)]
mod tests;
