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
                image: p.style.image.map(|i| ImageRefDto {
                    id: i.id,
                    width: i.width,
                    height: i.height,
                    float: i.float.map(|f| FloatDto {
                        x: f.x,
                        y: f.y,
                        wrap: wrap_name(f.wrap).to_string(),
                    }),
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
                tabs: p.style.tabs.clone(),
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
                columns: doc.page.columns,
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
                        float: i.float.map(|f| wv_core::Float {
                            x: f.x,
                            y: f.y,
                            wrap: wrap_from(&f.wrap),
                        }),
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
                    tabs: p.tabs.iter().copied().take(wv_core::MAX_TABS).collect(),
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
                columns: p.columns.clamp(1, wv_core::MAX_COLUMNS),
            };
        }
        doc.header = band_of(self.header, self.header_runs, &self.header_align);
        doc.footer = band_of(self.footer, self.footer_runs, &self.footer_align);
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
