use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use wv_core::{
    Cell, Document, ImageData, ImageFormat, ImageRef, NamedStyle, PageSpec, ParaStyle, Paragraph,
};

use super::convert::*;
use super::dto::*;
use super::variants::{VariantsDto, band_of, styled_runs};
use crate::IoError;

const FORMAT: &str = "wvd";
const VERSION: u32 = 1;

impl File {
    pub fn from_document(doc: &Document) -> Self {
        let paragraphs = doc
            .paragraphs
            .iter()
            .map(|p| Para {
                align: align_name(p.style.align).to_string(),
                quote: p.style.quote,
                toc: p.style.toc,
                section: p.style.section,
                custom: p.style.custom,
                note: p.style.note,
                image: p.style.image.map(|i| ImageRefDto {
                    id: i.id,
                    width: i.width,
                    height: i.height,
                    float: i.float.map(|f| FloatDto {
                        x: f.x,
                        y: f.y,
                        wrap: wrap_name(f.wrap).to_string(),
                    }),
                    angle: i.angle,
                    crop: (!i.crop.is_none()).then_some([
                        i.crop.left,
                        i.crop.top,
                        i.crop.right,
                        i.crop.bottom,
                    ]),
                }),
                cell: p.style.cell.map(|c| CellDto {
                    table: c.table,
                    row: c.row,
                    col: c.col,
                    shade: c.shade,
                    colspan: c.colspan,
                    rowspan: c.rowspan,
                }),
                style: if p.style.named == NamedStyle::Normal {
                    String::new()
                } else {
                    p.style.named.id().to_string()
                },
                list: list_name(p.style.list).to_string(),
                indent: p.style.indent,
                line_spacing: p.style.line_spacing,
                first_line: p.style.first_line,
                page_break: p.style.page_break,
                left_extra: p.style.left_extra,
                tabs: p.style.tabs.iter().map(|t| t.pos).collect(),
                tab_kinds: if p
                    .style
                    .tabs
                    .iter()
                    .all(|t| t.kind == wv_core::TabKind::Left)
                {
                    Vec::new()
                } else {
                    p.style
                        .tabs
                        .iter()
                        .map(|t| {
                            wv_core::TabKind::ALL
                                .iter()
                                .position(|k| *k == t.kind)
                                .unwrap_or(0) as u8
                        })
                        .collect()
                },
                tab_leaders: if p
                    .style
                    .tabs
                    .iter()
                    .all(|t| t.leader == wv_core::TabLeader::None)
                {
                    Vec::new()
                } else {
                    p.style
                        .tabs
                        .iter()
                        .map(|t| {
                            wv_core::TabLeader::ALL
                                .iter()
                                .position(|k| *k == t.leader)
                                .unwrap_or(0) as u8
                        })
                        .collect()
                },
                right_indent: p.style.right_indent,
                space_before: p.style.space_before,
                space_after: p.style.space_after,
                runs: p.runs.iter().map(run_dto).collect(),
            })
            .collect();
        Self {
            format: FORMAT.into(),
            version: VERSION,
            paragraphs,
            header: doc.header.text(),
            footer: doc.footer.text(),
            header_align: band_align_name(doc.header.align),
            footer_align: band_align_name(doc.footer.align),
            header_runs: styled_runs(&doc.header),
            footer_runs: styled_runs(&doc.footer),
            band_variants: VariantsDto::of(&doc.variants),
            table_widths: Vec::new(),
            comments: doc
                .comments
                .iter()
                .map(|c| CommentDto {
                    id: c.id,
                    author: c.author.clone(),
                    text: c.text.clone(),
                    date: c.date.clone(),
                })
                .collect(),
            revisions: doc
                .revisions
                .iter()
                .map(|r| CommentDto {
                    id: r.id,
                    author: r.author.clone(),
                    text: String::new(),
                    date: r.date.clone(),
                })
                .collect(),
            table_looks: doc
                .table_looks
                .iter()
                .map(|(table, look)| TableLookDto {
                    table: *table,
                    widths: look.widths.clone(),
                    indent: look.indent,
                    rows: look.rows.clone(),
                })
                .collect(),
            page: (doc.page != PageSpec::default()).then(|| PageDto::of(&doc.page)),
            bookmarks: doc
                .bookmarks
                .iter()
                .map(|b| BookmarkDto {
                    id: b.id,
                    name: b.name.clone(),
                })
                .collect(),
            default_font: doc.default_font.clone(),
            default_size_pt: doc.default_size_pt,
            section_bands: doc
                .section_bands
                .iter()
                .map(|(id, bands)| super::variants::SectionBandsDto::of(*id, bands))
                .collect(),
            styles: doc
                .styles
                .iter()
                .map(|(id, s)| super::styles::UserStyleDto::of(*id, s))
                .collect(),
            sections: doc
                .sections
                .iter()
                .map(|(id, page)| SectionDto {
                    id: *id,
                    page: PageDto::of(page),
                })
                .collect(),
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
                    section: p.section,
                    custom: p.custom,
                    note: p.note,
                    image: p.image.map(|i| ImageRef {
                        id: i.id,
                        width: i.width,
                        height: i.height,
                        float: i.float.map(|f| wv_core::Float {
                            x: f.x,
                            y: f.y,
                            wrap: wrap_from(&f.wrap),
                        }),
                        angle: i.angle.rem_euclid(360.0),
                        crop: i.crop.map_or(
                            wv_core::Crop::default(),
                            |[left, top, right, bottom]| wv_core::Crop {
                                left: left.clamp(0.0, 0.9),
                                top: top.clamp(0.0, 0.9),
                                right: right.clamp(0.0, 0.9),
                                bottom: bottom.clamp(0.0, 0.9),
                            },
                        ),
                    }),
                    cell: p.cell.map(|c| Cell {
                        shade: c.shade,
                        colspan: c.colspan.max(1),
                        rowspan: c.rowspan.max(1),
                        ..Cell::at(c.table, c.row, c.col)
                    }),
                    named: NamedStyle::from_id(&p.style).unwrap_or_default(),
                    list: list_from(&p.list),
                    indent: p.indent.min(8),
                    line_spacing: p.line_spacing,
                    first_line: p.first_line,
                    page_break: p.page_break,
                    left_extra: p.left_extra,
                    tabs: p
                        .tabs
                        .iter()
                        .enumerate()
                        .take(wv_core::MAX_TABS)
                        .map(|(i, pos)| wv_core::TabStop {
                            pos: *pos,
                            kind: p
                                .tab_kinds
                                .get(i)
                                .and_then(|k| wv_core::TabKind::ALL.get(usize::from(*k)))
                                .copied()
                                .unwrap_or_default(),
                            leader: p
                                .tab_leaders
                                .get(i)
                                .and_then(|k| wv_core::TabLeader::ALL.get(usize::from(*k)))
                                .copied()
                                .unwrap_or_default(),
                        })
                        .collect(),
                    right_indent: p.right_indent,
                    space_before: p.space_before,
                    space_after: p.space_after,
                },
                runs: p.runs.into_iter().map(run_of).collect(),
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
        if let Some(spec) = self.page.and_then(PageDto::spec) {
            doc.page = spec;
        }
        doc.bookmarks = self
            .bookmarks
            .into_iter()
            .map(|b| wv_core::Bookmark {
                id: b.id,
                name: b.name,
            })
            .collect();
        doc.default_font = self.default_font;
        doc.default_size_pt = self.default_size_pt;
        doc.section_bands = self
            .section_bands
            .into_iter()
            .map(super::variants::SectionBandsDto::bands)
            .collect();
        doc.styles = self
            .styles
            .into_iter()
            .map(super::styles::UserStyleDto::style)
            .collect();
        doc.sections = self
            .sections
            .into_iter()
            .filter_map(|s| Some((s.id, s.page.spec()?)))
            .collect();
        doc.header = band_of(self.header, self.header_runs, &self.header_align);
        doc.footer = band_of(self.footer, self.footer_runs, &self.footer_align);
        doc.comments = self
            .comments
            .into_iter()
            .map(|c| wv_core::Comment {
                id: c.id,
                author: c.author,
                text: c.text,
                date: c.date,
            })
            .collect();
        doc.revisions = self
            .revisions
            .into_iter()
            .map(|r| wv_core::Revision {
                id: r.id,
                author: r.author,
                date: r.date,
            })
            .collect();
        for (table, widths) in self.table_widths {
            doc.table_looks.insert(
                table,
                wv_core::TableLook {
                    widths,
                    ..Default::default()
                },
            );
        }
        for look in self.table_looks {
            doc.table_looks.insert(
                look.table,
                wv_core::TableLook {
                    widths: look.widths,
                    indent: look.indent,
                    rows: look.rows,
                },
            );
        }
        doc.variants = self
            .band_variants
            .map(VariantsDto::variants)
            .unwrap_or_default();
        Ok(doc)
    }
}
