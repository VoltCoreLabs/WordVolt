use super::Run;
use crate::style::{CharStyle, ParaStyle};

/// A paragraph: runs of text plus paragraph style. Offsets are UTF-8 bytes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Paragraph {
    pub runs: Vec<Run>,
    pub style: ParaStyle,
}

impl Paragraph {
    pub fn plain(text: &str) -> Self {
        let runs = if text.is_empty() {
            vec![]
        } else {
            vec![Run::new(text, CharStyle::default())]
        };
        Self {
            runs,
            style: ParaStyle::default(),
        }
    }

    pub fn text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }

    /// Length in bytes.
    pub fn len(&self) -> usize {
        self.runs.iter().map(|r| r.text.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Style a character typed at `offset` would get: that of the char before it
    /// (or of the first run at offset 0).
    pub fn style_at(&self, offset: usize) -> CharStyle {
        let mut pos = 0;
        for run in &self.runs {
            pos += run.text.len();
            if pos >= offset && !run.text.is_empty() {
                return run.style.clone();
            }
        }
        self.runs
            .last()
            .map(|r| r.style.clone())
            .unwrap_or_default()
    }

    /// The paragraph with its named style's defaults written into the runs (what is drawn);
    /// `None` for Normal paragraphs, which need no change. Text and offsets stay the same.
    pub fn with_named_style(&self) -> Option<Paragraph> {
        if self.style.named == crate::style::NamedStyle::Normal {
            return None;
        }
        let mut resolved = self.clone();
        for run in &mut resolved.runs {
            self.style.named.apply(&mut run.style);
        }
        Some(resolved)
    }

    /// Style of the character right after `offset` (`None` at the end of the paragraph).
    pub fn style_after(&self, offset: usize) -> Option<CharStyle> {
        let mut pos = 0;
        for run in &self.runs {
            pos += run.text.len();
            if pos > offset {
                return Some(run.style.clone());
            }
        }
        None
    }

    /// Makes sure a run boundary exists at `offset`; returns the index of the run starting there.
    fn split_at(&mut self, offset: usize) -> usize {
        let mut pos = 0;
        for i in 0..self.runs.len() {
            let len = self.runs[i].text.len();
            if offset == pos {
                return i;
            }
            if offset < pos + len {
                let tail = self.runs[i].text.split_off(offset - pos);
                let style = self.runs[i].style.clone();
                self.runs.insert(i + 1, Run::new(tail, style));
                return i + 1;
            }
            pos += len;
        }
        self.runs.len()
    }

    /// Merges neighbouring runs with equal style and drops empty ones.
    fn normalize(&mut self) {
        self.runs.retain(|r| !r.text.is_empty());
        let mut out: Vec<Run> = Vec::with_capacity(self.runs.len());
        for run in self.runs.drain(..) {
            match out.last_mut() {
                Some(last) if last.style == run.style => last.text.push_str(&run.text),
                _ => out.push(run),
            }
        }
        self.runs = out;
    }

    pub fn insert(&mut self, offset: usize, text: &str, style: &CharStyle) {
        if text.is_empty() {
            return;
        }
        let i = self.split_at(offset);
        self.runs.insert(i, Run::new(text, style.clone()));
        self.normalize();
    }

    pub fn delete(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }
        let a = self.split_at(start);
        let b = self.split_at(end);
        self.runs.drain(a..b);
        self.normalize();
    }

    /// Cuts everything from `offset` on and returns it as a new paragraph with the same style.
    pub fn split_off(&mut self, offset: usize) -> Paragraph {
        let i = self.split_at(offset);
        let tail = self.runs.split_off(i);
        self.normalize();
        let mut right = Paragraph {
            runs: tail,
            style: self.style.clone(),
        };
        right.normalize();
        right
    }

    pub fn append(&mut self, other: Paragraph) {
        self.runs.extend(other.runs);
        self.normalize();
    }

    /// Rewrites the text of `start..end` run by run through `f` (run styles are kept).
    pub fn map_text(&mut self, start: usize, end: usize, f: &mut dyn FnMut(&str) -> String) {
        if start >= end {
            return;
        }
        let a = self.split_at(start);
        let b = self.split_at(end);
        for run in &mut self.runs[a..b] {
            run.text = f(&run.text);
        }
        self.normalize();
    }

    pub fn map_style(&mut self, start: usize, end: usize, f: impl Fn(&mut CharStyle)) {
        if start >= end {
            return;
        }
        let a = self.split_at(start);
        let b = self.split_at(end);
        self.runs[a..b].iter_mut().for_each(|r| f(&mut r.style));
        self.normalize();
    }

    pub fn all_chars(&self, start: usize, end: usize, pred: &impl Fn(&CharStyle) -> bool) -> bool {
        let mut pos = 0;
        for run in &self.runs {
            let (rs, re) = (pos, pos + run.text.len());
            pos = re;
            if re > start && rs < end && !pred(&run.style) {
                return false;
            }
        }
        true
    }
}
