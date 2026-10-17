use gpui_kit::*;
use wv_core::Position;

use super::view::EditorView;

impl EditorView {
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
