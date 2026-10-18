use std::collections::HashMap;
use std::rc::Rc;

use wv_core::{Document, Wrap};

use crate::Measurer;
use crate::page::{Page, PageSetup};
use crate::paginate::paginate;
use crate::paragraph::{Obstacle, Options, ParagraphLayout, layout_paragraph_around};
use crate::table::Block;

/// Space kept between a picture and the text beside it.
const GAP: f32 = 8.0;

/// How often the text is laid out again around pictures: moving a line changes what is beside a
/// picture, and which page a picture is on.
const PASSES: usize = 3;

/// What `layout` needs of a paragraph to lay it out again: its width and list marker.
pub(crate) struct Source {
    pub options: Options,
    pub marker: Option<String>,
}

/// Lays the paragraphs beside "square" pictures out again so that the text goes around the pictures,
/// and flows the pages again. Only paragraphs that touch such a picture are laid out again.
pub(crate) fn wrap_around(
    doc: &Document,
    setup: &PageSetup,
    blocks: &[Block],
    sources: &[Source],
    paragraphs: &mut [Rc<ParagraphLayout>],
    pages: &mut Vec<Page>,
    m: &dyn Measurer,
) {
    let base: Vec<Rc<ParagraphLayout>> = paragraphs.to_vec();
    let mut current: HashMap<usize, Vec<Obstacle>> = HashMap::new();
    for _ in 0..PASSES {
        let next = obstacles(doc, pages, paragraphs);
        if next == current {
            break;
        }
        for para in current
            .keys()
            .chain(next.keys())
            .copied()
            .collect::<Vec<_>>()
        {
            paragraphs[para] = match next.get(&para) {
                Some(around) => {
                    let p = &doc.paragraphs[para];
                    let resolved = p.with_named_style();
                    let source = &sources[para];
                    let mut layout = layout_paragraph_around(
                        resolved.as_ref().unwrap_or(p),
                        &source.options,
                        m,
                        around,
                    );
                    layout.marker = source.marker.clone();
                    Rc::new(layout)
                }
                None => base[para].clone(),
            };
        }
        current = next;
        *pages = paginate(paragraphs, blocks, setup);
    }
}

/// The pictures with square wrapping as obstacles of the body paragraphs that start on their page.
fn obstacles(
    doc: &Document,
    pages: &[Page],
    paragraphs: &[Rc<ParagraphLayout>],
) -> HashMap<usize, Vec<Obstacle>> {
    // Round to half a pixel so that tiny shifts do not look like changes.
    let round = |v: f32| (v * 2.0).round() / 2.0;
    let mut out: HashMap<usize, Vec<Obstacle>> = HashMap::new();
    for page in pages {
        for float in page.floats.iter().filter(|f| f.wrap == Wrap::Square) {
            for slice in page.slices.iter().filter(|s| s.lines.start == 0) {
                let para = &paragraphs[slice.para];
                if slice.para == float.para || doc.paragraphs[slice.para].style.cell.is_some() {
                    continue;
                }
                let top = float.y - GAP - slice.y;
                if top >= para.height || float.y + float.height + GAP - slice.y <= 0.0 {
                    continue;
                }
                out.entry(slice.para).or_default().push(Obstacle {
                    top: round(top),
                    bottom: round(float.y + float.height + GAP - slice.y),
                    x0: round(float.x - GAP - slice.x - para.indent),
                    x1: round(float.x + float.width + GAP - slice.x - para.indent),
                });
            }
        }
    }
    out
}
