/// Header or footer text, repeated on every page. `{page}` and `{pages}` stand for the page number
/// and the page count.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Band {
    pub text: String,
}

impl Band {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
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

#[cfg(test)]
mod tests {
    use super::Band;

    #[test]
    fn renders_page_tokens() {
        let band = Band::new("Стр. {page} из {pages}");
        assert!(band.has_page_number());
        assert_eq!(band.render(2, 5), "Стр. 2 из 5");
    }
}
