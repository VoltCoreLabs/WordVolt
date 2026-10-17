use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use gpui_kit::*;

use super::state::MainWindow;

/// Print copies older than this are removed the next time something is printed.
const KEEP: Duration = Duration::from_secs(24 * 3600);

fn print_dir() -> PathBuf {
    std::env::temp_dir().join("WordVolt-print")
}

/// Deletes old print copies.
fn clean(dir: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let old = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| SystemTime::now().duration_since(t).ok())
            .is_some_and(|age| age > KEEP);
        if old {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

impl MainWindow {
    /// Print: the document is written as a PDF in a temporary folder and opened in the system's
    /// PDF viewer, which previews it and prints it (Ctrl+P there) with the printer's own dialog.
    pub fn print_document(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let doc = self.editor.read(cx).core.doc.clone();
        let name = self.file_stem();
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let dir = print_dir();
                    std::fs::create_dir_all(&dir)?;
                    clean(&dir);
                    let stamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map_or(0, |d| d.as_secs());
                    let path = dir.join(format!("{name}-{stamp}.pdf"));
                    let bytes = wv_io::encode_pdf(&doc, &wv_io::SystemFonts)?;
                    std::fs::write(&path, bytes)?;
                    Ok::<_, wv_io::IoError>(path)
                })
                .await;
            this.update_in(cx, |this, window, cx| match result {
                Ok(path) => cx.open_with_system(&path),
                Err(err) => this.show_error(window, cx, &err.to_string()),
            })
            .ok();
        })
        .detach();
    }
}
