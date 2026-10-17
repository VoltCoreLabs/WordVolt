use std::path::{Path, PathBuf};

use gpui_kit::*;
use wv_core::Document;
use wv_i18n::{Key, Lang, t, unsaved_detail};
use wv_io::Format;

use super::state::MainWindow;

/// What to do once unsaved changes are dealt with.
type Next = Box<dyn FnOnce(&mut MainWindow, &mut Window, &mut Context<MainWindow>)>;

impl MainWindow {
    pub fn new_document(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.confirm_then(
            window,
            cx,
            Box::new(|this, _, cx| this.install(Document::default(), None, cx)),
        );
    }

    pub fn open_document(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.confirm_then(
            window,
            cx,
            Box::new(|this, window, cx| this.pick_and_open(window, cx)),
        );
    }

    pub fn save_document(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.save_then(window, cx, None);
    }

    pub fn save_document_as(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.save_as_then(window, cx, None);
    }

    /// Window close request: with unsaved changes ask first (returns `false` = keep the window open for now).
    pub fn should_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if !self.dirty {
            return true;
        }
        cx.spawn_in(window, async move |this, cx| {
            this.update_in(cx, |this, window, cx| {
                this.confirm_then(window, cx, Box::new(|_, window, _| window.remove_window()));
            })
            .ok();
        })
        .detach();
        false
    }

    /// Runs `next` now if there is nothing unsaved; otherwise asks Save / Don't save / Cancel first.
    fn confirm_then(&mut self, window: &mut Window, cx: &mut Context<Self>, next: Next) {
        if !self.dirty {
            next(self, window, cx);
            return;
        }
        let lang = self.lang;
        let detail = unsaved_detail(lang, &self.file_label());
        let answer = window.prompt(
            PromptLevel::Warning,
            t(lang, Key::UnsavedTitle),
            Some(&detail),
            &[
                t(lang, Key::Save),
                t(lang, Key::DontSave),
                t(lang, Key::Cancel),
            ],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            let Ok(choice) = answer.await else { return };
            this.update_in(cx, |this, window, cx| match choice {
                0 => this.save_then(window, cx, Some(next)),
                1 => next(this, window, cx),
                _ => {}
            })
            .ok();
        })
        .detach();
    }

    fn save_then(&mut self, window: &mut Window, cx: &mut Context<Self>, next: Option<Next>) {
        let Some(path) = self.file_path.clone() else {
            return self.save_as_then(window, cx, next);
        };
        if self.write_to(&path, window, cx)
            && let Some(next) = next
        {
            next(self, window, cx);
        }
    }

    /// "Save as": the dialog offers our formats; the chosen filter decides the extension.
    fn save_as_then(&mut self, window: &mut Window, cx: &mut Context<Self>, next: Option<Next>) {
        let lang = self.lang;
        let current = self
            .file_path
            .as_deref()
            .and_then(Format::from_path)
            .unwrap_or(Format::Wvd);
        let mut dialog = rfd::AsyncFileDialog::new()
            .set_parent(&*window)
            .set_file_name(format!("{}.{}", self.file_stem(), current.extension()));
        if let Some(dir) = self.file_path.as_deref().and_then(Path::parent) {
            dialog = dialog.set_directory(dir);
        }
        // Current format first = preselected.
        let order =
            std::iter::once(current).chain(Format::ALL.into_iter().filter(|f| *f != current));
        for format in order {
            dialog = dialog.add_filter(format_label(lang, format), format.extensions());
        }
        let picked = dialog.save_file();
        cx.spawn_in(window, async move |this, cx| {
            let Some(handle) = picked.await else { return };
            let mut path = handle.path().to_path_buf();
            if Format::from_path(&path).is_none() {
                path.set_extension(Format::Wvd.extension());
            }
            this.update_in(cx, |this, window, cx| {
                if this.write_to(&path, window, cx)
                    && let Some(next) = next
                {
                    next(this, window, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    /// Writes the document; on success the file becomes the current one. Shows an error dialog otherwise.
    fn write_to(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let doc = self.editor.read(cx).core.doc.clone();
        match wv_io::save_file(path, &doc) {
            Ok(()) => {
                self.file_path = Some(path.to_path_buf());
                self.editor.update(cx, |view, cx| {
                    view.core.mark_saved();
                    cx.notify();
                });
                cx.notify();
                true
            }
            Err(err) => {
                self.show_error(window, cx, &format!("{}: {err}", path.display()));
                false
            }
        }
    }

    fn pick_and_open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let lang = self.lang;
        let all: Vec<&str> = Format::ALL
            .iter()
            .flat_map(|f| f.extensions().iter().copied())
            .collect();
        let mut dialog = rfd::AsyncFileDialog::new()
            .set_parent(&*window)
            .add_filter(t(lang, Key::AllSupported), &all);
        for format in Format::ALL {
            dialog = dialog.add_filter(format_label(lang, format), format.extensions());
        }
        let picked = dialog.pick_file();
        cx.spawn_in(window, async move |this, cx| {
            let Some(handle) = picked.await else { return };
            let path = handle.path().to_path_buf();
            this.update_in(cx, |this, window, cx| this.load_path(path, window, cx))
                .ok();
        })
        .detach();
    }

    /// Opens a file (asks about unsaved changes first).
    pub(super) fn open_path(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.confirm_then(
            window,
            cx,
            Box::new(move |this, window, cx| this.load_path(path, window, cx)),
        );
    }

    fn load_path(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let to_read = path.clone();
        let loaded = cx
            .background_executor()
            .spawn(async move { wv_io::load_file(&to_read) });
        cx.spawn_in(window, async move |this, cx| {
            let loaded = loaded.await;
            this.update_in(cx, |this, window, cx| match loaded {
                Ok(doc) => this.install(doc, Some(path), cx),
                Err(err) => this.show_error(window, cx, &format!("{}: {err}", path.display())),
            })
            .ok();
        })
        .detach();
    }

    /// Makes `doc` the current document.
    pub(crate) fn install(&mut self, doc: Document, path: Option<PathBuf>, cx: &mut Context<Self>) {
        self.file_path = path;
        self.editor.update(cx, |view, cx| view.load(doc, cx));
        cx.notify();
    }

    pub(super) fn show_error(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        message: &str,
    ) {
        let title = t(self.lang, Key::ErrorTitle);
        drop(window.prompt(PromptLevel::Critical, title, Some(message), &["OK"], cx));
    }
}

fn format_label(lang: Lang, format: Format) -> &'static str {
    t(
        lang,
        match format {
            Format::Wvd => Key::FormatWvd,
            Format::Docx => Key::FormatDocx,
            Format::Txt => Key::FormatTxt,
            Format::Md => Key::FormatMd,
        },
    )
}
