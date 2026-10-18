use std::ops::Range;
use std::rc::Rc;

use wv_core::Document;

use crate::Measurer;
use crate::around::{Around, Source, wrap_around};
use crate::notes::NoteSet;
use crate::page::{Page, PageSetup};
use crate::paginate::paginate;
use crate::paragraph::ParagraphLayout;
use crate::table::Block;

/// The paragraph ranges of the sections and their page setup. `base` (the document's own page
/// setup and text spacing) is the setup of the last section.
pub(crate) fn setups(doc: &Document, base: &PageSetup) -> Vec<(Range<usize>, PageSetup)> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, p) in doc.paragraphs.iter().enumerate() {
        if let Some(spec) = p.style.section.and_then(|id| doc.sections.get(&id)) {
            let setup = PageSetup {
                paragraph_spacing: base.paragraph_spacing,
                line_spacing: base.line_spacing,
                ..PageSetup::from_spec(spec)
            };
            out.push((start..i + 1, setup));
            start = i + 1;
        }
    }
    if start < doc.paragraphs.len() || out.is_empty() {
        out.push((start..doc.paragraphs.len(), base.clone()));
    }
    out
}

/// The first paragraph a block is made of.
fn first_para(doc: &Document, block: &Block) -> usize {
    match block {
        Block::Para(i) => *i,
        Block::Table(t) => doc
            .paragraphs
            .iter()
            .position(|p| p.style.cell.is_some_and(|c| c.table == t.table))
            .unwrap_or(0),
    }
}

/// Flows every section onto its own pages (a section starts on a new page, with its own page
/// size and margins) and joins the pages. Without section breaks this is one `paginate`.
pub(crate) fn paginate_sections(
    doc: &Document,
    paragraphs: &mut [Rc<ParagraphLayout>],
    blocks: &[Block],
    sections: &[(Range<usize>, PageSetup)],
    (sources, notes): (&[Source], &NoteSet),
    m: &dyn Measurer,
) -> Vec<Page> {
    let mut pages: Vec<Page> = Vec::new();
    let mut from = 0;
    for (range, setup) in sections {
        let to = from + blocks[from..].partition_point(|b| first_para(doc, b) < range.end);
        let part_blocks = &blocks[from..to];
        from = to;
        if part_blocks.is_empty() {
            continue;
        }
        let mut part = paginate(paragraphs, part_blocks, setup, notes);
        if part
            .iter()
            .any(|p| p.floats.iter().any(|f| crate::around::beside(f.wrap)))
        {
            let around = Around {
                doc,
                setup,
                blocks: part_blocks,
                sources,
                notes,
            };
            wrap_around(&around, paragraphs, &mut part, m);
        }
        let section = doc.paragraphs[range.end - 1].style.section;
        part.iter_mut().for_each(|p| p.section = section);
        pages.extend(part);
    }
    if pages.is_empty()
        && let Some((_, setup)) = sections.last()
    {
        pages = paginate(paragraphs, blocks, setup, notes);
    }
    pages
}

impl crate::DocLayout {
    /// Page size and margins of page `page` (pages of different sections may differ).
    pub fn setup_of(&self, page: usize) -> &PageSetup {
        self.pages.get(page).map_or(&self.setup, |p| &p.setup)
    }

    /// The widest and the tallest page.
    pub fn max_page_size(&self) -> (f32, f32) {
        self.pages
            .iter()
            .fold((self.setup.width, self.setup.height), |(w, h), p| {
                (w.max(p.setup.width), h.max(p.setup.height))
            })
    }
}
