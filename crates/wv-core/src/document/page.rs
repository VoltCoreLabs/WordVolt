/// Page size and margins in px (96 dpi).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageSpec {
    pub width: f32,
    pub height: f32,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    /// Extra room added to the left margin for binding.
    pub gutter: f32,
    /// Distance from the top edge to the header text and from the bottom edge to the footer text.
    pub header_distance: f32,
    pub footer_distance: f32,
    /// Text columns (1..=4); they flow top to bottom, left to right.
    pub columns: u8,
}

/// Margin presets of the "Margins" menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Margins {
    Normal,
    Narrow,
    Moderate,
    Wide,
}

const INCH: f32 = 96.0;

/// Most text columns of a page.
pub const MAX_COLUMNS: u8 = 4;

/// Word's default header/footer distance: 1.25 cm (709 twips).
pub const BAND_DISTANCE: f32 = 709.0 / 15.0;

impl PageSpec {
    /// A4 (210 x 297 mm), 2.54 cm margins.
    pub fn a4() -> Self {
        Self::sized(793.7, 1122.5)
    }

    fn sized(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            margin_top: INCH,
            margin_right: INCH,
            margin_bottom: INCH,
            margin_left: INCH,
            gutter: 0.0,
            header_distance: BAND_DISTANCE,
            footer_distance: BAND_DISTANCE,
            columns: 1,
        }
    }

    /// Paper sizes of the "Size" menu, portrait: A4, A5, Letter, Legal.
    pub const SIZES: [(&'static str, f32, f32); 4] = [
        ("A4", 793.7, 1122.5),
        ("A5", 559.4, 793.7),
        ("Letter", 816.0, 1056.0),
        ("Legal", 816.0, 1344.0),
    ];

    /// Snaps a size read from a file (rounded to twips) to the standard paper it is within half a
    /// pixel of, so files round-trip exactly.
    pub fn snapped(mut self) -> Self {
        if let Some(index) = self.paper_index() {
            let (_, w, h) = Self::SIZES[index];
            (self.width, self.height) = if self.is_landscape() { (h, w) } else { (w, h) };
        }
        self
    }

    /// The same page with `n` text columns (1..=4).
    pub fn with_columns(mut self, n: u8) -> Self {
        self.columns = n.clamp(1, MAX_COLUMNS);
        self
    }

    pub fn is_landscape(&self) -> bool {
        self.width > self.height
    }

    /// Switches to paper `index` of [`Self::SIZES`], keeping orientation and margins.
    pub fn with_paper(mut self, index: usize) -> Self {
        if let Some((_, w, h)) = Self::SIZES.get(index) {
            (self.width, self.height) = if self.is_landscape() {
                (*h, *w)
            } else {
                (*w, *h)
            };
        }
        self
    }

    /// Index of the paper in [`Self::SIZES`] (either orientation), if it is a standard one.
    pub fn paper_index(&self) -> Option<usize> {
        let (short, long) = (self.width.min(self.height), self.width.max(self.height));
        Self::SIZES
            .iter()
            .position(|(_, w, h)| (w - short).abs() < 0.5 && (h - long).abs() < 0.5)
    }

    /// Portrait or landscape; the margins turn with the page.
    pub fn with_landscape(mut self, landscape: bool) -> Self {
        if landscape != self.is_landscape() {
            (self.width, self.height) = (self.height, self.width);
            (
                self.margin_top,
                self.margin_right,
                self.margin_bottom,
                self.margin_left,
            ) = (
                self.margin_left,
                self.margin_top,
                self.margin_right,
                self.margin_bottom,
            );
        }
        self
    }

    /// Page of `width` x `height` px with the given margins (top, right, bottom, left) and
    /// `extras` = gutter, header distance, footer distance. Sizes are kept sane: the sheet is at
    /// least 2 inches each way, at least an inch of text remains, and the header and footer stay
    /// inside their margins.
    pub fn custom(width: f32, height: f32, margins: [f32; 4], extras: [f32; 3]) -> Self {
        let width = width.clamp(2.0 * INCH, 40.0 * INCH);
        let height = height.clamp(2.0 * INCH, 40.0 * INCH);
        let fit = |m: f32, other: f32, size: f32| m.clamp(0.0, (size - other - INCH).max(0.0));
        let [top, right, bottom, left] = margins;
        let top = fit(top, 0.0, height);
        let bottom = fit(bottom, top, height);
        let right = fit(right, 0.0, width);
        let left = fit(left, right, width);
        let [gutter, header, footer] = extras;
        let gutter = fit(gutter, left + right, width);
        Self {
            width,
            height,
            margin_top: top,
            margin_right: right,
            margin_bottom: bottom,
            margin_left: left,
            gutter,
            header_distance: header.clamp(0.0, top),
            footer_distance: footer.clamp(0.0, bottom),
            columns: 1,
        }
    }

    pub fn with_margins(mut self, preset: Margins) -> Self {
        let (vertical, horizontal) = match preset {
            Margins::Normal => (INCH, INCH),
            Margins::Narrow => (INCH / 2.0, INCH / 2.0),
            Margins::Moderate => (INCH, INCH * 0.75),
            Margins::Wide => (INCH, INCH * 2.0),
        };
        (self.margin_top, self.margin_bottom) = (vertical, vertical);
        (self.margin_left, self.margin_right) = (horizontal, horizontal);
        self
    }
}

impl Default for PageSpec {
    fn default() -> Self {
        Self::a4()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_orientation_and_margins() {
        let letter = PageSpec::a4().with_paper(2);
        assert_eq!((letter.width, letter.height), (816.0, 1056.0));
        let wide = letter.with_landscape(true).with_margins(Margins::Narrow);
        assert!(wide.is_landscape());
        assert_eq!((wide.width, wide.margin_left), (1056.0, 48.0));
        assert_eq!(wide.paper_index(), Some(2));
        assert_eq!(wide.with_paper(1).width, 793.7);
    }

    #[test]
    fn custom_page_keeps_an_inch_of_text() {
        let p = PageSpec::custom(800.0, 1000.0, [50.0, 60.0, 70.0, 80.0], [10.0, 20.0, 99.0]);
        assert_eq!((p.margin_top, p.margin_left), (50.0, 80.0));
        assert_eq!((p.gutter, p.header_distance), (10.0, 20.0));
        assert_eq!(
            p.footer_distance, 70.0,
            "the footer stays inside its margin"
        );
        let tight = PageSpec::custom(300.0, 300.0, [500.0; 4], [500.0; 3]);
        assert!(tight.width - tight.margin_left - tight.margin_right - tight.gutter >= 96.0);
        assert!(tight.height - tight.margin_top - tight.margin_bottom >= 96.0);
        assert_eq!(PageSpec::custom(1.0, 1.0, [0.0; 4], [0.0; 3]).width, 192.0);
        assert_eq!(
            PageSpec::custom(800.0, 800.0, [-5.0; 4], [0.0; 3]).margin_top,
            0.0
        );
    }
}
