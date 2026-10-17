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
}

impl NamedStyle {
    pub const ALL: [NamedStyle; 6] = [
        Self::Normal,
        Self::Title,
        Self::Subtitle,
        Self::Heading1,
        Self::Heading2,
        Self::Heading3,
    ];

    pub fn size_pt(self) -> Option<u16> {
        match self {
            Self::Normal => None,
            Self::Title => Some(28),
            Self::Subtitle => Some(14),
            Self::Heading1 => Some(20),
            Self::Heading2 => Some(16),
            Self::Heading3 => Some(14),
        }
    }

    pub fn bold(self) -> bool {
        matches!(self, Self::Heading1 | Self::Heading2 | Self::Heading3)
    }

    pub fn color(self) -> Option<u32> {
        (self == Self::Subtitle).then_some(0x6B7280)
    }

    /// Gap above the paragraph in px (dropped at the top of a page).
    pub fn space_before(self) -> f32 {
        match self {
            Self::Heading1 => 18.0,
            Self::Heading2 => 14.0,
            Self::Heading3 => 10.0,
            _ => 0.0,
        }
    }

    /// Gap below the paragraph in px; `None` = the document default.
    pub fn space_after(self) -> Option<f32> {
        match self {
            Self::Normal => None,
            Self::Title | Self::Subtitle => Some(12.0),
            Self::Heading1 => Some(6.0),
            Self::Heading2 | Self::Heading3 => Some(4.0),
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
    fn ids_round_trip() {
        for s in NamedStyle::ALL {
            assert_eq!(NamedStyle::from_id(s.id()), Some(s));
        }
        assert_eq!(NamedStyle::from_id("nope"), None);
    }
}
