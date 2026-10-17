use gpui_kit::*;
use wv_core::CharStyle;
use wv_core::{Document, Editor, TocEntry};
use wv_layout::{DocLayout, INDENT_STEP, Measurer, PageSetup};

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
            dots: 4,
        })
        .collect()
}

/// Sizes the leader dots so the page numbers end at the right edge of the text.
fn fit_dots(entries: &mut [TocEntry], doc: &Document, m: &dyn Measurer) {
    let style = CharStyle::default();
    let (dot, space) = (m.width("·", &style), m.width(" ", &style));
    let total = PageSetup::from_spec(&doc.page).content_width();
    for e in entries {
        let indent = f32::from(e.level.saturating_sub(1)) * INDENT_STEP;
        let used = m.width(&e.text, &style) + m.width(&e.page.to_string(), &style) + 2.0 * space;
        e.dots = if dot > 0.0 {
            ((total - indent - used) / dot).floor().max(2.0) as usize
        } else {
            4
        };
    }
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
        let mut final_entries = entries(&trial.doc, &measurer);
        fit_dots(&mut final_entries, &trial.doc, &measurer);
        let title = title.to_string();
        self.apply(cx, move |c| c.set_toc(&title, &final_entries));
    }
}
