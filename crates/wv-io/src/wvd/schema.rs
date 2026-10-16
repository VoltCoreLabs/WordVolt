use serde::{Deserialize, Serialize};
use wv_core::{Align, CharStyle, Document, ParaStyle, Paragraph, Run};

use crate::IoError;

const FORMAT: &str = "wvd";
const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct File {
    format: String,
    version: u32,
    paragraphs: Vec<Para>,
}

#[derive(Serialize, Deserialize)]
struct Para {
    #[serde(default)]
    align: String,
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
                runs: p
                    .runs
                    .iter()
                    .map(|r| RunDto {
                        text: r.text.clone(),
                        bold: r.style.bold,
                        italic: r.style.italic,
                        underline: r.style.underline,
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
                },
                runs: p
                    .runs
                    .into_iter()
                    .map(|r| {
                        let style = CharStyle {
                            bold: r.bold,
                            italic: r.italic,
                            underline: r.underline,
                            size_pt: r.size_pt,
                            color: r.color,
                            font: r.font,
                        };
                        Run::new(r.text, style)
                    })
                    .collect(),
            })
            .collect();
        Ok(if paragraphs.is_empty() {
            Document::default()
        } else {
            Document { paragraphs }
        })
    }
}
