use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use wv_core::{Document, Wrap};

use crate::Measurer;
use crate::page::{Page, PageSetup, PlacedFloat};
use crate::paginate::paginate;
use crate::paragraph::{Obstacle, Options, ParagraphLayout, layout_paragraph_around};
use crate::raster::{Strip, silhouette};
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

/// Lays the paragraphs beside pictures with a square or tight wrap out again so that the text goes around the pictures,
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

/// The pictures the text goes beside (square or following the outline) as obstacles of the body
/// paragraphs that start on their page.
fn obstacles(
    doc: &Document,
    pages: &[Page],
    paragraphs: &[Rc<ParagraphLayout>],
) -> HashMap<usize, Vec<Obstacle>> {
    // Round to half a pixel so that tiny shifts do not look like changes.
    let round = |v: f32| (v * 2.0).round() / 2.0;
    let mut out: HashMap<usize, Vec<Obstacle>> = HashMap::new();
    for page in pages {
        for float in page.floats.iter().filter(|f| beside(f.wrap)) {
            // The parts of the picture the text keeps out of: its box, or its outline in strips.
            let parts: Vec<(f32, f32, f32, f32)> = if float.wrap == Wrap::Tight {
                outline(doc, float)
            } else {
                vec![(0.0, float.height, 0.0, float.width)]
            };
            for slice in &page.slices {
                let para = &paragraphs[slice.para];
                if slice.para == float.para || doc.paragraphs[slice.para].style.cell.is_some() {
                    continue;
                }
                // The slice may be the continuation of a paragraph from the page before: its
                // first line is `base` px down the paragraph.
                let base = para.lines[slice.lines.start].y;
                for &(top, bottom, x0, x1) in &parts {
                    let (top, bottom) = (
                        float.y + top - GAP - slice.y + base,
                        float.y + bottom + GAP - slice.y + base,
                    );
                    if top >= base + slice.height.max(1.0) || bottom <= base {
                        continue;
                    }
                    out.entry(slice.para).or_default().push(Obstacle {
                        top: round(top),
                        bottom: round(bottom),
                        x0: round(float.x + x0 - GAP - slice.x - para.indent),
                        x1: round(float.x + x1 + GAP - slice.x - para.indent),
                    });
                }
            }
        }
    }
    out
}

/// Text goes beside the picture (not above/below it or under/over it).
pub(crate) fn beside(wrap: Wrap) -> bool {
    matches!(wrap, Wrap::Square | Wrap::Tight)
}

type OutlineKey = (usize, usize, u32, u32, [u32; 4], u32);

thread_local! {
    /// Outlines worked out before: reading a picture is slow, an edit must not repeat it.
    static OUTLINES: RefCell<HashMap<OutlineKey, Rc<Vec<Strip>>>> = RefCell::default();
}

/// The strips (top, bottom, x0, x1 from the box's corner) a picture with a tight wrap takes up.
fn outline(doc: &Document, float: &PlacedFloat) -> Vec<(f32, f32, f32, f32)> {
    let Some(data) = doc.images.get(float.id as usize) else {
        return vec![(0.0, float.height, 0.0, float.width)];
    };
    let c = float.crop;
    let key = (
        data.bytes.as_ptr() as usize,
        data.bytes.len(),
        float.frame.0.to_bits(),
        float.frame.1.to_bits(),
        [c.left, c.top, c.right, c.bottom].map(f32::to_bits),
        float.angle.to_bits(),
    );
    let strips = OUTLINES.with(|cache| {
        if let Some(found) = cache.borrow().get(&key) {
            return found.clone();
        }
        let strips = Rc::new(silhouette(&data.bytes, float.frame, c, float.angle));
        let mut cache = cache.borrow_mut();
        if cache.len() > 64 {
            cache.clear();
        }
        cache.insert(key, strips.clone());
        strips
    });
    if strips.is_empty() {
        return vec![(0.0, float.height, 0.0, float.width)];
    }
    strips
        .iter()
        .map(|s| (s.top, s.bottom, s.x0, s.x1))
        .collect()
}
