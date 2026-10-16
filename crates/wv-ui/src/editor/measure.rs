use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::*;
use wv_core::CharStyle;
use wv_layout::{DEFAULT_SIZE_PT, Measurer, font_px};

use crate::theme::ui_font;

/// bold, italic, size (pt), hash of the font family (0 = default).
type StyleKey = (bool, bool, u16, u64);

fn key(style: &CharStyle) -> StyleKey {
    let family = style.font.as_deref().map_or(0, |f| {
        let mut hasher = DefaultHasher::new();
        f.hash(&mut hasher);
        hasher.finish()
    });
    (
        style.bold,
        style.italic,
        style.size_pt.unwrap_or(DEFAULT_SIZE_PT),
        family,
    )
}

/// Cached text widths and font metrics, shared between frames.
#[derive(Default)]
pub struct MeasureCache {
    widths: HashMap<StyleKey, HashMap<String, f32>>,
    metrics: HashMap<StyleKey, (f32, f32)>,
}

pub fn font_for(style: &CharStyle) -> Font {
    let mut font = ui_font();
    if let Some(family) = &style.font {
        font.family = family.clone().into();
    }
    if style.bold {
        font.weight = FontWeight::BOLD;
    }
    if style.italic {
        font.style = FontStyle::Italic;
    }
    font
}

pub fn text_run(len: usize, style: &CharStyle, color: Hsla) -> TextRun {
    TextRun {
        len,
        font: font_for(style),
        color,
        background_color: None,
        underline: style.underline.then(|| UnderlineStyle {
            thickness: px(1.),
            color: None,
            wavy: false,
        }),
        strikethrough: None,
    }
}

/// wv-layout's `Measurer` on top of GPUI's text shaping.
pub struct GpuiMeasurer {
    pub system: Arc<WindowTextSystem>,
    cache: Rc<RefCell<MeasureCache>>,
}

impl GpuiMeasurer {
    pub fn new(window: &Window, cache: Rc<RefCell<MeasureCache>>) -> Self {
        Self {
            system: window.text_system().clone(),
            cache,
        }
    }

    pub fn shape(&self, text: &str, style: &CharStyle, color: Hsla) -> ShapedLine {
        self.system.shape_line(
            text.to_string().into(),
            px(font_px(style)),
            &[text_run(text.len(), style, color)],
            None,
        )
    }

    fn metrics(&self, style: &CharStyle) -> (f32, f32) {
        let k = key(style);
        if let Some(found) = self.cache.borrow().metrics.get(&k) {
            return *found;
        }
        let line = self.shape("x", style, black());
        let found = (f32::from(line.ascent), f32::from(line.descent));
        self.cache.borrow_mut().metrics.insert(k, found);
        found
    }
}

impl Measurer for GpuiMeasurer {
    fn width(&self, text: &str, style: &CharStyle) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let k = key(style);
        if let Some(found) = self.cache.borrow().widths.get(&k).and_then(|m| m.get(text)) {
            return *found;
        }
        let width = f32::from(self.shape(text, style, black()).width());
        let mut cache = self.cache.borrow_mut();
        let per_style = cache.widths.entry(k).or_default();
        if per_style.len() > 20_000 {
            per_style.clear();
        }
        per_style.insert(text.to_string(), width);
        width
    }

    fn ascent(&self, style: &CharStyle) -> f32 {
        self.metrics(style).0
    }

    fn descent(&self, style: &CharStyle) -> f32 {
        self.metrics(style).1
    }
}
