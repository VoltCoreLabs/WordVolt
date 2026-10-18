use gpui_kit::*;
use wv_core::Position;

use super::view::EditorView;

impl EditorView {
    /// The page number (as written) of the page paragraph `para` starts on, from the layout.
    pub fn page_text_of(&self, para: usize) -> Option<String> {
        let layout = self.layout.as_ref()?;
        let page = layout.page_of_paragraph(para)?;
        Some(layout.pages[page].number.clone())
    }

    /// Types the page number of bookmark `name` as a field.
    pub fn insert_page_reference(&mut self, name: &str, cx: &mut Context<Self>) {
        let text = self
            .core
            .bookmark_position(name)
            .and_then(|p| self.page_text_of(p.para))
            .unwrap_or_else(|| "?".to_string());
        self.apply(cx, |c| c.insert_page_reference(name, &text));
    }

    /// Types the page number of heading paragraph `para` as a field.
    pub fn insert_heading_page_reference(&mut self, para: usize, cx: &mut Context<Self>) {
        let text = self.page_text_of(para).unwrap_or_else(|| "?".to_string());
        self.apply(cx, |c| {
            if let Some(name) = c.heading_bookmark(para) {
                c.insert_page_reference(&name, &text);
            }
        });
    }

    /// Brings the page-number fields up to date with the layout.
    pub fn update_fields(&mut self, cx: &mut Context<Self>) {
        let layout = self.layout.clone();
        self.apply(cx, |c| {
            let page = |p: Position| {
                let layout = layout.as_ref()?;
                let page = layout.page_of_paragraph(p.para)?;
                Some(layout.pages[page].number.clone())
            };
            c.update_fields(&page);
        });
    }

    /// The 1-based page each position is on (for the navigation pane).
    pub fn pages_of(&mut self, positions: &[Position], window: &Window) -> Vec<usize> {
        let layout = self.ensure_layout(window);
        let m = self.measurer(window);
        positions
            .iter()
            .map(|p| layout.caret(*p, &m).map_or(1, |c| c.page + 1))
            .collect()
    }
}
