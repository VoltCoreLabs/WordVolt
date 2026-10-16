use std::collections::HashMap;
use std::io::{Cursor, Read, Write};

use wv_core::ListKind;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::IoError;

/// Refuse to inflate more than this (zip-bomb guard).
const MAX_XML_BYTES: u64 = 256 * 1024 * 1024;

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/><Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/></Types>"#;

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

const DOCUMENT_RELS_HEAD: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>"#;

/// Default font and size for the document (Calibri 11 pt, like a fresh Word file).
const STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:cs="Calibri"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="ru-RU"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/><w:basedOn w:val="Normal"/><w:pPr><w:pBdr><w:left w:val="single" w:sz="18" w:space="8" w:color="9CA3AF"/></w:pBdr><w:ind w:left="480"/></w:pPr></w:style><w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr></w:style></w:styles>"#;

fn zip_error(err: zip::result::ZipError) -> IoError {
    IoError::Parse(format!("not a valid .docx archive: {err}"))
}

pub fn read_document_xml(bytes: &[u8]) -> Result<Vec<u8>, IoError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(zip_error)?;
    let file = archive
        .by_name("word/document.xml")
        .map_err(|_| IoError::Parse("word/document.xml is missing".into()))?;
    let mut xml = Vec::new();
    file.take(MAX_XML_BYTES).read_to_end(&mut xml)?;
    Ok(xml)
}

/// Relationships of word/document.xml: styles plus one external relationship per hyperlink.
fn document_rels(links: &[String]) -> String {
    let mut xml = String::from(DOCUMENT_RELS_HEAD);
    for (i, url) in links.iter().enumerate() {
        let target = url
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('"', "&quot;");
        xml.push_str(&format!(
            "<Relationship Id=\"rIdL{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink\" Target=\"{target}\" TargetMode=\"External\"/>",
            i + 1
        ));
    }
    xml.push_str("</Relationships>");
    xml
}

/// Hyperlink targets by relationship id, read from word/_rels/document.xml.rels (missing file = no links).
pub fn read_links(bytes: &[u8]) -> HashMap<String, String> {
    let Ok(mut archive) = ZipArchive::new(Cursor::new(bytes)) else {
        return HashMap::new();
    };
    let Ok(file) = archive.by_name("word/_rels/document.xml.rels") else {
        return HashMap::new();
    };
    let mut xml = Vec::new();
    if file.take(MAX_XML_BYTES).read_to_end(&mut xml).is_err() {
        return HashMap::new();
    }
    let mut reader = quick_xml::Reader::from_reader(xml.as_slice());
    let (mut buf, mut links) = (Vec::new(), HashMap::new());
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            quick_xml::events::Event::Eof => break,
            quick_xml::events::Event::Start(e) | quick_xml::events::Event::Empty(e)
                if e.local_name().as_ref() == b"Relationship" =>
            {
                let attr = |name: &[u8]| {
                    e.attributes()
                        .flatten()
                        .find(|a| a.key.as_ref() == name)
                        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
                };
                if let (Some(id), Some(target)) = (attr(b"Id"), attr(b"Target")) {
                    links.insert(id, target);
                }
            }
            _ => {}
        }
        buf.clear();
    }
    links
}

/// word/numbering.xml: abstract list 0 = bullets, abstract list 1 = numbers (1. / a. / i. by level);
/// `num` 1 = bullets, `num` 2.. = one numbered list each (every separate numbered list restarts at 1).
fn numbering_xml(number_lists: usize) -> String {
    let level = |i: usize, fmt: &str, text: &str| {
        format!(
            "<w:lvl w:ilvl=\"{i}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"{fmt}\"/><w:lvlText w:val=\"{text}\"/><w:lvlJc w:val=\"left\"/><w:pPr><w:ind w:left=\"{}\" w:hanging=\"360\"/></w:pPr></w:lvl>",
            720 * i + 360
        )
    };
    let bullets: String = (0..9)
        .map(|i| level(i, "bullet", ["\u{2022}", "\u{25E6}", "\u{25AA}"][i % 3]))
        .collect();
    let numbers: String = (0..9)
        .map(|i| {
            level(
                i,
                ["decimal", "lowerLetter", "lowerRoman"][i % 3],
                &format!("%{}.", i + 1),
            )
        })
        .collect();
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:abstractNum w:abstractNumId=\"0\"><w:multiLevelType w:val=\"hybridMultilevel\"/>{bullets}</w:abstractNum><w:abstractNum w:abstractNumId=\"1\"><w:multiLevelType w:val=\"hybridMultilevel\"/>{numbers}</w:abstractNum><w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>"
    );
    for n in 0..number_lists.max(1) {
        xml.push_str(&format!(
            "<w:num w:numId=\"{}\"><w:abstractNumId w:val=\"1\"/><w:lvlOverride w:ilvl=\"0\"><w:startOverride w:val=\"1\"/></w:lvlOverride></w:num>",
            n + 2
        ));
    }
    xml.push_str("</w:numbering>");
    xml
}

fn read_part(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).ok()?;
    let file = archive.by_name(name).ok()?;
    let mut xml = Vec::new();
    file.take(MAX_XML_BYTES).read_to_end(&mut xml).ok()?;
    Some(xml)
}

/// List kind of every `numId` in word/numbering.xml (missing file = no lists).
pub fn read_numbering(bytes: &[u8]) -> HashMap<String, ListKind> {
    use quick_xml::events::Event;
    let Some(xml) = read_part(bytes, "word/numbering.xml") else {
        return HashMap::new();
    };
    let mut reader = quick_xml::Reader::from_reader(xml.as_slice());
    let mut buf = Vec::new();
    let mut abstract_kinds: HashMap<String, ListKind> = HashMap::new();
    let mut nums: Vec<(String, String)> = Vec::new();
    let (mut current_abstract, mut current_num) = (None::<String>, None::<String>);
    let (mut level0, mut got_kind) = (false, false);
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => {
                let attr = |name: &[u8]| {
                    e.attributes()
                        .flatten()
                        .find(|a| a.key.local_name().as_ref() == name)
                        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
                };
                match e.local_name().as_ref() {
                    b"abstractNum" => {
                        current_abstract = attr(b"abstractNumId");
                        (level0, got_kind) = (false, false);
                    }
                    b"lvl" => level0 = attr(b"ilvl").as_deref() == Some("0"),
                    b"numFmt" if level0 && !got_kind => {
                        if let (Some(id), Some(fmt)) = (&current_abstract, attr(b"val")) {
                            abstract_kinds.insert(
                                id.clone(),
                                if fmt == "bullet" {
                                    ListKind::Bullet
                                } else {
                                    ListKind::Number
                                },
                            );
                            got_kind = true;
                        }
                    }
                    b"num" => current_num = attr(b"numId"),
                    b"abstractNumId" => {
                        if let (Some(num), Some(abs)) = (&current_num, attr(b"val")) {
                            nums.push((num.clone(), abs));
                        }
                    }
                    _ => {}
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                b"abstractNum" => current_abstract = None,
                b"num" => current_num = None,
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }
    nums.into_iter()
        .filter_map(|(id, abs)| abstract_kinds.get(&abs).map(|k| (id, *k)))
        .collect()
}

pub fn write(
    document_xml: &str,
    links: &[String],
    number_lists: usize,
) -> Result<Vec<u8>, IoError> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let rels = document_rels(links);
    let numbering = numbering_xml(number_lists);
    let parts = [
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", ROOT_RELS),
        ("word/_rels/document.xml.rels", rels.as_str()),
        ("word/styles.xml", STYLES),
        ("word/numbering.xml", numbering.as_str()),
        ("word/document.xml", document_xml),
    ];
    for (name, content) in parts {
        zip.start_file(name, options).map_err(zip_error)?;
        zip.write_all(content.as_bytes())?;
    }
    Ok(zip.finish().map_err(zip_error)?.into_inner())
}
