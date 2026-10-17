use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use serde::{Deserialize, Serialize};
use wv_core::{
    Align, Band, Cell, CharStyle, Document, ImageData, ImageFormat, ImageRef, ListKind, NamedStyle,
    PageSpec, ParaStyle, Paragraph, Run, Script,
};

use crate::IoError;

const FORMAT: &str = "wvd";
const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct File {
    format: String,
    version: u32,
    paragraphs: Vec<Para>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    header: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    footer: String,
    /// "left" / "right"; empty = centred.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    header_align: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    footer_align: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    images: Vec<ImageDto>,
    /// Page size and margins in px; absent = A4 with 2.54 cm margins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    page: Option<PageDto>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct PageDto {
    width: f32,
    height: f32,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
    #[serde(default)]
    gutter: f32,
    #[serde(default = "band_distance")]
    header_distance: f32,
    #[serde(default = "band_distance")]
    footer_distance: f32,
}

fn band_distance() -> f32 {
    wv_core::BAND_DISTANCE
}

/// Embedded picture: format name and base64 data.
#[derive(Serialize, Deserialize)]
struct ImageDto {
    format: String,
    data: String,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct ImageRefDto {
    id: u32,
    width: f32,
    height: f32,
}

/// Table cell of a paragraph.
#[derive(Serialize, Deserialize, Clone, Copy)]
struct CellDto {
    table: u32,
    row: u16,
    col: u16,
}

#[derive(Serialize, Deserialize)]
struct Para {
    #[serde(default)]
    align: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    style: String,
    #[serde(default, skip_serializing_if = "is_false")]
    quote: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    list: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cell: Option<CellDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    image: Option<ImageRefDto>,
    #[serde(default, skip_serializing_if = "is_false")]
    toc: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    indent: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    line_spacing: Option<f32>,
    runs: Vec<RunDto>,
}

#[derive(Serialize, Deserialize)]
struct RunDto {
    text: String,
    #[serde(default, skip_serializing_if = "is_false")]
    bold: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    italic: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    underline: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    strike: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    highlight: Option<u32>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    script: String,
    #[serde(default, skip_serializing_if = "is_false")]
    caps: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    mono: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    spoiler: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    link: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    size_pt: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    font: Option<String>,
}

fn is_false(v: &bool) -> bool {
    !*v
}

fn align_name(a: Align) -> &'static str {
    match a {
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
        Align::Justify => "justify",
    }
}

fn align_from(name: &str) -> Align {
    match name {
        "center" => Align::Center,
        "right" => Align::Right,
        "justify" => Align::Justify,
        _ => Align::Left,
    }
}

impl File {
    pub fn from_document(doc: &Document) -> Self {
        let paragraphs = doc
            .paragraphs
            .iter()
            .map(|p| Para {
                align: align_name(p.style.align).to_string(),
                quote: p.style.quote,
                toc: p.style.toc,
                image: p.style.image.map(|i| ImageRefDto {
                    id: i.id,
                    width: i.width,
                    height: i.height,
                }),
                cell: p.style.cell.map(|c| CellDto {
                    table: c.table,
                    row: c.row,
                    col: c.col,
                }),
                style: if p.style.named == NamedStyle::Normal {
                    String::new()
                } else {
                    p.style.named.id().to_string()
                },
                list: list_name(p.style.list).to_string(),
                indent: p.style.indent,
                line_spacing: p.style.line_spacing,
                runs: p
                    .runs
                    .iter()
                    .map(|r| RunDto {
                        text: r.text.clone(),
                        bold: r.style.bold,
                        italic: r.style.italic,
                        underline: r.style.underline,
                        strike: r.style.strike,
                        highlight: r.style.highlight,
                        script: script_name(r.style.script).to_string(),
                        caps: r.style.caps,
                        mono: r.style.mono,
                        spoiler: r.style.spoiler,
                        link: r.style.link.clone(),
                        size_pt: r.style.size_pt,
                        color: r.style.color,
                        font: r.style.font.clone(),
                    })
                    .collect(),
            })
            .collect();
        Self {
            format: FORMAT.into(),
            version: VERSION,
            paragraphs,
            header: doc.header.text.clone(),
            footer: doc.footer.text.clone(),
            header_align: band_align_name(doc.header.align),
            footer_align: band_align_name(doc.footer.align),
            page: (doc.page != PageSpec::default()).then_some(PageDto {
                width: doc.page.width,
                height: doc.page.height,
                top: doc.page.margin_top,
                right: doc.page.margin_right,
                bottom: doc.page.margin_bottom,
                left: doc.page.margin_left,
                gutter: doc.page.gutter,
                header_distance: doc.page.header_distance,
                footer_distance: doc.page.footer_distance,
            }),
            images: doc
                .images
                .iter()
                .map(|i| ImageDto {
                    format: i.format.extension().to_string(),
                    data: B64.encode(i.bytes.as_slice()),
                })
                .collect(),
        }
    }

    pub fn into_document(self) -> Result<Document, IoError> {
        if self.format != FORMAT {
            return Err(IoError::Parse(format!(
                "unknown format \"{}\"",
                self.format
            )));
        }
        if self.version > VERSION {
            return Err(IoError::Parse(format!(
                "file version {} is newer than supported {VERSION}",
                self.version
            )));
        }
        let paragraphs: Vec<Paragraph> = self
            .paragraphs
            .into_iter()
            .map(|p| Paragraph {
                style: ParaStyle {
                    align: align_from(&p.align),
                    quote: p.quote,
                    toc: p.toc,
                    image: p.image.map(|i| ImageRef {
                        id: i.id,
                        width: i.width,
                        height: i.height,
                    }),
                    cell: p.cell.map(|c| Cell {
                        table: c.table,
                        row: c.row,
                        col: c.col,
                    }),
                    named: NamedStyle::from_id(&p.style).unwrap_or_default(),
                    list: list_from(&p.list),
                    indent: p.indent.min(8),
                    line_spacing: p.line_spacing,
                },
                runs: p
                    .runs
                    .into_iter()
                    .map(|r| {
                        let style = CharStyle {
                            bold: r.bold,
                            italic: r.italic,
                            underline: r.underline,
                            strike: r.strike,
                            highlight: r.highlight,
                            script: script_from(&r.script),
                            caps: r.caps,
                            mono: r.mono,
                            spoiler: r.spoiler,
                            link: r.link,
                            size_pt: r.size_pt,
                            color: r.color,
                            font: r.font,
                        };
                        Run::new(r.text, style)
                    })
                    .collect(),
            })
            .collect();
        let mut doc = if paragraphs.is_empty() {
            Document::default()
        } else {
            Document {
                paragraphs,
                ..Document::default()
            }
        };
        for image in self.images {
            let format = ImageFormat::from_extension(&image.format).ok_or_else(|| {
                IoError::Parse(format!("unknown image format \"{}\"", image.format))
            })?;
            let bytes = B64
                .decode(image.data)
                .map_err(|e| IoError::Parse(format!("image data: {e}")))?;
            doc.images.push(ImageData {
                format,
                bytes: std::sync::Arc::new(bytes),
            });
        }
        // Pictures that point past the stored images are dropped.
        let count = doc.images.len() as u32;
        for p in &mut doc.paragraphs {
            if p.style.image.is_some_and(|i| i.id >= count) {
                p.style.image = None;
            }
        }
        if let Some(p) = self.page.filter(|p| p.width >= 100.0 && p.height >= 100.0) {
            doc.page = PageSpec {
                width: p.width,
                height: p.height,
                margin_top: p.top.max(0.0),
                margin_right: p.right.max(0.0),
                margin_bottom: p.bottom.max(0.0),
                margin_left: p.left.max(0.0),
                gutter: p.gutter.max(0.0),
                header_distance: p.header_distance.max(0.0),
                footer_distance: p.footer_distance.max(0.0),
            };
        }
        doc.header = Band::new(self.header).aligned(band_align_from(&self.header_align));
        doc.footer = Band::new(self.footer).aligned(band_align_from(&self.footer_align));
        Ok(doc)
    }
}

fn band_align_name(align: Align) -> String {
    match align {
        Align::Center => String::new(),
        other => align_name(other).to_string(),
    }
}

fn band_align_from(name: &str) -> Align {
    match name {
        "left" => Align::Left,
        "right" => Align::Right,
        _ => Align::Center,
    }
}

fn script_name(s: Script) -> &'static str {
    match s {
        Script::Normal => "",
        Script::Super => "super",
        Script::Sub => "sub",
    }
}

fn script_from(name: &str) -> Script {
    match name {
        "super" => Script::Super,
        "sub" => Script::Sub,
        _ => Script::Normal,
    }
}

fn is_zero(v: &u8) -> bool {
    *v == 0
}

fn list_name(kind: Option<ListKind>) -> &'static str {
    match kind {
        None => "",
        Some(ListKind::Bullet) => "bullet",
        Some(ListKind::Number) => "number",
    }
}

fn list_from(name: &str) -> Option<ListKind> {
    match name {
        "bullet" => Some(ListKind::Bullet),
        "number" => Some(ListKind::Number),
        _ => None,
    }
}
