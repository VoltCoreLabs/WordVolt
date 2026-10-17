use gpui_kit::*;
use wv_core::{Document, Editor, TocEntry};
use wv_layout::{DocLayout, PageSetup};

use super::view::EditorView;

/// Contents entries for the headings of `doc` with their page numbers.
fn entries(doc: &Document, measurer: &super::measure::GpuiMeasurer) -> Vec<TocEntry> {
    let editor = Editor::new(doc.clone());
    let layout = DocLayout::new(doc, PageSetup::from_spec(&doc.page), measurer);
    editor
        .headings()
        .into_iter()
        .map(|h| TocEntry {
            level: h.level,
            page: layout
                .pages
                .iter()
                .position(|p| p.slices.iter().any(|s| s.para == h.para))
                .map_or(1, |i| i + 1),
            text: h.text,
        })
        .collect()
}

impl EditorView {
    /// Inserts or updates the table of contents. The page numbers are computed for the document
    /// as it will be once the contents are in (they take space themselves).
    pub fn update_toc(&mut self, title: &str, window: &Window, cx: &mut Context<Self>) {
        let measurer = self.measurer(window);
        let first = entries(&self.core.doc, &measurer);
        let mut trial = Editor::new(self.core.doc.clone());
        trial.sel = self.core.sel;
        trial.set_toc(title, &first);
        let final_entries = entries(&trial.doc, &measurer);
        let title = title.to_string();
        self.apply(cx, move |c| c.set_toc(&title, &final_entries));
    }
}
