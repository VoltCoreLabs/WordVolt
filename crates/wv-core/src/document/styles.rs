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
        let user = p.style.custom.and_then(|id| self.styles.get(&id));
        let defaults = self.default_font.is_some() || self.default_size_pt.is_some();
        if user.is_none() && !defaults {
            return p.with_named_style();
        }
        let mut q = p.clone();
        if let Some(user) = user {
            user.apply_to_paragraph(&mut q.style);
            for run in &mut q.runs {
                user.apply_to_text(&mut run.style);
            }
        }
        let mut q = q.with_named_style().unwrap_or(q);
        // What neither the text, its style nor the built-in style sets comes from the file.
        for run in &mut q.runs {
            if run.style.font.is_none() {
                run.style.font = self.default_font.clone();
            }
            if run.style.size_pt.is_none() {
                run.style.size_pt = self.default_size_pt;
            }
        }
        Some(q)
    }
}
