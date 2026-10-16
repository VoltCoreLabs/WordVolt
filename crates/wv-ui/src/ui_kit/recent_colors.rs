use std::cell::RefCell;

use gpui_kit::Global;

const MAX: usize = 10;

/// Most recently used colours (shared by the text-colour and highlight pickers), newest first.
#[derive(Default)]
pub struct RecentColors(RefCell<Vec<u32>>);

impl Global for RecentColors {}

impl RecentColors {
    pub fn push(&self, color: u32) {
        let mut list = self.0.borrow_mut();
        list.retain(|c| *c != color);
        list.insert(0, color);
        list.truncate(MAX);
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
            r.push(c);
        }
        r.push(10);
        let list = r.list();
        assert_eq!(list.len(), 10);
        assert_eq!(list[0], 10);
        assert_eq!(list.iter().filter(|c| **c == 10).count(), 1);
    }
}
