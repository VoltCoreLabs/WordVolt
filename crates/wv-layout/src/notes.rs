use std::collections::HashMap;
use std::rc::Rc;

use crate::line::Line;
use crate::page::{Page, Slice};
use crate::paragraph::ParagraphLayout;

/// Space above the first footnote of a page (the separator line is drawn in it).
pub const NOTES_GAP: f32 = 12.0;

/// Space after every paragraph of a footnote.
const NOTE_PARA_GAP: f32 = 2.0;

/// The paragraphs of one footnote and the room they take.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteBox {
    pub paras: Vec<usize>,
    pub height: f32,
}

/// The footnotes that have a mark in the text, by id.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NoteSet {
    pub boxes: HashMap<u32, NoteBox>,
}

impl NoteSet {
    /// Gathers the footnote paragraphs (`note_of(i)` is the footnote paragraph `i` belongs to) of
    /// the notes that have a mark.
    pub fn new(
        paragraphs: &[Rc<ParagraphLayout>],
        note_of: impl Fn(usize) -> Option<u32>,
        marked: impl Fn(u32) -> bool,
    ) -> Self {
        let mut boxes: HashMap<u32, NoteBox> = HashMap::new();
        for (i, p) in paragraphs.iter().enumerate() {
            let Some(id) = note_of(i).filter(|id| marked(*id)) else {
                continue;
            };
            let b = boxes.entry(id).or_insert_with(|| NoteBox {
                paras: Vec::new(),
                height: 0.0,
            });
            b.paras.push(i);
            b.height += p.height + NOTE_PARA_GAP;
        }
        Self { boxes }
    }

    /// The footnotes with a mark on `line`, in order.
    pub fn refs_in(&self, line: &Line) -> Vec<u32> {
        if self.boxes.is_empty() {
            return Vec::new();
        }
        line.fragments
            .iter()
            .filter_map(|f| f.style.note)
            .filter(|id| self.boxes.contains_key(id))
            .collect()
    }

    /// Room the notes `on_page` and `extra` take together at the bottom of a page.
    pub fn reserved(&self, on_page: &[u32], extra: &[u32]) -> f32 {
        let height: f32 = on_page
            .iter()
            .chain(extra.iter().filter(|id| !on_page.contains(id)))
            .map(|id| self.boxes[id].height)
            .sum();
        if height > 0.0 {
            height + NOTES_GAP
        } else {
            0.0
        }
    }

    /// Puts the notes of every page at its bottom: a slice per note paragraph, and where the
    /// separator is (`Page::note_top`).
    pub fn place(
        &self,
        pages: &mut [Page],
        page_notes: &[Vec<u32>],
        paragraphs: &[Rc<ParagraphLayout>],
        limit: f32,
        width: f32,
    ) {
        for (page, ids) in pages.iter_mut().zip(page_notes) {
            let total = self.reserved(ids, &[]);
            if total <= 0.0 {
                continue;
            }
            let top = limit - total;
            page.note_top = Some(top);
            let mut y = top + NOTES_GAP;
            for id in ids {
                for &para in &self.boxes[id].paras {
                    let height = paragraphs[para].height;
                    page.slices.push(Slice {
                        para,
                        lines: 0..paragraphs[para].lines.len(),
                        y,
                        height,
                        x: 0.0,
                        width,
                    });
                    y += height + NOTE_PARA_GAP;
                }
            }
        }
    }
}
