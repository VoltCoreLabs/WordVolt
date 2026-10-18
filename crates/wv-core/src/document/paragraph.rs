use super::Run;
use crate::style::{Change, CharStyle, ParaStyle};

/// Size (pt) of the text of a footnote unless it says otherwise.
pub const NOTE_SIZE_PT: u16 = 10;

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
        let mut found = None;
        for run in &self.runs {
            pos += run.text.len();
            if pos >= offset && !run.text.is_empty() {
                found = Some(typed(&run.style));
                break;
            }
        }
        let mut style = found.unwrap_or_else(|| {
            self.runs
                .last()
                .map(|r| typed(&r.style))
                .unwrap_or_default()
        });
        // Text typed at the end of a commented range is not part of it; inside, it is.
        let next = self.style_at_raw(offset).and_then(|s| s.comment);
        if style.comment != next {
            style.comment = None;
        }
        style
    }

    /// The paragraph with its named style's defaults written into the runs (what is drawn);
    /// `None` for Normal paragraphs, which need no change. Text and offsets stay the same.
    pub fn with_named_style(&self) -> Option<Paragraph> {
        let note = self.style.note.is_some();
        let tracked = self.runs.iter().any(|r| r.style.change.is_some());
        if self.style.named == crate::style::NamedStyle::Normal && !note && !tracked {
            return None;
        }
        let mut resolved = self.clone();
        for run in &mut resolved.runs {
            self.style.named.apply(&mut run.style);
            // Tracked changes show as coloured: inserted text underlined, deleted text struck.
            if let Some(change) = run.style.change {
                run.style.color = Some(CHANGE_COLOR);
                if change.deleted {
                    run.style.strike = true;
                } else {
                    run.style.underline = true;
                }
            }
            // The text of a footnote is smaller than the text.
            if note && run.style.size_pt.is_none() {
                run.style.size_pt = Some(NOTE_SIZE_PT);
            }
        }
        Some(resolved)
    }

    /// Style of the run that holds the character right after `offset`, footnote mark included.
    pub fn style_at_raw(&self, offset: usize) -> Option<CharStyle> {
        let mut pos = 0;
        for run in &self.runs {
            pos += run.text.len();
            if pos > offset {
                return Some(run.style.clone());
            }
        }
        None
    }

    /// Style of the character right after `offset` (`None` at the end of the paragraph).
    pub fn style_after(&self, offset: usize) -> Option<CharStyle> {
        let mut pos = 0;
        for run in &self.runs {
            pos += run.text.len();
            if pos > offset {
                return Some(typed(&run.style));
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
    pub(crate) fn normalize(&mut self) {
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

    /// Inserts runs (with their own styles) at `offset`.
    pub fn insert_runs(&mut self, offset: usize, runs: Vec<Run>) {
        let at = self.split_at(offset);
        self.runs.splice(at..at, runs);
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

    /// Tracked deletion of `start..end`: text inserted by `own` changes is removed for good, the
    /// rest stays, marked as deleted by change `id`. Returns how many bytes were removed.
    pub fn track_delete(
        &mut self,
        (start, end): (usize, usize),
        id: u32,
        own: &dyn Fn(u32) -> bool,
    ) -> usize {
        if start >= end {
            return 0;
        }
        let a = self.split_at(start);
        let b = self.split_at(end);
        let mut removed = 0;
        let mut kept = Vec::with_capacity(b - a);
        for mut run in self.runs.drain(a..b) {
            match run.style.change {
                Some(c) if !c.deleted && own(c.id) => removed += run.text.len(),
                Some(_) => kept.push(run),
                None => {
                    run.style.change = Some(Change { id, deleted: true });
                    kept.push(run);
                }
            }
        }
        self.runs.splice(a..a, kept);
        self.normalize();
        removed
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

/// The style of text typed next to a run with `style` (it is not a footnote mark itself).
/// Colour of tracked changes (Word's default red).
const CHANGE_COLOR: u32 = 0xC0392B;

fn typed(style: &CharStyle) -> CharStyle {
    CharStyle {
        note: None,
        change: None,
        ..style.clone()
    }
}
