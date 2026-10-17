use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use wv_core::Paragraph;

use crate::Measurer;
use crate::paragraph::{Options, ParagraphLayout, layout_paragraph};

/// Laid-out paragraphs kept between layouts: a paragraph that did not change (same text, formatting
/// and width) is not laid out again, so an edit costs one paragraph, not the whole document. The
/// layouts are shared, not copied.
#[derive(Default)]
pub struct LayoutCache {
    /// key -> (generation of the last use, layout)
    map: HashMap<u64, (u64, Rc<ParagraphLayout>)>,
    generation: u64,
}

/// A fast hash (FxHash): the key of every paragraph is computed on every layout.
#[derive(Default)]
struct Fx(u64);

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl Fx {
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for Fx {
    fn write(&mut self, bytes: &[u8]) {
        let (words, rest) = bytes.as_chunks::<8>();
        for word in words {
            self.add(u64::from_le_bytes(*word));
        }
        for byte in rest {
            self.add(u64::from(*byte));
        }
    }

    fn write_u8(&mut self, i: u8) {
        self.add(u64::from(i));
    }

    fn write_u16(&mut self, i: u16) {
        self.add(u64::from(i));
    }

    fn write_u32(&mut self, i: u32) {
        self.add(u64::from(i));
    }

    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }

    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

fn key(p: &Paragraph, opts: &Options) -> u64 {
    let mut h = Fx::default();
    opts.width.to_bits().hash(&mut h);
    opts.line_spacing.to_bits().hash(&mut h);
    p.style.hash(&mut h);
    p.runs.hash(&mut h);
    h.finish()
}

impl LayoutCache {
    /// Starts a layout pass; entries not used by the pass are dropped when it ends.
    pub(crate) fn begin(&mut self) {
        self.generation += 1;
    }

    pub(crate) fn end(&mut self) {
        let generation = self.generation;
        self.map.retain(|_, (g, _)| *g == generation);
    }

    /// The layout of `p` with list marker `marker` (a list item's number depends on the items
    /// before it, so it is not part of the key).
    pub(crate) fn layout(
        &mut self,
        p: &Paragraph,
        opts: &Options,
        marker: Option<String>,
        m: &dyn Measurer,
    ) -> Rc<ParagraphLayout> {
        let key = key(p, opts);
        let generation = self.generation;
        if let Some((g, layout)) = self.map.get_mut(&key) {
            *g = generation;
            if layout.marker != marker {
                let mut changed = (**layout).clone();
                changed.marker = marker;
                *layout = Rc::new(changed);
            }
            return layout.clone();
        }
        let mut layout = layout_paragraph(p, opts, m);
        layout.marker = marker;
        let layout = Rc::new(layout);
        self.map.insert(key, (generation, layout.clone()));
        layout
    }
}
