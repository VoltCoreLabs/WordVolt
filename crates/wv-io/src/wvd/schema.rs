use serde::{Deserialize, Serialize};
use wv_core::{Align, CharStyle, Document, ParaStyle, Paragraph, Run, Script};

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
    #[serde(default, skip_serializing_if = "is_false")]
    quote: bool,
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
        Ok(if paragraphs.is_empty() {
            Document::default()
        } else {
            Document { paragraphs }
        })
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
