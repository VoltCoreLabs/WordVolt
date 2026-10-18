use super::{Paragraph, Run};
use crate::style::{Align, CharStyle};

/// Default size of header and footer text (pt).
pub const BAND_SIZE_PT: u16 = 10;

/// Header or footer: formatted text repeated on every page. `{page}` and `{pages}` stand for the
/// page number and the page count.
#[derive(Clone, Debug, PartialEq)]
pub struct Band {
    /// The text with its formatting (only the runs matter; offsets are UTF-8 bytes).
    pub para: Paragraph,
    /// Left, Center (default) or Right; Justify counts as Left.
    pub align: Align,
}

impl Default for Band {
    fn default() -> Self {
        Self::new("")
    }
}

impl Band {
    pub fn new(text: impl AsRef<str>) -> Self {
        Self {
            para: Paragraph::plain(text.as_ref()),
            align: Align::Center,
        }
    }

    pub fn from_runs(runs: Vec<Run>) -> Self {
        let mut para = Paragraph {
            runs,
            ..Paragraph::default()
        };
        para.append(Paragraph::default());
        Self {
            para,
            align: Align::Center,
        }
    }

    pub fn aligned(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn runs(&self) -> &[Run] {
        &self.para.runs
    }

    pub fn text(&self) -> String {
        self.para.text()
    }

    pub fn is_empty(&self) -> bool {
        self.para.is_empty()
    }

    pub fn has_page_number(&self) -> bool {
        self.text().contains("{page}")
    }

    /// Replaces the text keeping the formatting of what stays: only the changed middle is
    /// rewritten, and typed text takes the style of the character before it.
    pub fn set_text(&mut self, new: &str) {
        let old = self.text();
        if old == new {
            return;
        }
        let prefix = old
            .char_indices()
            .zip(new.chars())
            .take_while(|((_, a), b)| a == b)
            .last()
            .map_or(0, |((i, c), _)| i + c.len_utf8());
        let max_suffix = old.len().min(new.len()) - prefix;
        let suffix = old[prefix..]
            .chars()
            .rev()
            .zip(new[prefix..].chars().rev())
            .take_while(|(a, b)| a == b)
            .map(|(c, _)| c.len_utf8())
            .sum::<usize>()
            .min(max_suffix);
        let style = self.para.style_at(prefix);
        self.para.delete(prefix, old.len() - suffix);
        self.para
            .insert(prefix, &new[prefix..new.len() - suffix], &style);
    }

    /// Changes the formatting of the text in `start..end`.
    pub fn restyle(&mut self, start: usize, end: usize, f: impl Fn(&mut CharStyle)) {
        self.para.map_style(start, end, f);
    }

    /// Changes the formatting of the whole text.
    pub fn restyle_all(&mut self, f: impl Fn(&mut CharStyle)) {
        let len = self.para.len();
        self.restyle(0, len, f);
    }

    /// True if every character of `start..end` satisfies `pred` (an empty range: false).
    pub fn all_chars(&self, start: usize, end: usize, pred: impl Fn(&CharStyle) -> bool) -> bool {
        start < end && self.para.all_chars(start, end, &pred)
    }

    /// Formatting a character typed at `offset` would get.
    pub fn style_at(&self, offset: usize) -> CharStyle {
        self.para.style_at(offset)
    }

    /// The text for page `page` (1-based) of `pages`.
    pub fn render(&self, page: usize, pages: usize) -> String {
        replace_tokens(&self.text(), &page.to_string(), pages)
    }

    /// The runs for the page numbered `page` of `pages`, tokens replaced (a token split over two
    /// runs stays).
    pub fn render_runs(&self, page: &str, pages: usize) -> Vec<Run> {
        self.runs()
            .iter()
            .map(|r| Run::new(replace_tokens(&r.text, page, pages), r.style.clone()))
            .collect()
    }
}

fn replace_tokens(text: &str, page: &str, pages: usize) -> String {
    text.replace("{page}", page)
        .replace("{pages}", &pages.to_string())
}

/// Headers and footers of the first page and of even pages, used when the matching switch is on.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BandVariants {
    /// Page 1 has its own header and footer.
    pub different_first: bool,
    /// Even pages have their own header and footer.
    pub odd_even: bool,
    /// (header, footer) of the first page.
    pub first: (Band, Band),
    /// (header, footer) of even pages.
    pub even: (Band, Band),
}

impl BandVariants {
    /// The (header, footer) shown on page `page` (0-based); `header`/`footer` are the main ones.
    pub fn pick<'a>(
        &'a self,
        page: usize,
        header: &'a Band,
        footer: &'a Band,
    ) -> (&'a Band, &'a Band) {
        if self.different_first && page == 0 {
            (&self.first.0, &self.first.1)
        } else if self.odd_even && page % 2 == 1 {
            (&self.even.0, &self.even.1)
        } else {
            (header, footer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_page_tokens() {
        let band = Band::new("Стр. {page} из {pages}");
        assert!(band.has_page_number());
        assert_eq!(band.render(2, 5), "Стр. 2 из 5");
    }

    #[test]
    fn typing_keeps_the_formatting_around() {
        let mut band = Band::new("Привет мир");
        band.restyle(0, "Привет".len(), |s| s.bold = true);
        band.set_text("Привет, мир");
        assert_eq!(band.text(), "Привет, мир");
        assert!(
            band.all_chars(0, "Привет,".len(), |s| s.bold),
            "typed after bold text"
        );
        assert!(!band.all_chars(0, band.para.len(), |s| s.bold));
        band.set_text("мир");
        assert_eq!(band.text(), "мир");
        assert!(!band.all_chars(0, band.para.len(), |s| s.bold));
        band.set_text("");
        assert!(band.is_empty());
    }

    #[test]
    fn restyle_splits_runs_and_tokens_render_per_run() {
        let mut band = Band::new("a {page} b");
        band.restyle(2, 8, |s| s.italic = true);
        assert_eq!(band.runs().len(), 3);
        let shown: String = band
            .render_runs("3", 9)
            .iter()
            .map(|r| r.text.clone())
            .collect();
        assert_eq!(shown, "a 3 b");
    }

    #[test]
    fn picks_first_even_or_main_band() {
        let (h, f) = (Band::new("h"), Band::new("f"));
        let mut v = BandVariants {
            first: (Band::new("h1"), Band::new("f1")),
            even: (Band::new("h2"), Band::new("f2")),
            ..Default::default()
        };
        assert_eq!(v.pick(0, &h, &f).0.text(), "h");
        v.different_first = true;
        v.odd_even = true;
        assert_eq!(v.pick(0, &h, &f).0.text(), "h1");
        assert_eq!(v.pick(1, &h, &f).1.text(), "f2");
        assert_eq!(v.pick(2, &h, &f).0.text(), "h");
    }
}
