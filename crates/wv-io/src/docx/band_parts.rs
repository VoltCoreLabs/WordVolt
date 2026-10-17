use wv_core::{Band, Document};

use super::bands::part_xml;

/// One header or footer part of the .docx.
pub struct BandPart {
    /// "hdr" / "ftr" (root element of the part).
    pub root: &'static str,
    /// "header" / "footer" (relationship and content type name).
    pub kind: &'static str,
    /// "default" / "first" / "even" (`w:type` of the reference).
    pub slot: &'static str,
    /// File name inside word/ (header1.xml, footer2.xml, ...).
    pub name: String,
    /// Relationship id (rIdH1, rIdF2, ...).
    pub rid: String,
    pub band: Band,
}

/// The parts to write: the main header/footer, plus the first-page and even-page ones when their
/// switches are on. Empty bands are left out.
pub fn parts(doc: &Document) -> Vec<BandPart> {
    let v = &doc.variants;
    let slots: [(&str, bool, (&Band, &Band)); 3] = [
        ("default", true, (&doc.header, &doc.footer)),
        ("first", v.different_first, (&v.first.0, &v.first.1)),
        ("even", v.odd_even, (&v.even.0, &v.even.1)),
    ];
    let mut out = Vec::new();
    for (n, (slot, on, (header, footer))) in slots.into_iter().enumerate() {
        if !on {
            continue;
        }
        let n = n + 1;
        for (band, root, kind, letter) in [
            (header, "hdr", "header", 'H'),
            (footer, "ftr", "footer", 'F'),
        ] {
            if !band.is_empty() {
                out.push(BandPart {
                    root,
                    kind,
                    slot,
                    name: format!("{kind}{n}.xml"),
                    rid: format!("rId{letter}{n}"),
                    band: band.clone(),
                });
            }
        }
    }
    out
}

impl BandPart {
    pub fn xml(&self) -> String {
        part_xml(self.root, &self.band)
    }

    /// `<w:headerReference .../>` / `<w:footerReference .../>`.
    pub fn reference(&self) -> String {
        format!(
            "<w:{}Reference w:type=\"{}\" r:id=\"{}\"/>",
            self.kind, self.slot, self.rid
        )
    }
}
