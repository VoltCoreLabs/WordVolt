use wv_core::NamedStyle;
use wv_i18n::{Key, Lang, t};

pub fn style_key(style: NamedStyle) -> Key {
    match style {
        NamedStyle::Normal => Key::StyleNormal,
        NamedStyle::Title => Key::StyleTitle,
        NamedStyle::Subtitle => Key::StyleSubtitle,
        NamedStyle::Heading1 => Key::StyleHeading1,
        NamedStyle::Heading2 => Key::StyleHeading2,
        NamedStyle::Heading3 => Key::StyleHeading3,
        NamedStyle::Heading4 => Key::StyleHeading4,
        NamedStyle::Heading5 => Key::StyleHeading5,
        NamedStyle::Heading6 => Key::StyleHeading6,
        NamedStyle::Heading7 => Key::StyleHeading7,
        NamedStyle::Heading8 => Key::StyleHeading8,
        NamedStyle::Heading9 => Key::StyleHeading9,
    }
}

/// Display name of a built-in style.
pub fn style_label(lang: Lang, style: NamedStyle) -> &'static str {
    t(lang, style_key(style))
}

/// The style whose display name is `label` (as picked from the list).
pub fn style_from_label(lang: Lang, label: &str) -> Option<NamedStyle> {
    NamedStyle::ALL
        .into_iter()
        .find(|s| style_label(lang, *s) == label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_round_trip_in_both_languages() {
        for lang in [Lang::Ru, Lang::En] {
            for style in NamedStyle::ALL {
                assert_eq!(
                    style_from_label(lang, style_label(lang, style)),
                    Some(style)
                );
            }
        }
        assert_eq!(
            style_from_label(Lang::Ru, "Заголовок 1"),
            Some(NamedStyle::Heading1)
        );
        assert_eq!(style_from_label(Lang::Ru, "что-то"), None);
    }
}
