/// Character formatting of a run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CharStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    /// Font size in points; `None` = document default.
    pub size_pt: Option<u16>,
    /// 0xRRGGBB; `None` = default text colour.
    pub color: Option<u32>,
    /// Font family name; `None` = the document default.
    pub font: Option<String>,
}
