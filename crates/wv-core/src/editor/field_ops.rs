use super::Editor;
use crate::position::Position;
use crate::style::CharStyle;

impl Editor {
    /// Types the page number `page_text` of bookmark `name` as a field: a link to the bookmark
    /// whose text is the number (kept up to date by `update_fields`). One undo step.
    pub fn insert_page_reference(&mut self, name: &str, page_text: &str) {
        let Some(id) = self.anchored_id(name) else {
            return;
        };
        self.insert_reference(name, page_text);
        // The inserted text is the one run (just after the start of the selection) that is a link
        // to the bookmark; mark it as a page-number field.
        let at = self.sel.head;
        let mut para = self.doc.paragraphs[at.para].clone();
        let end = at.offset;
        let start = end.saturating_sub(page_text.len());
        para.map_style(start, end, |s: &mut CharStyle| s.page_ref = Some(id));
        self.amend_last_edit(at.para, para);
    }

    /// Brings the page-number fields up to date: `page_text` gives the page number (as written)
    /// of a position. Returns how many fields changed; one undo step.
    pub fn update_fields(&mut self, page_text: &dyn Fn(Position) -> Option<String>) -> usize {
        let changed = |this: &Editor| -> Vec<usize> {
            this.doc
                .paragraphs
                .iter()
                .enumerate()
                .filter(|(_, p)| p.runs.iter().any(|r| r.style.page_ref.is_some()))
                .map(|(i, _)| i)
                .collect()
        };
        let hit = changed(self);
        let (Some(&first), Some(&last)) = (hit.first(), hit.last()) else {
            return 0;
        };
        let mut paras = self.doc.paragraphs[first..=last].to_vec();
        let mut count = 0;
        for p in &mut paras {
            for run in &mut p.runs {
                let Some(id) = run.style.page_ref else {
                    continue;
                };
                let Some(text) = self.bookmark_start(id).and_then(page_text) else {
                    continue;
                };
                if run.text != text {
                    run.text = text;
                    count += 1;
                }
            }
            p.normalize();
        }
        if count > 0 {
            let sel = self.sel;
            self.edit_paragraphs(first, paras.len(), paras, sel);
        }
        count
    }
}
