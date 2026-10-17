use crate::style::Align;

/// Header or footer text, repeated on every page. `{page}` and `{pages}` stand for the page number
/// and the page count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Band {
    pub text: String,
    /// Left, Center (default) or Right; Justify counts as Left.
    pub align: Align,
}

impl Default for Band {
    fn default() -> Self {
        Self::new("")
    }
}

impl Band {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            align: Align::Center,
        }
    }

    pub fn aligned(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn has_page_number(&self) -> bool {
        self.text.contains("{page}")
    }

    /// The text for page `page` (1-based) of `pages`.
    pub fn render(&self, page: usize, pages: usize) -> String {
        self.text
            .replace("{page}", &page.to_string())
            .replace("{pages}", &pages.to_string())
    }
}

/// Headers and footers of the first page and of even pages, used when the matching switch is on.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    use super::Band;

    #[test]
    fn renders_page_tokens() {
        let band = Band::new("Стр. {page} из {pages}");
        assert!(band.has_page_number());
        assert_eq!(band.render(2, 5), "Стр. 2 из 5");
    }

    #[test]
    fn picks_first_even_or_main_band() {
        let (h, f) = (Band::new("h"), Band::new("f"));
        let mut v = super::BandVariants {
            first: (Band::new("h1"), Band::new("f1")),
            even: (Band::new("h2"), Band::new("f2")),
            ..Default::default()
        };
        assert_eq!(v.pick(0, &h, &f).0.text, "h");
        v.different_first = true;
        v.odd_even = true;
        assert_eq!(v.pick(0, &h, &f).0.text, "h1");
        assert_eq!(v.pick(1, &h, &f).1.text, "f2");
        assert_eq!(v.pick(2, &h, &f).0.text, "h");
    }
}
