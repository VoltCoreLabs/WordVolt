use super::CharStyle;

/// Built-in paragraph styles. A style supplies default size, weight, colour and spacing;
/// formatting applied directly to the text wins over it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NamedStyle {
    #[default]
    Normal,
    Title,
    Subtitle,
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    Heading7,
    Heading8,
    Heading9,
}

impl NamedStyle {
    pub const ALL: [NamedStyle; 12] = [
        Self::Normal,
        Self::Title,
        Self::Subtitle,
        Self::Heading1,
        Self::Heading2,
        Self::Heading3,
        Self::Heading4,
        Self::Heading5,
        Self::Heading6,
        Self::Heading7,
        Self::Heading8,
        Self::Heading9,
    ];

    /// 1..=9 for the headings.
    pub fn heading_level(self) -> Option<u8> {
        Self::ALL[3..]
            .iter()
            .position(|s| *s == self)
            .map(|i| i as u8 + 1)
    }

    /// Heading `level` (1..=9; deeper levels give Heading 9).
    pub fn heading(level: u8) -> Self {
        Self::ALL[3 + usize::from(level.clamp(1, 9)) - 1]
    }

    pub fn size_pt(self) -> Option<u16> {
        match self {
            Self::Normal => None,
            Self::Title => Some(28),
            Self::Subtitle => Some(14),
            Self::Heading1 => Some(20),
            Self::Heading2 => Some(16),
            Self::Heading3 => Some(14),
            Self::Heading4 => Some(12),
            Self::Heading5 | Self::Heading6 | Self::Heading7 | Self::Heading8 | Self::Heading9 => {
                None
            }
        }
    }

    pub fn bold(self) -> bool {
        self.heading_level().is_some()
    }

    pub fn color(self) -> Option<u32> {
        match self {
            Self::Subtitle => Some(0x6B7280),
            Self::Heading6 | Self::Heading7 | Self::Heading8 | Self::Heading9 => Some(0x4B5563),
            _ => None,
        }
    }

    /// Gap above the paragraph in px (dropped at the top of a page).
    pub fn space_before(self) -> f32 {
        match self {
            Self::Heading1 => 18.0,
            Self::Heading2 => 14.0,
            Self::Heading3 => 10.0,
            Self::Heading4 | Self::Heading5 | Self::Heading6 => 8.0,
            Self::Heading7 | Self::Heading8 | Self::Heading9 => 6.0,
            _ => 0.0,
        }
    }

    /// Gap below the paragraph in px; `None` = the document default.
    pub fn space_after(self) -> Option<f32> {
        match self {
            Self::Normal => None,
            Self::Title | Self::Subtitle => Some(12.0),
            Self::Heading1 => Some(6.0),
            _ => Some(4.0),
        }
    }

    /// Style of the paragraph created by pressing Enter at the end of this one.
    pub fn next(self) -> Self {
        Self::Normal
    }

    /// Fills in what the style defines and the text leaves open.
    pub fn apply(self, style: &mut CharStyle) {
        if style.size_pt.is_none() {
            style.size_pt = self.size_pt();
        }
        style.bold |= self.bold();
        if style.color.is_none() {
            style.color = self.color();
        }
    }

    /// Stable id used in files.
    pub fn id(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Title => "title",
            Self::Subtitle => "subtitle",
            Self::Heading1 => "heading1",
            Self::Heading2 => "heading2",
            Self::Heading3 => "heading3",
            Self::Heading4 => "heading4",
            Self::Heading5 => "heading5",
            Self::Heading6 => "heading6",
            Self::Heading7 => "heading7",
            Self::Heading8 => "heading8",
            Self::Heading9 => "heading9",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_fills_gaps_but_direct_formatting_wins() {
        let mut plain = CharStyle::default();
        NamedStyle::Heading1.apply(&mut plain);
        assert_eq!((plain.size_pt, plain.bold), (Some(20), true));
        let mut direct = CharStyle {
            size_pt: Some(11),
            color: Some(1),
            ..Default::default()
        };
        NamedStyle::Subtitle.apply(&mut direct);
        assert_eq!((direct.size_pt, direct.color), (Some(11), Some(1)));
    }

    #[test]
    fn heading_levels() {
        assert_eq!(NamedStyle::Normal.heading_level(), None);
        assert_eq!(NamedStyle::Title.heading_level(), None);
        for level in 1..=9 {
            assert_eq!(NamedStyle::heading(level).heading_level(), Some(level));
        }
        assert_eq!(NamedStyle::heading(20), NamedStyle::Heading9);
    }

    #[test]
    fn ids_round_trip() {
        for s in NamedStyle::ALL {
            assert_eq!(NamedStyle::from_id(s.id()), Some(s));
        }
        assert_eq!(NamedStyle::from_id("nope"), None);
    }
}
