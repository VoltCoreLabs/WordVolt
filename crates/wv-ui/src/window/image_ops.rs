use gpui_kit::*;
use wv_i18n::{Key, t};

use super::state::MainWindow;

impl MainWindow {
    /// Puts the selected picture in the line of text (`None`) or lets it float.
    pub(super) fn set_picture_wrap(&mut self, wrap: Option<wv_core::Wrap>, cx: &mut Context<Self>) {
        self.editor
            .update(cx, |view, cx| view.set_picture_wrap(wrap, cx));
    }

    /// Opens "size and position" for the selected picture where the context menu was.
    pub(super) fn open_image_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(settings) = self.editor.read(cx).picture_settings() else {
            return;
        };
        let at = self.last_menu_at.unwrap_or_default();
        self.image_dialog
            .update(cx, |dialog, cx| dialog.open(at, settings, window, cx));
    }
    /// "Insert picture": pick a PNG/JPEG/GIF/WebP file and put it after the caret's paragraph.
    pub fn pick_and_insert_image(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let lang = self.lang;
        let picked = rfd::AsyncFileDialog::new()
            .set_parent(&*window)
            .add_filter(
                t(lang, Key::ImageFiles),
                &["png", "jpg", "jpeg", "gif", "webp"],
            )
            .pick_file();
        cx.spawn_in(window, async move |this, cx| {
            let Some(handle) = picked.await else { return };
            let path = handle.path().to_path_buf();
            this.update_in(cx, |this, window, cx| {
                this.insert_image_file(path, window, cx)
            })
            .ok();
        })
        .detach();
    }

    /// Reads the picture file and puts it after the caret's paragraph (an error dialog if it is
    /// not a picture).
    pub(super) fn insert_image_file(
        &mut self,
        path: std::path::PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let lang = self.lang;
        let to_read = path.clone();
        let read = cx
            .background_executor()
            .spawn(async move { std::fs::read(&to_read) });
        cx.spawn_in(window, async move |this, cx| {
            let bytes = read.await;
            this.update_in(cx, |this, window, cx| {
                let probed = bytes
                    .as_ref()
                    .ok()
                    .and_then(|b| wv_io::probe_image(b).map(|p| (p, b.clone())));
                match probed {
                    Some(((format, width, height), bytes)) => {
                        this.edit(cx, move |c| {
                            c.insert_image(format, bytes, width as f32, height as f32)
                        });
                        this.editor.update(cx, |view, cx| view.request_focus(cx));
                    }
                    None => {
                        let message = format!("{}: {}", path.display(), t(lang, Key::ImageBad));
                        this.show_error(window, cx, &message);
                    }
                }
            })
            .ok();
        })
        .detach();
    }
}
