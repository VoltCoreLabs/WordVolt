use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Wvd,
    Docx,
    Txt,
    Md,
}

impl Format {
    /// Formats we can save to, in the order shown in the Save dialog.
    pub const ALL: [Format; 4] = [Format::Wvd, Format::Docx, Format::Txt, Format::Md];

    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "wvd" => Some(Self::Wvd),
            "docx" => Some(Self::Docx),
            "txt" => Some(Self::Txt),
            "md" | "markdown" => Some(Self::Md),
            _ => None,
        }
    }

    /// Preferred extension, without the dot.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Wvd => "wvd",
            Self::Docx => "docx",
            Self::Txt => "txt",
            Self::Md => "md",
        }
    }

    /// All extensions that open as this format.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Wvd => &["wvd"],
            Self::Docx => &["docx"],
            Self::Txt => &["txt"],
            Self::Md => &["md", "markdown"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_by_extension_case_insensitively() {
        assert_eq!(Format::from_path(Path::new("a.WVD")), Some(Format::Wvd));
        assert_eq!(Format::from_path(Path::new("a.markdown")), Some(Format::Md));
        assert_eq!(Format::from_path(Path::new("a.DOCX")), Some(Format::Docx));
        assert_eq!(Format::from_path(Path::new("a")), None);
        assert_eq!(Format::from_path(Path::new("a.rtf")), None);
    }

    #[test]
    fn every_format_round_trips_its_extension() {
        for format in Format::ALL {
            let name = format!("file.{}", format.extension());
            assert_eq!(Format::from_path(Path::new(&name)), Some(format));
        }
    }
}
