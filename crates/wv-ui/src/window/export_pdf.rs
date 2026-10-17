use gpui_kit::*;
use wv_i18n::Key;

use super::state::MainWindow;

impl MainWindow {
    /// Asks for a file name and writes the document as a PDF (the file stays the current one).
    pub fn export_pdf(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let lang = self.lang;
        let mut dialog = rfd::AsyncFileDialog::new()
            .set_parent(&*window)
            .set_file_name(format!("{}.pdf", self.file_stem()))
            .add_filter(wv_i18n::t(lang, Key::FormatPdf), &["pdf"]);
        if let Some(dir) = self.file_path.as_deref().and_then(std::path::Path::parent) {
            dialog = dialog.set_directory(dir);
        }
        let picked = dialog.save_file();
        let doc = self.editor.read(cx).core.doc.clone();
        cx.spawn_in(window, async move |this, cx| {
            let Some(handle) = picked.await else { return };
            let mut path = handle.path().to_path_buf();
            path.set_extension("pdf");
            let target = path.clone();
            let result = cx
                .background_executor()
                .spawn(async move {
                    let bytes = wv_io::encode_pdf(&doc, &wv_io::SystemFonts)?;
                    std::fs::write(&target, bytes)?;
                    Ok::<_, wv_io::IoError>(())
                })
                .await;
            if let Err(err) = result {
                this.update_in(cx, |this, window, cx| {
                    this.show_error(window, cx, &format!("{}: {err}", path.display()))
                })
                .ok();
            }
        })
        .detach();
    }
}
