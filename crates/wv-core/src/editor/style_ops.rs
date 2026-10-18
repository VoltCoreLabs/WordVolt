use super::Editor;
use crate::position::Selection;
use crate::style::{Align, CharStyle, Script};

impl Editor {
    /// Applies `f` to every selected character; with a caret it changes the style of the next typed text.
    fn apply_char_style(&mut self, f: impl Fn(&mut CharStyle)) {
        let (start, end) = self.sel.range();
        if start == end {
            let mut style = self
                .pending_style
                .clone()
                .unwrap_or_else(|| self.doc.paragraphs[start.para].style_at(start.offset));
            f(&mut style);
            self.pending_style = Some(style);
            return;
        }
        let mut paras = self.doc.paragraphs[start.para..=end.para].to_vec();
        let last = paras.len() - 1;
        for (i, p) in paras.iter_mut().enumerate() {
            let from = if i == 0 { start.offset } else { 0 };
            let to = if i == last { end.offset } else { p.len() };
            p.map_style(from, to, &f);
        }
        let sel = self.sel;
        self.edit_paragraphs(start.para, last + 1, paras, sel);
    }

    /// Gives the selected text the character formatting `style` (a link stays as it is):
    /// the format painter.
    pub fn paint_char_style(&mut self, style: &CharStyle) {
        self.apply_char_style(|s| {
            let link = s.link.take();
            *s = style.clone();
            s.link = link;
        });
    }

    /// Switches a property on unless the whole selection (or the caret) already has it, then off.
    fn toggle_char_style(
        &mut self,
        has: impl Fn(&CharStyle) -> bool,
        set: impl Fn(&mut CharStyle, bool),
    ) {
        let (start, end) = self.sel.range();
        let on = if start == end {
            let style = self
                .pending_style
                .clone()
                .unwrap_or_else(|| self.doc.paragraphs[start.para].style_at(start.offset));
            !has(&style)
        } else {
            !self.doc.all_chars(start, end, &has)
        };
        self.apply_char_style(|s| set(s, on));
    }

    /// The value of `get` if every selected character agrees on it.
    fn uniform<T: PartialEq>(&self, get: impl Fn(&CharStyle) -> T) -> Option<T> {
        let (s, e) = self.sel.range();
        let first = get(&self.doc.paragraphs[s.para].style_at(s.offset + 1));
        self.doc
            .all_chars(s, e, |c| get(c) == first)
            .then_some(first)
    }

    /// Formatting at the caret (or the common formatting of the selection) for toolbar state.
    pub fn current_style(&self) -> CharStyle {
        let (s, e) = self.sel.range();
        if s == e {
            return self
                .pending_style
                .clone()
                .unwrap_or_else(|| self.doc.paragraphs[s.para].style_at(s.offset));
        }
        CharStyle {
            bold: self.doc.all_chars(s, e, |c| c.bold),
            italic: self.doc.all_chars(s, e, |c| c.italic),
            underline: self.doc.all_chars(s, e, |c| c.underline),
            strike: self.doc.all_chars(s, e, |c| c.strike),
            size_pt: self.uniform(|c| c.size_pt).flatten(),
            color: self.uniform(|c| c.color).flatten(),
            highlight: self.uniform(|c| c.highlight).flatten(),
            font: self.uniform(|c| c.font.clone()).flatten(),
            script: self.uniform(|c| c.script).unwrap_or_default(),
            caps: self.doc.all_chars(s, e, |c| c.caps),
            mono: self.doc.all_chars(s, e, |c| c.mono),
            spoiler: self.doc.all_chars(s, e, |c| c.spoiler),
            link: self.uniform(|c| c.link.clone()).flatten(),
        }
    }

    pub fn toggle_bold(&mut self) {
        self.toggle_char_style(|s| s.bold, |s, on| s.bold = on);
    }

    pub fn toggle_italic(&mut self) {
        self.toggle_char_style(|s| s.italic, |s, on| s.italic = on);
    }

    pub fn toggle_underline(&mut self) {
        self.toggle_char_style(|s| s.underline, |s, on| s.underline = on);
    }

    pub fn toggle_strike(&mut self) {
        self.toggle_char_style(|s| s.strike, |s, on| s.strike = on);
    }

    pub fn toggle_caps(&mut self) {
        self.toggle_char_style(|s| s.caps, |s, on| s.caps = on);
    }

    /// Inline code (monospace).
    pub fn toggle_mono(&mut self) {
        self.toggle_char_style(|s| s.mono, |s, on| s.mono = on);
    }

    /// Spoiler: hidden until the caret enters it.
    pub fn toggle_spoiler(&mut self) {
        self.toggle_char_style(|s| s.spoiler, |s, on| s.spoiler = on);
    }

    /// Makes the selection a hyperlink; `None` removes the link.
    pub fn set_link(&mut self, url: Option<String>) {
        self.apply_char_style(|s| s.link = url.clone());
    }

    /// Turns the block-quote style on for the selected paragraphs, or off if they all have it.
    pub fn toggle_quote(&mut self) {
        let (start, end) = self.sel.range();
        let mut paras = self.doc.paragraphs[start.para..=end.para].to_vec();
        let on = !paras.iter().all(|p| p.style.quote);
        paras.iter_mut().for_each(|p| p.style.quote = on);
        let sel: Selection = self.sel;
        self.edit_paragraphs(start.para, paras.len(), paras, sel);
    }

    pub fn quote_active(&self) -> bool {
        self.doc.paragraphs[self.sel.head.para].style.quote
    }

    /// Superscript / subscript; pressing it again returns to normal.
    pub fn toggle_script(&mut self, script: Script) {
        self.toggle_char_style(
            |s| s.script == script,
            |s, on| s.script = if on { script } else { Script::Normal },
        );
    }

    /// Sets the font size (pt) of the selection; with a caret it applies to the next typed text.
    pub fn set_size_pt(&mut self, size: u16) {
        self.apply_char_style(|s| s.size_pt = Some(size));
    }

    /// Next bigger (`step > 0`) or smaller (`step < 0`) size from the usual Word size list.
    pub fn bump_size(&mut self, step: i32) {
        const SIZES: [u16; 21] = [
            8, 9, 10, 11, 12, 14, 16, 18, 20, 22, 24, 26, 28, 36, 48, 72, 96, 120, 144, 192, 288,
        ];
        let mut shown = self.current_style();
        self.named_style().apply(&mut shown);
        let current = shown.size_pt.unwrap_or(12);
        let next = if step > 0 {
            SIZES
                .iter()
                .copied()
                .find(|s| *s > current)
                .unwrap_or(current)
        } else {
            SIZES
                .iter()
                .rev()
                .copied()
                .find(|s| *s < current)
                .unwrap_or(current)
        };
        self.set_size_pt(next);
    }

    /// Removes all character formatting from the selection (or the next typed text).
    pub fn clear_formatting(&mut self) {
        self.apply_char_style(|s| *s = CharStyle::default());
    }

    /// Sets the font family; `None` = back to the default font.
    pub fn set_font_family(&mut self, font: Option<String>) {
        self.apply_char_style(|s| s.font = font.clone());
    }

    /// Text colour (0xRRGGBB); `None` = automatic.
    pub fn set_color(&mut self, color: Option<u32>) {
        self.apply_char_style(|s| s.color = color);
    }

    /// Highlight (marker) colour; `None` = no highlight.
    pub fn set_highlight(&mut self, color: Option<u32>) {
        self.apply_char_style(|s| s.highlight = color);
    }

    /// Sets alignment of every paragraph touched by the selection.
    pub fn set_align(&mut self, align: Align) {
        let (start, end) = self.sel.range();
        let mut paras = self.doc.paragraphs[start.para..=end.para].to_vec();
        paras.iter_mut().for_each(|p| p.style.align = align);
        let sel: Selection = self.sel;
        self.edit_paragraphs(start.para, paras.len(), paras, sel);
    }

    /// Alignment of the paragraph with the caret.
    pub fn align(&self) -> Align {
        self.doc.paragraphs[self.sel.head.para].style.align
    }
}
