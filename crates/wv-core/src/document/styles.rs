use super::{Document, Paragraph};

impl Document {
    /// An id no user style has.
    pub fn next_style_id(&self) -> u32 {
        let used = self.paragraphs.iter().filter_map(|p| p.style.custom);
        let known = self.styles.keys().copied();
        used.chain(known).max().map_or(1, |m| m + 1)
    }

    /// The paragraph as it is drawn: the formatting of its user style and of its built-in style
    /// written in (what the text sets itself wins). `None` when nothing needs to change.
    pub fn resolve(&self, p: &Paragraph) -> Option<Paragraph> {
        let Some(user) = p.style.custom.and_then(|id| self.styles.get(&id)) else {
            return p.with_named_style();
        };
        let mut q = p.clone();
        user.apply_to_paragraph(&mut q.style);
        for run in &mut q.runs {
            user.apply_to_text(&mut run.style);
        }
        Some(q.with_named_style().unwrap_or(q))
    }
}
