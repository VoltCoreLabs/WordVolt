use gpui_kit::*;
use wv_core::{Float, Wrap};

use super::view::EditorView;

/// Size and placement of a picture as the "size and position" dialog edits them.
#[derive(Clone, Copy)]
pub struct ImageSettings {
    pub width: f32,
    pub height: f32,
    /// `None`: in the line of text.
    pub float: Option<Float>,
}

impl EditorView {
    /// The selected picture's size and placement.
    pub fn picture_settings(&self) -> Option<ImageSettings> {
        let image = self.core.image_at_caret()?;
        let placed = self.selected_image();
        let float = image.float.or(None).map(|f| match placed {
            Some(p) => Float {
                x: p.x,
                y: p.y,
                wrap: f.wrap,
            },
            None => f,
        });
        Some(ImageSettings {
            width: image.width,
            height: image.height,
            float,
        })
    }

    /// Puts the selected picture in the line of text (`None`) or lets it float with the given
    /// wrapping; a picture that starts to float stays where it is on the page.
    pub fn set_picture_wrap(&mut self, wrap: Option<Wrap>, cx: &mut Context<Self>) {
        let Some(image) = self.core.image_at_caret() else {
            return;
        };
        let float = wrap.map(|wrap| match (image.float, self.selected_image()) {
            (Some(f), _) => Float { wrap, ..f },
            (None, Some(p)) => Float {
                x: p.x,
                y: p.y,
                wrap,
            },
            (None, None) => Float {
                x: 0.0,
                y: 0.0,
                wrap,
            },
        });
        self.apply(cx, |c| {
            c.set_image_geometry(image.width, image.height, float)
        });
    }
}
