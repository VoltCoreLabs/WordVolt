use std::rc::Rc;

use wv_core::Wrap;

use crate::notes::NoteSet;
use crate::page::{COLUMN_GAP, EPS, Page, PageSetup, PlacedFloat, Slice};
use crate::paragraph::ParagraphLayout;
use crate::table::{Block, TABLE_GAP};

/// Space kept between a picture and the text above/below it.
const FLOAT_GAP: f32 = 6.0;

/// A band of a page that text stays out of (the vertical extent of a top-and-bottom picture).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Band {
    page: usize,
    top: f32,
    bottom: f32,
}

/// Flows body paragraphs and tables onto pages. A paragraph may continue on the next page (between
/// lines); a table row never splits (the table continues on the next page). The gap above a
/// paragraph is dropped at the top of a page. Text stops above and goes on below floating pictures
/// with top-and-bottom wrapping; which page such a picture is on depends on the text before it, so
/// the flow is repeated until the pictures stay where they are.
pub fn paginate(
    paragraphs: &[Rc<ParagraphLayout>],
    blocks: &[Block],
    setup: &PageSetup,
    notes: &NoteSet,
) -> Vec<Page> {
    let mut bands: Vec<Band> = Vec::new();
    let mut result = pass(paragraphs, blocks, setup, notes, &bands);
    for _ in 0..3 {
        let next: Vec<Band> = result
            .1
            .iter()
            .filter(|(_, f)| f.wrap == Wrap::TopBottom)
            .map(|(page, f)| Band {
                page: *page,
                top: f.y,
                bottom: f.y + f.height,
            })
            .collect();
        if next == bands {
            break;
        }
        bands = next;
        result = pass(paragraphs, blocks, setup, notes, &bands);
    }
    let (mut pages, floats) = result;
    for (page, float) in floats {
        if let Some(p) = pages.get_mut(page) {
            p.floats.push(float);
        }
    }
    pages
}

/// One flow of the blocks: the pages and the floating pictures with their page.
fn pass(
    paragraphs: &[Rc<ParagraphLayout>],
    blocks: &[Block],
    setup: &PageSetup,
    notes: &NoteSet,
    bands: &[Band],
) -> (Vec<Page>, Vec<(usize, PlacedFloat)>) {
    let columns = usize::from(setup.columns.max(1));
    let mut floats = Vec::new();
    if columns == 1 {
        let pages = flow(paragraphs, blocks, setup, notes, 1, bands, &mut floats);
        return (pages, floats);
    }
    // Flow the text through column-wide "pages", then set each run of `columns` of them side by
    // side on one real page.
    let narrow = PageSetup {
        margin_right: setup.margin_right + setup.content_width() - setup.column_width(),
        ..setup.clone()
    };
    let step = setup.column_width() + COLUMN_GAP;
    let pages = flow(
        paragraphs,
        blocks,
        &narrow,
        notes,
        columns,
        bands,
        &mut floats,
    )
    .chunks(columns)
    .map(|group| {
        let mut page = Page::default();
        for (i, part) in group.iter().enumerate() {
            let dx = i as f32 * step;
            page.slices.extend(part.slices.iter().cloned().map(|mut s| {
                s.x += dx;
                s
            }));
            page.tables.extend(part.tables.iter().cloned().map(|mut t| {
                t.shift(dx);
                t
            }));
        }
        page
    })
    .collect();
    (pages, floats)
}

/// Flows the blocks onto `setup`-sized pages; a page break skips to the next multiple of
/// `columns` (the next real page when the pages are columns). `floats` collects the floating
/// pictures with the (real) page they are on.
fn flow(
    paragraphs: &[Rc<ParagraphLayout>],
    blocks: &[Block],
    setup: &PageSetup,
    notes: &NoteSet,
    columns: usize,
    bands: &[Band],
    floats: &mut Vec<(usize, PlacedFloat)>,
) -> Vec<Page> {
    let limit = setup.content_height();
    let content_width = setup.content_width();
    let mut pages = vec![Page::default()];
    let mut y = 0.0;
    // The footnotes with a mark on each page (they take room at its bottom).
    let mut page_notes: Vec<Vec<u32>> = vec![Vec::new()];
    for block in blocks {
        match block {
            Block::Para(para_ix) => {
                let para = &paragraphs[*para_ix];
                if para.page_break && !pages.last().is_some_and(Page::is_empty) {
                    let next = (pages.len() - 1) / columns * columns + columns;
                    while pages.len() <= next {
                        pages.push(Page::default());
                    }
                    y = 0.0;
                }
                for (line_ix, line) in para.lines.iter().enumerate() {
                    // `gap`: the line did not fit beside a picture and starts below it.
                    let mut lead = line.gap
                        + if line_ix == 0 && y > 0.0 {
                            para.space_before
                        } else {
                            0.0
                        };
                    let marks = notes.refs_in(line);
                    loop {
                        page_notes.resize_with(pages.len(), Vec::new);
                        let reserve = notes.reserved(&page_notes[pages.len() - 1], &marks);
                        let real = (pages.len() - 1) / columns;
                        // Text does not run into a picture that has the page to itself in that band.
                        if line.height > 0.0 {
                            while let Some(band) =
                                band_at(bands, real, y + lead, y + lead + line.height)
                            {
                                y = band.bottom + FLOAT_GAP;
                                lead = 0.0;
                            }
                        }
                        if y + lead + line.height + reserve > limit + EPS
                            && !pages.last().is_some_and(Page::is_empty)
                        {
                            // The next page may have a picture in the way too: look again there.
                            pages.push(Page::default());
                            y = 0.0;
                            lead = 0.0;
                            continue;
                        }
                        y += lead;
                        break;
                    }
                    for id in marks {
                        let on_page = &mut page_notes[pages.len() - 1];
                        if !on_page.contains(&id) {
                            on_page.push(id);
                        }
                    }
                    let page = pages.last_mut().expect("at least one page");
                    match page.slices.last_mut() {
                        Some(s)
                            if s.para == *para_ix
                                && s.lines.end == line_ix
                                && s.x == 0.0
                                && (s.y + s.height - y).abs() < EPS =>
                        {
                            s.lines.end += 1;
                            s.height += line.height;
                        }
                        _ => page.slices.push(Slice {
                            para: *para_ix,
                            lines: line_ix..line_ix + 1,
                            y,
                            height: line.height,
                            x: 0.0,
                            width: content_width,
                        }),
                    }
                    y += line.height;
                }
                if let Some(image) = para.image
                    && let Some(float) = image.float
                {
                    floats.push((
                        (pages.len() - 1) / columns,
                        PlacedFloat {
                            para: *para_ix,
                            id: image.id,
                            x: float.x,
                            y: float.y,
                            width: image.bounds().0,
                            height: image.bounds().1,
                            wrap: float.wrap,
                            frame: (image.width, image.height),
                            angle: image.angle,
                            crop: image.crop,
                        },
                    ));
                }
                y += para.space_after.unwrap_or(setup.paragraph_spacing);
            }
            Block::Table(table) => {
                crate::table_page::paginate_table(paragraphs, table, setup, &mut pages, &mut y);
                y += TABLE_GAP;
            }
        }
    }
    page_notes.resize_with(pages.len(), Vec::new);
    notes.place(&mut pages, &page_notes, paragraphs, limit, content_width);
    pages
}

/// The band of `page` that a line from `top` to `bottom` runs into.
fn band_at(bands: &[Band], page: usize, top: f32, bottom: f32) -> Option<&Band> {
    bands.iter().find(|b| {
        b.page == page && top < b.bottom + FLOAT_GAP - EPS && bottom > b.top - FLOAT_GAP + EPS
    })
}
