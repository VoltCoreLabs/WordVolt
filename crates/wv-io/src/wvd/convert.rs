use serde::{Deserialize, Serialize};
use wv_core::{Align, CharStyle, ListKind, Run, Script, Wrap};

#[derive(Serialize, Deserialize)]
pub(super) struct RunDto {
    pub(super) text: String,
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
    /// Footnote mark: the id of the footnote.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<u32>,
    /// The text is the anchor of this comment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    comment: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    size_pt: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    font: Option<String>,
}

pub(super) fn is_false(v: &bool) -> bool {
    !*v
}

pub(super) fn align_name(a: Align) -> &'static str {
    match a {
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
        Align::Justify => "justify",
    }
}

pub(super) fn align_from(name: &str) -> Align {
    match name {
        "center" => Align::Center,
        "right" => Align::Right,
        "justify" => Align::Justify,
        _ => Align::Left,
    }
}

pub(super) fn band_align_name(align: Align) -> String {
    match align {
        Align::Center => String::new(),
        other => align_name(other).to_string(),
    }
}

pub(super) fn band_align_from(name: &str) -> Align {
    match name {
        "left" => Align::Left,
        "right" => Align::Right,
        _ => Align::Center,
    }
}

pub(super) fn script_name(s: Script) -> &'static str {
    match s {
        Script::Normal => "",
        Script::Super => "super",
        Script::Sub => "sub",
    }
}

pub(super) fn script_from(name: &str) -> Script {
    match name {
        "super" => Script::Super,
        "sub" => Script::Sub,
        _ => Script::Normal,
    }
}

pub(super) fn is_zero(v: &u8) -> bool {
    *v == 0
}

pub(super) fn list_name(kind: Option<ListKind>) -> &'static str {
    match kind {
        None => "",
        Some(ListKind::Bullet) => "bullet",
        Some(ListKind::Number) => "number",
    }
}

pub(super) fn list_from(name: &str) -> Option<ListKind> {
    match name {
        "bullet" => Some(ListKind::Bullet),
        "number" => Some(ListKind::Number),
        _ => None,
    }
}

pub(super) fn run_dto(r: &Run) -> RunDto {
    RunDto {
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
        note: r.style.note,
        comment: r.style.comment,
        size_pt: r.style.size_pt,
        color: r.style.color,
        font: r.style.font.clone(),
    }
}

pub(super) fn run_of(r: RunDto) -> Run {
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
        note: r.note,
        comment: r.comment,
        size_pt: r.size_pt,
        color: r.color,
        font: r.font,
    };
    Run::new(r.text, style)
}

pub(super) fn wrap_name(w: Wrap) -> &'static str {
    match w {
        Wrap::Square => "square",
        Wrap::Tight => "tight",
        Wrap::TopBottom => "topbottom",
        Wrap::Front => "front",
        Wrap::Behind => "behind",
    }
}

pub(super) fn wrap_from(name: &str) -> Wrap {
    match name {
        "square" => Wrap::Square,
        "tight" => Wrap::Tight,
        "front" => Wrap::Front,
        "behind" => Wrap::Behind,
        _ => Wrap::TopBottom,
    }
}
