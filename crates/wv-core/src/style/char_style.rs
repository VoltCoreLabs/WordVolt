/// Superscript / subscript.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Script {
    #[default]
    Normal,
    Super,
    Sub,
}

/// Character formatting of a run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CharStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    /// Font size in points; `None` = document default.
    pub size_pt: Option<u16>,
    /// 0xRRGGBB; `None` = default text colour.
    pub color: Option<u32>,
    /// 0xRRGGBB background marker; `None` = none.
    pub highlight: Option<u32>,
    /// Font family name; `None` = the document default.
    pub font: Option<String>,
    pub script: Script,
    /// ALL CAPS display (the text itself is unchanged).
    pub caps: bool,
    /// Inline code: monospace font and a light background.
    pub mono: bool,
    /// Hidden behind a block until the caret is inside it.
    pub spoiler: bool,
    /// Hyperlink target.
    pub link: Option<String>,
}
