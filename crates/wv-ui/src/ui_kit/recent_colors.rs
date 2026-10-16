use std::cell::RefCell;

use gpui_kit::Global;

const MAX: usize = 10;

/// Most recently used colours (shared by the text-colour and highlight pickers), newest first.
#[derive(Default)]
pub struct RecentColors(RefCell<Vec<u32>>);

impl Global for RecentColors {}

impl RecentColors {
    /// Starts with the colours saved by the previous session.
    pub fn load() -> Self {
        let mut saved = crate::settings::load_recent_colors();
        saved.truncate(MAX);
        Self(RefCell::new(saved))
    }

    /// Remembers `color` (newest first) and saves the list.
    pub fn push(&self, color: u32) {
        crate::settings::save_recent_colors(&self.record(color));
    }

    /// In-memory part of [`push`](Self::push); returns the new list.
    fn record(&self, color: u32) -> Vec<u32> {
        let mut list = self.0.borrow_mut();
        list.retain(|c| *c != color);
        list.insert(0, color);
        list.truncate(MAX);
        list.clone()
    }

    pub fn list(&self) -> Vec<u32> {
        self.0.borrow().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::RecentColors;

    #[test]
    fn newest_first_without_duplicates_and_capped() {
        let r = RecentColors::default();
        for c in 0..15 {
            r.record(c);
        }
        r.record(10);
        let list = r.list();
        assert_eq!(list.len(), 10);
        assert_eq!(list[0], 10);
        assert_eq!(list.iter().filter(|c| **c == 10).count(), 1);
    }
}
