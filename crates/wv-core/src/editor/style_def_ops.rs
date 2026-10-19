use super::Editor;
use crate::style::{NamedStyle, UserStyle};

impl Editor {
    /// The user style of the paragraph with the caret.
    pub fn user_style_here(&self) -> Option<u32> {
        self.doc.paragraphs[self.sel.head.para]
            .style
            .custom
            .filter(|id| self.doc.styles.contains_key(id))
    }

    /// Defines a style named `name` from the formatting at the caret and gives it to the selected
    /// paragraphs. One undo step. Returns its id.
    pub fn define_style(&mut self, name: &str) -> u32 {
        let id = self.doc.next_style_id();
        let para = self.doc.paragraphs[self.sel.head.para].style.clone();
        let style = UserStyle::from_formatting(name, &para, &self.current_style());
        let mut settings = self.doc.settings();
        settings.styles.insert(id, style);
        self.assign_style(Some(id), Some(settings));
        id
    }

    /// Makes style `id` look like the formatting at the caret (the paragraphs that use it follow).
    pub fn update_style(&mut self, id: u32) {
        let Some(old) = self.doc.styles.get(&id) else {
            return;
        };
        let para = self.doc.paragraphs[self.sel.head.para].style.clone();
        let mut style = UserStyle::from_formatting(&old.name, &para, &self.current_style());
        // The paragraph itself is drawn with the style's base: keep the base the style has.
        style.base = para.named;
        let mut settings = self.doc.settings();
        settings.styles.insert(id, style);
        self.change_settings(settings);
    }

    /// Replaces the definition of style `id` with `style` (edited in a dialog); an empty name or a
    /// changed base keeps the old one. One undo step; the paragraphs that use it follow.
    pub fn edit_style(&mut self, id: u32, mut style: UserStyle) {
        let Some(old) = self.doc.styles.get(&id) else {
            return;
        };
        if style.name.trim().is_empty() {
            style.name = old.name.clone();
        }
        style.name = style.name.trim().to_string();
        style.base = old.base;
        let mut settings = self.doc.settings();
        settings.styles.insert(id, style);
        self.change_settings(settings);
    }

    /// Deletes style `id`; its paragraphs go back to their built-in style.
    pub fn delete_style(&mut self, id: u32) {
        let mut settings = self.doc.settings();
        settings.styles.remove(&id);
        let first = self
            .doc
            .paragraphs
            .iter()
            .position(|p| p.style.custom == Some(id));
        let last = self
            .doc
            .paragraphs
            .iter()
            .rposition(|p| p.style.custom == Some(id));
        let (Some(first), Some(last)) = (first, last) else {
            self.change_settings(settings);
            return;
        };
        let mut paras = self.doc.paragraphs[first..=last].to_vec();
        for p in &mut paras {
            if p.style.custom == Some(id) {
                p.style.custom = None;
            }
        }
        let sel = self.sel;
        self.edit_paragraphs_with(first, paras.len(), paras, sel, Some(settings));
    }

    /// Gives user style `id` to the selected paragraphs.
    pub fn set_user_style(&mut self, id: u32) {
        if self.doc.styles.contains_key(&id) {
            self.assign_style(Some(id), None);
        }
    }

    /// Gives (or takes away) a user style; the paragraph's built-in style becomes the style's base.
    fn assign_style(&mut self, id: Option<u32>, settings: Option<crate::document::DocSettings>) {
        let (s, e) = self.sel.range();
        let base = match id {
            Some(id) => settings
                .as_ref()
                .map_or(&self.doc.styles, |s| &s.styles)
                .get(&id)
                .map_or(NamedStyle::Normal, |u| u.base),
            None => NamedStyle::Normal,
        };
        let mut paras = self.doc.paragraphs[s.para..=e.para].to_vec();
        for p in &mut paras {
            p.style.custom = id;
            p.style.named = base;
        }
        let sel = self.sel;
        self.edit_paragraphs_with(s.para, paras.len(), paras, sel, settings);
    }
}
