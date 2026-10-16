use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Wvd,
    Txt,
    Md,
}

impl Format {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "wvd" => Some(Self::Wvd),
            "txt" => Some(Self::Txt),
            "md" | "markdown" => Some(Self::Md),
            _ => None,
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
        assert_eq!(Format::from_path(Path::new("a")), None);
        assert_eq!(Format::from_path(Path::new("a.docx")), None);
    }
}
