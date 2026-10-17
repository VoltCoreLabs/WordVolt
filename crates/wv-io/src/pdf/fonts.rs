use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::Arc;

use wv_core::CharStyle;
use wv_layout::{Measurer, font_px};

/// Which font file a run needs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontRequest {
    /// Lower-case family name; `None` = the default font.
    pub family: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub mono: bool,
}

impl FontRequest {
    pub fn of(style: &CharStyle) -> Self {
        Self {
            family: style.font.as_ref().map(|f| f.to_lowercase()),
            bold: style.bold,
            italic: style.italic,
            mono: style.mono,
        }
    }
}

/// Supplies TrueType files for the PDF (the file is embedded as is).
pub trait FontSource {
    fn load(&self, request: &FontRequest) -> Option<Vec<u8>>;
}

type Files = [&'static [&'static str]; 4];

/// (family names, files for regular / bold / italic / bold italic). Segoe UI is first: it has Cyrillic.
const FAMILIES: &[(&[&str], Files)] = &[
    (
        &["segoe ui", "sf pro text", "inter", "inter variable"],
        [
            &["segoeui.ttf"],
            &["segoeuib.ttf"],
            &["segoeuii.ttf"],
            &["segoeuiz.ttf"],
        ],
    ),
    (
        &["arial"],
        [
            &["arial.ttf", "Arial.ttf"],
            &["arialbd.ttf", "Arial Bold.ttf"],
            &["ariali.ttf", "Arial Italic.ttf"],
            &["arialbi.ttf", "Arial Bold Italic.ttf"],
        ],
    ),
    (
        &["times new roman"],
        [
            &["times.ttf", "Times New Roman.ttf"],
            &["timesbd.ttf", "Times New Roman Bold.ttf"],
            &["timesi.ttf", "Times New Roman Italic.ttf"],
            &["timesbi.ttf", "Times New Roman Bold Italic.ttf"],
        ],
    ),
    (
        &["calibri"],
        [
            &["calibri.ttf"],
            &["calibrib.ttf"],
            &["calibrii.ttf"],
            &["calibriz.ttf"],
        ],
    ),
    (
        &["dejavu sans"],
        [
            &["DejaVuSans.ttf"],
            &["DejaVuSans-Bold.ttf"],
            &["DejaVuSans-Oblique.ttf"],
            &["DejaVuSans-BoldOblique.ttf"],
        ],
    ),
];

/// Monospace families, tried for inline code.
const MONO: &[(&[&str], Files)] = &[
    (
        &["consolas", "cascadia mono"],
        [
            &["consola.ttf"],
            &["consolab.ttf"],
            &["consolai.ttf"],
            &["consolaz.ttf"],
        ],
    ),
    (
        &["courier new"],
        [
            &["cour.ttf", "Courier New.ttf"],
            &["courbd.ttf", "Courier New Bold.ttf"],
            &["couri.ttf", "Courier New Italic.ttf"],
            &["courbi.ttf", "Courier New Bold Italic.ttf"],
        ],
    ),
    (
        &["dejavu sans mono"],
        [
            &["DejaVuSansMono.ttf"],
            &["DejaVuSansMono-Bold.ttf"],
            &["DejaVuSansMono-Oblique.ttf"],
            &["DejaVuSansMono-BoldOblique.ttf"],
        ],
    ),
];

const DIRS: &[&str] = &[
    "C:/Windows/Fonts",
    "/System/Library/Fonts/Supplemental",
    "/Library/Fonts",
    "/usr/share/fonts/truetype/dejavu",
    "/usr/share/fonts/TTF",
    "/usr/share/fonts/dejavu",
];

/// Fonts installed in the system.
pub struct SystemFonts;

fn read(files: &Files, slot: usize) -> Option<Vec<u8>> {
    [slot, 0].into_iter().find_map(|slot| {
        files[slot].iter().find_map(|name| {
            DIRS.iter()
                .find_map(|dir| std::fs::read(Path::new(dir).join(name)).ok())
        })
    })
}

impl FontSource for SystemFonts {
    fn load(&self, request: &FontRequest) -> Option<Vec<u8>> {
        let slot = usize::from(request.bold) + 2 * usize::from(request.italic);
        let named = request.family.as_deref().and_then(|name| {
            FAMILIES
                .iter()
                .chain(MONO)
                .find(|(names, _)| names.contains(&name))
        });
        let chain: Vec<_> = named
            .into_iter()
            .chain(if request.mono { MONO } else { &[] })
            .chain(FAMILIES)
            .collect();
        chain.into_iter().find_map(|(_, files)| read(files, slot))
    }
}

/// An embedded font and the glyphs used from it.
pub struct Face {
    pub data: Arc<Vec<u8>>,
    /// Glyph id -> (a character that maps to it, advance in 1/1000 em).
    pub used: BTreeMap<u16, (char, f32)>,
}

#[derive(Default)]
struct State {
    by_request: HashMap<FontRequest, Option<usize>>,
    faces: Vec<Face>,
}

/// The fonts of one export; also the `Measurer` that lays the document out with them.
pub struct FontSet<'a> {
    source: &'a dyn FontSource,
    state: RefCell<State>,
}

impl<'a> FontSet<'a> {
    pub fn new(source: &'a dyn FontSource) -> Self {
        Self {
            source,
            state: RefCell::default(),
        }
    }

    /// Index of the face for `style`; `None` when no font could be found at all.
    pub fn face_index(&self, style: &CharStyle) -> Option<usize> {
        let request = FontRequest::of(style);
        let mut state = self.state.borrow_mut();
        if let Some(found) = state.by_request.get(&request) {
            return *found;
        }
        let loaded = self.source.load(&request).map(Arc::new);
        let index = loaded
            .filter(|data| ttf_parser::Face::parse(data, 0).is_ok())
            .map(|data| {
                // The same file may serve several requests (fallbacks): share one face.
                let same = state.faces.iter().position(|f| Arc::ptr_eq(&f.data, &data));
                same.unwrap_or_else(|| {
                    state.faces.push(Face {
                        data,
                        used: BTreeMap::new(),
                    });
                    state.faces.len() - 1
                })
            });
        state.by_request.insert(request, index);
        index
    }

    pub fn into_faces(self) -> Vec<Face> {
        self.state.into_inner().faces
    }

    /// Glyphs of `text` in face `index`; remembers them for the embedded font.
    pub fn encode(&self, index: usize, text: &str, style: &CharStyle) -> Vec<u16> {
        let mut state = self.state.borrow_mut();
        let face = &mut state.faces[index];
        let data = face.data.clone();
        let Ok(parsed) = ttf_parser::Face::parse(&data, 0) else {
            return Vec::new();
        };
        let upem = f32::from(parsed.units_per_em());
        shown(text, style)
            .chars()
            .map(|c| {
                let gid = parsed.glyph_index(c).map_or(0, |g| g.0);
                let advance = parsed
                    .glyph_hor_advance(ttf_parser::GlyphId(gid))
                    .map_or(0.0, |a| f32::from(a) * 1000.0 / upem);
                face.used.entry(gid).or_insert((c, advance));
                gid
            })
            .collect()
    }

    fn with_face<T>(&self, style: &CharStyle, f: impl FnOnce(&ttf_parser::Face) -> T) -> Option<T> {
        let index = self.face_index(style)?;
        let data = self.state.borrow().faces[index].data.clone();
        ttf_parser::Face::parse(&data, 0).ok().map(|face| f(&face))
    }
}

/// The text as drawn (ALL CAPS styles show upper case).
pub fn shown(text: &str, style: &CharStyle) -> String {
    if style.caps {
        text.to_uppercase()
    } else {
        text.to_string()
    }
}

impl Measurer for FontSet<'_> {
    fn width(&self, text: &str, style: &CharStyle) -> f32 {
        let size = font_px(style);
        let text = shown(text, style);
        self.with_face(style, |face| {
            let upem = f32::from(face.units_per_em());
            let units: f32 = text
                .chars()
                .map(|c| {
                    face.glyph_index(c)
                        .and_then(|g| face.glyph_hor_advance(g))
                        .map_or(0.0, f32::from)
                })
                .sum();
            units * size / upem
        })
        .unwrap_or(0.0)
    }

    fn ascent(&self, style: &CharStyle) -> f32 {
        let size = font_px(style);
        self.with_face(style, |f| {
            f32::from(f.ascender()) * size / f32::from(f.units_per_em())
        })
        .unwrap_or(size)
    }

    fn descent(&self, style: &CharStyle) -> f32 {
        let size = font_px(style);
        self.with_face(style, |f| {
            -f32::from(f.descender()) * size / f32::from(f.units_per_em())
        })
        .unwrap_or(size * 0.25)
    }
}
