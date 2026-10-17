use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Write as _;
use std::hash::Hasher;

use wv_core::Paragraph;

use crate::Measurer;
use crate::paragraph::{Options, ParagraphLayout, layout_paragraph};

/// Laid-out paragraphs kept between layouts: a paragraph that did not change (same text, formatting
/// and width) is not laid out again, so an edit costs one paragraph, not the whole document.
#[derive(Default)]
pub struct LayoutCache {
    /// key -> (generation of the last use, layout)
    map: HashMap<u64, (u64, ParagraphLayout)>,
    generation: u64,
}

/// Feeds formatted text straight into a hasher.
struct HashWriter(DefaultHasher);

impl std::fmt::Write for HashWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write(s.as_bytes());
        Ok(())
    }
}

fn key(p: &Paragraph, opts: &Options) -> u64 {
    let mut w = HashWriter(DefaultHasher::new());
    let _ = write!(
        w,
        "{}|{}|{:?}|{:?}",
        opts.width.to_bits(),
        opts.line_spacing.to_bits(),
        p.style,
        p.runs
    );
    w.0.finish()
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

    pub(crate) fn layout(
        &mut self,
        p: &Paragraph,
        opts: &Options,
        m: &dyn Measurer,
    ) -> ParagraphLayout {
        let key = key(p, opts);
        if let Some((g, layout)) = self.map.get_mut(&key) {
            *g = self.generation;
            return layout.clone();
        }
        let layout = layout_paragraph(p, opts, m);
        self.map.insert(key, (self.generation, layout.clone()));
        layout
    }
}
