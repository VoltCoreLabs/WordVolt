use quick_xml::events::BytesStart;
use wv_core::{Align, Cell, CharStyle, Script};

use super::State;
use crate::docx::read_util::{attr, flag, named_from_docx, named_highlight};

impl State {
    pub(super) fn start(&mut self, e: &BytesStart, empty: bool) {
        let name = e.local_name();
        match name.as_ref() {
            b"p" => {
                self.begin_paragraph();
                if empty {
                    self.end_paragraph();
                }
            }
            b"tbl" if !empty => {
                self.table.depth += 1;
                if self.table.depth == 1 {
                    self.end_paragraph();
                    self.table.start = self.paragraphs.len();
                    self.table.row = None;
                    self.table.raw = Default::default();
                }
            }
            b"tr" if self.table.depth == 1 => {
                self.table.raw.rows.push(0);
                self.table.row = Some(self.table.row.map_or(0, |r| r + 1));
                self.table.col = 0;
            }
            b"tc" if self.table.depth == 1 && !empty => {
                self.table.span = 1;
                self.table.continues = false;
                self.table.cell_start = self.paragraphs.len();
                self.table.cell = Some(Cell::at(
                    self.table.next_id,
                    self.table.row.unwrap_or(0),
                    self.table.col,
                ));
            }
            b"tblGrid" if self.table.depth == 1 => self.table.raw.grid.clear(),
            b"gridCol" if self.table.depth == 1 => {
                let width = attr(e, b"w").and_then(|v| v.parse().ok()).unwrap_or(0);
                self.table.raw.grid.push(width);
            }
            b"tblInd" if self.table.depth == 1 => {
                self.table.raw.indent = attr(e, b"w").and_then(|v| v.parse().ok()).unwrap_or(0);
            }
            b"trHeight" if self.table.depth == 1 => {
                let height = attr(e, b"val").and_then(|v| v.parse().ok()).unwrap_or(0);
                if let Some(last) = self.table.raw.rows.last_mut() {
                    *last = height;
                }
            }
            b"shd"
                if self.table.depth == 1
                    && !self.in_paragraph_props
                    && !self.in_run_props
                    && self.table.cell.is_some() =>
            {
                let fill = attr(e, b"fill").and_then(|v| crate::docx::table_read::parse_fill(&v));
                if let Some(cell) = self.table.cell.as_mut() {
                    cell.shade = fill;
                }
            }
            b"gridSpan" if self.table.depth == 1 && self.table.cell.is_some() => {
                self.table.span = attr(e, b"val")
                    .and_then(|v| v.parse::<u16>().ok())
                    .map_or(1, |n| n.clamp(1, 63));
                if let Some(cell) = self.table.cell.as_mut() {
                    cell.colspan = self.table.span as u8;
                }
            }
            b"vMerge" if self.table.depth == 1 && self.table.cell.is_some() => {
                self.table.continues = attr(e, b"val").as_deref() != Some("restart");
            }
            b"drawing" if !empty => {
                self.pictures.in_drawing = true;
                self.pictures.embed = None;
                self.pictures.size = (0.0, 0.0);
            }
            b"extent" if self.pictures.in_drawing && self.pictures.size.0 == 0.0 => {
                let px = |name: &[u8]| {
                    attr(e, name).map_or(0.0, |v| crate::docx::picture::emu_to_px(&v))
                };
                self.pictures.size = (px(b"cx"), px(b"cy"));
            }
            b"blip" if self.pictures.in_drawing => self.pictures.embed = attr(e, b"embed"),
            b"pgSz" => {
                let px = |name: &[u8]| attr(e, name).and_then(|v| v.parse::<f32>().ok());
                if let (Some(w), Some(h)) = (px(b"w"), px(b"h"))
                    && w >= 1500.0
                    && h >= 1500.0
                {
                    (self.page.width, self.page.height) = (w / 15.0, h / 15.0);
                }
            }
            b"pgMar" => {
                let px = |name: &[u8], old: f32| {
                    attr(e, name)
                        .and_then(|v| v.parse::<f32>().ok())
                        .map_or(old, |v| (v / 15.0).max(0.0))
                };
                let p = &mut self.page;
                p.margin_top = px(b"top", p.margin_top);
                p.margin_right = px(b"right", p.margin_right);
                p.margin_bottom = px(b"bottom", p.margin_bottom);
                p.margin_left = px(b"left", p.margin_left);
                p.gutter = px(b"gutter", p.gutter);
                p.header_distance = px(b"header", p.header_distance);
                p.footer_distance = px(b"footer", p.footer_distance);
            }
            b"cols" => {
                let num = attr(e, b"num").and_then(|v| v.parse::<u8>().ok());
                self.page.columns = num.unwrap_or(1).clamp(1, wv_core::MAX_COLUMNS);
            }
            b"pPr" => self.in_paragraph_props = true,
            b"ilvl" if self.in_paragraph_props => {
                self.indent = attr(e, b"val")
                    .and_then(|v| v.parse::<u8>().ok())
                    .unwrap_or(0)
                    .min(8);
            }
            b"numId" if self.in_paragraph_props => {
                self.list = attr(e, b"val").and_then(|id| self.numbering.get(&id).copied());
            }
            b"ind" if self.in_paragraph_props => {
                let twips = |name: &[u8]| attr(e, name).and_then(|v| v.parse::<f32>().ok());
                self.first_line =
                    twips(b"firstLine").map_or(0, |t| (t / 20.0).round().clamp(0.0, 255.0) as u8);
                if self.list.is_none() {
                    let left = twips(b"left")
                        .or_else(|| twips(b"start"))
                        .unwrap_or(0.0)
                        .max(0.0);
                    // Whole 0.5 inch steps are the indent level, the rest is kept in points.
                    let steps = (left / 720.0).floor().min(8.0);
                    self.indent = steps as u8;
                    self.left_extra =
                        ((left - steps * 720.0) / 20.0).round().clamp(0.0, 255.0) as u8;
                }
            }
            b"tab" if self.in_paragraph_props => {
                let left = attr(e, b"val").is_none_or(|v| matches!(v.as_str(), "left" | "start"));
                let pos = attr(e, b"pos").and_then(|v| v.parse::<f32>().ok());
                if let (true, Some(pos), true) = (left, pos, self.tabs.len() < wv_core::MAX_TABS) {
                    self.tabs
                        .push((pos / 20.0).round().clamp(0.0, 65535.0) as u16);
                }
            }
            b"spacing" if self.in_paragraph_props => {
                let pt = |name: &[u8]| {
                    attr(e, name)
                        .and_then(|v| v.parse::<f32>().ok())
                        .map(|twips| (twips / 20.0).round().clamp(0.0, 255.0) as u8)
                };
                self.space_before = pt(b"before");
                self.space_after = pt(b"after");
                let auto = attr(e, b"lineRule").is_none_or(|rule| rule == "auto");
                if let (true, Some(line)) =
                    (auto, attr(e, b"line").and_then(|v| v.parse::<f32>().ok()))
                {
                    self.line_spacing = Some((line / 240.0).clamp(0.5, 5.0));
                }
            }
            b"hyperlink" if !empty => {
                self.link = attr(e, b"id").and_then(|id| self.links.get(&id).cloned())
            }
            b"jc" if self.in_paragraph_props => {
                self.align = match attr(e, b"val").as_deref() {
                    Some("center") => Align::Center,
                    Some("right" | "end") => Align::Right,
                    Some("both" | "distribute") => Align::Justify,
                    _ => Align::Left,
                };
            }
            b"pStyle" if self.in_paragraph_props => {
                let style = attr(e, b"val").unwrap_or_default();
                self.named = named_from_docx(&style);
                self.quote = style == "Quote";
            }
            b"r" => self.run = CharStyle::default(),
            b"rPr" if !self.in_paragraph_props => self.in_run_props = true,
            b"rFonts" if self.in_run_props => {
                self.run.font = attr(e, b"ascii")
                    .or_else(|| attr(e, b"hAnsi"))
                    .filter(|f| !f.is_empty());
            }
            b"b" if self.in_run_props => self.run.bold = flag(e),
            b"i" if self.in_run_props => self.run.italic = flag(e),
            b"u" if self.in_run_props => self.run.underline = flag(e),
            b"caps" if self.in_run_props => self.run.caps = flag(e),
            b"strike" | b"dstrike" if self.in_run_props => self.run.strike = flag(e),
            b"highlight" if self.in_run_props => {
                self.run.highlight = attr(e, b"val").as_deref().and_then(named_highlight)
            }
            b"shd" if self.in_run_props => {
                if let Some(fill) = attr(e, b"fill").filter(|f| f.len() == 6) {
                    self.run.highlight = u32::from_str_radix(&fill, 16).ok();
                }
            }
            b"vertAlign" if self.in_run_props => {
                self.run.script = match attr(e, b"val").as_deref() {
                    Some("superscript") => Script::Super,
                    Some("subscript") => Script::Sub,
                    _ => Script::Normal,
                };
            }
            b"sz" if self.in_run_props => {
                self.run.size_pt = attr(e, b"val")
                    .and_then(|v| v.parse::<u16>().ok())
                    .map(|half| half.div_ceil(2));
            }
            b"color" if self.in_run_props => {
                self.run.color = attr(e, b"val")
                    .and_then(|v| u32::from_str_radix(&v, 16).ok().filter(|_| v.len() == 6));
            }
            b"t" => self.in_text = !empty,
            b"tab" if !self.in_run_props && !self.in_paragraph_props => self.text("\t"),
            b"br" if attr(e, b"type").as_deref() == Some("page") => self.line_break(true),
            b"br" | b"cr" => self.line_break(false),
            b"pageBreakBefore" if self.in_paragraph_props => {
                self.page_break =
                    attr(e, b"val").is_none_or(|v| !matches!(v.as_str(), "0" | "false"));
            }
            _ => {}
        }
    }
}
