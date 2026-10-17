use std::path::PathBuf;

use gpui_kit::*;
use wv_core::ImageFormat;
use wv_io::Format;

use super::state::MainWindow;

impl MainWindow {
    /// Files dropped onto the window: pictures go into the document, the first document file opens.
    pub(super) fn drop_files(
        &mut self,
        paths: &[PathBuf],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let is_image = |p: &PathBuf| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| ImageFormat::from_extension(e).is_some())
        };
        for path in paths.iter().filter(|p| is_image(p)) {
            self.insert_image_file(path.clone(), window, cx);
        }
        if let Some(doc) = paths.iter().find(|p| Format::from_path(p).is_some()) {
            self.open_path(doc.clone(), window, cx);
        }
    }
}
