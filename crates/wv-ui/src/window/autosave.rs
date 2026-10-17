use std::time::Duration;

use gpui_kit::*;
use wv_io::recovery;

use super::state::MainWindow;

/// How often the "still running" mark is refreshed.
const TICK: Duration = Duration::from_secs(10);
/// Recovery copies are written every this many ticks.
const SAVE_EVERY: u32 = 3;
/// A window whose mark is older than this has stopped (crashed).
const STALE: Duration = Duration::from_secs(30);

impl MainWindow {
    /// Starts the timer: the window shows that it is alive and writes recovery copies of unsaved
    /// documents.
    pub fn start_autosave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        recovery::heartbeat(&self.recovery_dir, self.instance);
        cx.spawn_in(window, async move |this, cx| {
            let mut tick = 0u32;
            loop {
                cx.background_executor().timer(TICK).await;
                tick += 1;
                let alive = this
                    .update_in(cx, |this, _, cx| {
                        recovery::heartbeat(&this.recovery_dir, this.instance);
                        if tick.is_multiple_of(SAVE_EVERY) {
                            this.autosave(cx);
                        }
                    })
                    .is_ok();
                if !alive {
                    break;
                }
            }
        })
        .detach();
    }

    /// Writes a recovery copy of every document that changed since its last copy.
    fn autosave(&mut self, cx: &mut Context<Self>) {
        for i in 0..self.docs.len() {
            let (editor, path, dirty) = if i == self.active {
                (self.editor.clone(), self.file_path.clone(), self.dirty)
            } else {
                let slot = &self.docs[i];
                (slot.editor.clone(), slot.file_path.clone(), slot.dirty)
            };
            let revision = editor.read(cx).core.revision();
            if !dirty || self.docs[i].autosaved == Some(revision) {
                continue;
            }
            let doc = editor.read(cx).core.doc.clone();
            let (dir, instance, session) = (
                self.recovery_dir.clone(),
                self.instance,
                self.docs[i].session,
            );
            self.docs[i].autosaved = Some(revision);
            cx.background_executor()
                .spawn(async move {
                    let _ = recovery::write(&dir, instance, session, path.as_deref(), &doc);
                })
                .detach();
        }
    }

    /// Opens the documents a crashed window left behind.
    pub fn restore_recovered(&mut self, cx: &mut Context<Self>) {
        for rec in recovery::scan(&self.recovery_dir, STALE) {
            self.open_recovered(rec, cx);
        }
    }

    /// The active document was saved: its recovery copy is no longer needed.
    pub(super) fn forget_recovery(&mut self) {
        let slot = &mut self.docs[self.active];
        recovery::remove(&self.recovery_dir, slot.session);
        slot.autosaved = None;
    }

    /// The window closes for good: all copies and the "alive" mark go.
    pub(super) fn finish_session(&self) {
        for slot in &self.docs {
            recovery::remove(&self.recovery_dir, slot.session);
        }
        recovery::end_instance(&self.recovery_dir, self.instance);
    }
}
